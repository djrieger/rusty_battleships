use std::collections::{BTreeMap, HashMap};
use std::io::{BufReader, BufWriter, Write};
use std::net::{Ipv4Addr, TcpStream, UdpSocket, SocketAddr, SocketAddrV4};
use std::sync::mpsc;
use std::sync::mpsc::{RecvError, TryRecvError};
use std::thread;

extern crate argparse;
use argparse::{ArgumentParser, Print, Store};

extern crate byteorder;
use byteorder::{ByteOrder, BigEndian, ReadBytesExt};

#[macro_use]
extern crate qmlrs;

extern crate rustc_serialize;
use rustc_serialize::Encodable;
use rustc_serialize::json;
use rustc_serialize::json::{ToJson, Json};

extern crate rusty_battleships;
use rusty_battleships::message::{serialize_message, deserialize_message, Message, Direction};
use rusty_battleships::clientstate::{State, Status, tcp_poll};
use rusty_battleships::clientboard::{Board};
use rusty_battleships::board::{W, H};
use rusty_battleships::ship::Ship;

// http://stackoverflow.com/questions/35157399/how-to-concatenate-static-strings-in-rust/35159310
macro_rules! description {
    () => ( "rusty battleships: game client" )
}
macro_rules! version {
    () => ( env!("CARGO_PKG_VERSION") )
}
macro_rules! version_string {
    () => ( concat!(description!(), " v", version!()) )
}


static CONNECT_SCREEN: &'static str = include_str!("assets/connect_screen.qml");
static GAME_SCREEN: &'static str = include_str!("assets/game_screen.qml");
static LOBBY_SCREEN: &'static str = include_str!("assets/lobby_screen.qml");
static LOGIN_SCREEN: &'static str = include_str!("assets/login_screen.qml");
static MAIN_WINDOW: &'static str = include_str!("assets/main.qml");


struct Assets;

impl Assets {
    fn get_connect_screen(&self) -> String {
        CONNECT_SCREEN.to_owned()
    }

    fn get_game_screen(&self) -> String {
        GAME_SCREEN.to_owned()
    }

    fn get_lobby_screen(&self) -> String {
        LOBBY_SCREEN.to_owned()
    }

    fn get_login_screen(&self) -> String {
        LOGIN_SCREEN.to_owned()
    }
}

Q_OBJECT! { Assets:
    slot fn get_connect_screen();
    slot fn get_game_screen();
    slot fn get_lobby_screen();
    slot fn get_login_screen();
}


#[derive(RustcEncodable)]
struct Server {
    ip: [u8; 4],
    port: u16,
    name: String
}


struct Bridge {
    ui_sender: Option<mpsc::Sender<Message>>,

    msg_update_sender: mpsc::Sender<(Status, Message)>, //For the State object!
    msg_update_receiver:mpsc::Receiver<(Status, Message)>,

    lobby_sender : mpsc::Sender<Message>, //For the State object!
    lobby_receiver: mpsc::Receiver<Message>,

    board_receiver: Option<mpsc::Receiver<Board>>,

    state: Status,
    my_board: Option<Board>,
    their_board: Option<Board>,
    features_list: Vec<String>,
    last_rcvd_msg: Option<Message>,
    ready_players_list: Vec<String>,
    available_players_list: Vec<String>,

    udp_discovery_receiver: mpsc::Receiver<(Ipv4Addr, u16, String)>,
    discovered_servers: Vec<Server>,
}

