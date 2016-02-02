use std::cell::RefCell;
use std::env;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Read;
use std::io::Write;
use std::net::TcpListener;
use std::net::TcpStream;
use std::option::Option::None;
use std::str;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::mpsc;
use std::thread::Thread;
use std::thread::sleep;
use std::thread;
use std::time::Duration;

extern crate rusty_battleships;
use rusty_battleships::message::{
    serialize_message,
    deserialize_message,
    Message
};

fn handle_client(mut stream : TcpStream, tx : mpsc::SyncSender<Message>, rx : mpsc::Receiver<Message>) {
    println!("New incoming TCP stream");

    let mut response_stream = stream.try_clone().unwrap();
    let mut buff_reader = BufReader::new(stream);
    let mut buff_writer = BufWriter::new(response_stream);
    loop {
        let msg = deserialize_message(&mut buff_reader);
        let mut response_msg;
        match msg {
            Some(msg) => {
                tx.send(msg);
                response_msg = rx.recv().unwrap();
            },
            None => {
                response_msg = Message::InvalidRequestResponse;
                println!("Received invalid request");
            }
        }
        let serialized_msg = serialize_message(response_msg);
        buff_writer.write(&serialized_msg[..]).unwrap();
        buff_writer.flush();
    }
}

fn main() {
    let args: Vec<_> = env::args().collect(); // args[0] is the name of the program.
    let mut port:u16 = 50000;
    let ip = "127.0.0.1"; //"0.0.0.0";

    if args.len() == 2 {
        port = args[1].parse::<u16>().unwrap();
    } else if args.len() == 3 && args[2] == "threadtest" { //Just for Testing purposes. Will be prettyfied.
        port = args[1].parse::<u16>().unwrap();
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

    // let get_player_name = | p: &Player, username: String | {
    //     return p.nickname == username;
    // };

    loop {
        // Receive new players from tcp_loop
        if let Ok(player) = rx_main_players.try_recv() {
            players.push(player);
        }
        // Receive Messages from child threads
        for (i, mut player) in players.iter().enumerate() {
            if let Ok(msg) = player.from_child_endpoint.try_recv() {
                print!("[Child {}] {:?}", i, msg);
                if let Some(response_msg) = handle_main(msg, &player, &players) {
                    println!(" -> {:?}", response_msg);
                    player.to_child_endpoint.send(response_msg);
                }
            }
            // let mut nick = player.nickname.borrow_mut();
            // *nick = Some("David".to_string());
        }
    }
    // tcp_thread.join();
}

fn handle_main(msg: Message, player: &Player, players: &Vec<Player>) -> Option<Message> {
    match msg {
        Message::GetFeaturesRequest => {
            return Some(Message::FeaturesResponse {
                numfeatures: 1,
                features: vec!["Awesomeness".to_string()]
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
            // Store player name
            let mut nick = player.nickname.borrow_mut();
            *nick = Some(username);
            return Some(Message::OkResponse);
        },
        _ => { panic!("Not implemented yet"); }
    }
    return None;
}

struct Player {
    nickname: RefCell<Option<String>>,
    from_child_endpoint: mpsc::Receiver<Message>,
    to_child_endpoint: mpsc::Sender<Message>
}
