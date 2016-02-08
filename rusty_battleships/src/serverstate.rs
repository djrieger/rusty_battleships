use std::collections::{HashMap, HashSet};

use message::Message;
use message::Reason;
use board::{Board, PlayerState, Player, PlayerHandle, Game};

pub fn handle_get_features_request() -> Option<Message> {
    return Some(Message::FeaturesResponse {
        features: vec!["Awesomeness".to_owned()]
    });
}

pub fn handle_login_request(username: String, player: &mut PlayerHandle, player_names: &mut HashSet<String>, lobby: &mut HashMap<String, Player>) -> Option<Message> {
    if username.len() == 0 {
        return Some(Message::InvalidRequestResponse);
    }
    // Determine if we already have a player with name 'username'
    if lobby.contains_key(&username) {
        return Some(Message::NameTakenResponse { nickname: username });
    } else {
        // Update lobby hashtable
        lobby.insert(username.clone(), Player {
            state: PlayerState::Available,
            game: None,
        });
        // Update player struct
        player.nickname = Some(username.clone());
        player_names.insert(username);
        return Some(Message::OkResponse);
    }
}

pub fn handle_ready_request(player: &mut PlayerHandle, player_names: &mut HashSet<String>, lobby: &mut HashMap<String, Player>) -> Option<Message> {
    if let Some(ref username) = player.nickname {
        if let Some(ref mut x) = lobby.get_mut(username) {
            x.state = PlayerState::Ready;
            return Some(Message::OkResponse);
        }
    }
    panic!("Invalid state or request!");
}

pub fn handle_not_ready_request(player: &mut PlayerHandle, player_names: &mut HashSet<String>, lobby: &mut HashMap<String, Player>) -> Option<Message> {
    if let Some(ref username) = player.nickname {
        if let Some(ref mut x) = lobby.get_mut(username) {
            match x.game {
                // TODO: initialize game
                // TODO: Check if Game is running
                Some(_) => return Some(Message::OkResponse),
                None    => return Some(Message::GameAlreadyStartedResponse)
            }
        }
    }
    panic!("Invalid state or request!");
}

fn initialize_game(player1: &String, player2: &String) -> Game {
    let first_board = Board::new(vec![]);
    let second_board = Board::new(vec![]);

    return Game {
        players: ((*player1).clone(), (*player2).clone()),
        boards: (first_board, second_board),
    };
}

pub fn handle_challenge_player_request(challenged_player_name: String, player: &mut PlayerHandle, player_names: &mut HashSet<String>, lobby: &mut HashMap<String, Player>) -> Option<Message> {
    // DONE: Spiel starten!
    // DONE: Spielerstatus auf Playing setzen
    // DONE: check if other player exists and is ready
    // DONE: return one of OK, NOT_WAITING, NO_SUCH_PLAYER
    // STRUCTURE: Spieler schon im Spiel? => NOT_WAITING
    //TODO: Nicht?
    //Wartet der Spieler? => OkResponse
    //Nicht? => NOT_WAITING
    return Some(Message::OkResponse);
    if let Some(ref challenger_name) = player.nickname {
        if let Some(ref mut challenged_player) = lobby.get_mut(&challenged_player_name) {
            match challenged_player.game {
                Some(_) => return Some(Message::NotWaitingResponse {nickname:challenged_player_name}),
                None    => {
                    match challenged_player.state {
                        PlayerState::Ready => {
                            challenged_player.state = PlayerState::Playing;
                            // TODO: Cannot borrow lobby as mutable again, alternative?
                            // lobby.get_mut(challenger_name).unwrap().state = PlayerState::Playing;
                            initialize_game(challenger_name, &challenged_player_name);
                            // TODO save game in some collection
                            // Tell challenged player about game
                            // TODO: Need PlayerHandle for challenged player
                            // challenged_player_handle.to_child_endpoint.send(Message::GameStartUpdate {nickname: challenger_name }); 
                            return Some(Message::OkResponse);
                        },
                        _ => return Some(Message::NotWaitingResponse {nickname: challenged_player_name}),
                    }
                }
            }

        } else {
            return Some(Message::NoSuchPlayerResponse {nickname:challenged_player_name});
        }
    }
    panic!("Invalod state or request!");
}

pub fn handle_surrender_request(player: &mut PlayerHandle, player_names: &mut HashSet<String>, lobby: &mut HashMap<String, Player>) -> Option<Message> {
    // TODO: Tell other player!
    // STRUCTURE: If playing, set available, return GameOverUpdate to both players!
    if let Some(ref username) = player.nickname {
        if let Some(ref mut x) = lobby.get_mut(username) {
            match x.state {
                PlayerState::Playing =>  {
                    x.state = PlayerState::Available;
                    // TODO: Tell other player!
                    return Some(Message::GameOverUpdate {
                        victorious:false,
                        reason:Reason::Surrendered,
                    });
                },
                _ => return Some(Message::InvalidRequestResponse),
            }
        }
    }
    panic!("Invalod state or request!");
}

pub fn handle_report_error_request(errormessage: String, player: &mut PlayerHandle, player_names: &mut HashSet<String>, lobby: &mut HashMap<String, Player>) -> Option<Message> {
    // TODO: Tell other player!
    // TODO: "Reset" players to available state.
    if let Some(ref username) = player.nickname {
        if let Some(ref mut x) = lobby.get_mut(username) {
            println!("{}", errormessage);
            // TODO: Add further debugging information!
            x.state = PlayerState::Available;
            if let Some(ref mut g) = x.game {
                if &(g.players.0) == username {
                    // We're the left player
                    // TODO: Send message to other player and set them to available
                } else {
                    // We're the right player
                    // TODO: Send message to other player and set them to available
                }
            }
        }
    }
    panic!("Invalod state or request!");
}
