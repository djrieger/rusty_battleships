use std::env;
use std::net::TcpListener;
use std::io::Read;
use std::io::BufReader;
use std::str;
use std::option::Option::None;
//use std::thread::Thread;
use std::thread;
use std::sync::mpsc;
use std::thread::sleep;
use std::time::Duration;

/* tcpfun <PORT/IP:PORT>
 * In SERVER mode, the target port for the TCP socket is required.
 */

fn main() {
    let args: Vec<_> = env::args().collect(); // args[0] is the name of the program.
    let mut port:u16 = 5000;
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
	} else { //Just for Testing purposes. Will be prettyfied.
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
	}
}