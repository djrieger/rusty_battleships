use std::io::Read;
use std::io::BufReader;
use std::str;
use std::option::Option::None;

use std::net::TcpStream;

#[derive(Debug)]
pub enum Request {
    GetFeatures,
    Login {
        username:String
    },
    Ready,
    NotReady,
    ChallengePlayer {
        username:String
    },
    PlaceShips {
        placement:[ShipPlacement; 5]
    },
    Shoot {
        x:u8,
        y:u8,
    },
    MoveAndShoot {
        id:u8,
        direction:Direction,
        x:u8,
        y:u8,
    },
    Surrender,
    ReportError {
        errormessage:String,
    },
}

#[derive(Debug)]
pub enum Response {
    Ok,
    Features {
        numfeatures:u8,
        features:Vec<String>,
    },
    NameTaken{
        nickname:String,
    },
    NoSuchPlayer{
        nickname:String,
    },
    NotWaiting{
        nickname:String,
    },
    GameAlreadyStarted,
    IllegalPlacement,
    Hit {
        x:u8,
        y:u8,
    },
    Miss {
        x:u8,
        y:u8,
    },
    Destroyed {
        x:u8,
        y:u8,
    },
    InvalidRequest,
}

#[derive(Debug)]
pub enum Update {
    PlayerJoined {
        nickname:String,
    },
    PlayerLeft {
        nickname:String,
    },
    PlayerReady {
        nickname:String,
    },
    PlayerNotReady {
        nickname:String,
    },
    GameStart {
        nickname:String, //Opponent's name
    },
    YourTurn,
    EnemyTurn,
    EnemyVisible {
        x:u8,
        y:u8,
    },
    EnemyInvisible {
        x:u8,
        y:u8,
    },
    EnemyHit {
        x:u8,
        y:u8,
    },
    EnemyMiss {
        x:u8,
        y:u8,
    },
    GameOver {
        victorious:bool,
        reason:Reason,
    },
    AfkWarning {
        strikes:u8,
    },
    EnemyAfk {
        strikes:u8,
    },
    ServerGoingDown,
}

#[derive(Copy, Clone, Debug)]
pub enum Direction {
    North,
    East,
    South,
    West,
}

pub fn get_direction(direction_index: u8) -> Direction {
    return match direction_index {
        0 => Direction::North,
        1 => Direction::East,
        2 => Direction::South,
        3 => Direction::West,
        _ => panic!("Invalid direction index")
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ShipPlacement {
    pub x:u8,
    pub y:u8,
    pub direction:Direction
}

#[derive(Debug)]
pub enum Reason {
    Obliterated,
    Disconnected,
    Surrendered,
    Afk
}

pub fn get_reason(reason_index: u8) -> Reason {
    return match reason_index {
        0 => Reason::Obliterated,
        1 => Reason::Disconnected,
        2 => Reason::Surrendered,
        3 => Reason::Afk,
        _ => panic!("Invalid index for reason")
    }
}

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

fn extract_placement(mut reader: &mut BufReader<TcpStream>) -> [ShipPlacement; 5] {
    let mut placement:[ShipPlacement; 5] = [ShipPlacement { x: 0, y: 0, direction: Direction::North }; 5];
    for i in 0..4 {
        placement[i] = ShipPlacement {
            x: extract_number(&mut reader),
            y: extract_number(&mut reader),
            direction: get_direction(extract_number(&mut reader)),
        };
    }
    return placement;
}

fn extract_features(mut reader: &mut BufReader<TcpStream>) -> Response {
    let numfeatures = extract_number(&mut reader);
    let mut features = Vec::new();
    for i in 0..numfeatures - 1 {
        features.push(extract_string(&mut reader));
    }
    return Response::Features {
        numfeatures: numfeatures,
        features: features
    };
}

pub fn deserialize_message(mut reader: &mut BufReader<TcpStream>) -> (Option<Request>, Option<Response>, Option<Update>) {
    let mut request: Option<Request> = None;
    let mut response: Option<Response> = None;
    let mut update: Option<Update> = None;
    let opcode = extract_number(&mut reader);
    match opcode {
        000 => request = Some(Request::GetFeatures),
        001 => request = Some(Request::Login {
            username: extract_string(&mut reader)
        }),
        002 => request = Some(Request::Ready),
        003 => request = Some(Request::NotReady),
        004 => request = Some(Request::ChallengePlayer {
            username: extract_string(&mut reader)
        }),
        010 => request = Some(Request::PlaceShips {
            placement: extract_placement(&mut reader)
        }),
        11 => request = Some(Request::Shoot {
            x: extract_number(&mut reader),
            y: extract_number(&mut reader)
        }),
        012 => request = Some(Request::MoveAndShoot {
            id: extract_number(&mut reader),
            direction: get_direction(extract_number(&mut reader)),
            x: extract_number(&mut reader),
            y: extract_number(&mut reader)
        }),
        013 => request = Some(Request::Surrender),
        099 => request = Some(Request::ReportError {
            errormessage: extract_string(&mut reader) 
        }), 


        100 => response = Some(Response::Ok),
        101 => response = Some(extract_features(&mut reader)), 
        102 => response = Some(Response::NameTaken {
            nickname: extract_string(&mut reader)
        }),
        103 => response = Some(Response::NoSuchPlayer {
            nickname: extract_string(&mut reader)
        }),
        104 => response = Some(Response::NotWaiting {
            nickname: extract_string(&mut reader)
        }),
        105 => response = Some(Response::GameAlreadyStarted),
        110 => response = Some(Response::IllegalPlacement),
        111 => response = Some(Response::Hit {
            x: extract_number(&mut reader),
            y: extract_number(&mut reader)
        }),
        112 => response = Some(Response::Miss {
            x: extract_number(&mut reader),
            y: extract_number(&mut reader)
        }),
        113 => response = Some(Response::Destroyed {
            x: extract_number(&mut reader),
            y: extract_number(&mut reader)
        }),
        199 => response = Some(Response::InvalidRequest),


        200 => update = Some(Update::PlayerJoined {
            nickname: extract_string(&mut reader)
        }),
        201 => update = Some(Update::PlayerLeft {
            nickname: extract_string(&mut reader)
        }),
        202 => update = Some(Update::PlayerReady {
            nickname: extract_string(&mut reader)
        }),
        203 => update = Some(Update::PlayerNotReady {
            nickname: extract_string(&mut reader)
        }),
        204 => update = Some(Update::GameStart {
            nickname: extract_string(&mut reader)
        }),
        210 => update = Some(Update::YourTurn),
        211 => update = Some(Update::EnemyTurn),
        212 => update = Some(Update::EnemyVisible {
            x: extract_number(&mut reader),
            y: extract_number(&mut reader)
        }),
        213 => update = Some(Update::EnemyInvisible {
            x: extract_number(&mut reader),
            y: extract_number(&mut reader)
        }),
        214 => update = Some(Update::EnemyHit {
            x: extract_number(&mut reader),
            y: extract_number(&mut reader)
        }),
        215 => update = Some(Update::EnemyMiss {
            x: extract_number(&mut reader),
            y: extract_number(&mut reader)
        }),
        216 => update = Some(Update::GameOver {
            victorious: extract_bool(&mut reader),
            reason: get_reason(extract_number(&mut reader)),
        }),
        217 => update = Some(Update::AfkWarning {
            strikes: extract_number(&mut reader)
        }),
        218 => update = Some(Update::EnemyAfk{
            strikes: extract_number(&mut reader)
        }),

        255 => update = Some(Update::ServerGoingDown),

        _   => {}
    }
    return (request, response, update);
}
