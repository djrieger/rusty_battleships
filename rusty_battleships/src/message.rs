use std::io::Read;
use std::io::BufReader;
use std::str;
use std::option::Option::None;
use std::collections::HashMap;
use std::fmt;
use std::io::Write;

use std::net::TcpStream;

#[derive(Debug, Hash, Eq, PartialEq)]
pub enum Message {
    // Requests
    GetFeaturesRequest,
    LoginRequest {
        username:String
    },
    ReadyRequest,
    NotReadyRequest,
    ChallengePlayerRequest {
        username:String
    },
    PlaceShipsRequest {
        placement:[ShipPlacement; 5]
    },
    ShootRequest {
        x:u8,
        y:u8,
    },
    MoveAndShootRequest {
        id:u8,
        direction:Direction,
        x:u8,
        y:u8,
    },
    SurrenderRequest,
    ReportError{
        errormessage:String,
    },

    // Responses
    OkResponse,
    FeaturesResponse {
        numfeatures:u8,
        features:Vec<String>,
    },
    NameTakenResponse{
        nickname:String,
    },
    NoSuchPlayerResponse{
        nickname:String,
    },
    NotWaitingResponse{
        nickname:String,
    },
    GameAlreadyStartedResponse,
    IllegalPlacementResponse,
    HitResponse {
        x:u8,
        y:u8,
    },
    MissResponse {
        x:u8,
        y:u8,
    },
    DestroyedResponse {
        x:u8,
        y:u8,
    },
    InvalidRequestResponse,

