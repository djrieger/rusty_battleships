use std::io::{BufReader, BufWriter, Write, Read, stdin, BufRead};
use std::net::{TcpStream};
use std::sync::mpsc::{self, Sender, Receiver};
use std::thread;
use std::time::Duration;
use std::sync::mpsc::TryRecvError;

//extern crate argparse;
//use argparse::{ArgumentParser, Print, Store};

use message::{serialize_message, deserialize_message, Message, ShipPlacement, Direction};
use clientlobby::ClientLobby;
use game::Game;
use board::{Ship};
use clientboard::{Board, W, H};

fn send_message(msg: Message, stream: &mut BufWriter<TcpStream>) {
    let serialized_msg = serialize_message(msg);
    stream.write(&serialized_msg[..]).unwrap();
    stream.flush();
}

/* For quick user prompts. */
pub fn ask(question: String) -> String {
    println!("{}", question);
    let stdin = stdin();
    let answer = stdin.lock().lines().next().unwrap().unwrap();
    return answer;
}

/*Tries to read from the TCP stream. If there's no message, it waits patiently.*/
pub fn tcp_poll(br: &mut BufReader<TcpStream>, tx: Sender<Message>) {
    loop {
        println!("Waiting for a message");
        //This can take a while!
        let msg_from_server = deserialize_message(br);
        if msg_from_server.is_err() {
            panic!("FUUUUU!");
        }
        tx.send(msg_from_server.unwrap()).unwrap();
    }
}

pub struct State {
    pub lobby : ClientLobby,
    opponent : String,
    status : Status,
    my_board : Option<Board>,
    their_board : Option<Board>,
    buff_reader : BufReader<TcpStream>,
    buff_writer : BufWriter<TcpStream>,
}

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub enum Status {
    //LOBBY
    Unregistered,
    AwaitingFeatures,
    Register,
    Available,
    AwaitReady,
    AwaitGameStart,
    Waiting,
    AwaitNotReady,
    //GAME
    PlacingShips,
    OpponentPlacing,
    Planning,
    OpponentPlanning,
}

impl State {

    pub fn new (buff_reader: BufReader<TcpStream>, buff_writer: BufWriter<TcpStream>) -> State {
        State {
            lobby : ClientLobby::new(),
            opponent : String::from("None"),
            status : Status::Unregistered,
            my_board : None,
            their_board : None,
            buff_reader : buff_reader,
            buff_writer : buff_writer,
        }
    }

    pub fn get_features(&mut self) -> bool {
        send_message(Message::GetFeaturesRequest, &mut self.buff_writer);
        self.status = Status::AwaitingFeatures;
        let server_response = deserialize_message(&mut self.buff_reader);
        if server_response.is_err() {
            return false;
        } else if let Ok(Message::FeaturesResponse { features: fts }) = server_response {
            if let Ok(()) = self.handle_get_features_response(fts) {
                return true;
            } else {
                println!("Something went wrong with receiving the feature list.");
                self.status = Status::Unregistered;
                return false;
            }
        } else {
            println!("That is no FeaturesResponse! MSG={:?}", server_response);
            self.status = Status::Unregistered;
            return false;
        }
    }

    //FIXME: Change return value to Result<(),String)>
    pub fn login(&mut self, nickname: &str) -> bool {
        if self.status != Status::Unregistered {
            return false;
        }
        send_message(Message::LoginRequest { username: String::from(nickname) }, &mut self.buff_writer);
        self.status = Status::Register;
        return true;
    }

    //FIXME: Change return value to Result<(),String)>
    pub fn ready(&mut self) -> bool {
        if self.status != Status::Available {
            return false;
            println!("Möp");
        }
        self.status = Status::AwaitReady;
        send_message(Message::ReadyRequest, &mut self.buff_writer);
        println!("Sending READY_REQUEST");
        return true;
    }

