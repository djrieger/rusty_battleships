use std::io::Write;
use std::net::{Ipv4Addr, TcpStream};
use std::option::Option::None;

extern crate argparse;
use argparse::{ArgumentParser, Store};

fn main() {
    let mut port:u16 = 5000;
    let mut ip = Ipv4Addr::new(127, 0, 0, 1);

    {  // this block limits scope of borrows by ap.refer() method
        let mut ap = ArgumentParser::new();
        ap.set_description("Rusty battleships: Game client.");
        ap.refer(&mut ip).add_argument("IP", Store, "IPv4 address");
        ap.refer(&mut port).add_option(&["-p", "--port"], Store, "port the server listens on");
        ap.parse_args_or_exit();
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
