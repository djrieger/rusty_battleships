use std::io::Write;
use std::net::{Ipv4Addr, TcpStream};
use std::option::Option::None;

extern crate argparse;
use argparse::{ArgumentParser, Print, Store};

const DESCRIPTION: &'static str = "rusty battleships: game client";
const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() {
    let mut port:u16 = 5000;
    let mut ip = Ipv4Addr::new(127, 0, 0, 1);

    {  // this block limits scope of borrows by ap.refer() method
        let mut ap = ArgumentParser::new();
        ap.set_description(DESCRIPTION);
        ap.refer(&mut ip).add_argument("IP", Store, "IPv4 address");
        ap.refer(&mut port).add_option(&["-p", "--port"], Store, "port the server listens on");
        ap.add_option(&["-v", "--version"], Print(DESCRIPTION.to_owned() + " v" + VERSION),
            "show version number");
        ap.parse_args_or_exit();
    }

    println!("Operating as client on port {}.", port);
	println!("Connecting to {}.", ip);

    //Connect to the specified address and port.
	let mut sender = TcpStream::connect((ip, port)).unwrap();
	sender.set_write_timeout(None);
	let message = "123".to_owned();
	//	let message = b"123"; //Directly into bytes!
	let message_bytes:Vec<u8> = message.into_bytes();
	sender.write(&message_bytes);

}
