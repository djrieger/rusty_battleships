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
use rusty_battleships::board::{ToMainThreadCommand, ToChildCommand};
use rusty_battleships::game::Game;
use rusty_battleships::timer::timer_periodic;

mod server_;
use server_::state;

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

const TICK_DURATION_MS: u64 = 100;

fn start_udp_discovery(tcp_port: u16) {
    let socket = match UdpSocket::bind("0.0.0.0:49001") {
        Ok(s) => s,
        Err(e) => {
            println!("Couldn't bind socket: {}", e);
            return;
        }
    };
    match socket.join_multicast_v4(&Ipv4Addr::new(224, 0, 0, 250), &Ipv4Addr::new(0, 0, 0, 0)) {
        Err(why) => {
            println!("Couldn't join multicast group: {:?}", why);
            return;
        },
        Ok(_) => {
            println!("Joined UDP multicast group 224.0.0.250, listening on port 49001 for UDP discovery");
        },
    };
    socket.set_read_timeout(None).unwrap(); // blocking reads
    
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
                    response.push(version_string!().len() as u8);
                    write!(&mut response, version_string!()).unwrap();
                    if socket.send_to(&response[..], &src).is_err() {
                        println!("Unable to respond");
                    } else {
                        println!("Responded with port and host name");
                    }
                },
                Err(e) => {
                    println!("Couldn't receive a datagram: {}", e);
                }
            }
        }
    };
    thread::spawn(udp_discovery_loop);
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
    let tick = timer_periodic(TICK_DURATION_MS);

    // launch thread receiving messages from child TCP endpoint
    thread::spawn(move || {
        loop {
            let request = deserialize_message(&mut buff_reader);
            match request {
                Ok(msg) => tx.send(ToMainThreadCommand::Message(msg)).unwrap(),
                Err(e) => {
                    tx.send(ToMainThreadCommand::Error(e)).unwrap();
                    return;
                },
            }
            tick.recv().expect("Timer thread died unexpectedly."); // wait for next tick
        }
    });

    let mut buff_writer = BufWriter::new(response_stream);
    loop {
        let response = rx.recv().unwrap();
        match response {
            ToChildCommand::TerminateConnection => return,
            ToChildCommand::Message(Message::InvalidRequestResponse) => {
                respond(Message::InvalidRequestResponse, &mut buff_writer).is_err();
                return;
            },
            ToChildCommand::Message(response_msg) => if respond(response_msg, &mut buff_writer).is_err() { return },
        }
    }
}

fn handle_main(msg: Message, player: &mut board::PlayerHandle, lobby: &mut HashMap<String, board::Player>, games: &mut Vec<Rc<RefCell<Game>>>) -> state::Result {
    // These requests can be handled without any restrictions
    match msg {
        Message::GetFeaturesRequest => return state::handle_get_features_request(),
        Message::ReportErrorRequest { errormessage } => return state::handle_report_error_request(errormessage, player, lobby, games),
        _ => {},
    }

    // Login requests on the other hand are only valid if the client is not already logged in, i.e.
    // their nickname must be None
    if player.nickname.is_none() {
        if let Message::LoginRequest { username } = msg {
            return state::handle_login_request(username, player, lobby);
        }
    } else {
        // All other requests are only valid after logging in, i.e. with a user name
        assert!(lobby.contains_key(player.nickname.as_ref().unwrap()), "Invalid state: nickname not in lobby.");

        let nickname = player.nickname.as_ref().unwrap();

        match msg {
            Message::ReadyRequest => return state::handle_ready_request(nickname, lobby),
            Message::NotReadyRequest => return state::handle_not_ready_request(nickname, lobby),
            Message::ChallengePlayerRequest { username } => return state::handle_challenge_player_request(username, nickname, lobby, games),
            Message::SurrenderRequest => return state::handle_surrender_request(nickname, lobby, games),
            Message::PlaceShipsRequest { placement } => return state::handle_place_ships_request(placement, nickname, lobby),
            Message::ShootRequest { x, y } => return state::handle_move_shoot_request((x, y), None, nickname, lobby, games),
            Message::MoveAndShootRequest { id, direction, x, y } => return state::handle_move_shoot_request((x, y), Some((id as usize, direction)), nickname, lobby, games),
            _ => {},
        };
    }
    return state::Result::respond(Message::InvalidRequestResponse, false);
}

fn log_msg(index: usize, player_handle: &board::PlayerHandle, color: ansi_term::Colour, msg: &Message) {
    println!(
        "#{} ({}): {}", 
        index, 
        player_handle.nickname.clone().unwrap_or("".to_owned()), 
        color.paint(format!("{:?}", msg)));
}

/**
 * @return.0 Whether the player was terminated
 */
fn handle_msg(i: usize, player_handle: &mut board::PlayerHandle, msg: Message, mut lobby: &mut HashMap<String, board::Player>, mut games: &mut Vec<Rc<RefCell<Game>>>) -> (bool, state::Result) {
    log_msg(i, &player_handle, Green, &msg);
    // Handle Message received from child
    let result = handle_main(msg, player_handle, &mut lobby, &mut games);
    if let Some(ref response) = result.response {
        // handle_main generated a response -> send response Message back to child
        log_msg(i, &player_handle, Cyan, &response);
        player_handle.to_child_endpoint.send(ToChildCommand::Message((*response).clone())).unwrap();
    }
    if result.terminate_connection {
        println!("-- Closing connection to child {}", i);
        player_handle.to_child_endpoint.send(ToChildCommand::TerminateConnection).unwrap();
        return (false, result);
    }
    (true, result)
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
	    let mut valid = vec![true; player_handles.len()];
        for (i, player_handle) in player_handles.iter_mut().enumerate() {
            match player_handle.from_child_endpoint.try_recv() {
                Ok(ToMainThreadCommand::Message(msg)) => {
                    let (player_terminated, result) = handle_msg(i, player_handle, msg, &mut lobby, &mut games);
                    if player_terminated {
                        valid[i] = false;
                    }
                    if !result.updates.is_empty() {
                        message_store = result.updates;
                        break;
                    }
                },
                Ok(ToMainThreadCommand::Error(ref e)) => {
                    if let Some(ref name) = player_handle.nickname {
                        message_store = state::terminate_player(name, &mut lobby, &mut games);
                    }
                    valid[i] = false;
                    match e.kind() {
                        std::io::ErrorKind::UnexpectedEof => {
                            println!("Client terminated connection");
                            player_handle.to_child_endpoint.send(ToChildCommand::TerminateConnection).unwrap();
                        },
                        _ => {
                            println!("Got error: {:?}", e);
                            // Ignoring return value of respond() since we are exiting anyway
                            player_handle.to_child_endpoint.send(ToChildCommand::Message(Message::InvalidRequestResponse)).unwrap();
                        }
                    }
                },
                _ => {},
            }
        }

	    // delete old handles
	    for (i, is_valid) in valid.iter().enumerate() {
		    if !is_valid {
			    player_handles.remove(i);
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
            let mut result = state::handle_afk(game.clone(), &mut lobby, &mut games);
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
                    player_handle.to_child_endpoint.send(ToChildCommand::Message(message)).unwrap();
                }
            }
        }
    }
    message_store.clear();
}
