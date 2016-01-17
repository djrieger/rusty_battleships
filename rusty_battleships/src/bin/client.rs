use std::env;
use std::net::TcpStream;
use std::io::Write;
use std::option::Option::None;

/* vlient <IP> <PORT>
 * In CLIENT mode, ip and port of the server is required.
 */

fn main() {
    let args: Vec<_> = env::args().collect(); // args[0] is the name of the program.
    let mut port:u16 = 5000;
    let mut ip = "127.0.0.1";

    if args.len() == 3 {
    	ip = &args[1];
    	port = args[2].parse::<u16>().unwrap();
    }
    println!("Operating as client on port {}.", port);
	println!("Connecting to {}.", ip);

    //Connect to the specified address and port.
	let mut sender = TcpStream::connect((ip, port)).unwrap();
	sender.set_write_timeout(None);
	let message = "123".to_string();
	//	let message = b"123"; //Directly into bytes!
	let message_bytes:Vec<u8> = message.into_bytes();
	sender.write(&message_bytes);

}