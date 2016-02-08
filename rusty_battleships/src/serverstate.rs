use std::collections::{HashMap, HashSet};

use message::Message;
use message::Reason;
use board::{Board, PlayerState, Player, PlayerHandle, Game};

// From http://stackoverflow.com/a/28392068
macro_rules! hashmap {
    ($( $key: expr => $val: expr ),*) => {{
         let mut map = ::std::collections::HashMap::new();
         $( map.insert($key, $val); )*
         map
    }}
}

pub struct Result {
    pub response: Option<Message>,
    pub updates: HashMap<String, Vec<Message>>,
    pub terminate_connection: bool,
}

impl Result {
    pub fn respond(response: Message, terminate_connection: bool) -> Result {
        return Result::respond_and_update_single(response, HashMap::new(), false);
    }

    pub fn respond_and_update_single(response: Message, updates: HashMap<String, Vec<Message>>, terminate_connection: bool) -> Result {
        return Result {
            response: Some(response),
            updates: updates,
            terminate_connection: terminate_connection,
        }
    }
}


pub fn handle_get_features_request() -> Result {
    return Result::respond(Message::FeaturesResponse {
        features: vec!["Awesomeness".to_owned()]
    }, false);
}

pub fn handle_login_request(username: String, player: &mut PlayerHandle, player_names: &mut HashSet<String>, lobby: &mut HashMap<String, Player>) -> Result {
    if username.len() == 0 {
        return Result::respond(Message::InvalidRequestResponse, false);
    }
    // Determine if we already have a player with name 'username'
    if lobby.contains_key(&username) {
        return Result::respond(Message::NameTakenResponse { nickname: username }, false);
    } else {
        // Update lobby hashtable
        lobby.insert(username.clone(), Player {
            state: PlayerState::Available,
            game: None,
        });
        // Update player struct
        player.nickname = Some(username.clone());
        player_names.insert(username);
        return Result::respond(Message::OkResponse, false);
    }
}

macro_rules! get_player {
    ($player:expr, $lobby:expr ) => {{
        if $player.nickname.is_none() || !$lobby.contains_key($player.nickname.as_ref().unwrap()) {
            panic!("Invalid state. User has no nickname or nickname not in lobby HashTable");
        }
        let x = $lobby.get_mut($player.nickname.as_ref().unwrap()).unwrap();
        x
    }};
}

pub fn handle_ready_request(player: &mut PlayerHandle, player_names: &mut HashSet<String>, lobby: &mut HashMap<String, Player>) -> Result {
    let x = get_player!(player, lobby);
    x.state = PlayerState::Ready;
    return Result::respond(Message::OkResponse, false);
}

pub fn handle_not_ready_request(player: &mut PlayerHandle, player_names: &mut HashSet<String>, lobby: &mut HashMap<String, Player>) -> Result {
    if let Some(ref username) = player.nickname {
        if let Some(ref mut x) = lobby.get_mut(username) {
            match x.game {
                // TODO: initialize game
                // TODO: Check if Game is running
                Some(_) => return Result::respond(Message::OkResponse, false),
                None    => return Result::respond(Message::GameAlreadyStartedResponse, false)
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

pub fn handle_challenge_player_request(challenged_player_name: String, player: &mut PlayerHandle, player_names: &mut HashSet<String>, lobby: &mut HashMap<String, Player>, games: &mut Vec<Game>) -> Result {
    let challenger_name = player.nickname.as_ref().expect("Invalid state, challenging player has no nickname");
    let mut launch_game = false;

    let not_waiting_result = Result::respond(Message::NotWaitingResponse {nickname: challenged_player_name.clone() }, false);

    // Is there a player called challenged_player_name?
    if let Some(ref mut challenged_player) = lobby.get_mut(&challenged_player_name) {
        if let Some(_) = challenged_player.game {
            // Challenged player is already in a game -> NotWaiting
            return not_waiting_result;
        }
        if let PlayerState::Ready = challenged_player.state  {
            // Challenged player is not in a game and Ready
            challenged_player.state = PlayerState::Playing;
            launch_game = true;
        } else {
            return not_waiting_result;
        }
    } else {
        return Result::respond(Message::NoSuchPlayerResponse {nickname:challenged_player_name}, false);
    }

    if launch_game {
        // Create and save new game
        games.push( initialize_game(challenger_name, &challenged_player_name));
        lobby.get_mut(challenger_name).unwrap().state = PlayerState::Playing;
        // Tell challenged player about game
        let update_message = Message::GameStartUpdate {nickname: (*challenger_name).clone() }; 
        // OkResponse for player who issued challenge
        return Result::respond_and_update_single(Message::OkResponse, hashmap![challenged_player_name => vec![update_message]], false);
    }
    panic!("Invalod state or request!");
}

pub fn handle_surrender_request(player: &mut PlayerHandle, player_names: &mut HashSet<String>, lobby: &mut HashMap<String, Player>) -> Result {
    // STRUCTURE: If playing, set available, return GameOverUpdate to both players!
    let username = player.nickname.as_ref().expect("Invalid state, player has no nickname");
    let opponent_name;
    {
        let requesting_player = lobby.get_mut(username).expect("Invalid state, requesting player not in lobby");
        match requesting_player.state {
            PlayerState::Playing =>  {
                requesting_player.state = PlayerState::Available;
            },
            _ => return Result::respond(Message::InvalidRequestResponse, false),
        }

        opponent_name = requesting_player.game.unwrap().get_opponent_name(username);
        requesting_player.game = None;
    }

    let opponent = lobby.get_mut(opponent_name).expect("Invalid state, opponent not in lobby");
    opponent.game = None;
    // Send GameOver to player and opponent
    let updatemsg = Message::GameOverUpdate {
        victorious:false,
        reason:Reason::Surrendered,
    };
    let updatemsg2 = Message::GameOverUpdate {
        victorious:false,
        reason:Reason::Surrendered,
    };
    return Result::respond_and_update_single(updatemsg, hashmap![(*opponent_name).clone() => vec![updatemsg2]], false);
}

fn terminate_player() {
    // TODO: What to do here?
    // Remove from lobby HashTable
    // What else?
}

pub fn handle_report_error_request(errormessage: String, player: &mut PlayerHandle, player_names: &mut HashSet<String>, lobby: &mut HashMap<String, Player>) -> Result {
    let mut termination_result: Result = return Result {
        response: None,
        updates: HashMap::new(), 
        terminate_connection: true,
    };

    // For registered players we need to terminate a running game, if any
    if let Some(ref username) = player.nickname {
        let mut player2_name : Option<String> = None;
        {
            let mut user = lobby.get_mut(username).expect("Invalid state, requesting player not in lobby");
            println!("Client {} reported the following error: {}", username, errormessage);
            // player in game with player2?
            if let Some(game) = user.game {
                let player2_name = Some(game.get_opponent_name(username));
                let player_left_update = Message::PlayerLeftUpdate { nickname: (*username).clone() };
                let game_over_update = Message::GameOverUpdate {
                    victorious: false,
                    reason: Reason::Disconnected,
                };
                termination_result.updates.insert( (*player2_name.unwrap()).clone(), vec![player_left_update, game_over_update]);
                // TODO: remove game from games
            }
        } 
        if let Some(ref name) = player2_name {
            let player2 = lobby.get_mut(name).expect("Invalid state, opponent name not in lobby");
            // TODO: change this to player2.set_available(...), only partly implemented in board.rs so
            // far, not tested
            player2.state = PlayerState::Available;
        }
    }

    // Terminate connection to client reporting ErrorRequest
    return termination_result;
}
