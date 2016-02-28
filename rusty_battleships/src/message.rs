use std::io::{BufReader, Error, ErrorKind, Read, Result, Write};
use std::net::TcpStream;

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
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
        features:Vec<String>,
    },
    NameTakenResponse {
        nickname:String,
    },
    NoSuchPlayerResponse {
        nickname:String,
    },
    NotWaitingResponse {
        nickname:String,
    },
    GameAlreadyStartedResponse,
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
    NotYourTurnResponse,
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
    ServerGoingDownUpdate {
        errormessage:String,
    },
}

#[derive(Debug, Hash, Eq, PartialEq)]
enum MessageEnvironment {
    Lobby,
    Game,
    All,
}

#[derive(Debug, Hash, Eq, PartialEq)]
enum MessageType {
    Request,
    Response,
    Update,
}

fn message_type(msg: Message) -> (MessageEnvironment, MessageType) {
    match msg {
        Message::GetFeaturesRequest |
        Message::LoginRequest{..} |
        Message::ReadyRequest |
        Message::NotReadyRequest |
        Message::ChallengePlayerRequest {..} => (MessageEnvironment::Lobby, MessageType::Request),

        Message::PlaceShipsRequest {..} |
        Message::ShootRequest {..} |
        Message::MoveAndShootRequest {..} |
        Message::SurrenderRequest => (MessageEnvironment::Game, MessageType::Request),

        Message::ReportErrorRequest{..} => (MessageEnvironment::All, MessageType::Request),


        Message::FeaturesResponse{..} |
        Message::NameTakenResponse{..} |
        Message::NoSuchPlayerResponse{..} |
        Message::NotWaitingResponse{..} |
        Message::GameAlreadyStartedResponse => (MessageEnvironment::Lobby, MessageType::Response),

        Message::HitResponse {..} |
        Message::MissResponse {..} |
        Message::DestroyedResponse {..} |
        Message::NotYourTurnResponse => (MessageEnvironment::Game, MessageType::Response),

        Message::OkResponse |
        Message::InvalidRequestResponse => (MessageEnvironment::All, MessageType::Response),


        Message::PlayerJoinedUpdate{..} |
        Message::PlayerLeftUpdate{..} |
        Message::PlayerReadyUpdate{..} |
        Message::PlayerNotReadyUpdate{..} |
        Message::GameStartUpdate{..} => (MessageEnvironment::Lobby, MessageType::Update),

        Message::YourTurnUpdate |
        Message::EnemyTurnUpdate |
        Message::EnemyVisibleUpdate{..} |
        Message::EnemyInvisibleUpdate{..} |
        Message::EnemyHitUpdate{..} |
        Message::EnemyMissUpdate{..} |
        Message::GameOverUpdate{..} |
        Message::AfkWarningUpdate{..} |
        Message::EnemyAfkUpdate{..} => (MessageEnvironment::Game, MessageType::Update),

        Message::ServerGoingDownUpdate{..} => (MessageEnvironment::All, MessageType::Update),
    }
}

pub fn is_fatal_error(msg: Message) -> bool {
    match msg {
        Message::ReportErrorRequest{..} |
        Message::InvalidRequestResponse |
        Message::ServerGoingDownUpdate{..} => true,
        _ => false
    }
}

pub fn is_request(msg: Message) -> bool {
    message_type(msg).1 == MessageType::Request
}

pub fn is_response(msg: Message) -> bool {
    message_type(msg).1 == MessageType::Response
}

pub fn is_update(msg: Message) -> bool {
    message_type(msg).1 == MessageType::Update
}

pub fn is_game(msg: Message) -> bool {
    let env = message_type(msg).0;
    env == MessageEnvironment::Game || env == MessageEnvironment::All
}

pub fn is_lobby(msg: Message) -> bool {
    let env = message_type(msg).0;
    env == MessageEnvironment::Lobby || env == MessageEnvironment::All
}


#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum Direction {
    North = 0,
    East = 1,
    South = 2,
    West = 3,
}

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub struct ShipPlacement {
    pub x:u8,
    pub y:u8,
    pub direction:Direction
}

#[derive(Debug, Hash, Eq, PartialEq, Copy, Clone)]
pub enum Reason {
    Obliterated = 0,
    Disconnected = 1,
    Surrendered = 2,
    Afk = 3,
}

