

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

fn extract_string(mut reader: &mut BufReader<TcpStream>) -> String {
    let strlen = extract_number(&mut reader);

    let mut string_buffer = vec![];
    let mut chunk = reader.take(strlen as u64);
    let status = chunk.read_to_end(&mut string_buffer);
    return str::from_utf8(& string_buffer).unwrap().to_string();
}

fn extract_number(reader: &mut BufReader<TcpStream>) -> u8 {
    let mut message_buffer:[u8;1] = [0;1];
    let result = reader.read_exact(&mut message_buffer);
    return message_buffer[0];
}

fn extract_placement(mut reader: &mut BufReader<TcpStream>) -> [Message::ShipPlacement; 5] {
    let mut placement:[Message::ShipPlacement; 5] = [Message::ShipPlacement { x: 0, y: 0, direction: Message::Direction::North }; 5];
    for i in 0..4 {
        placement[i] = Message::ShipPlacement {
            x: extract_number(&mut reader),
            y: extract_number(&mut reader),
            direction: Message::get_direction(extract_number(&mut reader)),
        };
    }
    return placement;
}

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
        let mut message_buffer:[u8;1] = [0;1];
        let mut buff_reader = BufReader::new(tcpstream);
        let result = buff_reader.read_exact(&mut message_buffer);
        let opcode = message_buffer[0];
        let msg;
        match opcode {
            000 => msg = Some(Message::Request::GetFeatures),
            001 => msg = Some(Message::Request::Login {
                username: extract_string(&mut buff_reader)
            }),
            002 => msg = Some(Message::Request::Ready),
            003 => msg = Some(Message::Request::NotReady),
            004 => msg = Some(Message::Request::ChallengePlayer {
                username: extract_string(&mut buff_reader)
            }),
            010 => msg = Some(Message::Request::PlaceShips {
                placement: extract_placement(&mut buff_reader)
            }),
            11 => msg = Some(Message::Request::Shoot {
                x: extract_number(&mut buff_reader),
                y: extract_number(&mut buff_reader)
            }),
            012 => msg = Some(Message::Request::MoveAndShoot {
                id: extract_number(&mut buff_reader),
                direction: Message::get_direction(extract_number(&mut buff_reader)),
                x: extract_number(&mut buff_reader),
                y: extract_number(&mut buff_reader)
            }),
            013 => msg = Some(Message::Request::Surrender),
            014 => msg = Some(Message::Request::ReportError {
                errormessage: extract_string(&mut buff_reader) 
            }), 
            _   => msg = None
        }
        println!("Received {:?} message", msg);
        // match msg.unwrap() {
        //     Message::Request::GetFeatures => println!("FEATIU"),
        //     Message::Request::ChallengePlayer { username: nickname } => println!("{}", nickname),
        //     _ => println!("other")
        // }
        // match result {
        //     Result::Ok(_) => println!("Received message: {}", str::from_utf8(&	message_buffer).unwrap()),
        //     Result::Err(str) => println!("ERROR!")
        // }
    }
}
