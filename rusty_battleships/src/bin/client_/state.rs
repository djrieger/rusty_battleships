use std::io::{BufReader, BufWriter, Write};
use std::net::{TcpStream};
use std::sync::mpsc::{self, Sender, Receiver};
use std::thread;
use std::sync::mpsc::TryRecvError;
use std::cmp;

use rustc_serialize::Encodable;

use client_::board::{Board};
use client_::lobby::ClientLobby;

use rusty_battleships::message::{serialize_message, deserialize_message, Message, ShipPlacement, Direction, Reason};
use rusty_battleships::ship::{Ship};
use rusty_battleships::timer::timer_periodic;


const TICK_DURATION_MS: u64 = 100;


#[derive(Clone, Debug, PartialEq, RustcEncodable)]
pub struct LobbyList {
    pub available_players : Vec<String>,
    pub ready_players : Vec<String>,
}

impl LobbyList {
    pub fn new() -> LobbyList {
        LobbyList {
            available_players: Vec::new(),
            ready_players: Vec::new()
        }
    }
}

fn send_message(msg: Message, stream: &mut BufWriter<TcpStream>) {
    let serialized_msg = serialize_message(msg);
    // TODO: don't crash when TCP stream closes
    stream.write(&serialized_msg[..]).unwrap();
    stream.flush().unwrap();
}

/*Tries to read from the TCP stream. If there's no message, it waits patiently.*/
pub fn tcp_poll(br: &mut BufReader<TcpStream>, tx: Sender<Message>) {
    loop {
        println!(">>> TCP: Waiting for a message");
        //This can take a while!
        let msg_from_server = deserialize_message(br);
        if msg_from_server.is_err() {
            println!(">>> TCP connection lost");
            break;
        }
        tx.send(msg_from_server.unwrap()).unwrap();
    }
}

pub struct State {
    pub lobby : ClientLobby,
    opponent : String,
    status : Status,
    my_turn : bool,
    my_afks : u8,
    their_afks : u8,
    hits: u8,
    destroyed: u8,
    my_board : Option<Board>,
    their_board : Option<Board>,
    pub buff_reader : BufReader<TcpStream>,
    buff_writer : BufWriter<TcpStream>,
    ui_update_receiver : Receiver<Message>,
    ui_update_sender : Sender<(Status, Message)>,
    lobby_update_sender : Sender<LobbyList>,
    board_update_sender : Sender<(Board, Board, u8, u8)>,
    disconnect_update_sender : Sender<bool>
}

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub enum Status {
    //LOBBY
    Unregistered,
    AwaitFeatures,
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
    //INTERMEDIATE
    Surrendered,
}

impl State {

    pub fn new(rec_ui_update: Receiver<Message>,
                tx_ui_update: Sender<(Status, Message)>,
                tx_lobby_update: Sender<LobbyList>,
                tx_board_update: Sender<(Board, Board, u8, u8)>,
                tx_disconnect_update: Sender<bool>,
                buff_reader: BufReader<TcpStream>,
                buff_writer: BufWriter<TcpStream>) -> State {
        State {
            lobby : ClientLobby::new(),
            opponent : String::from("None"),
            status : Status::Unregistered,
            my_turn : false,
            my_afks : 0,
            their_afks : 0,
            hits : 0,
            destroyed: 0,
            my_board : None,
            their_board : None,
            buff_reader : buff_reader,
            buff_writer : buff_writer,
            ui_update_receiver : rec_ui_update,
            ui_update_sender : tx_ui_update,
            lobby_update_sender : tx_lobby_update,
            board_update_sender : tx_board_update,
            disconnect_update_sender : tx_disconnect_update
        }
    }

    pub fn get_features(&mut self) -> bool {
        send_message(Message::GetFeaturesRequest, &mut self.buff_writer);
        self.status = Status::AwaitFeatures;
        return true;
    }

    //FIXME: Change return value to Result<(),String)>
    pub fn login(&mut self, nickname: &str) -> bool {
        if self.status != Status::Unregistered {
            println!("You're already logged in! STATUS = {:?}", self.status);
            return false;
        }
        send_message(Message::LoginRequest { username: String::from(nickname) }, &mut self.buff_writer);
        self.lobby.set_player_name(nickname);
        self.status = Status::Register;
        return true;
    }

    //FIXME: Change return value to Result<(),String)>
    pub fn ready(&mut self) -> bool {
        if self.status != Status::Available {
            return false;
        }
        self.status = Status::AwaitReady;
        send_message(Message::ReadyRequest, &mut self.buff_writer);
        println!("Sending READY_REQUEST");
        return true;
    }

