

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

fn extract_bool(mut reader: &mut BufReader<TcpStream>) -> bool {
    let intval = extract_number(&mut reader);
    match intval {
        1 => return true,
        0 => return false,
        _ => panic!("invalid bool")
    }
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

fn extract_features(mut reader: &mut BufReader<TcpStream>) -> Message::Response {
    let numfeatures = extract_number(&mut reader);
    let mut features = Vec::new();
    for i in 0..numfeatures - 1 {
        features.push(extract_string(&mut reader));
    }
    return Message::Response::Features {
        numfeatures: numfeatures,
        features: features
    };
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
        let mut request: Option<Message::Request> = None;
        let mut response: Option<Message::Response> = None;
        let mut update:   Option<Message::Update> = None;
        match opcode {
            000 => request = Some(Message::Request::GetFeatures),
            001 => request = Some(Message::Request::Login {
                username: extract_string(&mut buff_reader)
            }),
            002 => request = Some(Message::Request::Ready),
            003 => request = Some(Message::Request::NotReady),
            004 => request = Some(Message::Request::ChallengePlayer {
                username: extract_string(&mut buff_reader)
            }),
            010 => request = Some(Message::Request::PlaceShips {
                placement: extract_placement(&mut buff_reader)
            }),
            11 => request = Some(Message::Request::Shoot {
                x: extract_number(&mut buff_reader),
                y: extract_number(&mut buff_reader)
            }),
            012 => request = Some(Message::Request::MoveAndShoot {
                id: extract_number(&mut buff_reader),
                direction: Message::get_direction(extract_number(&mut buff_reader)),
                x: extract_number(&mut buff_reader),
                y: extract_number(&mut buff_reader)
            }),
            013 => request = Some(Message::Request::Surrender),
            099 => request = Some(Message::Request::ReportError {
                errormessage: extract_string(&mut buff_reader) 
            }), 


            100 => response = Some(Message::Response::Ok),
            101 => response = Some(extract_features(&mut buff_reader)), 
            102 => response = Some(Message::Response::NameTaken {
                nickname: extract_string(&mut buff_reader)
            }),
            103 => response = Some(Message::Response::NoSuchPlayer {
                nickname: extract_string(&mut buff_reader)
            }),
            104 => response = Some(Message::Response::NotWaiting {
                nickname: extract_string(&mut buff_reader)
            }),
            105 => response = Some(Message::Response::GameAlreadyStarted),
            110 => response = Some(Message::Response::IllegalPlacement),
            111 => response = Some(Message::Response::Hit {
                x: extract_number(&mut buff_reader),
                y: extract_number(&mut buff_reader)
            }),
            112 => response = Some(Message::Response::Miss {
                x: extract_number(&mut buff_reader),
                y: extract_number(&mut buff_reader)
            }),
            113 => response = Some(Message::Response::Destroyed {
                x: extract_number(&mut buff_reader),
                y: extract_number(&mut buff_reader)
            }),
            199 => response = Some(Message::Response::InvalidRequest),


            200 => update = Some(Message::Update::PlayerJoined {
                nickname: extract_string(&mut buff_reader)
            }),
            201 => update = Some(Message::Update::PlayerLeft {
                nickname: extract_string(&mut buff_reader)
            }),
            202 => update = Some(Message::Update::PlayerReady {
                nickname: extract_string(&mut buff_reader)
            }),
            203 => update = Some(Message::Update::PlayerNotReady {
                nickname: extract_string(&mut buff_reader)
            }),
            204 => update = Some(Message::Update::GameStart {
                nickname: extract_string(&mut buff_reader)
            }),
            210 => update = Some(Message::Update::YourTurn),
            211 => update = Some(Message::Update::EnemyTurn),
            212 => update = Some(Message::Update::EnemyVisible {
                x: extract_number(&mut buff_reader),
                y: extract_number(&mut buff_reader)
            }),
            213 => update = Some(Message::Update::EnemyInvisible {
                x: extract_number(&mut buff_reader),
                y: extract_number(&mut buff_reader)
            }),
            214 => update = Some(Message::Update::EnemyHit {
                x: extract_number(&mut buff_reader),
                y: extract_number(&mut buff_reader)
            }),
            215 => update = Some(Message::Update::EnemyMiss {
                x: extract_number(&mut buff_reader),
                y: extract_number(&mut buff_reader)
            }),
            216 => update = Some(Message::Update::GameOver {
                victorious: extract_bool(&mut buff_reader),
                reason: Message::get_reason(extract_number(&mut buff_reader)),
            }),
            217 => update = Some(Message::Update::AfkWarning {
                strikes: extract_number(&mut buff_reader)
            }),
            218 => update = Some(Message::Update::EnemyAfk{
                strikes: extract_number(&mut buff_reader)
            }),

            255 => update = Some(Message::Update::ServerGoingDown),

            _   => response = None
        }
        println!("Request: {:?} ", request);
        println!("Response: {:?} ", response);
        println!("Update: {:?} ", update);
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
