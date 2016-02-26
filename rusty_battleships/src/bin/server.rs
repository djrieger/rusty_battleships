use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{BufReader, BufWriter, Write};
use std::net::{Ipv4Addr, TcpListener, TcpStream};
use std::option::Option::None;
use std::rc::Rc;
use std::sync::mpsc;
use std::thread;

extern crate time;

extern crate argparse;
use argparse::{ArgumentParser, Print, Store};

extern crate rusty_battleships;
use rusty_battleships::message::{serialize_message, deserialize_message, Message};
use rusty_battleships::board;
use rusty_battleships::board::{Player, ToMainThreadCommand, ToChildCommand};
use rusty_battleships::game::Game;
use rusty_battleships::timer::timer_periodic;
use rusty_battleships::serverstate;

// http://stackoverflow.com/questions/35157399/how-to-concatenate-static-strings-in-rust/35159310
macro_rules! description {
    () => ( "rusty battleships: game server" )
}
macro_rules! version {
    () => ( env!("CARGO_PKG_VERSION") )
}
macro_rules! version_string {
    () => ( concat!(description!(), " v", version!()) )
}

const TICK_DURATION_MS: u64 = 250;


// Tell server to perform proper shutdown, like removing player from their game, informing
// opponent etc.
fn shutdown_player(tx: &mpsc::SyncSender<ToMainThreadCommand>, rx: &mpsc::Receiver<ToChildCommand>) {
    tx.send(ToMainThreadCommand::TerminatePlayer).expect("Main thread died, exiting.");
    rx.recv().expect("Main thread died before answering, exiting.");
}

fn handle_client(stream: TcpStream, tx: mpsc::SyncSender<ToMainThreadCommand>, rx: mpsc::Receiver<ToChildCommand>) {
    println!("New incoming TCP stream");

    let response_stream = stream.try_clone().unwrap();
    let mut buff_reader = BufReader::new(stream);
    let mut buff_writer = BufWriter::new(response_stream);
    let tick = timer_periodic(TICK_DURATION_MS);
    loop {
        let request = deserialize_message(&mut buff_reader);
        let response_msg;
        match request {
            Ok(request_msg) => {
                // Send parsed Message to main thread
                tx.send(ToMainThreadCommand::Message(request_msg)).expect("Main thread died, exiting.");
                // Wait for response Message
                let response = rx.recv().expect("Main thread died before answering, exiting.");
                match response {
                    ToChildCommand::Message(msg) => {
                        response_msg = msg;
                        if response_msg == Message::InvalidRequestResponse {
                            shutdown_player(&tx, &rx);
                        }
                    },
                    ToChildCommand::TerminateConnection => return,
                }
            },
            // Normal connection termination
            Err(ref e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                shutdown_player(&tx, &rx);
                println!("Client terminated connection");
                // Client terminated TCP connection, exit
                return;
            },
            // Malformed message received
            Err(_) => {
                shutdown_player(&tx, &rx);
                println!("Received malformed message, terminating client");
                response_msg = Message::InvalidRequestResponse;
            }
        }
        // Serialize, send response and flush
        let clone_response = response_msg.clone();
        let serialized_msg = serialize_message(response_msg);
        buff_writer.write(&serialized_msg[..]).expect("Could not write to TCP steam, exiting.");
        buff_writer.flush().expect("Could not write to TCP steam, exiting.");
        if clone_response == Message::InvalidRequestResponse {
            return;
        }

        tick.recv().expect("Timer thread died unexpectedly."); // wait for next tick
    }
}

fn handle_main(msg: Message, player: &mut board::PlayerHandle, lobby: &mut HashMap<String, board::Player>, games: &mut Vec<Rc<RefCell<Game>>>) -> serverstate::Result {
    // These requests can be handled without any restrictions
    match msg {
        Message::GetFeaturesRequest => return serverstate::handle_get_features_request(),
        Message::ReportErrorRequest { errormessage } => return serverstate::handle_report_error_request(errormessage, player, lobby, games),
        _ => {},
    }

    // Login requests on the other hand are only valid if the client is not already logged in, i.e.
    // their nickname must be None
    if player.nickname.is_none() {
        if let Message::LoginRequest { username } = msg {
            return serverstate::handle_login_request(username, player, lobby);
        }
    } else {
        // All other requests are only valid after logging in, i.e. with a user name
        assert!(lobby.contains_key(player.nickname.as_ref().unwrap()), "Invalid state: nickname not in lobby.");

        let nickname = player.nickname.as_ref().unwrap();

        match msg {
            Message::ReadyRequest => return serverstate::handle_ready_request(nickname, lobby),
            Message::NotReadyRequest => return serverstate::handle_not_ready_request(nickname, lobby),
            Message::ChallengePlayerRequest { username } => return serverstate::handle_challenge_player_request(username, nickname, lobby, games),
            Message::SurrenderRequest => return serverstate::handle_surrender_request(nickname, lobby, games),
            Message::PlaceShipsRequest { placement } => return serverstate::handle_place_ships_request(placement, nickname, lobby),
            Message::ShootRequest { x, y } => return serverstate::handle_move_shoot_request((x, y), None, nickname, lobby, games),
            Message::MoveAndShootRequest { id, direction, x, y } => return serverstate::handle_move_shoot_request((x, y), Some((id as usize, direction)), nickname, lobby, games),
            _ => {},
        };
    }
    return serverstate::Result::respond(Message::InvalidRequestResponse, false);
}