    //FIXME: Change return value to Result<(),String)>
    pub fn unready(&mut self) -> bool {
        if self.status != Status::Waiting {
            return false;
        }
        self.status = Status::AwaitNotReady;
        send_message(Message::NotReadyRequest, &mut self.buff_writer);
        println!("Sending NOT_READY_REQUEST");
        return true;
    }

    /* Sends a challenge to the server, if and only if the opponent is in the ready-and-waiting-list */
    //FIXME: Change return value to Result<(),String)>
    pub fn challenge(&mut self, opponent: &str) -> bool {
        if self.status != Status::Available {
            println!("You can't challenge anyone unless you're in state AVAILABLE! STATE={:?}", self.status.clone());
            return false;
        }

        println!("Challenging captain {:?}", opponent);
        send_message(Message::ChallengePlayerRequest { username: String::from(opponent) }, &mut self.buff_writer);
        self.status = Status::AwaitGameStart;
        return true;
    }

    //FIXME: Change return value to Result<(),String)>
    pub fn place_ships(&mut self, ships: [ShipPlacement; 5]) -> bool {
        if self.status != Status::PlacingShips {
            return false;
        }

        let mut ship_vec = Vec::<Ship>::new();
        for i in 0..5 {
            let s = Ship {
                x: ships[i].x as isize,
                y: ships[i].y as isize,
                length: cmp::max(5-i, 2),
                direction: ships[i].direction,
                health_points: cmp::max(5-i, 2),
            };
            ship_vec.push(s);
        }
        self.my_board = Some(Board::new(ship_vec, true));
        self.my_board.as_mut().unwrap().compute_state();
        self.their_board = Some(Board::new(Vec::<Ship>::new(), false));

        send_message(Message::PlaceShipsRequest{ placement: ships }, &mut self.buff_writer);

        return true;
    }

    fn shoot(&mut self, x: u8, y: u8) {
        if self.status != Status::Planning {
            panic!("I cannot shoot when I'm not in Planning state! STATUS = {:?}", self.status);
        }

        send_message(Message::ShootRequest { x: x, y: y }, &mut self.buff_writer);
    }

    fn move_and_shoot(&mut self, x: u8, y: u8, id: u8, direction: Direction) {
        if self.status != Status::Planning {
            panic!("I cannot move and shoot when I'm not in Planning state! STATUS = {:?}", self.status);
        }

        self.my_board.as_mut().unwrap().move_ship(id as usize, direction);
        send_message(Message::MoveAndShootRequest { id: id, direction: direction, x: x, y: y }, &mut self.buff_writer);
    }


    pub fn handle_get_features_response(&mut self, features: Vec<String>) -> Result<(), String> {
        if self.status == Status::AwaitFeatures {
            self.lobby.set_feature_list(features);
            self.status = Status::Unregistered;
            return Ok(());
        } else {
            return Err(format!("ERROR: I did not expect a feature response! STATUS={:?}", self.status));
        }
    }

    pub fn surrender(&mut self) {
        if self.status == Status::Planning || self.status == Status::OpponentPlanning
            || self.status == Status::OpponentPlacing || self.status == Status::PlacingShips {
            self.status = Status::Surrendered;
            send_message(Message::SurrenderRequest, &mut self.buff_writer);
        }
    }

    pub fn handle_game_start_update(&mut self, nickname: &str) -> Result<(), String> {
        if self.status == Status::Waiting {
            self.status = Status::PlacingShips;
            self.opponent = nickname.to_owned();
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
                return Ok(());
            },
            Status::AwaitGameStart => {
                self.status = Status::PlacingShips;
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
            Status::Surrendered => {
                println!("Surrender request was received.");
                self.status = Status::Available;
                return Ok(());
            },
            _ => return Err(format!("Wrong message! STATUS={:?}, MESSAGE={:?}", self.status, msg)),
        }
    }

    pub fn handle_name_taken_response(&mut self, _: &str) {
        if self.status == Status::Register {
            self.status = Status::Unregistered;
        } else {
            panic!("Received a NAME_TAKEN_RESPONSE while not in Register State! STATUS={:?}", self.status);
        }
    }

    pub fn handle_no_such_player_response(&mut self, _: &str) {
        if self.status == Status::AwaitGameStart {
            self.status = Status::Available;
        } else {
            panic!("Received a NAME_TAKEN_RESPONSE while not in AwaitGameStart State! STATUS={:?}", self.status);
        }
    }

