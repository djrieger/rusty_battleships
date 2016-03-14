use std::io::{BufReader, BufWriter};
use std::net::{Ipv4Addr, TcpStream, UdpSocket, SocketAddr};
use std::sync::mpsc;
use std::sync::mpsc::{TryRecvError};
use std::thread;

extern crate byteorder;
use byteorder::{ByteOrder, BigEndian, ReadBytesExt};

mod client_;
use client_::state::{LobbyList, State, Status};
use client_::board::{Board};

#[macro_use]
extern crate qmlrs;

extern crate rustc_serialize;
use rustc_serialize::Encodable;
use rustc_serialize::json;
use rustc_serialize::json::{Json};

extern crate rusty_battleships;
use rusty_battleships::message::{Message, Direction, ShipPlacement};
use rusty_battleships::board::{CellState, W, H};
use rusty_battleships::timer::timer_periodic;

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

const TICK_DURATION_MS: u64 = 50;


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

    lobby_sender : mpsc::Sender<LobbyList>, //For the State object!
    lobby_receiver: mpsc::Receiver<LobbyList>,

    disconnect_sender : mpsc::Sender<bool>, // For state object
    disconnect_receiver : mpsc::Receiver<bool>,

    board_receiver: Option<mpsc::Receiver<(Board, Board)>>,
    my_board: Option<Board>,
    their_board: Option<Board>,

    state: Status,
    features_list: Vec<String>,
    last_rcvd_msg: Option<Message>,
    lobby_list: LobbyList,

    udp_discovery_receiver: mpsc::Receiver<(Ipv4Addr, u16, String)>,
    discovered_servers: Vec<Server>,
}

impl Bridge {
    fn send_login_request(&mut self, username: String) -> bool{
        println!(">>> UI: Sending login request for {} ...", username);
        self.ui_sender.as_mut().unwrap().send(Message::LoginRequest { username: username });
        // Wait for a OkResponse from the server, discard player state updates.
        let mut response_received = false;
        let mut success = false;

        while !response_received {
            //Block while receiving! At some point there MUST be an OkResponse or a NameTakenResponse
            let resp = self.msg_update_receiver.recv();
            if let Ok(tuple) = resp {
                match tuple.1.clone() {
                    Message::OkResponse => {
                        println!("Logged in.");
                        response_received = true;
                        success = true;
                        self.state = tuple.0;
                        self.last_rcvd_msg = Some(tuple.1.clone());
                    },
                    Message::NameTakenResponse { nickname } => {
                        println!("Name taken: {:?}", nickname.clone());
                        response_received = true;
                        self.state = tuple.0;
                        self.last_rcvd_msg = Some(tuple.1.clone());
                    },
                    x => {
                        println!("Received illegal response: {:?}", x);
                        break;
                    },
                }
            } else {
                println!("UI update channel hung up!");
            }
        }

        return success;
    }

    fn connection_closed(&mut self) -> bool {
        let mut result = false;

        while let Ok(disconnected) = self.disconnect_receiver.try_recv() {
            result = disconnected;
        }

        return result;
    }

    fn update_lobby(&mut self) -> String {
        while let Ok(ref lobby_list) = self.lobby_receiver.try_recv() {
            self.lobby_list = lobby_list.clone();
        }

        return json::encode(&self.lobby_list).unwrap();
    }

    fn update_boards(&mut self) {
        if let Some(ref recv) = self.board_receiver {
            while let Ok((ref my_board, ref their_board)) = recv.try_recv() {
                self.my_board = Some(my_board.clone());
                self.their_board = Some(their_board.clone());
            }
        }
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
        while let Ok(tuple) = self.msg_update_receiver.try_recv() {
            self.state = tuple.0;
            self.last_rcvd_msg = Some(tuple.1);
        }
        let state_description = match self.state {
            Status::Unregistered => String::from("Not registered"),
            Status::Available => String::from("Registered"),
            _ => format!("{:?}", self.state),
        };
        return state_description;
    }

