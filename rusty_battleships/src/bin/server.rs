use std::collections::HashMap;
use std::cell::RefCell;
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
use rusty_battleships::board::Player;
use rusty_battleships::board::PlayerState;

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

fn handle_client(stream : TcpStream, tx : mpsc::SyncSender<Message>, rx : mpsc::Receiver<Message>) {
    println!("New incoming TCP stream");

    let response_stream = stream.try_clone().unwrap();
    let mut buff_reader = BufReader::new(stream);
    let mut buff_writer = BufWriter::new(response_stream);
    loop {
        let msg = deserialize_message(&mut buff_reader);
        let response_msg;
        match msg {
            Ok(msg) => {
                // Send parsed Message to main thread
                tx.send(msg);
                // Wait for response Message
                response_msg = rx.recv().unwrap();
            },
            Err(e) => {
                response_msg = Message::InvalidRequestResponse;
                println!("Received invalid request: {}", e);
            }
        }
        // Serialize, send response and flush
        let serialized_msg = serialize_message(response_msg);
        buff_writer.write(&serialized_msg[..]).unwrap();
        buff_writer.flush();
    }
}

fn handle_main(msg: Message, player: &board::PlayerHandle, players: &Vec<board::PlayerHandle>, lobby: &mut HashMap<String, board::Player>) -> Option<Message> {
    match msg {
        Message::GetFeaturesRequest => {
            return Some(Message::FeaturesResponse {
                features: vec!["Awesomeness".to_owned()]
            });
        },
        Message::LoginRequest { username } => {
            if username.len() == 0 {
                return Some(Message::InvalidRequestResponse);
            }
            // Determine if we already have a player with name 'username'
            if lobby.contains_key(&username) {
                return Some(Message::NameTakenResponse { nickname: username });
            } else {
                // Update lobby hashtable
                lobby.insert(username.clone(), board::Player {
                    state: board::PlayerState::Available,
                    game: None,
                });
                // Update player struct
                *player.nickname.borrow_mut() = Some(username);
                return Some(Message::OkResponse);
            }
        },
        Message::ReadyRequest => {
            // TODO: Save ready state for this player
            lobby.get(&(player.nickname)).unwrap().state = PlayerState::Ready;
            return Some(Message::OkResponse);
        },
        Message::NotReadyRequest => {
            // TODO: Check if client is part of a Game and if Game is running
            // return Some(Message::GameAlreadyStartedResponse);
            let p:&Player = lobby.get(&(player.nickname)).unwrap();
            match p.game {
                Some(_) => return Some(Message::OkResponse),
                None    => return Some(Message::GameAlreadyStartedResponse)
            }
        },
        Message::ChallengePlayerRequest { username } => {
            // TODO: check if other player exists and is ready
            // TODO: return one of OK, NOT_WAITING, NO_SUCH_PLAYER
            return Some(Message::OkResponse);
        },
        _ => { return None; }
    }
}

fn main() {
    let mut map = HashMap::new();
    let player_bob = board::Player {
        state: board::PlayerState::Available,
        game: None,
    };
    let player_alice = board::Player {
        state: board::PlayerState::Ready,
        game: None,
    };
    let name_bob = "Bob";
    let name_alice = "Alice";
    map.insert(name_bob, player_bob);
    map.insert(name_alice, player_alice);

    let first_ship = board::Ship { x: 1, y: 2, length: 4, horizontal: true, health_points: 4 };
    let second_ship = board::Ship { x: 5, y: 2, length: 2, horizontal: false, health_points: 2 };
    let first_board = board::Board::new(5, 8, [ first_ship, second_ship ]);
    let second_board = first_board.clone();

    let game = board::Game {
        players: (String::from(name_alice), String::from(name_bob)),
        boards: (first_board, second_board),
    };

    // let alice_entry = map.entry(name_alice);
    // let alice = board::RegisteredPlayer {
    //     nickname: name_alice,
    //     map_entry: alice_entry,
    // };
    // next: Game, Boards








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

    let listener = TcpListener::bind((ip, port)).unwrap();
    let address = listener.local_addr().unwrap();
    println!("Started listening on port {} at address {}.", port, address);
    let mut players = Vec::new();
    let mut lobby = HashMap::new();

    // channel for letting tcp_loop tell main loop about new players
    let (tx_tcp_players, rx_main_players) : (mpsc::Sender<board::PlayerHandle>, mpsc::Receiver<board::PlayerHandle>) = mpsc::channel();

    let tcp_loop = move || {
        for stream in listener.incoming() {
            // channel child --> main
            let (tx_child, rx_main) = mpsc::sync_channel(0);
            // channel main --> child
            let (tx_main, rx_child) = mpsc::channel();

            let child = thread::spawn(move || {
                handle_client(stream.unwrap(), tx_child, rx_child);
            });
            tx_tcp_players.send(
                board::PlayerHandle {
                    nickname: RefCell::new(None),
                    from_child_endpoint: rx_main,
                    to_child_endpoint: tx_main,
                }
                );
        }
    };

    let tcp_thread = thread::spawn(tcp_loop);

    // Main loop
    loop {
        // Receive new players from tcp_loop
        if let Ok(player) = rx_main_players.try_recv() {
            players.push(player);
        }
        // Receive Messages from child threads
        for (i, mut player) in players.iter().enumerate() {
            if let Ok(msg) = player.from_child_endpoint.try_recv() {
                print!("[Child {}] {:?}", i, msg);
                // Handle Message received from child
                let opt_response = handle_main(msg, &player, &players, &mut lobby);
                if let Some(response) = opt_response {
                    // handle_main generated a response -> send response Message back to child
                    println!(" -> {:?}", response);
                    player.to_child_endpoint.send(response);
                }
            }
        }
    }
    // tcp_thread.join();
}
