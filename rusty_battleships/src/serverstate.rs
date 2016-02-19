use std::collections::{HashMap, HashSet};
use std::time::Duration;
use std::thread;

use message::Message;
use message::{ShipPlacement, Direction};
use message::Reason;
use board::{Board, PlayerState, Player, PlayerHandle, Ship, HitResult};
use game::{Game, GameState};

// From http://stackoverflow.com/a/28392068
macro_rules! hashmap {
    ($( $key: expr => $val: expr ),*) => {{
         let mut map = ::std::collections::HashMap::new();
         $( map.insert($key, $val); )*
         map
    }}
}

macro_rules! get_player {
    ($player:expr, $lobby:expr ) => {{
        assert!($player.nickname.is_some() || $lobby.contains_key($player.nickname.as_ref().unwrap()));
        let name = $player.nickname.as_ref().unwrap();
        let player = $lobby.get_mut(name).unwrap();
        (player, name)
    }};
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

pub fn terminate_player(player_handle: &PlayerHandle, lobby: &mut HashMap<String, Player>, games: &mut Vec<Game>) -> Option<Message> {
    assert!(player_handle.nickname.is_some() || lobby.contains_key(player_handle.nickname.as_ref().unwrap()));
    let name = player_handle.nickname.as_ref().unwrap().clone();
    {
        let mut player = lobby.get_mut(&name).unwrap();
        if player.game.is_some() {
            return Some(Message::GameOverUpdate { victorious: true, reason: Reason::Disconnected });
        }
    }
    lobby.remove(&name);
    return None;
}

pub fn handle_get_features_request() -> Result {
    return Result::respond(Message::FeaturesResponse {
        features: vec!["Awesomeness".to_owned()]
    }, false);
}

pub fn handle_login_request(username: String, player: &mut PlayerHandle, lobby: &mut HashMap<String, Player>) -> Result {
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
        return Result::respond(Message::OkResponse, false);
    }
}


pub fn handle_ready_request(player: &mut PlayerHandle, lobby: &mut HashMap<String, Player>) -> Result {
    let (player, _) = get_player!(player, lobby);
    if player.game.is_some() {
        return Result::respond(Message::InvalidRequestResponse, false);
    } else {
        player.state = PlayerState::Ready;
        return Result::respond(Message::OkResponse, false);
    }
}

pub fn handle_not_ready_request(player_handle: &mut PlayerHandle, lobby: &mut HashMap<String, Player>) -> Result {
    let (player, _) = get_player!(player_handle, lobby);
    match player.game {
        None => {
            player.state = PlayerState::Available;
            return Result::respond(Message::OkResponse, false);
        },
        Some(_) => return Result::respond(Message::GameAlreadyStartedResponse, false),
    }
    panic!("Invalid state or request!");
}

fn initialize_game(player1: &String, player2: &String) -> Game {
    let first_board = Board::new(vec![]);
    let second_board = Board::new(vec![]);

    return Game::new(first_board, second_board, (*player1).clone(), (*player1).clone());
}

pub fn handle_challenge_player_request(challenged_player_name: String, player: &mut PlayerHandle, lobby: &mut HashMap<String, Player>, games: &mut Vec<Game>) -> Result {
    let challenger_name = player.nickname.as_ref().expect("Invalid state, challenging player has no nickname");
    let mut launch_game = false;

    let not_waiting_result = Result::respond(Message::NotWaitingResponse {nickname: challenged_player_name.clone() }, false);

    // Is there a player called challenged_player_name?
    if let Some(ref mut challenged_player) = lobby.get_mut(&challenged_player_name) {
        if challenged_player.game.is_some() {
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
        let new_game = initialize_game(challenger_name, &challenged_player_name);
        lobby.get_mut(challenger_name).unwrap().state = PlayerState::Playing;
        // TODO: Set game reference for both players!!!
        // lobby.get_mut(challenger_name).unwrap().game = Some(&mut new_game);
        games.push(new_game);
        // Tell challenged player about game
        let update_message = Message::GameStartUpdate {nickname: (*challenger_name).clone() };
        // OkResponse for player who issued challenge
        return Result::respond_and_update_single(Message::OkResponse, hashmap![challenged_player_name => vec![update_message]], false);
    }
    panic!("Invalid state or request!");
}

pub fn handle_surrender_request(player: &mut PlayerHandle, lobby: &mut HashMap<String, Player>) -> Result {
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

        opponent_name = requesting_player.game.as_ref().unwrap().get_opponent_name(username).clone();
        requesting_player.game = None;
    }

    let opponent = lobby.get_mut(&opponent_name).expect("Invalid state, opponent not in lobby");
    opponent.game = None;
    // Send GameOver to player and opponent
    let updatemsg = Message::GameOverUpdate {
        victorious:false,
        reason:Reason::Surrendered,
    };
    let updatemsg2 = updatemsg.clone();
    return Result::respond_and_update_single(updatemsg, hashmap![opponent_name => vec![updatemsg2]], false);
}

