use std::io::Read;
use std::io::BufReader;
use std::str;
use std::option::Option::None;

use std::net::TcpStream;

#[derive(Debug)]
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
    ReportErrorRequest {
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
        099 => msg = Some(Message::ReportErrorRequest {
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