fn main() {
    let mut port:u16 = 5000;
    let mut ip = Ipv4Addr::new(127, 0, 0, 1);

    {  // this block limits scope of borrows by ap.refer() method
        let mut ap = ArgumentParser::new();
        ap.set_description(description!());
        ap.refer(&mut ip).add_argument("IP", Store, "IPv4 address to listen to");
        ap.refer(&mut port).add_option(&["-p", "--port"], Store, "port to listen on");
        ap.add_option(&["-v", "--version"], Print(version_string!().to_owned()),
        "show version number");
        ap.parse_args_or_exit();
    }

    println!("Operating as server on port {}.", port);

    let listener = TcpListener::bind((ip, port))
            .expect(&format!("Could not bind to port {}", port));
    let address = listener.local_addr()
            .expect("Could not get local address.");
    println!("Started listening on port {} at address {}.", port, address);
    let mut player_handles = Vec::new();
    let mut lobby = HashMap::new();

    // channel for letting tcp_loop tell main loop about new players
    let (tx_tcp_players, rx_main_players) : (mpsc::Sender<board::PlayerHandle>, mpsc::Receiver<board::PlayerHandle>) = mpsc::channel();

    let tcp_loop = move || {
        for stream in listener.incoming() {
            // channel child --> main
            let (tx_child, rx_main) = mpsc::sync_channel(0);
            // channel main --> child
            let (tx_main, rx_child) = mpsc::channel();

            thread::spawn(move || {
                handle_client(stream.unwrap(), tx_child, rx_child);
            });
            tx_tcp_players.send(
                board::PlayerHandle {
                    nickname: None,
                    from_child_endpoint: rx_main,
                    to_child_endpoint: tx_main,
                }
            ).expect("Main thread died, exiting.");
        }
    };

    thread::spawn(tcp_loop);
    let mut message_store: HashMap<String, Vec<Message>> = HashMap::new();
    let mut games: Vec<Rc<RefCell<Game>>> = vec![];
    // stores player name -> game

    let tick = timer_periodic(TICK_DURATION_MS);
    // Main loop
    loop {
        // Receive new players from tcp_loop
        if let Ok(player_handle) = rx_main_players.try_recv() {
            player_handles.push(player_handle);
        }
        // Receive Messages from child threads
        for (i, player_handle) in player_handles.iter_mut().enumerate() {
            if let Ok(maybe_msg) = player_handle.from_child_endpoint.try_recv() {
                match maybe_msg {
                    ToMainThreadCommand::Message(msg) => {
                        print!("From child {} ({}): {:?}", i, player_handle.nickname.clone().unwrap_or("".to_owned()), msg);
                        // Handle Message received from child
                        let result = handle_main(msg, player_handle, &mut &mut lobby, &mut games);
                        if let Some(response) = result.response {
                            // handle_main generated a response -> send response Message back to child
                            println!(" -> {:?}", response);
                            player_handle.to_child_endpoint.send(ToChildCommand::Message(response));
                        }
                        if result.terminate_connection {
                            print!("-- Closing connection to child {}", i);
                            player_handle.to_child_endpoint.send(ToChildCommand::TerminateConnection);
                        }
                        if !result.updates.is_empty() {
                            message_store = result.updates;
                            break;
                        }
                    },
                    ToMainThreadCommand::TerminatePlayer => {
                        if let Some(ref name) = player_handle.nickname {
                            message_store = serverstate::terminate_player(name, &mut lobby, &mut games);
                        }
                        player_handle.to_child_endpoint.send(ToChildCommand::TerminateConnection);
                    }
                }
            }
        }
        // Send all updates from result
        send_updates(&mut player_handles, &mut message_store);

        // Send updates issued by handle_afk() for all games with exceeded turn times
        let mut afk_games: Vec<Rc<RefCell<Game>>> = vec![];
        for game in games.iter() {
            if game.borrow_mut().turn_time_exceeded() {
                afk_games.push(game.clone());
            }
        }
        for game in afk_games {
            let mut result = serverstate::handle_afk(game.clone(), &mut lobby, &mut games);
            send_updates(&mut player_handles, &mut result);
        }

        tick.recv().expect("Timer thread died unexpectedly."); // wait for next tick
    }
}

fn send_updates(player_handles: &mut Vec<board::PlayerHandle>, message_store: &mut HashMap<String, Vec<Message>>) {
    // player_handles
    //     .iter_mut()
    //     .filter(|player_handle| player_handle.nickname.is_some() )
    //     .filter(|player_handle| message_store.contains_key(&player_handle.nickname.unwrap()))
    //     .map(|player_handle| message_store.remove(&player_handle.nickname.unwrap()).unwrap().iter().map(|msg| player_handle.to_child_endpoint.send(ToChildCommand::Message(*msg)) ) );
    for player_handle in player_handles.iter_mut() {
        if let Some(ref name) = player_handle.nickname {
            if message_store.contains_key(name) {
                for message in message_store.remove(name).unwrap() {
                    println!("Update to {}: {:?}", name, message);
                    player_handle.to_child_endpoint.send(ToChildCommand::Message(message));
                }
            }
        }
    }
    message_store.clear();
}