    // Updates
    PlayerJoinedUpdate {
        nickname:String,
    },
    PlayerLeftUpdate {
        nickname:String,
    },
    PlayerReadyUpdate {
        nickname:String,
    },
    PlayerNotReadyUpdate {
        nickname:String,
    },
    GameStartUpdate {
        nickname:String, //Opponent's name
    },
    YourTurnUpdate,
    EnemyTurnUpdate,
    EnemyVisibleUpdate {
        x:u8,
        y:u8,
    },
    EnemyInvisibleUpdate {
        x:u8,
        y:u8,
    },
    EnemyHitUpdate {
        x:u8,
        y:u8,
    },
    EnemyMissUpdate {
        x:u8,
        y:u8,
    },
    GameOverUpdate {
        victorious:bool,
        reason:Reason,
    },
    AfkWarningUpdate {
        strikes:u8,
    },
    EnemyAfkUpdate {
        strikes:u8,
    },
    ServerGoingDownUpdate,
}

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum Direction {
    North = 0,
    East = 1,
    South = 2,
    West = 3,
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

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub struct ShipPlacement {
    pub x:u8,
    pub y:u8,
    pub direction:Direction
}

#[derive(Debug, Hash, Eq, PartialEq)]
pub enum Reason {
    Obliterated = 0,
    Disconnected = 1,
    Surrendered = 2,
    Afk = 3,
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

fn extract_features(mut reader: &mut BufReader<TcpStream>) -> Message {
    let numfeatures = extract_number(&mut reader);
    let mut features = Vec::new();
    for i in 0..numfeatures - 1 {
        features.push(extract_string(&mut reader));
    }
    return Message::FeaturesResponse {
        numfeatures: numfeatures,
        features: features
    };
}

pub fn deserialize_message(mut reader: &mut BufReader<TcpStream>) -> Option<Message> {
    let mut msg: Option<Message> = None;
    let opcode = extract_number(&mut reader);
    match opcode {
        000 => msg = Some(Message::GetFeaturesRequest),
        001 => msg = Some(Message::LoginRequest {
            username: extract_string(&mut reader)
        }),
        002 => msg = Some(Message::ReadyRequest),
        003 => msg = Some(Message::NotReadyRequest),
        004 => msg = Some(Message::ChallengePlayerRequest {
            username: extract_string(&mut reader)
        }),
        010 => msg = Some(Message::PlaceShipsRequest {
            placement: extract_placement(&mut reader)
        }),
        11 => msg = Some(Message::ShootRequest {
            x: extract_number(&mut reader),
            y: extract_number(&mut reader)
        }),
        012 => msg = Some(Message::MoveAndShootRequest {
            id: extract_number(&mut reader),
            direction: get_direction(extract_number(&mut reader)),
            x: extract_number(&mut reader),
            y: extract_number(&mut reader)
        }),
        013 => msg = Some(Message::SurrenderRequest),
        099 => msg = Some(Message::ReportError{
            errormessage: extract_string(&mut reader) 
        }), 


        100 => msg = Some(Message::OkResponse),
        101 => msg = Some(extract_features(&mut reader)), 
        102 => msg = Some(Message::NameTakenResponse {
            nickname: extract_string(&mut reader)
        }),
        103 => msg = Some(Message::NoSuchPlayerResponse {
            nickname: extract_string(&mut reader)
        }),
        104 => msg = Some(Message::NotWaitingResponse {
            nickname: extract_string(&mut reader)
        }),
        105 => msg = Some(Message::GameAlreadyStartedResponse),
        110 => msg = Some(Message::IllegalPlacementResponse),
        111 => msg = Some(Message::HitResponse {
            x: extract_number(&mut reader),
            y: extract_number(&mut reader)
        }),
        112 => msg = Some(Message::MissResponse {
            x: extract_number(&mut reader),
            y: extract_number(&mut reader)
        }),
        113 => msg = Some(Message::DestroyedResponse {
            x: extract_number(&mut reader),
            y: extract_number(&mut reader)
        }),
        199 => msg = Some(Message::InvalidRequestResponse),


        200 => msg = Some(Message::PlayerJoinedUpdate {
            nickname: extract_string(&mut reader)
        }),
        201 => msg = Some(Message::PlayerLeftUpdate {
            nickname: extract_string(&mut reader)
        }),
        202 => msg = Some(Message::PlayerReadyUpdate {
            nickname: extract_string(&mut reader)
        }),
        203 => msg = Some(Message::PlayerNotReadyUpdate {
            nickname: extract_string(&mut reader)
        }),
        204 => msg = Some(Message::GameStartUpdate {
            nickname: extract_string(&mut reader)
        }),
        210 => msg = Some(Message::YourTurnUpdate),
        211 => msg = Some(Message::EnemyTurnUpdate),
        212 => msg = Some(Message::EnemyVisibleUpdate {
            x: extract_number(&mut reader),
            y: extract_number(&mut reader)
        }),
        213 => msg = Some(Message::EnemyInvisibleUpdate {
            x: extract_number(&mut reader),
            y: extract_number(&mut reader)
        }),
        214 => msg = Some(Message::EnemyHitUpdate {
            x: extract_number(&mut reader),
            y: extract_number(&mut reader)
        }),
        215 => msg = Some(Message::EnemyMissUpdate {
            x: extract_number(&mut reader),
            y: extract_number(&mut reader)
        }),
        216 => msg = Some(Message::GameOverUpdate {
            victorious: extract_bool(&mut reader),
            reason: get_reason(extract_number(&mut reader)),
        }),
        217 => msg = Some(Message::AfkWarningUpdate {
            strikes: extract_number(&mut reader)
        }),
        218 => msg = Some(Message::EnemyAfkUpdate{
            strikes: extract_number(&mut reader)
        }),

        255 => msg = Some(Message::ServerGoingDownUpdate),

        _   => {}
    }
    return msg;
}

fn append_string(mut buf: &mut Vec<u8>, string: String) {
    if string.len() > 255 {
        panic!("String exceeds maximum allowed length.");
    }
    buf.push(string.len() as u8);
    write!(&mut buf, "{}", string);
}

pub fn serialize_message(msg: Message) -> Vec<u8> {
    let mut msgbuf = Vec::new();
    match msg {
        Message::GetFeaturesRequest => msgbuf.push(000), 
        Message::LoginRequest { username } => {
            msgbuf.push(001);
            append_string(&mut msgbuf, username);
        },
        Message::ReadyRequest => msgbuf.push(002),
        Message::NotReadyRequest => msgbuf.push(003),
        Message::ChallengePlayerRequest { username } => {
            msgbuf.push(004);
            append_string(&mut msgbuf, username);
        },
        Message::PlaceShipsRequest { placement } => {
            msgbuf.push(010);
            for shipPlacement in &placement {
                msgbuf.push(shipPlacement.x);
                msgbuf.push(shipPlacement.y);
                msgbuf.push(shipPlacement.direction as u8);
            }
        },
        Message::ShootRequest { x, y } =>{
            msgbuf.push(11);
            msgbuf.push(x);
            msgbuf.push(y);
        },
        Message::MoveAndShootRequest { id, direction, x, y } => {
            msgbuf.push(012);
            msgbuf.push(id);
            msgbuf.push(direction as u8);
            msgbuf.push(x);
            msgbuf.push(y);
        },
        Message::SurrenderRequest => msgbuf.push(013),
        Message::ReportError{ errormessage } => {
            msgbuf.push(099); 
            append_string(&mut msgbuf, errormessage);
        }, 


        Message::OkResponse => msgbuf.push(100),
        Message::FeaturesResponse { numfeatures, features } => {
            msgbuf.push(101);
            msgbuf.push(features.len() as u8);
            for feature in features {
                append_string(&mut msgbuf, feature);
            }
        },

        Message::NameTakenResponse { nickname } => {
            msgbuf.push(102);
            append_string(&mut msgbuf, nickname);
        },
        Message::NoSuchPlayerResponse { nickname } => {
            msgbuf.push(103);
            append_string(&mut msgbuf, nickname);
        },
        Message::NotWaitingResponse { nickname } => {
            msgbuf.push(104);
            append_string(&mut msgbuf, nickname);
        },
        Message::GameAlreadyStartedResponse => msgbuf.push(105),
        Message::IllegalPlacementResponse => msgbuf.push(110),
        Message::HitResponse { x, y } => {
            msgbuf.push(111); 
            msgbuf.push(x);
            msgbuf.push(y);
        },
        Message::MissResponse { x, y } => {
            msgbuf.push(112); 
            msgbuf.push(x);
            msgbuf.push(y);
        },
        Message::DestroyedResponse { x, y } => {
            msgbuf.push(113); 
            msgbuf.push(x);
            msgbuf.push(y);
        },
        Message::InvalidRequestResponse => msgbuf.push(199),


        Message::PlayerJoinedUpdate { nickname } => {
            msgbuf.push(200); 
            append_string(&mut msgbuf, nickname);
        },
        Message::PlayerLeftUpdate { nickname } => {
            msgbuf.push(201); 
            append_string(&mut msgbuf, nickname);
        },
        Message::PlayerReadyUpdate { nickname } => {
            msgbuf.push(202); 
            append_string(&mut msgbuf, nickname);
        },
        Message::PlayerNotReadyUpdate { nickname } => {
            msgbuf.push(203); 
            append_string(&mut msgbuf, nickname);
        },
        Message::GameStartUpdate { nickname } => {
            msgbuf.push(204); 
            append_string(&mut msgbuf, nickname);
        },

        Message::YourTurnUpdate => msgbuf.push(210),
        Message::EnemyTurnUpdate => msgbuf.push(211),
        Message::EnemyVisibleUpdate { x, y} => {
            msgbuf.push(212);
            msgbuf.push(x);
            msgbuf.push(y);
        },
        Message::EnemyInvisibleUpdate { x, y} => {
            msgbuf.push(213);
            msgbuf.push(x);
            msgbuf.push(y);
        },
        Message::EnemyHitUpdate { x, y} => {
            msgbuf.push(214);
            msgbuf.push(x);
            msgbuf.push(y);
        },
        Message::EnemyMissUpdate { x, y} => {
            msgbuf.push(215);
            msgbuf.push(x);
            msgbuf.push(y);
        },

        Message::GameOverUpdate { victorious, reason } => {
            msgbuf.push(216);
            msgbuf.push(victorious as u8);
            msgbuf.push(reason as u8);
        },
        Message::AfkWarningUpdate { strikes } => {
            msgbuf.push(217);
            msgbuf.push(strikes);
        },
        Message::EnemyAfkUpdate { strikes } => {
            msgbuf.push(218);
            msgbuf.push(strikes);
        },

        Message::ServerGoingDownUpdate => msgbuf.push(255),
    }
    return msgbuf;
}