    pub fn handle_not_waiting_response(&mut self, _: &str) {
        if self.status == Status::AwaitGameStart {
            self.status = Status::Available;
        } else {
            panic!("Received a NOT_WAITING_RESPONSE while not in AwaitGameStart State! STATUS={:?}", self.status);
        }
    }

    pub fn handle_hit_response(&mut self, x: u8, y: u8) {
        if self.status == Status::Planning {
            if let Some(ref mut board) = self.their_board {
                board.hit(x as usize, y as usize);
            }
            self.my_turn = false;
            self.hits += 1;
            self.status = Status::OpponentPlanning;
        } else {
            panic!("Received a HIT_RESPONSE while not in PLANNING state! STATUS={:?}", self.status);
        }
    }

    pub fn handle_enemy_hit_update(&mut self, x: u8, y: u8) {
        if self.status == Status::OpponentPlanning {
            if let Some(ref mut board) = self.my_board {
                board.hit(x as usize, y as usize);
            }
            self.my_turn = true;
            self.status = Status::Planning;
        } else {
            panic!("Received a ENEMY_HIT_UPDATE while not in OPPONENT_PLANNING state! STATUS={:?}", self.status);
        }
    }

    pub fn handle_miss_response(&mut self, x: u8, y: u8) {
        if self.status == Status::Planning {
            if let Some(ref mut board) = self.their_board {
                board.miss(x as usize, y as usize);
            }
            self.my_turn = false;
            self.status = Status::OpponentPlanning;
        } else {
            panic!("Received a MISS_RESPONSE while not in PLANNING state! STATUS={:?}", self.status);
        }
    }

    pub fn handle_enemy_miss_update(&mut self, x: u8, y: u8) {
        if self.status == Status::OpponentPlanning {
            if let Some(ref mut board) = self.my_board {
                board.miss(x as usize, y as usize);
            }
            self.my_turn = true;
            self.status = Status::Planning;
        } else {
            panic!("Received a MISS_RESPONSE while not in PLANNING state! STATUS={:?}", self.status);
        }
    }

    pub fn handle_destroyed_response(&mut self, x: u8, y: u8) {
        if self.status == Status::Planning {
            if let Some(ref mut board) = self.their_board {
                board.destroyed(x as usize, y as usize);
            }
            self.my_turn = false;
            self.hits += 1;
            self.destroyed += 1;
            self.status = Status::OpponentPlanning;
        } else {
            panic!("Received a DESTROYED_RESPONSE while not in PLANNING state! STATUS={:?}", self.status);
        }
    }

    pub fn handle_your_turn_update(&mut self) {
        if self.status == Status::OpponentPlacing {
            self.my_turn = true;
            self.status = Status::Planning;
        } else {
            panic!("Received a YOUR_TURN_UPDATE while not in OPPONENT_PLACING state! STATUS={:?}", self.status);
        }
    }

    pub fn handle_enemy_turn_update(&mut self) {
        if self.status == Status::OpponentPlacing {
            self.my_turn = false;
            self.status = Status::OpponentPlanning;
        } else {
            panic!("Received a ENEMY_TURN_UPDATE while not in OPPONENT_PLACING state! STATUS={:?}", self.status);
        }
    }

    pub fn handle_enemy_visible_update(&mut self, x: u8, y: u8) {
        if let Some(ref mut board) = self.their_board {
            board.visible(x as usize, y as usize);
        }
    }

    pub fn handle_enemy_invisible_update(&mut self, x: u8, y: u8) {
        if let Some(ref mut board) = self.their_board {
            board.invisible(x as usize, y as usize);
        }
    }

    pub fn handle_afk_warning_update(&mut self, strikes: u8) {
        if self.status == Status::Planning {
            self.my_turn = false;
            self.my_afks += 1;
            if self.my_afks != strikes {
                panic!("Inconsistent strike count for **me**! MINE={}, SERVER={}", self.my_afks, strikes);
            }
            self.status = Status::OpponentPlanning;
        } else {
            panic!("Received a AFK_WARNING_UPDATE while not in PLANNING state! STATUS={:?}", self.status);
        }
    }

