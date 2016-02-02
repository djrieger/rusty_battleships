use std::env;
use std::net::TcpListener;
use std::io::Read;
use std::io::Write;
use std::io::BufReader;
use std::io::BufWriter;
use std::str;
use std::option::Option::None;
use std::thread::Thread;
use std::thread;
use std::sync::mpsc;
use std::thread::sleep;
use std::time::Duration;
use std::sync::mpsc::Sender;
use std::sync::mpsc::Receiver;

use std::net::TcpStream;

extern crate rusty_battleships;
use rusty_battleships::message::{
    serialize_message,
    deserialize_message,
    Message
};

fn handle_get_features_request(tx : mpsc::SyncSender<Message>, rx : mpsc::Receiver<Message>) -> Message {
    Message::FeaturesResponse {
        numfeatures: 1,
        features: vec!["Awesomeness".to_string()]
    }
}

// fn handle_login_request(username: Message, tx : mpsc::SyncSender<Message>, rx : mpsc::Receiver<Message>) -> Message {
//     println!("Sending LoginRequest to main thread");
//     tx.send(msg);
//     return rx.recv().unwrap();
// }

fn handle_client(mut stream : TcpStream, tx : mpsc::SyncSender<Message>, rx : mpsc::Receiver<Message>) {
    println!("Received stream.");

    // need to receive client messages in a loop here?
    let mut response_stream = stream.try_clone().unwrap();
    let mut buff_reader = BufReader::new(stream);
    let mut buff_writer = BufWriter::new(response_stream);
    let msg = deserialize_message(&mut buff_reader);
    // If message received from client is valid...
    let mut response_msg;
    match msg {
        Some(Message::GetFeaturesRequest) => response_msg = handle_get_features_request(tx, rx),
        // Some(Message::LoginRequest { username }) => response_msg = handle_login_request(username, tx, rx),
        Some(msg) => {
            println!("Sending LoginRequest to main thread");
            tx.send(msg);
            response_msg = rx.recv().unwrap();
        },
        None => response_msg = Message::ReportErrorRequest { 
            errormessage: "Malformed message".to_string()
        }
    }
    let serialized_msg = serialize_message(response_msg);
    buff_writer.write(&serialized_msg[..]);
}

// map JoinHandle -> Player

fn main() {
    let args: Vec<_> = env::args().collect(); // args[0] is the name of the program.
    let mut port:u16 = 50000;
    let ip = "127.0.0.1"; //"0.0.0.0";
    let mut do_thread_testing = false; //Just for Testing purposes. Will be prettyfied.

    if args.len() == 2 {
        port = args[1].parse::<u16>().unwrap();
    } else if args.len() == 3 && args[2] == "threadtest" { //Just for Testing purposes. Will be prettyfied.
        port = args[1].parse::<u16>().unwrap();
        do_thread_testing = true;
    }
    println!("Operating as server on port {}.", port);

    if !do_thread_testing { //Just for Testing purposes. Will be prettyfied.
        let listener = TcpListener::bind((ip, port)).unwrap();
        let address = listener.local_addr().unwrap();
        println!("Started listening on port {} at address {}.", port, address);
        // let mut endpoints: Vec<(&mpsc::Sender<Message>, &mpsc::Receiver<Message>)> = Vec::new();
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
                        nickname: "".to_string(),
                        from_child_endpoint: rx_main,
                        to_child_endpoint: tx_main,
                    }
                );
            }
        };

        let tcp_thread = thread::spawn(tcp_loop);

        let get_player_name = | p: &Player, username: String | {
            return p.nickname == username;
        };

        loop {
            // Receive messages from tcp_loop
            if let Ok(player) = rx_main_players.try_recv() {
                // endpoints.push((&player.to_child_endpoint, &player.from_child_endpoint));
                players.push(player);
            }
            // Receive messages from child threads
            for player in &players {
            // for &(ref to_child_endpoint, ref from_child_endpoint) in &endpoints {
                if let Ok(msg) = player.from_child_endpoint.try_recv() {
                    handle_main(msg, /*player,*/ &players);
                }
            }
        }
        // tcp_thread.join();
    }
}

fn handle_main(msg: Message, /*sending_player: &mut Player, */ players: &Vec<Player>) {
    match msg {
        Message::LoginRequest { username }=> {
            let mut player_exists = false;
            for player in players {
                if player.nickname == username {
                    println!("Player {} already exists", username);
                    player_exists = true;
                    break;
                }
            }
            if !player_exists {
                println!("Adding player {}", username);
                // now we need to store the player's name
                // sending_player.nickname = username;
            }
        },
        _ => { panic!("Not implemented yet"); }
    }
}

struct Player {
    nickname: String,
    from_child_endpoint: mpsc::Receiver<Message>,
    to_child_endpoint: mpsc::Sender<Message>
    // thread_handle: JoinHandle
}
