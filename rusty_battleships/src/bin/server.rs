use std::collections::{HashMap, HashSet};
use std::io::{BufReader, BufWriter, Write};
use std::net::{Ipv4Addr, TcpListener, TcpStream};
use std::option::Option::None;
use std::sync::mpsc;
use std::thread;

extern crate argparse;
use argparse::{ArgumentParser, Print, Store};

extern crate rusty_battleships;
use rusty_battleships::message::{serialize_message, deserialize_message, Message, Reason};
use rusty_battleships::board;
use rusty_battleships::board::{Game, Player, PlayerState};
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

fn handle_client(stream: TcpStream, tx: mpsc::SyncSender<Message>, rx: mpsc::Receiver<Message>) {
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
                tx.send(msg).expect("Main thread died, exiting.");
                // Wait for response Message
                response_msg = rx.recv().expect("Main thread died before answering, exiting.");
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

fn handle_main(msg: Message, player: &mut board::PlayerHandle, player_names: &mut HashSet<String>, lobby: &mut HashMap<String, board::Player>, games: &mut Vec<Game>) -> (Option<Message>, Option<(String, Message)>) {
    match msg {
        Message::GetFeaturesRequest => return (serverstate::handle_get_features_request(), None),
        Message::LoginRequest { username } => return (serverstate::handle_login_request(username, player, player_names, lobby), None), 
        Message::ReadyRequest => return (serverstate::handle_ready_request(player, player_names, lobby), None),
        Message::NotReadyRequest => return (serverstate::handle_not_ready_request(player, player_names, lobby), None),
        Message::ChallengePlayerRequest { username } => return serverstate::handle_challenge_player_request(username, player, player_names, lobby, games),  
        Message::SurrenderRequest => return (serverstate::handle_surrender_request(player, player_names, lobby), None),
        Message::ReportErrorRequest { errormessage } => return (serverstate::handle_report_error_request(errormessage, player, player_names, lobby), None),  
        Message::PlaceShipsRequest { placement } => {
            // TODO: Fill me with life!
            /* TODO: Return OkResponse after saving the placement.
             * The RFC tells us to assume a correct placement. Nevertheless - for testing purposes - we should check it and return an INVALID_REQUEST.
             */
            return (Some(Message::InvalidRequestResponse), None);
        },
        Message::ShootRequest { x, y } => {
            // TODO: Fill me with life!
            // TODO: Return either one of HIT, MISS, DESTROYED, NOT_YOUR_TURN.
            return (Some(Message::InvalidRequestResponse), None);
        },
        Message::MoveAndShootRequest { id, direction, x, y } => {
            // TODO: Fill me with life!
            // TODO: HIT, MISS, DESTROYED, NOT_YOUR_TURN
            return (Some(Message::InvalidRequestResponse), None);
        },
        _ => { return (None, None); },
    };
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

            let child = thread::spawn(move || {
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

    let tcp_thread = thread::spawn(tcp_loop);
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
                let (opt_response, opt_update) = handle_main(msg, player_handle, &mut player_names, &mut lobby, &mut games);
                if let Some(response) = opt_response {
                    // handle_main generated a response -> send response Message back to child
                    println!(" -> {:?}", response);
                    player_handle.to_child_endpoint.send(response);
                }
                if let Some(update) = opt_update {
                    // handle_main generated an additional message (always update?) to be sent to
                    // another player, store in message_store
                    let (recipient_name, message) = update;
                    if message_store.contains_key(&recipient_name) {
                        let mut messages = message_store.get_mut(&recipient_name).unwrap();
                        messages.push(message);
                    } else {
                        message_store.insert(recipient_name, vec![message]);
                    }
                }
            }
        }
        // Send all messages saved in message_store
        for (i, player_handle) in player_handles.iter_mut().enumerate() {
            if let Some(ref name) = player_handle.nickname {
                if message_store.contains_key(name) {
                    for message in message_store.remove(name).unwrap() {
                        player_handle.to_child_endpoint.send(message);
                    }
                }
            }
        }
    }
    // tcp_thread.join();
}