fn read_into_buffer(mut message_buffer: &mut [u8], reader: &mut BufReader<TcpStream>)
        -> Result<()> {
    reader.read_exact(&mut message_buffer).map_err(|e| {
        if e.kind() == ErrorKind::UnexpectedEof {
            Error::new(ErrorKind::UnexpectedEof, "TCP stream closed unexpectedly.")
        } else {
            e
        }
    })
}

fn extract_number(mut reader: &mut BufReader<TcpStream>) -> Result<u8> {
    let mut message_buffer:[u8;1] = [0;1];
    try!(read_into_buffer(&mut message_buffer, &mut reader));
    return Ok(message_buffer[0]);
}

fn extract_bool(mut reader: &mut BufReader<TcpStream>) -> Result<bool> {
    match try!(extract_number(&mut reader)) {
        1 => Ok(true),
        0 => Ok(false),
        _ => Err(Error::new(ErrorKind::InvalidData, "Invalid boolean value."))
    }
}

pub fn extract_direction(mut reader: &mut BufReader<TcpStream>) -> Result<Direction> {
    match try!(extract_number(&mut reader)) {
        0 => Ok(Direction::North),
        1 => Ok(Direction::East),
        2 => Ok(Direction::South),
        3 => Ok(Direction::West),
        _ => Err(Error::new(ErrorKind::InvalidData, "Invalid direction value."))
    }
}

pub fn extract_reason(mut reader: &mut BufReader<TcpStream>) -> Result<Reason> {
    match try!(extract_number(&mut reader)) {
        0 => Ok(Reason::Obliterated),
        1 => Ok(Reason::Disconnected),
        2 => Ok(Reason::Surrendered),
        3 => Ok(Reason::Afk),
        _ => Err(Error::new(ErrorKind::InvalidData, "Invalid reason value."))
    }
}

fn extract_string(mut reader: &mut BufReader<TcpStream>, allow_space: bool)
        -> Result<String> {
    let strlen = try!(extract_number(&mut reader)) as usize;
    let mut string_buffer = vec![0;strlen];
    try!(read_into_buffer(&mut string_buffer, &mut reader));

    // check whether characters are in range
    let valid: bool = string_buffer.iter().fold(true, |in_range, &character| {
        in_range && character <= 0x7E && character >= (if allow_space { 0x20 } else { 0x21 })
    });
    if !valid {
        return Err(Error::new(ErrorKind::InvalidData, "String contains invalid characters."));
    }

    return Ok(String::from_utf8(string_buffer).unwrap());
}

fn extract_features(mut reader: &mut BufReader<TcpStream>) -> Result<Vec<String>> {
    let numfeatures = try!(extract_number(&mut reader));
    let mut features = Vec::new();
    for _ in 0..numfeatures {
        features.push(try!(extract_string(&mut reader, true)));
    }
    return Ok(features);
}

fn extract_placement(mut reader: &mut BufReader<TcpStream>) -> Result<[ShipPlacement; 5]> {
    let mut placement:[ShipPlacement; 5]
            = [ShipPlacement { x: 0, y: 0, direction: Direction::North }; 5];
    for i in 0..5 {
        placement[i] = ShipPlacement {
            x: try!(extract_number(&mut reader)),
            y: try!(extract_number(&mut reader)),
            direction: try!(extract_direction(&mut reader)),
        };
    }
    return Ok(placement);
}

