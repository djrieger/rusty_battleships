use std::collections::{HashMap, HashSet};

use message::Message;
use message::Reason;
use board::{PlayerState, Player, PlayerHandle};

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
    // TODO: Check if client is part of a Game and if Game is running
    // return Some(Message::GameAlreadyStartedResponse);
    if let Some(ref username) = player.nickname {
        if let Some(ref mut x) = lobby.get_mut(username) {
            match x.game {
                // TODO: initialize game
                Some(_) => return Some(Message::OkResponse),
                None    => return Some(Message::GameAlreadyStartedResponse)
            }
        }
    }
    panic!("Invalid state or request!");
}

pub fn handle_challenge_player_request(username: String, player: &mut PlayerHandle, player_names: &mut HashSet<String>, lobby: &mut HashMap<String, Player>) -> Option<Message> {
    // TODO: Spiel starten!
    // DONE: Spielerstatus auf Playing setzen
    // DONE: check if other player exists and is ready
    // DONE: return one of OK, NOT_WAITING, NO_SUCH_PLAYER
    // STRUCTURE: Spieler schon im Spiel? => NOT_WAITING
    //TODO: Nicht?
    //Wartet der Spieler? => OkResponse
    //Nicht? => NOT_WAITING
    return Some(Message::OkResponse);
    if let Some(ref challenger_name) = player.nickname {
        if let Some(ref mut challenged_player) = lobby.get_mut(&username) {
            match challenged_player.game {
                Some(_) => return Some(Message::NotWaitingResponse {nickname:username}),
                None    => {
                    match challenged_player.state {
                        PlayerState::Ready => {
                            challenged_player.state = PlayerState::Playing;
                            return Some(Message::OkResponse);
                        },
                        _ => return Some(Message::NotWaitingResponse {nickname:username}),
                    }
                }
            }

        } else {
            return Some(Message::NoSuchPlayerResponse {nickname:username});
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