impl Bridge {
    fn send_login_request(&mut self, username: String) {
        println!(">>> UI: Sending login request for {} ...", username);
        self.ui_sender.as_mut().unwrap().send(Message::LoginRequest { username: username });
        // Wait for a OkResponse from the server, discard player state updates.
        let mut response_received = false;
        while !response_received {
            //Block while receiving! At some point there MUST be an OkResponse or a NameTakenResponse
            let resp = self.msg_update_receiver.recv();
            if let Ok(tuple) = resp {
                match tuple.1.clone() {
                    Message::OkResponse => {
                        println!("Logged in.");
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

    fn send_get_features_request(&mut self) {
        self.ui_sender.as_mut().unwrap().send(Message::GetFeaturesRequest);
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
        return format!("{:?}", self.ready_players_list);
    }

    fn get_available_players(&self) -> String {
        return format!("{:?}", self.available_players_list);
    }

    fn get_features_list(&self) -> String {
        return format!("{:?}", self.features_list);
    }

    fn send_challenge(&mut self, username: String) {
        println!(">>> UI: Sending challenge request for {} ...", username);
        self.ui_sender.as_mut().unwrap().send(Message::ChallengePlayerRequest { username: username });
        if let Ok(tuple) = self.msg_update_receiver.try_recv() {
            self.state = tuple.0;
            self.last_rcvd_msg = Some(tuple.1);
        }
    }

    fn poll_state(&mut self) -> String {
        if let Ok(tuple) = self.msg_update_receiver.try_recv() {
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
        if let Ok(tuple) = self.msg_update_receiver.try_recv() {
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

    fn connect(&mut self, hostname: String, port: i64, nickname: String) {
        println!("Connecting to {}, {}, {}", hostname, port, nickname);
        /* From UI-Thread (this one) to Status-Update-Thread.
           Since every UI input corresponds to a Request, we can recycle message.rs for encoding user input. */
        let (tx_ui_update, rcv_ui_update) : (mpsc::Sender<Message>, mpsc::Receiver<Message>) = mpsc::channel();
        let (tx_board_update, rcv_board_update) : (mpsc::Sender<Board>, mpsc::Receiver<Board>) = mpsc::channel();
        self.ui_sender = Some(tx_ui_update);
        tcp_loop(hostname, port, rcv_ui_update, self.msg_update_sender.clone(), self.lobby_sender.clone());
        self.send_login_request(nickname);
    }

    fn discover_servers(&mut self) -> String {
        if let Ok((ip, port, server_name)) = self.udp_discovery_receiver.try_recv() {
            self.discovered_servers.push(Server { ip: ip.octets(), port: port, name: server_name });
        }

        return json::encode(&self.discovered_servers).unwrap();
    }

    fn get_coords_from_button_index(button_index: i64) -> (u8, u8) {
        ((button_index % 10) as u8, (button_index / 10) as u8)
    }

    fn on_clicked_my_board(&mut self, button_index: i64) {
        let (x, y) = Bridge::get_coords_from_button_index(button_index);
        println!("Button clicked at {}, {}", x, y);
    }

    fn on_clicked_opp_board(&mut self, button_index: i64) {
        let (x, y) = Bridge::get_coords_from_button_index(button_index);
        println!("Button clicked at {}, {}", x, y);
    }

    fn get_boards(&self) -> String {
        let mut ships = Vec::<Ship>::new();
        ships.push(Ship { x: 0, y: 0, length: 2, direction: Direction::East, health_points: 2});
        ships.push(Ship { x: 0, y: 1, length: 2, direction: Direction::East, health_points: 2});
        ships.push(Ship { x: 0, y: 2, length: 3, direction: Direction::East, health_points: 3});
        ships.push(Ship { x: 0, y: 3, length: 4, direction: Direction::East, health_points: 4});
        ships.push(Ship { x: 0, y: 4, length: 5, direction: Direction::East, health_points: 5});
        let mut board = Board::new(ships, true);
        board.compute_state();

        let mut result = String::new();
        {
            for y in 0..H {
                for x in 0..W {
                    match board.state[x][y].ship_index {
                        Some(index) => result.push_str(&index.to_string()),
                        None => result.push_str(" ")
                    }
                    if x == W-1 {
                        result.push_str("|\n");
                    } else {
                        result.push_str("|");
                    }
                }
            }
            println!("{}", result);
        }

        board.move_ship(0, Direction::East);
        {
            result = String::new();
            for y in 0..H {
                for x in 0..W {
                    match board.state[x][y].ship_index {
                        Some(index) => result.push_str(&index.to_string()),
                        None => result.push_str(" ")
                    }
                    if x == W-1 {
                        result.push_str("|\n");
                    } else {
                        result.push_str("|");
                    }
                }
            }
            println!("{}", result);
        }


        return result;
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
    slot fn on_clicked_my_board(i64);
    slot fn on_clicked_opp_board(i64);
    slot fn get_boards();
}

fn tcp_loop(hostname: String, port: i64, rcv_ui_update: mpsc::Receiver<Message>,
    tx_message_update: mpsc::Sender<(Status, Message)>, tx_lobby_update: mpsc::Sender<Message>) {

    //Connect to the specified address and port.
    let mut sender;
    match TcpStream::connect((&hostname[..], port as u16)) {
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
}

fn main() {
    // Channel pair for connecting the Bridge and ???
    let (tx_main, rcv_tcp) : (mpsc::Sender<Message>, mpsc::Receiver<Message>) = mpsc::channel();
    let (tx_message_update, rcv_main) : (mpsc::Sender<(Status, Message)>, mpsc::Receiver<(Status, Message)>) = mpsc::channel();

    let (tx_lobby_update, rcv_lobby_update) : (mpsc::Sender<Message>, mpsc::Receiver<Message>) = mpsc::channel();
    let (tx_udp_discovery, rcv_udp_discovery) = mpsc::channel();

    let tcp_loop = move || { //-> fn with params

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
    let assets = Assets;
    let mut bridge = Bridge {
        state: Status::Unregistered,
        my_board: None,
        their_board: None,
        ui_sender: None,
        msg_update_sender: tx_message_update, //For the State object!
        msg_update_receiver: rcv_main,
        lobby_sender : tx_lobby_update, //For the State object!
        lobby_receiver: rcv_lobby_update,
        board_receiver: None,
        last_rcvd_msg: None,
        udp_discovery_receiver: rcv_udp_discovery,
        discovered_servers: Vec::<Server>::new(),
        ready_players_list : Vec::<String>::new(),
        available_players_list : Vec::<String>::new(),
        features_list : Vec::<String>::new(),
    };
    bridge.state = Status::Unregistered;
    engine.set_property("assets", assets);
    engine.set_property("bridge", bridge);
    engine.load_data(MAIN_WINDOW);
    engine.exec();
}
