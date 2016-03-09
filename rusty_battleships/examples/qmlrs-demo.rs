use std::net::{Ipv4Addr, TcpStream, UdpSocket, SocketAddr, SocketAddrV4};

extern crate byteorder;
use byteorder::{ByteOrder, BigEndian, ReadBytesExt};

use std::io::{BufReader, BufWriter, Write};
use std::option::Option::None;
use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::mpsc::{RecvError, TryRecvError};
use std::thread;

#[macro_use]
extern crate qmlrs;

extern crate argparse;
use argparse::{ArgumentParser, Print, Store};

extern crate rusty_battleships;
use rusty_battleships::message::{serialize_message, deserialize_message, Message};
use rusty_battleships::clientstate::{State, Status, tcp_poll};

// http://stackoverflow.com/questions/35157399/how-to-concatenate-static-strings-in-rust/35159310
macro_rules! description {
    () => ( "rusty battleships: game client" )
}
macro_rules! version {
    () => ( env!("CARGO_PKG_VERSION") )
}
macro_rules! version_string {            // Like this (with a literal instead of a String variable), everything works just fine:

    () => ( concat!(description!(), " v", version!()) )
}

static WINDOW: &'static str = include_str!("assets/main_window.qml");
static CONNECT_WINDOW: &'static str = include_str!("assets/connect_window.qml");


struct Bridge {
    sender: mpsc::Sender<Message>,
    receiver: mpsc::Receiver<(Status, Message)>,
    lobby_receiver: mpsc::Receiver<Message>,
    state: Status,
    last_rcvd_msg: Option<Message>,

    udp_discovery_receiver: mpsc::Receiver<(Ipv4Addr, u16, String)>,
    discovered_servers: HashMap<(Ipv4Addr, u16), String>,
    available_players_list: Vec<String>,
    ready_players_list: Vec<String>,
    features_list: Vec<String>,
}

impl Bridge {
    fn send_login_request(&mut self, username: String) {
        println!(">>> UI: Sending login request for {} ...", username);
        self.sender.send(Message::LoginRequest { username: username });
        // Wait for a OkResponse from the server, discard player state updates.
        let mut response_received = false;
        while !response_received {
            //Block while receiving! At some point there MUST be an OkResponse or a NameTakenResponse
            let resp = self.receiver.recv();
            if let Ok(tuple) = resp {
                match tuple.1.clone() {
                    Message::OkResponse => {
                        println!("Loggresp.unwrap()ed in.");
                        response_received = true;
                        self.state = tuple.0;
                        self.last_rcvd_msg = Some(tuple.1.clone());
                    },
                    Message::NameTakenResponse { nickname } => {
                        println!("Name taken: {:?}", nickname.clone());
                        response_received = true;
                        self.state = tuple.0;
                        self.last_rcvd_msg = Some(tuple.1.clone());
                    }
                    Message::PlayerReadyUpdate{..} | Message::PlayerJoinedUpdate{..}
                    | Message::PlayerNotReadyUpdate{..} | Message::PlayerLeftUpdate{..} => continue,
                    x => {
                        println!("Received illegal response: {:?}", x);
                        break;
                    },
                }
            } else {
                println!("UI update channel hung up!");
            }
        }

    }

    fn send_get_features_request(&self) {
        self.sender.send(Message::GetFeaturesRequest);
    }

    fn update_lobby(&mut self) {
        let mut response = Err(TryRecvError::Disconnected);
        let mut available = Vec::<String>::new();
        let mut ready = Vec::<String>::new();
        while response != Err(TryRecvError::Empty) {
            response = self.lobby_receiver.try_recv();
            if let Ok(Message::LobbyList {ref available_players, ref ready_players}) = response {
                available = available_players.clone();
                ready = ready_players.clone();
            } else if let Err(TryRecvError::Disconnected) = response {
                panic!("Lobby update list was closed. Probably because the sender thread died.");
            } /*else {
                panic!("You shall not pass Non-LobbyList messages via the lobby update channel!");
            }*/
            self.available_players_list = available.clone();
            self.ready_players_list = ready.clone();
        }
    }

    fn get_ready_players(&self) -> String {
        return String::new();
    }

    fn get_available_players(&self) -> String {
        return String::new();
    }

    fn get_features_list(&self) -> String {
        return format!("{:?}", self.features_list);
    }

    fn send_challenge(&mut self, username: String) {
        println!(">>> UI: Sending challenge request for {} ...", username);
        self.sender.send(Message::ChallengePlayerRequest { username: username });
        if let Ok(tuple) = self.receiver.try_recv() {
            self.state = tuple.0;
            self.last_rcvd_msg = Some(tuple.1);
        }
    }

    fn poll_state(&mut self) -> String {
        if let Ok(tuple) = self.receiver.try_recv() {
            self.state = tuple.0;
            self.last_rcvd_msg = Some(tuple.1);
        }
        let state_description = match self.state {
            Status::Unregistered => String::from("Noch nicht registriert"),
            Status::Available => String::from("Registriert"),
            _ => format!("{:?}", self.state),
        };
        return state_description;
    }

    fn poll_log(&mut self) -> String {
        if let Ok(tuple) = self.receiver.try_recv() {
            self.state = tuple.0;
            self.last_rcvd_msg = Some(tuple.1);
        }

        return match self.last_rcvd_msg {
            Some(ref msg) => format!("{:?}", msg),
            None => String::new(),
        }
    }

