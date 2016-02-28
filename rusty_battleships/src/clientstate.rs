use std::io::{self, BufReader, BufWriter, Write, Read, stdin, BufRead};
use std::net::{TcpStream};
use std::sync::mpsc::{self, Sender};
use std::thread;
use std::time::Duration;

//extern crate argparse;
//use argparse::{ArgumentParser, Print, Store};

use message::{serialize_message, deserialize_message, Message, ShipPlacement, Direction};
use clientlobby::ClientLobby;
use game::Game;
use board::Board;
use board::Ship;

fn send_message(msg: Message, stream: &mut BufWriter<TcpStream>) {
    let serialized_msg = serialize_message(msg);
    stream.write(&serialized_msg[..]).unwrap();
    stream.flush();
}

/*Tries to read from the TCP stream. If there's no message, it waits patiently.*/
pub fn tcp_poll(br: &mut BufReader<TcpStream>, tx: Sender<Message>) {
    println!("Waiting for a message");
    //This can take a while!
    let msg_from_server = deserialize_message(br);
    if msg_from_server.is_err() {
        panic!("FUUUUU!");
    }
    tx.send(msg_from_server.unwrap()).unwrap();
}

pub struct State {
    pub lobby : ClientLobby,
    game : Option<Game>,
    opponent : String,
    //game : ClientGame;  <--- LATER TODAY
    buff_reader : BufReader<TcpStream>,
    buff_writer : BufWriter<TcpStream>,
}

impl State {

    pub fn new (buff_reader: BufReader<TcpStream>, buff_writer: BufWriter<TcpStream>) -> State {
        State {
            lobby : ClientLobby::new(),
            game : None,
            opponent : String::from("None"),
            buff_reader : buff_reader,
            buff_writer : buff_writer,
        }
    }

    pub fn get_features(&mut self) -> bool {
        send_message(Message::GetFeaturesRequest, &mut self.buff_writer);
        let server_response = deserialize_message(&mut self.buff_reader);
        if server_response.is_err() {
            return false;
        } else {
            return true;
        }
    }

    pub fn login(&mut self, nickname: &str) -> bool {
        send_message(Message::LoginRequest { username: String::from(nickname) }, &mut self.buff_writer);
        let server_response = deserialize_message(&mut self.buff_reader);
        if server_response.is_err() {
            return false;
        } else {
            self.lobby.set_player_name(nickname);
            return true;
        }
    }

    pub fn ready(&mut self) -> bool {
        send_message(Message::ReadyRequest, &mut self.buff_writer);
        let server_response = deserialize_message(&mut self.buff_reader);
        if server_response.is_err() {
            return false;
        } else {
            return true;
        }
    }

    /* Sends a challenge to the server, if and only if the opponent is in the ready-and-waiting-list */
    pub fn challenge(&mut self, opponent: &str) -> bool {
        if self.lobby.player_name != opponent {
        //if self.lobby.player_list.contains(&String::from(opponent)) {
            send_message(Message::ChallengePlayerRequest { username: String::from(opponent) }, &mut self.buff_writer);
            let server_response = deserialize_message(&mut self.buff_reader);
            if server_response.is_err() {
                return false;
            } else {
                match server_response.unwrap() {
                    Message::OkResponse => {
                        println!("FOUND {:?}!", opponent);
                        return true;
                    },
                    _ => return false,
                };
            }
        } else {
            return false;
        }
    }

    pub fn place_ships(&mut self) -> bool {
        //Dummy Values
        //TODO:Ask user!
        let ship_placements0 = ShipPlacement { x: 0, y: 0, direction: Direction::East};
        let ship_placements1 = ShipPlacement { x: 0, y: 1, direction: Direction::East};
        let ship_placements2 = ShipPlacement { x: 0, y: 2, direction: Direction::East};
        let ship_placements3 = ShipPlacement { x: 0, y: 3, direction: Direction::East};
        let ship_placements4 = ShipPlacement { x: 0, y: 4, direction: Direction::East};
        let ship_placements : [ShipPlacement; 5] = [ship_placements0, ship_placements1, ship_placements2, ship_placements3, ship_placements4];
        println!("{:?}", ship_placements);
        println!("REQUEST {:?}", Message::PlaceShipsRequest { placement: ship_placements });


        send_message(Message::PlaceShipsRequest { placement: ship_placements }, &mut self.buff_writer);
        let server_response =  deserialize_message(&mut self.buff_reader);
        if server_response.is_err() {
            return false;
        } else {
            match server_response.unwrap() {
                Message::OkResponse => {
                    let mut ships = Vec::<Ship>::new();
                    ships.push(Ship {x: 0, y: 0, length: 5, horizontal: true, health_points: 5});
                    ships.push(Ship {x: 0, y: 1, length: 4, horizontal: true, health_points: 4});
                    ships.push(Ship {x: 0, y: 2, length: 3, horizontal: true, health_points: 3});
                    ships.push(Ship {x: 0, y: 3, length: 2, horizontal: true, health_points: 2});
                    ships.push(Ship {x: 0, y: 4, length: 2, horizontal: true, health_points: 2});
                    let myboard = Board::new(ships);
                    let oppboard = Board::new(Vec::<Ship>::new());
                    self.game = Some(Game::new(myboard, oppboard, self.lobby.player_name.clone(), self.opponent.clone()));
                    return true;
                },
                _ => return false,
            }
        }
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



    /* Main loop; does most of the work. Main-Function should hand over control to this function as
    soon as a tcp connection has been established.*/
    pub fn handle_communication(&mut self/*, br: BufReader<TcpStream>, bw: BufWriter<TcpStream>*/) {
        println!("Hello! Please state your desired Username.");
        let mut stdin = stdin();
        let nickname = stdin.lock().lines().next().unwrap().unwrap();

        if self.login(&nickname) {
            println!("Logged in with playername {:?}", self.lobby.player_name);
        } else {
            println!("oops");
        }

        if self.ready() {
            println!("Your are now ready to be challenged.");
        } else {
            println!("Not ready. Oops.");
        }

        let opp = &"test2";
        let mut playing = false;
        let mut challenge_failed = false;

        if self.challenge(opp) {
            println!("You're now playing with {:?}", opp);
            playing = true;
        } else {
            challenge_failed = true;
            println!("Player not found: {:?}", opp);
            //state.handle_communication();
        }

        if playing {
            if self.place_ships() {
                println!("Ship placement succesful!");
            } else {
                println!("Ship placement failed!");
            }
        } else if challenge_failed {
            let (tx, rx) = mpsc::channel();
            let mut one_time_reader = BufReader::<TcpStream>::new(self.buff_reader.get_ref().try_clone().unwrap());
            thread::spawn(move || tcp_poll(&mut one_time_reader, tx));
            loop {
                println!("Checking for an incoming challenge.");
                if let Ok(server_response) = rx.try_recv() {
                    println!("Oh, a message for me!");
                    match server_response { //May contain traces of state transisions
                        _ => println!("Message received: {:?}", server_response),
                    }
                    println!("{:?}", server_response);
                } else {
                    thread::sleep(Duration::new(1, 0));
                }
            }

            /*
            loop {
                println!("Checking for challenges...");
                if let (true, opponent) = state.have_i_been_challenged() {
                    println!("Thou art being challenged by the fiendish captain {:?}", opponent);
                    println!("May the sea be with you, captain");
                } else {
                    println!("None so far...");
                    std::thread::sleep(std::time::Duration::new(1, 0));
                }
            }
            */
        }

    }

}