    /* Sends a challenge to the server, if and only if the opponent is in the ready-and-waiting-list */
    //FIXME: Change return value to Result<(),String)>
    pub fn challenge(&mut self, opponent: &str) -> bool {
        if self.status != Status::Available {
            println!("You can't challenge anyone unless you're in state AVAILABLE! STATE={:?}", self.status.clone());
            return false;
        }

        if self.lobby.player_name != opponent {
            //if self.lobby.player_list.contains(&String::from(opponent)) //FIXME: Lobby updates and related stuff!
            println!("Challenging captain {:?}", opponent);
            send_message(Message::ChallengePlayerRequest { username: String::from(opponent) }, &mut self.buff_writer);
            self.status = Status::AwaitGameStart;
            return true;
        } else {
            return false;
        }
    }

    //FIXME: Change return value to Result<(),String)>
    pub fn place_ships(&mut self) -> bool {
        if self.status != Status::PlacingShips {
            return false;
        }
        //Dummy Values
        //TODO:Ask user!
        let ship_placements0 = ShipPlacement { x: 0, y: 0, direction: Direction::East};
        let ship_placements1 = ShipPlacement { x: 0, y: 1, direction: Direction::East};
        let ship_placements2 = ShipPlacement { x: 0, y: 2, direction: Direction::East};
        let ship_placements3 = ShipPlacement { x: 0, y: 3, direction: Direction::East};
        let ship_placements4 = ShipPlacement { x: 0, y: 4, direction: Direction::East};
        let ship_placements : [ShipPlacement; 5] = [ship_placements0, ship_placements1, ship_placements2, ship_placements3, ship_placements4];
        println!("{:?}", ship_placements);
        send_message(Message::PlaceShipsRequest { placement: ship_placements }, &mut self.buff_writer);

        let mut ships = Vec::<Ship>::new();
        ships.push(Ship { x: 0, y: 0, length: 2, horizontal:true, health_points: 2});
        ships.push(Ship { x: 0, y: 1, length: 2, horizontal:true, health_points: 2});
        ships.push(Ship { x: 0, y: 2, length: 3, horizontal:true, health_points: 3});
        ships.push(Ship { x: 0, y: 3, length: 4, horizontal:true, health_points: 4});
        ships.push(Ship { x: 0, y: 4, length: 5, horizontal:true, health_points: 5});
        self.my_board = Some(Board::new(ships, true));
        self.their_board = Some(Board::new(Vec::<Ship>::new(), false));

        return true;
    }

    pub fn have_i_been_challenged(&mut self) -> (bool, Option<String>) {
        {
            let message_waiting = false;
            println!("Counting waiting bytes");
            let waiting = self.buff_reader.get_ref().bytes().count();//.clone();
            println!("Möp");
        }

        let server_response =  deserialize_message(&mut self.buff_reader);
        if server_response.is_err() {
            print!("foo");
            return (false, None);
        } else {
            match server_response.unwrap() {
                Message::GameStartUpdate { nickname: x } => {
                    return (true,Some(x));
                },
                _ => return (false,None),
            };
        }
    }

    //FIXME: Change return value to Result<(),String)>
    pub fn shoot(&mut self, x: usize, y: usize) -> Result<bool, String> {
        if self.status != Status::Planning {
            return Err(String::from("Not your turn!"));
        }
        if x >= W || y >= H {
            return Err(format!("Out of bounds! x={:?} y={:?}", x, y));
        }
        let msg = Message::ShootRequest {x: x as u8, y: y as u8};
        send_message(msg, &mut self.buff_writer);
        let server_response = deserialize_message(&mut self.buff_reader);
        if server_response.is_err() {
            return Err(String::from("Does not compute."));
        } else {
            match server_response.unwrap() {
                Message::OkResponse => return Ok(true),
                x => return Err(format!("Does not compute! MSG={:?}", x)),
            }
        }
    }

