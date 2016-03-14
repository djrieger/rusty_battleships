use std::cell::RefCell;
use std::net::UdpSocket;
use std::collections::HashMap;
use std::io::{BufReader, BufWriter, Write};
use std::net::{Ipv4Addr, TcpListener, TcpStream};
use std::option::Option::None;
use std::rc::Rc;
use std::sync::mpsc;
use std::thread;

extern crate net2;
use net2::UdpSocketExt;

extern crate byteorder;
use byteorder::{BigEndian, WriteBytesExt};

extern crate ansi_term;
use ansi_term::Colour::{Green, Yellow, Cyan};

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

const TICK_DURATION_MS: u64 = 5;

fn start_udp_discovery(tcp_port: u16) {
    let socket = match UdpSocket::bind("0.0.0.0:49001") {
        Ok(s) => s,
        Err(e) => panic!("couldn't bind socket: {}", e)
    };
    match socket.join_multicast_v4(&Ipv4Addr::new(224, 0, 0, 250), &Ipv4Addr::new(0, 0, 0, 0)) {
        Err(why) => println!("{:?}", why),
        Ok(_) => {},
    };
    println!("Joined UDP multicast group 224.0.0.250, listening on port 49001 for UDP discovery");
    let udp_discovery_loop = move || {
        let mut buf = [0; 2048];
        loop {
            match socket.recv_from(&mut buf) {
                Ok((num_bytes, src)) => {
                    println!("num_bytes: {}", num_bytes);
                    println!("src: {}", src);
                    println!("{}", std::str::from_utf8(&buf).unwrap_or(""));
                    let mut response = vec![];
                    response.write_u16::<BigEndian>(tcp_port).unwrap();
                    write!(&mut response, "Some host name");
                    socket.send_to(&response[..], &src).expect("unable to respond");
                    println!("Responded with port and host name");
                },
                Err(e) => {
                    println!("couldn't recieve a datagram: {}", e);
                }
            }
        }
    };
    thread::spawn(udp_discovery_loop);
}

// Tell server to perform proper shutdown, like removing player from their game, informing
// opponent etc.
fn shutdown_player(tx: &mpsc::SyncSender<ToMainThreadCommand>, rx: &mpsc::Receiver<ToChildCommand>) {
    tx.send(ToMainThreadCommand::TerminatePlayer).expect("Main thread died, exiting.");
    rx.recv().expect("Main thread died before answering, exiting.");
}

fn respond(response_msg: Message, buff_writer: &mut BufWriter<TcpStream>) -> Result<(), std::io::Error> {
    let serialized_msg = serialize_message(response_msg);
    try!(buff_writer.write(&serialized_msg[..])); //.expect("Could not write to TCP steam, exiting.");
    try!(buff_writer.flush()); //.expect("Could not write to TCP steam, exiting.");
    Ok(())
}

fn handle_client(stream: TcpStream, tx: mpsc::SyncSender<ToMainThreadCommand>, rx: mpsc::Receiver<ToChildCommand>) {
    println!("New incoming TCP stream");

    let response_stream = stream.try_clone().unwrap();
    let mut buff_reader = BufReader::new(stream);
    let mut buff_writer = BufWriter::new(response_stream);
    let tick = timer_periodic(TICK_DURATION_MS);
    let (tx_client_msg, rx_client_msg) = mpsc::channel();

    // launch thread receiving messages from child TCP endpoint
    thread::spawn(move || {
        loop {
            let request = deserialize_message(&mut buff_reader);
            let is_error = request.is_err();
            tx_client_msg.send(request).unwrap();
            if is_error {
                return;
            }
        }
    });

    loop {
        // Do we have any deserialized message from this child?
        if let Ok(request) = rx_client_msg.try_recv() {
            match request {
                Ok(request_msg) => {
                    // Send parsed Message to main thread
                    tx.send(ToMainThreadCommand::Message(request_msg)).expect("Main thread died, exiting.");
                },
                Err(ref e) => {
                    shutdown_player(&tx, &rx);
                    if e.kind() == std::io::ErrorKind::UnexpectedEof {
                        println!("Client terminated connection");
                    } else {
                        println!("Got error: {:?}", e);
                        respond(Message::InvalidRequestResponse, &mut buff_writer);
                    }
                    return;
                },
            }
        }

        // Check for response from main thread
        if let Ok(response) = rx.try_recv() {
            match response {
                ToChildCommand::TerminateConnection => return,
                ToChildCommand::Message(Message::InvalidRequestResponse) => {
                    shutdown_player(&tx, &rx);
                    respond(Message::InvalidRequestResponse, &mut buff_writer);
                    return;
                },
                ToChildCommand::Message(response_msg) => {
                    if respond(response_msg, &mut buff_writer).is_err() {
                        return;
                    }
                },
            }
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
    let mut ip = Ipv4Addr::new(0,0,0,0);

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
    start_udp_discovery(port);

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
                        println!("#{} ({}): {}", i, player_handle.nickname.clone().unwrap_or("".to_owned()), Green.paint(format!("{:?}", msg)));
                        // Handle Message received from child
                        let result = handle_main(msg, player_handle, &mut &mut lobby, &mut games);
                        if let Some(response) = result.response {
                            // handle_main generated a response -> send response Message back to child
                            println!("#{} ({}): {}", i, player_handle.nickname.clone().unwrap_or("".to_owned()), Cyan.paint(format!("{:?}", response)));
                            player_handle.to_child_endpoint.send(ToChildCommand::Message(response)).unwrap();
                        }
                        if result.terminate_connection {
                            println!("-- Closing connection to child {}", i);
                            player_handle.to_child_endpoint.send(ToChildCommand::TerminateConnection).unwrap();
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
                        player_handle.to_child_endpoint.send(ToChildCommand::TerminateConnection).unwrap();
                    }
                }
            }
        }
        // Send all updates from result
        send_updates(&mut player_handles, &mut message_store);

        // Send updates issued by handle_afk() for all games with exceeded turn times
        let mut afk_games: Vec<Rc<RefCell<Game>>> = vec![];
        for game in &games {
            if (*game).borrow_mut().turn_time_exceeded() {
                afk_games.push(game.clone());
            }
        }
        for game in &afk_games {
            let mut result = serverstate::handle_afk(game.clone(), &mut lobby, &mut games);
            send_updates(&mut player_handles, &mut result);
        }

        tick.recv().expect("Timer thread died unexpectedly."); // wait for next tick
    }
}

fn send_updates(player_handles: &mut Vec<board::PlayerHandle>, message_store: &mut HashMap<String, Vec<Message>>) {
    for (i, player_handle) in player_handles.iter_mut().enumerate() {
        if let Some(ref name) = player_handle.nickname {
            if message_store.contains_key(name) {
                for message in message_store.remove(name).unwrap() {
                    println!("#{} ({}): {}", i, name, Yellow.paint(format!("{:?}", message)));
                    player_handle.to_child_endpoint.send(ToChildCommand::Message(message));
                }
            }
        }
    }
    message_store.clear();
}