    pub fn handle_enemy_afk_update(&mut self, strikes: u8) {
        if self.status == Status::OpponentPlanning {
            self.my_turn = true;
            self.their_afks += 1;
            if self.their_afks != strikes {
                panic!("Inconsistent strike count for **the enemy**! MINE={}, SERVER={}", self.their_afks, strikes);
            }
            self.status = Status::Planning;
        } else {
            panic!("Received a ENEMY_AFK_UPDATE while not in OPPONENT_PLANNING state! STATUS={:?}", self.status);
        }
    }

    pub fn handle_game_over_update(&mut self, victory: bool, reason: Reason) {
        if self.status == Status::OpponentPlanning || self.status == Status::Planning ||
                self.status == Status::OpponentPlacing || self.status == Status::PlacingShips ||
                self.status == Status::Available || self.status == Status::Surrendered {
            println!("The game is over.");
            if victory {
                println!("Congratulations, captain! You've won!");
            } else {
                println!("You've lost.", );
            }
            println!("Reason: {:?}", reason);
            self.status = Status::Available;

            // reset game state
            self.my_turn = false;
            self.my_board = None;
            self.my_afks = 0;
            self.their_board = None;
            self.their_afks = 0;
            self.hits = 0;
            self.destroyed = 0;

            self.lobby = ClientLobby::new();
        } else {
            panic!("Received a GAME_OVER_UPDATE while not in an ingame state! STATUS={:?}", self.status);
        }
    }

    /* Contains the maain loop that does most of the work. Main-Function should hand over control to this function as
    soon as a tcp connection has been established.*/
    pub fn handle_communication(&mut self/*, br: BufReader<TcpStream>, bw: BufWriter<TcpStream>*/) {
        let (tx, rx) = mpsc::channel();
        let mut one_time_reader = BufReader::<TcpStream>::new(self.buff_reader.get_ref().try_clone().unwrap());
        thread::spawn(move || tcp_poll(&mut one_time_reader, tx));

        self.update_listen_loop(rx);
    }

    fn send_updated_lobby(&mut self) {
        let l = &self.lobby;
        self.lobby_update_sender.send(LobbyList {
            available_players: l.get_available_players(),
            ready_players: l.get_ready_players(),
        }).unwrap();
    }

    fn send_updated_boards(&mut self) {
        let mb;
        let tb;
        if let Some(ref b) = self.my_board {
            mb = b.clone();
        } else {
            panic!("I was told there would be boards! But there's no board for me...");
        }
        if let Some(ref c) = self.their_board {
            tb = c.clone();
        } else {
            panic!("I was told there would be boards! But there's no board for them...");
        }
        let boards = (mb, tb, self.hits, self.destroyed);
        self.board_update_sender.send(boards).unwrap();
    }

