

use std::env;
use std::process as process;
use std::net::TcpListener;
use std::net::TcpStream;
use std::io::Write;
use std::io::Read;
use std::io::BufReader;
use std::str;
use std::option::Option::None;
use std::str::FromStr;
use std::string::String;

/* tcpfun <PORT/IP:PORT>
 * In SERVER mode, the target port for the TCP socket is required.
 */

fn main() {
    let args: Vec<_> = env::args().collect(); // args[0] is the name of the program.
    let mut port:u16 = 5000;
    let mut ip = "127.0.0.1";

    if args.len() == 2 {
    	port = args[1].parse::<u16>().unwrap();
    } else {
    	println!("Computer says no.");
    }
    println!("Operating as server on port {}.", port);
	
	let listener = TcpListener::bind((ip, port)).unwrap();
	println!("Started listening on port {}.", port);
	for stream in listener.incoming() {
		let message = stream.unwrap();
		message.set_read_timeout(None);
		let mut message_buffer:[u8;3] = [0;3];
		let mut buff_reader = BufReader::new(message);
		let result = buff_reader.read_exact(&mut message_buffer);
		match result {
			Result::Ok(_) => println!("Received message: {}", str::from_utf8(&	message_buffer).unwrap()),
			Result::Err(str) => println!("ERROR!")
		}
	}
}



