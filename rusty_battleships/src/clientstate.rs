use std::io::{BufReader, BufWriter, Write};
use std::net::{TcpStream};


use message::{serialize_message, deserialize_message, Message};
use clientlobby::ClientLobby;

fn send_message(msg: Message, stream: &mut BufWriter<TcpStream>) {
    let serialized_msg = serialize_message(msg);
    stream.write(&serialized_msg[..]).unwrap();
    stream.flush();
}

pub struct State {
    pub lobby : ClientLobby,
    //game : ClientGame;  <--- LATER TODAY
    buff_reader : BufReader<TcpStream>,
    buff_writer : BufWriter<TcpStream>,
}

impl State {

    pub fn new (buff_reader: BufReader<TcpStream>, buff_writer: BufWriter<TcpStream>) -> State {
        State {
            lobby : ClientLobby::new(),
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
                    Message::OkResponse => {println!("FOUND {:?}!", opponent);return true},
                    _ => return false,
                };
            }
        } else {
            return false;
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
//            match server_response { //May contain traces of state transisions
//                Message::OkResponse => println!("You're now playing with {:?}", opp);
//            }
            println!("{:?}", server_response.unwrap());
        }
    }
}
