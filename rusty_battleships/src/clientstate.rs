use std::io::{BufReader, BufWriter, Write};
use std::net::{TcpStream};


use message::{serialize_message, deserialize_message, Message};
use clientlobby::ClientLobby;

fn send_message(msg: Message, stream: &mut BufWriter<TcpStream>) {
    let serialized_msg = serialize_message(msg);
    stream.write(&serialized_msg[..]).unwrap();
    stream.flush();
}

pub struct state {
    lobby : ClientLobby,
    //game : ClientGame;  <--- LATER TODAY
    buff_reader : BufReader<TcpStream>,
    buff_writer : BufWriter<TcpStream>,
}

impl state {

    pub fn new (buff_reader: BufReader<TcpStream>, buff_writer: BufWriter<TcpStream>) -> state {
        state {
            lobby : ClientLobby::new(),
            buff_reader : buff_reader,
            buff_writer : buff_writer,
        }
    }

    /* Requests features. */
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

    pub fn challenge(&mut self, opponent: &str) -> bool {
        if self.lobby.get_player_list().contains(&String::from(opponent)) {
            send_message(Message::ChallengePlayerRequest { username: String::from(opponent) }, &mut self.buff_writer);
            let server_response = deserialize_message(&mut self.buff_reader);
            if server_response.is_err() {
                return false;
            } else {
                return true;
            }
        } else {
            return false;
        }
    }
}