    pub fn update_listen_loop(&mut self, rx: Receiver<Message>) {
        println!(">>>Starting update_listen_loop.");

        let tick = timer_periodic(TICK_DURATION_MS);

        /*check-for-messages-loop*/
        loop {
            //println!("Checking for messages.");
            let received = rx.try_recv();
            if let Ok(server_response) = received {
                println!(">>>Oh, a message for me! MSG={:?}", server_response.clone());

                //let outcome: Result<(), String>;

                /* Handle messages from the server. */
                match server_response.clone() {
                    // UPDATES
                    Message::PlayerJoinedUpdate {nickname: nn} => {
                        println!("Welcome our new captain {:?}", nn);
                        self.lobby.add_player(&nn.clone());
                        self.send_updated_lobby();
                    },
                    Message::PlayerLeftUpdate {nickname: nn} => {
                        println!("Say goodbye to captain {:?}", nn);
                        self.lobby.remove_player(&nn.clone());
                        self.send_updated_lobby();
                    }
                    Message::PlayerReadyUpdate {nickname: nn} => {
                        println!("Captain {:?} is now ready to be challenged.", nn);
                        self.lobby.ready_player(&nn.clone());
                        self.send_updated_lobby();
                    },
                    Message::PlayerNotReadyUpdate {nickname : nn} => {
                        println!("Captain {:?} is not ready.", nn);
                        self.lobby.unready_player(&nn.clone());
                        self.send_updated_lobby();
                    },
                    Message::GameStartUpdate {nickname: nn} => {
                        println!("Received a challenge by captain {:?}", nn);
                        self.handle_game_start_update(&nn.clone()).unwrap();
                    },
                    Message::GameOverUpdate {victorious, reason} => {
                        self.handle_game_over_update(victorious, reason);
                    }
                    Message::ServerGoingDownUpdate {errormessage: err}=> {
                        println!("The server is going down!");
                        println!("REASON:{:?}",err);
                    },
                    Message::YourTurnUpdate => {
                        println!("It's yout turn!");
                        self.handle_your_turn_update();
                        self.send_updated_boards();
                    },
                    Message::EnemyTurnUpdate => {
                        println!("It's the enemy's turn!");
                        self.handle_enemy_turn_update();
                        self.send_updated_boards();
                    },
                    Message::NotYourTurnResponse => {
                        //println!("I'm sorry dave, I'm afraid I can't do that.");
                        // FIXME: handle properly, race condition occurred!
                        panic!("It's not your turn! Protocol demands termination.");
                    },
                    Message::AfkWarningUpdate {strikes} => {
                        self.handle_afk_warning_update(strikes);
                    }
                    Message::EnemyHitUpdate {x, y} => {
                        println!("We're hit! ({}, {})", x, y);
                        self.handle_enemy_hit_update(x, y);
                        self.send_updated_boards();
                    },
                    Message::EnemyMissUpdate {x, y} => {
                        println!("They missed! ({}, {})", x, y);
                        self.handle_enemy_miss_update(x, y);
                        self.send_updated_boards();
                    },
                    Message::EnemyVisibleUpdate {x, y} => {
                        println!("The enemy has been sighted! ({}, {})", x, y);
                        self.handle_enemy_visible_update(x, y);
                        self.send_updated_boards();
                    },
                    Message::EnemyInvisibleUpdate {x, y} => {
                        println!("We lost track of the enemy! ({}, {})", x, y);
                        self.handle_enemy_invisible_update(x, y);
                        self.send_updated_boards();
                    },
                    Message::EnemyAfkUpdate {strikes} => {
                        println!("The enemy is sleeping! ({})", strikes);
                        self.handle_enemy_afk_update(strikes);
                    },
                    // RESPONSES
                    Message::OkResponse => self.handle_ok_response(server_response.clone()).unwrap(),
                    Message::InvalidRequestResponse => {
                        println!("Received an INVALID_REQUEST_RESPONSE in state {:?}.", self.status);
                    },
                    Message::FeaturesResponse {features: fts} => {
                        println!("Received features list!");
                        self.handle_get_features_response(fts).unwrap();
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
                    },
                    Message::GameAlreadyStartedResponse => {
                        println!("The game has already started.");
                    },
                    Message::HitResponse {x, y} => {
                        println!("You have hit a ship! ({}, {})", x, y);
                        self.handle_hit_response(x, y);
                        self.send_updated_boards();
                    },
                    Message::MissResponse {x, y} => {
                        println!("You have missed.({}, {})", x, y);
                        self.handle_miss_response(x, y);
                        self.send_updated_boards();
                    },
                    Message::DestroyedResponse {x, y} => {
                        println!("Congratulations! You destroyed an enemy ship!");
                        self.handle_destroyed_response(x, y);
                        self.send_updated_boards();
                    },
                    _ => println!(">>>RECEIVED: {:?}", server_response),
                }

                self.ui_update_sender.send((self.status.clone(), server_response.clone())).unwrap();

            } else if received == Err(TryRecvError::Empty) {
                //println!("Nothing there =(");
            } else if received == Err(TryRecvError::Disconnected) {
                break;
            }

            let input = self.ui_update_receiver.try_recv();

            if let Ok(received) = input {
//                    println!(">>>UI input!");
                match received {
                    Message::GetFeaturesRequest => {
                        self.get_features();
                    },
                    Message::LoginRequest { username } => {
                        self.login(&username);
                    },
                    Message::ReadyRequest => {
                        self.ready();
                    },
                    Message::NotReadyRequest => {
                        self.unready();
                    },
                    Message::ChallengePlayerRequest { username } => {
                        self.challenge(&username);
                    },
                    Message::PlaceShipsRequest { placement } => {
                        self.place_ships( placement );
                    },
                    Message::ShootRequest { x, y } => {
                        self.shoot( x, y );
                    },
                    Message::MoveAndShootRequest { id, direction, x, y } => {
                        self.move_and_shoot( x, y, id, direction );
                    },
                    Message::SurrenderRequest => {
                        self.surrender();
                    },
                    m => panic!("Received illegal request from client: {:?}", m),
                }
            } else {
//                    println!(">>>No UI input: {:?}", input);
            }

            tick.recv().expect("Timer thread died unexpectedly."); // wait for next tick
        }

        self.disconnect_update_sender.send(true).unwrap();
    }
}
