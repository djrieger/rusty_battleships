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

const DESCRIPTION: &'static str = "rusty battleships: game client";
const VERSION: &'static str = env!("CARGO_PKG_VERSION");

struct Player {
    nickname: RefCell<Option<String>>,
    from_child_endpoint: mpsc::Receiver<Message>,
    to_child_endpoint: mpsc::Sender<Message>
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
            Some(msg) => {
                // Send parsed Message to main thread
                tx.send(msg);
                // Wait for response Message
                response_msg = rx.recv().unwrap();
            },
            None => {
                response_msg = Message::InvalidRequestResponse;
                println!("Received invalid request");
            }
        }
        // Serialize, send response and flush
        let serialized_msg = serialize_message(response_msg);
        buff_writer.write(&serialized_msg[..]).unwrap();
        buff_writer.flush();
    }
}

fn handle_main(msg: Message, player: &Player, players: &Vec<Player>) -> Option<Message> {
    match msg {
        Message::GetFeaturesRequest => {
            return Some(Message::FeaturesResponse {
                numfeatures: 1,
                features: vec!["Awesomeness".to_owned()]
            });
        },
        Message::LoginRequest { username }=> {
            // Determine if we already have a player with name 'username'
            for player in players {
                let nick = player.nickname.borrow();
                if let Some(ref nickname) = *nick {
                    if *nickname == username {
                        return Some(Message::NameTakenResponse {
                            nickname: username
                        });
                    }
                }
            }

            // Store new name
            // TODO: This currently does not work (setting the name has no effect):
            // *player.nickname.borrow_mut() = Some(username);
            // Like this (with a literal instead of a String variable), everything works just fine:
            *player.nickname.borrow_mut() = Some("Eva".to_owned());
            return Some(Message::OkResponse);
        },
        Message::ReadyRequest => {
            // TODO: Save ready state for this player
            return Some(Message::OkResponse);
        },
        Message::NotReadyRequest => {
            // TODO: Check if client is part of a Game and if Game is running
            // return Some(Message::GameAlreadyStartedResponse);
            return Some(Message::OkResponse);
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
    let mut port:u16 = 5000;
    let mut ip = Ipv4Addr::new(127, 0, 0, 1);

    {  // this block limits scope of borrows by ap.refer() method
        let mut ap = ArgumentParser::new();
        ap.set_description(DESCRIPTION);
        ap.refer(&mut ip).add_argument("IP", Store, "IPv4 address to listen to");
        ap.refer(&mut port).add_option(&["-p", "--port"], Store, "port to listen on");
        ap.add_option(&["-v", "--version"], Print(DESCRIPTION.to_owned() + " v" + VERSION),
            "show version number");
        ap.parse_args_or_exit();
    }

    println!("Operating as server on port {}.", port);

    let listener = TcpListener::bind((ip, port)).unwrap();
    let address = listener.local_addr().unwrap();
    println!("Started listening on port {} at address {}.", port, address);
    let mut players = Vec::new();

    // channel for letting tcp_loop tell main loop about new players
    let (tx_tcp_players, rx_main_players) : (mpsc::Sender<Player>, mpsc::Receiver<Player>) = mpsc::channel();

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
                Player {
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
                let opt_response = handle_main(msg, &player, &players);
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
