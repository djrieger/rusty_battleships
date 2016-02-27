use std::io::{BufReader, BufWriter, Write};
use std::net::{TcpStream};

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
        let ship_placements0 = ShipPlacement { x: 1, y: 1, direction: Direction::East};
        let ship_placements1 = ShipPlacement { x: 1, y: 2, direction: Direction::East};
        let ship_placements2 = ShipPlacement { x: 1, y: 3, direction: Direction::East};
        let ship_placements3 = ShipPlacement { x: 1, y: 4, direction: Direction::East};
        let ship_placements4 = ShipPlacement { x: 1, y: 5, direction: Direction::East};
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
                    ships.push(Ship {x: 1, y: 1, length: 5, horizontal: true, health_points: 5});
                    ships.push(Ship {x: 1, y: 2, length: 4, horizontal: true, health_points: 4});
                    ships.push(Ship {x: 1, y: 3, length: 3, horizontal: true, health_points: 3});
                    ships.push(Ship {x: 1, y: 4, length: 2, horizontal: true, health_points: 2});
                    ships.push(Ship {x: 1, y: 5, length: 2, horizontal: true, health_points: 2});
                    let myboard = Board::new(ships);
                    let oppboard = Board::new(Vec::<Ship>::new());
                    self.game = Some(Game::new(myboard, oppboard, self.lobby.player_name.clone(), self.opponent.clone()));
                    return true;
                },
                _ => return false,
            }
        }
    }

    /* Main loop; does most of the work. Main-Function should hand over control to this function as
    soon as a tcp connection has been established.*/
    pub fn handle_communication(&mut self) {
        loop {
            let server_response = deserialize_message(&mut self.buff_reader);
            if server_response.is_err() {
                return;
            }
            match server_response { //May contain traces of state transisions
                _ => println!("Message received: {:?}", server_response),
            }
            println!("{:?}", server_response.unwrap());
        }
    }
}
