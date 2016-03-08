use std::net::{Ipv4Addr, TcpStream};
use std::net::UdpSocket;

extern crate byteorder;
use byteorder::{BigEndian, ReadBytesExt};

use std::io::{BufReader, BufWriter, Write};
use std::option::Option::None;
use std::sync::mpsc;
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

struct Bridge {
    sender: mpsc::Sender<Message>,
    receiver: mpsc::Receiver<(Status, Message)>,
    state: Status,
    last_rcvd_msg: Option<Message>,
}

impl Bridge {
    fn send_login_request(&self, username: String) {
        println!("Sending login request for {} ...", username);
        self.sender.send(Message::LoginRequest { username: username });
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

    fn connect(&self, hostname: String) {
    }

    fn discover_servers() {
        let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
        let response = vec![];
        socket.send_to(&response[..], &(Ipv4Addr::new(224, 0, 0, 250), 49001 as u16));
        let udp_discovery_loop = move || {
            let mut buf = [0; 2048];
            loop {
                match socket.recv_from(&mut buf) {
                    Ok((num_bytes, src)) => {
                        if num_bytes < 2 {
                            panic!("Received invalid response from {} to UDP discovery request", src);
                        }
                        // println!("num_bytes: {}", num_bytes);
                        // println!("src: {}", src);
                        // Wozu Port?
                        // let port = BigEndian::read_u16(&buf[0..1]);
                        let server_name = std::str::from_utf8(&buf[2..]).unwrap_or("");
                        println!("Found server '{}' listening on {}", server_name, src);
                    },
                    Err(e) => {
                        println!("couldn't recieve a datagram: {}", e);
                    }
                }
            }
        };
        thread::spawn(udp_discovery_loop);
    }
}

Q_OBJECT! { Bridge:
    slot fn send_login_request(String);
    slot fn poll_state();
    slot fn poll_log();
    slot fn connect(String);
}

fn main() {
    // Channel pair for connecting the Bridge and ???
    let (tx_main, rcv_tcp) : (mpsc::Sender<Message>, mpsc::Receiver<Message>) = mpsc::channel();
    let (tx_tcp, rcv_main) : (mpsc::Sender<(Status, Message)>, mpsc::Receiver<(Status, Message)>) = mpsc::channel();
    /* From UI-Thread (this one) to Status-Update-Thread.
    Since every UI input corresponds to a Request, we can recycle message.rs for encoding user input. */
    let (tx_ui_update, rcv_ui_update) : (mpsc::Sender<Message>, mpsc::Receiver<Message>) = mpsc::channel();

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
        let mut current_state = State::new(true, Some(rcv_ui_update), buff_reader, buff_writer);

        // This thread blocks while waiting for messages from the server.
        // Any received messages are sent to the main thread via tx_tcp
        // let subloop = move || {
        //     loop {
        //         // Parse server response and send to main thread
        //         let server_response = deserialize_message(&mut buff_reader).unwrap();
        //         println!("Got {:?} from server", server_response);
        //         let cloned_response = server_response.clone();
        //         match server_response {
        //             Message::OkResponse => current_state = State::Registered,
        //             Message::NameTakenResponse { nickname } => current_state = State::NameTaken,
        //             _ => {},
        //         }
        //         tx_tcp.send((current_state, cloned_response));
        //     }tx_tcp
        // };

        // Spawn a TCP-Polling-Thread and a Status-Update-Thread, connected via a "Channel<Message>"


        // Request features from server
        // let serialized_msg = serialize_message(Message::GetFeaturesRequest);
        // buff_writer.write(&serialized_msg[..]).unwrap();
        // buff_writer.flush();

        loop {
            // See if main thread has any messages to be sent to the server
            // FIXME kann hier blocken bzw kein try, sync_channel...?
            // if let Ok(msg) = rcv_tcp.try_recv() {
            //     let serialized_msg = serialize_message(msg);
            //     buff_writer.write(&serialized_msg[..]).unwrap();
            //     buff_writer.flush();
            // }

        }
    };

    Bridge::discover_servers();

    let tcp_thread = thread::spawn(tcp_loop);
    let mut engine = qmlrs::Engine::new();
    let mut bridge = Bridge {
        state: Status::Unregistered,
        sender: tx_main,
        receiver: rcv_main,
        last_rcvd_msg: None,
    };
    bridge.state = Status::Unregistered;
    engine.set_property("bridge", bridge);
    engine.load_data(WINDOW);
    engine.exec();
}