    fn get_last_message(&self) -> String {
        if let Some(ref msg) = self.last_rcvd_msg {
            return format!("{:?}", msg);
        } else {
            return String::from("Nothing to display.");
        }
    }

    fn connect(&self, hostname: String, port: i64, nickname: String) {
        println!("Connecting to {}, {}, {}", hostname, port, nickname);
    }

    fn discover_servers(&mut self) -> String {
        if let Ok((ip, port, server_name)) = self.udp_discovery_receiver.try_recv() {
            self.discovered_servers.insert((ip, port), server_name);
        }

        let mut result = String::new();
        for (&(ip, port), server_name) in &self.discovered_servers {
            result.push_str(&format!("{},{},{}\n", ip, port, server_name));
        }
        return String::from(result.to_owned().trim());
    }
}

Q_OBJECT! { Bridge:
    slot fn send_login_request(String);
    slot fn send_get_features_request();
    slot fn send_challenge(String);
    slot fn poll_state();
    slot fn update_lobby();
    slot fn poll_log();
    slot fn get_last_message();
    slot fn connect(String, i64, String);
    slot fn discover_servers();
    slot fn get_ready_players();
    slot fn get_available_players();
    slot fn get_features_list();
}

fn main() {
    // Channel pair for connecting the Bridge and ???
    let (tx_main, rcv_tcp) : (mpsc::Sender<Message>, mpsc::Receiver<Message>) = mpsc::channel();
    let (tx_message_update, rcv_main) : (mpsc::Sender<(Status, Message)>, mpsc::Receiver<(Status, Message)>) = mpsc::channel();
    /* From UI-Thread (this one) to Status-Update-Thread.
       Since every UI input corresponds to a Request, we can recycle message.rs for encoding user input. */
    let (tx_ui_update, rcv_ui_update) : (mpsc::Sender<Message>, mpsc::Receiver<Message>) = mpsc::channel();
    let (tx_lobby_update, rcv_lobby_update) : (mpsc::Sender<Message>, mpsc::Receiver<Message>) = mpsc::channel();
    let (tx_udp_discovery, rcv_udp_discovery) = mpsc::channel();

    let tcp_loop = move || {
        let mut port:u16 = 5000;
        let mut ip = Ipv4Addr::new(127, 0, 0, 1);

        {  // this block limits scope of borrows by ap.refer() method
            let mut ap = ArgumentParser::new();
            ap.set_description(description!());
            ap.refer(&mut ip).add_argument("IP", Store, "IPv4 address");
            ap.refer(&mut port).add_option(&["-p", "--port"], Store, "port the server listens on");
            ap.add_option(&["-v", "--version"], Print(version_string!().to_owned()),
            "show version number");
            ap.parse_args_or_exit();
        }

        println!("Operating as client on port {}.", port);
        println!("Connecting to {}.", ip);

        //Connect to the specified address and port.
        let mut sender;
        match TcpStream::connect((ip, port)) {
            Ok(foo) => sender = foo,
            Err(why) => {
                println!("{:?}", why);
                return;
            }
        };
        sender.set_write_timeout(None);

        let receiver = sender.try_clone().unwrap();
        let mut buff_writer = BufWriter::new(sender);
        let mut buff_reader = BufReader::new(receiver);

        /* Holds the current state and provides state-based services such as shoot(), move-and-shoot() as well as state- and server-message-dependant state transitions. */
        let mut current_state = State::new(true, Some(rcv_ui_update), Some(tx_message_update), Some(tx_lobby_update), buff_reader, buff_writer);

        thread::spawn(move || {
            current_state.handle_communication();
        });
        //println!("Sending FeatureRequest from UI to core.");
        //tx_ui_update.send(Message::GetFeaturesRequest);
    };

    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    let response = vec![];
    socket.send_to(&response[..], &(Ipv4Addr::new(224, 0, 0, 250), 49001 as u16));
    let udp_discovery_loop = move || {
        let mut buf = [0; 2048];
        loop {
            match socket.recv_from(&mut buf) {
                Ok((num_bytes, SocketAddr::V4(src))) => {
                    if num_bytes < 2 {
                        panic!("Received invalid response from {} to UDP discovery request", src);
                    }
                    let port = BigEndian::read_u16(&buf[0..2]);
                    let server_name = std::str::from_utf8(&buf[2..]).unwrap_or("");
                    tx_udp_discovery.send((*src.ip(), port, String::from(server_name)));
                },
                Ok((num_bytes, SocketAddr::V6(_))) => panic!("Currently not supporting Ipv6"),
                Err(e) => {
                    println!("couldn't recieve a datagram: {}", e);
                }
            }
        }
    };
    thread::spawn(udp_discovery_loop);

    let tcp_thread = thread::spawn(tcp_loop);
    let mut engine = qmlrs::Engine::new();
    let mut bridge = Bridge {
        state: Status::Unregistered,
        sender: tx_ui_update,
        receiver: rcv_main,
        last_rcvd_msg: None,
        lobby_receiver: rcv_lobby_update,
        udp_discovery_receiver: rcv_udp_discovery,
        discovered_servers: HashMap::new(),
        ready_players_list : Vec::<String>::new(),
        available_players_list : Vec::<String>::new(),
        features_list : Vec::<String>::new(),
    };
    bridge.state = Status::Unregistered;
    engine.set_property("bridge", bridge);
    // engine.load_data(WINDOW);
    engine.load_data(CONNECT_WINDOW);
    engine.exec();
}
