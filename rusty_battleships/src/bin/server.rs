use std::collections::{HashMap, HashSet};
use std::io::{BufReader, BufWriter, Write};
use std::net::{Ipv4Addr, TcpListener, TcpStream};
use std::option::Option::None;
use std::sync::mpsc;
use std::thread;

extern crate argparse;
use argparse::{ArgumentParser, Print, Store};

extern crate rusty_battleships;
use rusty_battleships::message::{serialize_message, deserialize_message, Message};
use rusty_battleships::board;
use rusty_battleships::board::{Game, Player};
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

fn handle_client(stream: TcpStream, tx: mpsc::SyncSender<Message>, rx: mpsc::Receiver<Option<Message>>) {
    println!("New incoming TCP stream");

    let response_stream = stream.try_clone().unwrap();
    let mut buff_reader = BufReader::new(stream);
    let mut buff_writer = BufWriter::new(response_stream);
    loop {
        let request = deserialize_message(&mut buff_reader);
        let response_msg;
        match request {
            Ok(request_msg) => {
                // Send parsed Message to main thread
                tx.send(request_msg).expect("Main thread died, exiting.");
                // Wait for response Message
                let response = rx.recv().expect("Main thread died before answering, exiting.");
                match response {
                    Some(msg) => response_msg = msg,
                    None => break, // server wants to terminate this client
                }
            },
            Err(e) => {
                response_msg = Message::InvalidRequestResponse;
                println!("Received invalid request: {}", e);
            }
        }
        // Serialize, send response and flush
        let serialized_msg = serialize_message(response_msg);
        buff_writer.write(&serialized_msg[..]).expect("Could not write to TCP steam, exiting.");
        buff_writer.flush().expect("Could not write to TCP steam, exiting.");
    }
}

fn handle_main(msg: Message, player: &mut board::PlayerHandle, player_names: &mut HashSet<String>, lobby: &mut HashMap<String, board::Player>, games: &mut Vec<Game>) -> serverstate::Result {
    // These requests can be handled without any restrictions
    match msg {
        Message::GetFeaturesRequest => return serverstate::handle_get_features_request(),
        Message::ReportErrorRequest { errormessage } => return serverstate::handle_report_error_request(errormessage, player, player_names, lobby, games),
        _ => {},
    }

    // Login requests on the other hand are only valid if the client is not already logged in, i.e.
    // their nickname must be None
    if player.nickname.is_none() {
        if let Message::LoginRequest { username } = msg {
            return serverstate::handle_login_request(username, player, player_names, lobby); 
        }
    } else {
        // All other requests are only valid after logging in, i.e. with a user name
        match msg {
            Message::LoginRequest { username } => return serverstate::handle_login_request(username, player, player_names, lobby), 
            Message::ReadyRequest => return serverstate::handle_ready_request(player, player_names, lobby),
            Message::NotReadyRequest => return serverstate::handle_not_ready_request(player, player_names, lobby),
            Message::ChallengePlayerRequest { username } => return serverstate::handle_challenge_player_request(username, player, player_names, lobby, games),  
            Message::SurrenderRequest => return serverstate::handle_surrender_request(player, player_names, lobby),
            Message::PlaceShipsRequest { placement } => return serverstate::handle_place_ships_request(placement, player, player_names, lobby),
            Message::ShootRequest { x, y } => return serverstate::handle_move_shoot_request((x, y), None, player, player_names, lobby),
            Message::MoveAndShootRequest { id, direction, x, y } => return serverstate::handle_move_shoot_request((x, y), Some((id as usize, direction)), player, player_names, lobby),
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
    let mut player_names = HashSet::new();
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
    let mut games: Vec<Game> = vec![];
    // stores player name -> game
    // let mut games: HashMap<String, &Game> = HashMap::new();

    // Main loop
    loop {
        // Receive new players from tcp_loop
        if let Ok(player_handle) = rx_main_players.try_recv() {
            player_handles.push(player_handle);
        }
        // Receive Messages from child threads
        for (i, player_handle) in player_handles.iter_mut().enumerate() {
            if let Ok(msg) = player_handle.from_child_endpoint.try_recv() {
                print!("[Child {}] {:?}", i, msg);
                // Handle Message received from child
                let result = handle_main(msg, player_handle, &mut player_names, &mut lobby, &mut games);
                if let Some(response) = result.response {
                    // handle_main generated a response -> send response Message back to child
                    println!(" -> {:?}", response);
                    player_handle.to_child_endpoint.send(Some(response));
                }
                if result.terminate_connection {
                    print!("-- Closing connection to child {}", i);
                    player_handle.to_child_endpoint.send(None);
                }
                if !result.updates.is_empty() {
                    message_store = result.updates;
                    break;
                }
            }
        }
        // Send all messages saved in message_store
        // TODO: Does not work yet (compiler complains about moving out of borrowed context)
        let player_names_iter = player_handles
            .iter()
            // .by_ref()
            .filter_map(|player_handle| player_handle.nickname.clone());

        // for (i, player_handle) in player_handles.iter_mut().enumerate() {
        //     if let Some(ref name) = player_handle.nickname {
        //         if message_store.contains_key(name) {
        //             for message in message_store.remove(name).unwrap() {
        //                 player_handle.to_child_endpoint.send(Some(message));
        //             }
        //         }
        //     }
        // }
        message_store.clear();
    }
    // tcp_thread.join();
}
