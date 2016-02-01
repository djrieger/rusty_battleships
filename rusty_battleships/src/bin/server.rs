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

use std::net::TcpStream;

extern crate rusty_battleships;
use rusty_battleships::message::{
    serialize_message,
    deserialize_message,
    Message
};

/* tcpfun <PORT/IP:PORT>
 * In SERVER mode, the target port for the TCP socket is required.
 */

fn handle_get_features_request(tx : mpsc::SyncSender<Message>, rx : mpsc::Receiver<Message>) -> Message {
    Message::FeaturesResponse { 
        numfeatures: 1, 
        features: vec!["Awesomeness".to_string()] 
    }
}

fn handle_client(mut stream : TcpStream, tx : mpsc::SyncSender<Message>, rx : mpsc::Receiver<Message>) {
    println!("Received stream.");
    // stream.write(b"Test");

    let mut response_stream = stream.try_clone().unwrap();
    let mut buff_reader = BufReader::new(stream);
    let mut buff_writer = BufWriter::new(response_stream);
    let msg = deserialize_message(&mut buff_reader);
    // If message received from client is valid...
    let mut response_msg;
    match msg {
        Some(Message::GetFeaturesRequest) => response_msg = handle_get_features_request(tx, rx),
        Some(_) => response_msg = Message::ReportError { 
            errormessage: "Cannot handle opcode".to_string() 
        },
        None => response_msg = Message::ReportError { 
            errormessage: "Malformed message".to_string() 
        }
    }
    let serialized_msg = serialize_message(response_msg);
    buff_writer.write(&serialized_msg[..]);
    // println!("{:?}", x); 
    // stream.write(
    // println!("{}", String::from_utf8(serialize_message(x)).unwrap());
    // } else {
    //     let serialized_msg = serialize_message(Message::ReportError { errormessage: "Malformed message received".to_string() });
    //     buff_writer.write(&serialized_msg[..]);
    // }
}

fn main() {
    let args: Vec<_> = env::args().collect(); // args[0] is the name of the program.
    let mut port:u16 = 50000;
    let ip = "0.0.0.0";
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
        let mut children = Vec::new();
        let mut transmitters = Vec::new();

       	let (tx_child, rx_main) = mpsc::sync_channel(0);
        for stream in listener.incoming() {
        	let tx_child = tx_child.clone();
        	let (tx_main, rx_child) = mpsc::channel();
            let child = thread::spawn(move || {
            	handle_client(stream.unwrap(), tx_child, rx_child);
            });
            children.push(child);
            transmitters.push(tx_main);

			//let (transmitter, receiver) = mpsc::sync_channel(0);
            /*
            let tcpstream = stream.unwrap();
            tcpstream.set_read_timeout(None);
            let mut buff_reader = BufReader::new(tcpstream);
            let msg = deserialize_message(&mut buff_reader);
            if let Some(x) = msg {
                println!("{:?}", x); 
                println!("{}", String::from_utf8(serialize_message(x)).unwrap());
            }
            */
        }
    } 
    /*else { //Just for Testing purposes. Will be prettyfied.
        let message = "RANDOMSTUFF";
        let (transmitter, receiver) = mpsc::sync_channel(0);

        let function = move || {
            println!("Child sending {} to parent.",  message);
            sleep(Duration::new(5, 0));
            transmitter.send(message).unwrap();
            println!("Child will now terminate.");
        };
        let child = thread::spawn(function);

        let received_message = receiver.recv().unwrap();
        println!("Parent thread received {}.", received_message);
        println!("Parent will now terminate.");
    }*/
}