    fn poll_log(&mut self) -> String {
        while let Ok(tuple) = self.msg_update_receiver.try_recv() {
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

    fn connect(&mut self, hostname: String, port: i64) -> bool {
        println!("Connecting to {}, {}", hostname, port);
        /* From UI-Thread (this one) to Status-Update-Thread.
           Since every UI input corresponds to a Request, we can recycle message.rs for encoding user input. */
        let (tx_ui_update, rcv_ui_update) : (mpsc::Sender<Message>, mpsc::Receiver<Message>) = mpsc::channel();
        /* From Statuis-Update-Thread to UI-Thread (this one). Transmits the current board situation. */
        let (tx_board_update, rcv_board_update) : (mpsc::Sender<(Board, Board)>, mpsc::Receiver<(Board, Board)>) = mpsc::channel();
        self.ui_sender = Some(tx_ui_update);
        self.board_receiver = Some(rcv_board_update);
        return tcp_loop(hostname, port, rcv_ui_update, self.msg_update_sender.clone(),
            self.lobby_sender.clone(), tx_board_update, self.disconnect_sender.clone());
    }

    fn discover_servers(&mut self) -> String {
        // FIXME: handle removed servers somehow
        while let Ok((ip, port, server_name)) = self.udp_discovery_receiver.try_recv() {
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

    fn on_clicked_opp_board(&mut self, x: i64, y: i64) {
        println!("Button clicked at {}, {}", x, y);
        self.ui_sender.as_mut().unwrap().send(Message::ShootRequest { x: x as u8, y: y as u8 });
    }

    fn set_ready_state(&mut self, ready: i64) {
        if ready == 1 {
            self.ui_sender.as_mut().unwrap().send(Message::ReadyRequest);
        } else {
            self.ui_sender.as_mut().unwrap().send(Message::NotReadyRequest);
        }
    }

    fn handle_placement(&mut self, placement_json: String) {
        let data = Json::from_str(&placement_json).unwrap();
        let obj = data.as_object().unwrap();
        let mut json_placements = vec![];
        for i in 0..5 {
            let placement = obj.get(&i.to_string()).unwrap().as_object().unwrap();
            json_placements.push(placement);
        }

        let get_bool = |obj: &rustc_serialize::json::Object, key| obj.get(key).unwrap().as_boolean().unwrap();
        let get_u64 = |obj: &rustc_serialize::json::Object, key| obj.get(key).unwrap().as_u64().unwrap();

        let get_length = |a: &rustc_serialize::json::Object | get_u64(a, "length");
        json_placements.sort_by(|&a, &b| get_length(a).cmp(&get_length(b)));
        json_placements.reverse();

        let mut placements = vec![];
        for placement_object in &json_placements {
            let reverse = get_bool(placement_object, "reverse");
            let horizontal = get_bool(placement_object, "horizontal");
            placements.push(ShipPlacement {
                x: get_u64(placement_object, "x") as u8,
                y: get_u64(placement_object, "y") as u8,
                direction: match (reverse, horizontal) {
                    (true, true) => Direction::West,
                    (true, false) => Direction::North,
                    (false, true) => Direction::East,
                    (false, false) => Direction::South,
                },
            });
        }
        self.ui_sender.as_mut().unwrap().send(Message::PlaceShipsRequest { placement: [
            placements[0],
            placements[1],
            placements[2],
            placements[3],
            placements[4]
        ] });
        println!("{:?}", placements);
    }

    fn get_opp_board(&mut self) -> String {
        self.update_boards();
        let mut result = String::new();
        for y in 0..H {
            for x in 0..W {
                if let Some(ref board) = self.their_board {
                    let character = match board.state[x][y] {
                        CellState { visible: false, ship_index: _ } => '"',
                        CellState { visible: true, ship_index: Some(_) } => 'X',
                        CellState { visible: true, ship_index: None } => '-',
                    };
                    result.push(character);
                } else {
                    result.push('?');
                }
            }
        }
        result
    }
}

Q_OBJECT! { Bridge:
    slot fn send_login_request(String);
    slot fn send_challenge(String);
    slot fn poll_state();
    slot fn update_lobby();
    slot fn poll_log();
    slot fn get_last_message();
    slot fn connect(String, i64);
    slot fn discover_servers();
    slot fn get_features_list();
    slot fn on_clicked_my_board(i64);
    slot fn on_clicked_opp_board(i64, i64);
    slot fn handle_placement(String);
    // slot fn get_boards();
    slot fn connection_closed();

    slot fn set_ready_state(i64);
    slot fn get_opp_board();
}

fn tcp_loop(hostname: String, port: i64, rcv_ui_update: mpsc::Receiver<Message>,
    tx_message_update: mpsc::Sender<(Status, Message)>, tx_lobby_update: mpsc::Sender<LobbyList>,
    tx_board_update: mpsc::Sender<(Board, Board)>, tx_disconnect_update: mpsc::Sender<bool>)
    -> bool {

    //Connect to the specified address and port.
    let mut sender;
    match TcpStream::connect((&hostname[..], port as u16)) {
        Ok(foo) => sender = foo,
        Err(why) => {
            println!("{:?}", why);
            return false;
        }
    };
    sender.set_write_timeout(None);

    let receiver = sender.try_clone().unwrap();
    let buff_writer = BufWriter::new(sender);
    let buff_reader = BufReader::new(receiver);

    /* Holds the current state and provides state-based services such as shoot(), move-and-shoot() as well as state- and server-message-dependant state transitions. */
    let mut current_state = State::new(rcv_ui_update, tx_message_update, tx_lobby_update,
        tx_board_update, tx_disconnect_update, buff_reader, buff_writer);

    thread::spawn(move || {
        current_state.handle_communication();
    });

    return true;
}

fn main() {
    // Channel pair for connecting the Bridge and ???
    let (tx_main, rcv_tcp) : (mpsc::Sender<Message>, mpsc::Receiver<Message>) = mpsc::channel();
    let (tx_message_update, rcv_main) = mpsc::channel();
    let (tx_disconnect_update, rcv_disconnect) = mpsc::channel();

    let (tx_lobby_update, rcv_lobby_update) = mpsc::channel();
    let (tx_udp_discovery, rcv_udp_discovery) = mpsc::channel();

    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    let response = vec![];
    socket.send_to(&response[..], &(Ipv4Addr::new(224, 0, 0, 250), 49001 as u16));
    let udp_discovery_loop = move || {
        let mut buf = [0; 2048];
        let tick = timer_periodic(TICK_DURATION_MS);
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
                    println!("Couldn't receive a datagram: {}", e);
                }
            }

            tick.recv().expect("Timer thread died unexpectedly."); // wait for next tick
        }
    };
    thread::spawn(udp_discovery_loop);

    let mut engine = qmlrs::Engine::new();
    let assets = Assets;
    // FIXME: why the hell does the bridge create the State???
    let mut bridge = Bridge {
        state: Status::Unregistered,
        my_board: None,
        their_board: None,
        ui_sender: None,
        msg_update_sender: tx_message_update, //For the State object!
        msg_update_receiver: rcv_main,
        lobby_sender : tx_lobby_update, //For the State object!
        lobby_receiver: rcv_lobby_update,
        disconnect_sender : tx_disconnect_update,
        disconnect_receiver : rcv_disconnect,
        board_receiver : None,
        last_rcvd_msg: None,
        udp_discovery_receiver: rcv_udp_discovery,
        discovered_servers: Vec::<Server>::new(),
        lobby_list: LobbyList::new(),
        features_list : Vec::<String>::new(),
    };
    bridge.state = Status::Unregistered;
    engine.set_property("assets", assets);
    engine.set_property("bridge", bridge);
    engine.load_data(MAIN_WINDOW);
    engine.exec();
}