pub fn deserialize_message(mut reader: &mut BufReader<TcpStream>) -> Result<Message> {
    match try!(extract_number(&mut reader)) {
        000 => Ok(Message::GetFeaturesRequest),
        001 => Ok(Message::LoginRequest {
            username: try!(extract_string(&mut reader, false))
        }),
        002 => Ok(Message::ReadyRequest),
        003 => Ok(Message::NotReadyRequest),
        004 => Ok(Message::ChallengePlayerRequest {
            username: try!(extract_string(&mut reader, false))
        }),
        010 => Ok(Message::PlaceShipsRequest {
            placement: try!(extract_placement(&mut reader))
        }),
        011 => Ok(Message::ShootRequest {
            x: try!(extract_number(&mut reader)),
            y: try!(extract_number(&mut reader))
        }),
        012 => Ok(Message::MoveAndShootRequest {
            id: try!(extract_number(&mut reader)),
            direction: try!(extract_direction(&mut reader)),
            x: try!(extract_number(&mut reader)),
            y: try!(extract_number(&mut reader))
        }),
        013 => Ok(Message::SurrenderRequest),
        099 => Ok(Message::ReportErrorRequest {
            errormessage: try!(extract_string(&mut reader, true))
        }),


        100 => Ok(Message::OkResponse),
        101 => Ok(Message::FeaturesResponse {
            features: try!(extract_features(&mut reader))
        }),
        102 => Ok(Message::NameTakenResponse {
            nickname: try!(extract_string(&mut reader, false))
        }),
        103 => Ok(Message::NoSuchPlayerResponse {
            nickname: try!(extract_string(&mut reader, false))
        }),
        104 => Ok(Message::NotWaitingResponse {
            nickname: try!(extract_string(&mut reader, false))
        }),
        105 => Ok(Message::GameAlreadyStartedResponse),
        111 => Ok(Message::HitResponse {
            x: try!(extract_number(&mut reader)),
            y: try!(extract_number(&mut reader))
        }),
        112 => Ok(Message::MissResponse {
            x: try!(extract_number(&mut reader)),
            y: try!(extract_number(&mut reader))
        }),
        113 => Ok(Message::DestroyedResponse {
            x: try!(extract_number(&mut reader)),
            y: try!(extract_number(&mut reader))
        }),
        114 => Ok(Message::NotYourTurnResponse),
        199 => Ok(Message::InvalidRequestResponse),


        200 => Ok(Message::PlayerJoinedUpdate {
            nickname: try!(extract_string(&mut reader, false))
        }),
        201 => Ok(Message::PlayerLeftUpdate {
            nickname: try!(extract_string(&mut reader, false))
        }),
        202 => Ok(Message::PlayerReadyUpdate {
            nickname: try!(extract_string(&mut reader, false))
        }),
        203 => Ok(Message::PlayerNotReadyUpdate {
            nickname: try!(extract_string(&mut reader, false))
        }),
        204 => Ok(Message::GameStartUpdate {
            nickname: try!(extract_string(&mut reader, false))
        }),
        210 => Ok(Message::YourTurnUpdate),
        211 => Ok(Message::EnemyTurnUpdate),
        212 => Ok(Message::EnemyVisibleUpdate {
            x: try!(extract_number(&mut reader)),
            y: try!(extract_number(&mut reader))
        }),
        213 => Ok(Message::EnemyInvisibleUpdate {
            x: try!(extract_number(&mut reader)),
            y: try!(extract_number(&mut reader))
        }),
        214 => Ok(Message::EnemyHitUpdate {
            x: try!(extract_number(&mut reader)),
            y: try!(extract_number(&mut reader))
        }),
        215 => Ok(Message::EnemyMissUpdate {
            x: try!(extract_number(&mut reader)),
            y: try!(extract_number(&mut reader))
        }),
        216 => Ok(Message::GameOverUpdate {
            victorious: try!(extract_bool(&mut reader)),
            reason: try!(extract_reason(&mut reader))
        }),
        217 => Ok(Message::AfkWarningUpdate {
            strikes: try!(extract_number(&mut reader))
        }),
        218 => Ok(Message::EnemyAfkUpdate{
            strikes: try!(extract_number(&mut reader))
        }),

        255 => Ok(Message::ServerGoingDownUpdate{
            errormessage: try!(extract_string(&mut reader, true))
        }),

        _   => Err(Error::new(ErrorKind::InvalidData, "Invalid opcode."))
    }
}

fn append_string(mut buf: &mut Vec<u8>, string: String) {
    assert!(string.len() <= 255, "String exceeds maximum allowed length.");
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
            for ship_placement in &placement {
                msgbuf.push(ship_placement.x);
                msgbuf.push(ship_placement.y);
                msgbuf.push(ship_placement.direction as u8);
            }
        },
        Message::ShootRequest { x, y } =>{
            msgbuf.push(011);
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
        Message::ReportErrorRequest { errormessage } => {
            msgbuf.push(099);
            append_string(&mut msgbuf, errormessage);
        },


        Message::OkResponse => msgbuf.push(100),
        Message::FeaturesResponse { features } => {
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
        Message::NotYourTurnResponse => msgbuf.push(114),
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

        Message::ServerGoingDownUpdate { errormessage } => {
            msgbuf.push(255);
            append_string(&mut msgbuf, errormessage);
        },
    }
    return msgbuf;
}
