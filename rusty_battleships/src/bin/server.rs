

use std::env;
use std::net::TcpListener;
use std::io::Read;
use std::io::BufReader;
use std::str;
use std::option::Option::None;

use std::net::TcpStream;

mod Message;
/* tcpfun <PORT/IP:PORT>
 * In SERVER mode, the target port for the TCP socket is required.
 */

fn main() {
    let args: Vec<_> = env::args().collect(); // args[0] is the name of the program.
    let mut port:u16 = 5000;
    let ip = "127.0.0.1";

    if args.len() == 2 {
        port = args[1].parse::<u16>().unwrap();
    } else {
        println!("Computer says no.");
    }
    println!("Operating as server on port {}.", port);

    let listener = TcpListener::bind((ip, port)).unwrap();
    println!("Started listening on port {}.", port);
    for stream in listener.incoming() {
        let tcpstream = stream.unwrap();
        tcpstream.set_read_timeout(None);
        let mut buff_reader = BufReader::new(tcpstream);
        let (request, response, update) = Message::deserialize_message(&mut buff_reader);
        if let Some(x) = request { println!("Request: {:?}", x); }
        if let Some(x) = response { println!("Response: {:?}", x); }
        if let Some(x) = update { println!("Update: {:?}", x); }
    }
}
