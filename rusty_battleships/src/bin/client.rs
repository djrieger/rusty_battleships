use std::net::{Ipv4Addr, TcpStream};
use std::io::{BufReader, BufWriter, Write};
use std::option::Option::None;

extern crate argparse;
use argparse::{ArgumentParser, Print, Store};

extern crate rusty_battleships;
use rusty_battleships::message::{serialize_message, deserialize_message, Message};

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

fn send_message(msg: Message, stream: &mut BufWriter<TcpStream>) {
    let serialized_msg = serialize_message(msg);
    stream.write(&serialized_msg[..]).unwrap();
}

fn main() {
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
	let mut sender = TcpStream::connect((ip, port)).unwrap();
	sender.set_write_timeout(None);

    let receiver = sender.try_clone().unwrap();
    let mut buff_writer = BufWriter::new(sender);
    let mut buff_reader = BufReader::new(receiver);

    send_message(Message::GetFeaturesRequest, &mut buff_writer);
    send_message(Message::LoginRequest { username: String::from("foobar") }, &mut buff_writer);
    buff_writer.flush();
    loop {
        let server_response = deserialize_message(&mut buff_reader).unwrap();
        println!("{:?}", server_response);
    }
}