    //FIXME: Change return value to Result<(),String)>
    pub fn move_and_shoot(&mut self, ship: usize, dir: Direction, x: usize, y: usize) -> Result<bool, String> {
        if self.status != Status::Planning {
            return Err(String::from("Not your turn!"));
        }
        if 4 < ship { //Destroyed ships shall also be unable to move! Note that ship is unsigned! => no 0 > ship condition necessary!
            return Err(format!("Ship id out of bounds! id={:?}", ship));
        }
        if x >= W || y >= H {
            return Err(format!("Shot location out of bounds! x={:?} y={:?}", x, y));
        }
        let msg = Message::MoveAndShootRequest { id: ship as u8, direction: dir, x: x as u8, y: y as u8 };
        send_message(msg, &mut self.buff_writer);
        let server_response = deserialize_message(&mut self.buff_reader);
        if server_response.is_err() {
            return Err(String::from("Does not compute."));
        } else {
            match server_response.unwrap() {
                Message::OkResponse => return Ok(true),
                x => return Err(format!("Does not compute! MSG={:?}", x)),
            }
        }
    }

    pub fn handle_get_features_response(&mut self, features: Vec<String>) -> Result<(), String> {
        if self.status == Status::AwaitingFeatures {
            self.lobby.set_feature_list(features);
            self.status = Status::Unregistered;
            return Ok(());
        } else {
            return Err(format!("ERROR: I did not expect a feature response! STATUS={:?}", self.status));
        }
    }

    pub fn handle_game_start_update(&mut self, nickname: &str) -> Result<(), String> {
        if self.status == Status::Waiting {
            self.status = Status::PlacingShips;
            //Place the ships!
            self.place_ships();
            return Ok(());
        } else {
            return Err(format!("ERROR: I did not expect a GameStartUpdate! STATUS={:?}", self.status));
        }
    }

    /* Program flow guideline: Set your values when you're sending the Requests and hand over to
     * the usual message loop. If everythin goes the way it's meant to go, all's fine. If not, then
     * we'll panic anyway. */
    pub fn handle_ok_response(&mut self, msg: Message) -> Result<(), String> {
        match self.status {
            Status::Register => {
                self.status = Status::Available;
                self.challenge("test2"); //FIXME ONLY FOR TESTING!
                return Ok(());
            },
            Status::AwaitGameStart => {
                self.status = Status::PlacingShips;
                self.place_ships(); //FIXME ONLY FOR TESING!
                return Ok(());
            },
            Status::AwaitReady => {
                println!("Waiting to be challenged.");
                self.status = Status::Waiting;
                return Ok(());
            },
            Status::AwaitNotReady => {
                self.status = Status::Available;
                return Ok(());
            },
            Status::PlacingShips => {
                println!("Ships ok.");
                self.status = Status::OpponentPlacing;
                return Ok(());
            },
            _ => return Err(format!("Wrong message! STATUS={:?}, MESSAGE={:?}", self.status, msg)),
        }
    }

    pub fn handle_name_taken_response(&mut self, nickname: &str) {
        if self.status == Status::Register {
            self.status = Status::Unregistered;
            let new_name = ask(String::from("What was yer name again?")); //FIXME: FOR TESTING ONLY!
            send_message(Message::LoginRequest { username: String::from(new_name) }, &mut self.buff_writer); //FIXME: FOR TESTING ONLY!
            self.status = Status::Register; //FIXME: FOR TESTING ONLY!
        } else {
            panic!("Received a NAME_TAKEN_RESPONSE while not in Register State! STATUS={:?}", self.status);
        }
    }

    pub fn handle_no_such_player_response(&mut self, nickname: &str) {
        if self.status == Status::AwaitGameStart {
            self.status = Status::Available;
        } else {
            panic!("Received a NAME_TAKEN_RESPONSE while not in AwaitGameStart State! STATUS={:?}", self.status);
        }
    }

    pub fn handle_not_waiting_response(&mut self, nickname: &str) {
        if self.status == Status::AwaitGameStart {
            self.status = Status::Available;
        } else {
            panic!("Received a NOT_WAITING_RESPONSE while not in AwaitGameStart State! STATUS={:?}", self.status);
        }
    }

    pub fn handle_hit_response(&mut self, x: u8, y: u8) {
        if self.status == Status::Planning {
            if let Some(ref mut board) = self.my_board {
                board.hit(x as usize, y as usize);
            }
        } else {
            panic!("Received a HIT_RESPONSE while not in PLANNING state! STATUS={:?}", self.status);
        }
    }