pub fn handle_report_error_request(errormessage: String, player: &mut PlayerHandle, lobby: &mut HashMap<String, Player>, games: &mut Vec<Game>) -> Result {
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
            if let Some(ref mut game) = user.game {
                let player2_name;
                {
                    player2_name = game.get_opponent_name(username).clone();
                }
                let player_left_update = Message::PlayerLeftUpdate { nickname: (*username).clone() };
                let game_over_update = Message::GameOverUpdate {
                    victorious: true,
                    reason: Reason::Disconnected,
                };
                let mut updates = vec![player_left_update, game_over_update];
                termination_result.updates.insert( player2_name, updates);
            }
        }
        if let Some(ref name) = player2_name {
            let player2 = lobby.get_mut(name).expect("Invalid state, opponent name not in lobby");
            player2.state = PlayerState::Available;
        }
    }

    // Terminate connection to client reporting ErrorRequest
    return termination_result;
}

pub fn handle_place_ships_request(placement: [ShipPlacement; 5], player_handle: &mut PlayerHandle, lobby: &mut HashMap<String, Player>) -> Result {
    let player_name = player_handle.nickname.as_ref().unwrap();
    let player = lobby.get_mut(player_name).unwrap();

    if let Some(ref mut game) = player.game {
        if let GameState::Running = game.state {
            return Result::respond(Message::InvalidRequestResponse, false);
        }

        // Translate placement to ships vector
        let mut ships = vec![];
        let lengths_and_hp = vec![5, 4, 3, 2, 2];
        for (&ship_placement, &length_and_hp) in placement.iter().zip(lengths_and_hp.iter()) {
            let ShipPlacement { x, y, direction } = ship_placement;
            let ship = Ship {
                x: x as isize,
                y: y as isize,
                horizontal: direction == Direction::West || direction == Direction::East, // FIXME?
                length: length_and_hp,
                health_points: length_and_hp,
            };
            ships.push(ship);
        }
        // Get board for current player
        let ref mut board = if *game.player1 == *player_name { &mut game.board2 } else { &mut game.board1 };
        board.ships = ships;
        // Check if new state is valid
        if !board.compute_state() {
            return Result::respond(Message::InvalidRequestResponse, false);
        } else {
            return Result::respond(Message::OkResponse, false);
        }
    }

    return Result::respond(Message::InvalidRequestResponse, false);
}

fn handle_move(game: &mut Game, player_name: &String, movement: (usize, Direction)) -> Option<Result> {
    let (ship_index, direction) = movement;
    if ship_index < 1 || ship_index > 5 {
        // ship index is out of bounds
        return Some(Result::respond(Message::InvalidRequestResponse, false));
    }

    let mut movement_allowed = true;
    let ref mut my_board = if *game.player1 == *player_name { &mut game.board2 } else { &mut game.board1 };
    {
        let ref mut my_ship = my_board.ships[ship_index - 1];
        movement_allowed = my_ship.move_me(direction);
    }

    if !movement_allowed || !my_board.compute_state() {
        return Some(Result::respond(Message::InvalidRequestResponse, false));
    }

    None
}

fn handle_shoot(game: &mut Game, player_name: &String, target_x: u8, target_y: u8) -> Result {
    let opponent_name = game.get_opponent_name(player_name).to_owned();
    let response_msg;
    let mut updates = hashmap![];
    {
        let ref mut opponent_board = if *game.player1 != *player_name { &mut game.board2 } else { &mut game.board1 };
        match opponent_board.hit(target_x as usize, target_y as usize) {
            HitResult::Hit => {
                response_msg = Message::HitResponse { x: target_x, y: target_y };
                updates = hashmap![opponent_name.clone() => vec![Message::EnemyHitUpdate { x: target_x, y: target_y }]];
            },
            HitResult::Miss => {
                response_msg = Message::MissResponse { x: target_x, y: target_y };
                updates = hashmap![opponent_name.clone() => vec![Message::EnemyMissUpdate { x: target_x, y: target_y }]];
            },
            HitResult::Destroyed => {
                if opponent_board.is_dead() {
                    // TODO: terminate_game(won) with appropriate updates
                }
                response_msg = Message::DestroyedResponse { x: target_x, y: target_y };
                // TODO which update for enemy?
            },
        }
    }

    // make turn switch
    game.switch_turns();
    updates.get_mut(&opponent_name).unwrap().push(Message::YourTurnUpdate);
    updates.insert((*player_name).clone(), vec![Message::EnemyTurnUpdate]);
    return Result::respond_and_update_single(response_msg, updates, false);
}

pub fn handle_move_shoot_request(target_coords: (u8, u8), ship_movement: Option<(usize, Direction)>, player_handle: &mut PlayerHandle, lobby: &mut HashMap<String, Player>) -> Result {
    let player_name = player_handle.nickname.as_ref().unwrap();
    let player = lobby.get_mut(player_name).unwrap();

    // TODO update state, update other player, game over, afk,

    // Make sure player has a running game
    if let Some(ref mut game) = player.game {
        if let GameState::Running = game.state {
            if game.my_turn(player_name) {
                // move if requested
                if let Some(movement) = ship_movement {
                    if let Some(result) = handle_move( game, player_name, movement) {
                        return result;
                    }
                }

                let (target_x, target_y) = (target_coords.0 as u8, target_coords.1 as u8);
                return handle_shoot(game, player_name, target_x, target_y);
            } else {
                return Result::respond(Message::NotYourTurnResponse, false);
            }
        }
    }

    return Result::respond(Message::InvalidRequestResponse, false);
}