    /* Main loop; does most of the work. Main-Function should hand over control to this function as
    soon as a tcp connection has been established.*/
    pub fn handle_communication(&mut self/*, br: BufReader<TcpStream>, bw: BufWriter<TcpStream>*/) {
        if self.get_features() {
            println!("Supported features: {:?}", self.lobby.feature_list);
        } else {
            println!("No features.");
        }

        let nickname = ask(String::from("What's yer name, captain!?"));

        if self.login(&nickname) {
            println!("G'day, captain {:?}!", self.lobby.player_name);
        } else {
            println!("Login error.");
        }


        let (tx, rx) = mpsc::channel();
        let mut one_time_reader = BufReader::<TcpStream>::new(self.buff_reader.get_ref().try_clone().unwrap());
        thread::spawn(move || tcp_poll(&mut one_time_reader, tx));

        /*check-for-messages-loop*/
        loop {
            println!("Checking for messages.");
            let received = rx.try_recv();
            if let Ok(server_response) = received {
                println!("Oh, a message for me! MSG={:?}", server_response.clone());

                let outcome: Result<(), String>;
                match server_response { //May contain traces of state transisions
                    // UPDATES
                    Message::PlayerJoinedUpdate {nickname: nn} => {
                        println!("Welcome our new captain {:?}", nn);
                        self.lobby.add_player(&nn.clone());
                    },
                    Message::PlayerLeftUpdate {nickname: nn} => {
                        println!("Say goodbye to captain {:?}", nn);
                        self.lobby.remove_player(&nn.clone());
                    }
                    Message::PlayerReadyUpdate {nickname: nn} => {
                        println!("Captain {:?} is now ready to be challenged.", nn);
                        self.lobby.ready_player(&nn.clone());
                    },
                    Message::PlayerNotReadyUpdate {nickname : nn} => {
                        println!("Captain {:?} is not ready.", nn);
                        self.lobby.unready_player(&nn.clone());
                    },
                    Message::GameStartUpdate {nickname: nn} => {
                        println!("Received a challenge by captain {:?}", nn);
                        self.handle_game_start_update(&nn.clone());
                    },
                    Message::ServerGoingDownUpdate {errormessage: err}=> {
                        println!("The server is going down!");
                        println!("REASON:{:?}",err);
                    }
                    // RESPONSES
                    Message::OkResponse => outcome = self.handle_ok_response(server_response),
                    Message::InvalidRequestResponse => {
                        println!("Received an INVALID_REQUEST_RESPONSE in state {:?}.", self.status);
                    },
                    Message::FeaturesResponse {features: fts} => {
                        println!("Received features list!");
                        self.handle_get_features_response(fts);
                    },
                    Message::NameTakenResponse {nickname: nn} => {
                        println!("There is already a captain {:?} registered. Choose a different name.", nn);
                        self.handle_name_taken_response(&nn);
                    },
                    Message::NoSuchPlayerResponse {nickname: nn} => {
                        println!("There is no captain {:?} registered.", nn);
                        self.handle_no_such_player_response(&nn);
                    },
                    Message::NotWaitingResponse {nickname: nn} => {
                        println!("Captain {:?} is not waiting to be challenged.", nn);
                        self.handle_not_waiting_response(&nn);
                        self.ready();
                    },
                    Message::GameAlreadyStartedResponse => {
                        println!("The game has already started.");
                    },
                    Message::HitResponse {x: x, y: y} => {
                        println!("You have been hit! ({}, {})", x, y);
                        self.handle_hit_response(x, y);
                    }
                    _ => println!("Message received: {:?}", server_response),
                }

            } else if received == Err(TryRecvError::Empty) {
                //println!("Nothing there =(");
                thread::sleep(Duration::new(0, 500000000));
            } else if received == Err(TryRecvError::Disconnected) {
                panic!("Server terminated connection. =(");
            }

        }

    }

}
