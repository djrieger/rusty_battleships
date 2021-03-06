use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use rusty_battleships::message::{ShipPlacement, Direction, Message, Reason};
use rusty_battleships::board::{Board, PlayerState, Player, PlayerHandle, HitResult};
use rusty_battleships::ship::Ship;
use rusty_battleships::game::Game;

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
    pub fn add_list_lobby_for(&mut self, lobby: &HashMap<String, Player>, username: &String) {
        let player_updates = self.updates.entry(username.clone()).or_insert(vec![]);
        player_updates.extend(list_lobby_for(lobby, username));
    }

    pub fn add_update_lobby(&mut self, lobby: &HashMap<String, Player>, new_update: Message) {
        add_update_lobby(lobby, &mut self.updates, new_update);
    }

    pub fn add_update_lobby_except(&mut self, lobby: &HashMap<String, Player>,
            except_player: &String, new_update: Message) {
        add_update_lobby_except(lobby, except_player, &mut self.updates, new_update);
    }

    pub fn respond(response: Message, terminate_connection: bool) -> Result {
        return Result::respond_and_update_single(response, HashMap::new(), terminate_connection);
    }

    pub fn respond_and_update_single(response: Message, updates: HashMap<String, Vec<Message>>, terminate_connection: bool) -> Result {
        return Result {
            response: Some(response),
            updates: updates,
            terminate_connection: terminate_connection,
        }
    }
}

/// Send update to all players who are not currently playing a game.
fn add_update_lobby(lobby: &HashMap<String, Player>,
        updates: &mut HashMap<String, Vec<Message>>, new_update: Message) {
    for player in lobby.keys()
                       .filter(|p| lobby.get(*p).unwrap().game.is_none()) {
        let player_updates = updates.entry(player.clone()).or_insert(vec![]);
        player_updates.push(new_update.clone());
    }
}

fn add_update_lobby_except(lobby: &HashMap<String, Player>, except_player: &String,
        updates: &mut HashMap<String, Vec<Message>>, new_update: Message) {
    for player in lobby.keys()
                       .filter(|p| (*p) != except_player)
                       .filter(|p| lobby.get(*p).unwrap().game.is_none()) {
        let player_updates = updates.entry(player.clone()).or_insert(vec![]);
        player_updates.push(new_update.clone());
    }
}

fn list_lobby_for(lobby: &HashMap<String, Player>, username: &String) -> Vec<Message> {
    let mut result = vec![];

    for (name, player) in lobby {
        if name != username {
            result.push(Message::PlayerJoinedUpdate { nickname: name.clone() });
            if player.state == PlayerState::Ready {
                result.push(Message::PlayerReadyUpdate { nickname: name.clone() });
            }
        }
    }

    return result;
}

fn merge_updates(updates: &mut HashMap<String, Vec<Message>>, mut additional_updates: HashMap<String, Vec<Message>>) {
    for (receiver_name, update_vec) in additional_updates.drain() {
        let receiver_updates = updates.entry(receiver_name.clone()).or_insert(vec![]);
        receiver_updates.extend(update_vec);
    }
}

/// Terminates the game and returns the update messages to send
fn terminate_game(games: &mut Vec<Rc<RefCell<Game>>>, game: Rc<RefCell<Game>>,
        lobby: &mut HashMap<String, Player>, active_player_name: &String, victorious: bool,
        reason: Reason) -> HashMap<String, Vec<Message>> {
    let game_ref = (*game).borrow();
    let opponent_name = game_ref.get_opponent_name(active_player_name);
    let mut updates = HashMap::new();

    // delete game for active player
    {
        let p1 = lobby.get_mut(active_player_name).unwrap();
        p1.game = None;
        p1.state = PlayerState::Available;
    }
    if reason != Reason::Disconnected {
        let mut active_player_updates = vec![Message::GameOverUpdate {
            reason: reason,
            victorious: victorious
        }];
        active_player_updates.extend(list_lobby_for(lobby, active_player_name));
        updates.insert(active_player_name.clone(), active_player_updates);
    }

    // delete game for opponent
    {
        let p2 = lobby.get_mut(opponent_name).unwrap();
        p2.game = None;
        p2.state = PlayerState::Available;
    }
    let mut opponent_updates = vec![Message::GameOverUpdate {
        reason: reason,
        victorious: !victorious
    }];
    opponent_updates.extend(list_lobby_for(lobby, opponent_name));
    updates.insert(opponent_name.clone(), opponent_updates);

    // delete game
    games.retain(|g| (*(*g).borrow()) != (*game_ref));

    return updates;
}

pub fn terminate_player(name: &String, lobby: &mut HashMap<String, Player>,
        games: &mut Vec<Rc<RefCell<Game>>>) -> HashMap<String, Vec<Message>> {
    assert!(lobby.contains_key(name));
    let game;
    let mut result;

    {
        let player = lobby.get(name).unwrap();
        if let Some(ref game_ref) = player.game {
            game = Some(game_ref.clone());
        } else {
            game = None;
        }
    }

    if let Some(ref game_ref) = game {
        result = terminate_game(games, game_ref.clone(), lobby, &name, false, Reason::Disconnected);
    } else {
        result = HashMap::new();
    }

    lobby.remove(name);

    add_update_lobby(lobby, &mut result, Message::PlayerLeftUpdate{ nickname: name.clone() });

    return result;
}

pub fn handle_get_features_request() -> Result {
    return Result::respond(Message::FeaturesResponse {
        features: vec!["UDP Server Discovery".to_owned()]
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
        let mut result = Result::respond(Message::OkResponse, false);

        result.add_list_lobby_for(lobby, &username);
        result.add_update_lobby(lobby, Message::PlayerJoinedUpdate { nickname: username.clone() });

        // Update lobby hashtable
        lobby.insert(username.clone(), Player {
            state: PlayerState::Available,
            game: None,
        });
        // Update player struct
        player.nickname = Some(username.clone());

        return result;
    }
}


pub fn handle_ready_request(username: &String, lobby: &mut HashMap<String, Player>) -> Result {
    let mut result;

    {
        let mut player = lobby.get_mut(username).unwrap();

        if player.game.is_some() {
            return Result::respond(Message::InvalidRequestResponse, false);
        } else {
            player.state = PlayerState::Ready;
            result = Result::respond(Message::OkResponse, false);
        }
    }

    result.add_update_lobby_except(lobby, username,
        Message::PlayerReadyUpdate { nickname: username.clone() });

    return result;
}

pub fn handle_not_ready_request(username: &String, lobby: &mut HashMap<String, Player>) -> Result {
    let mut result;

    {
        let mut player = lobby.get_mut(username).unwrap();

        if player.game.is_some() {
            return Result::respond(Message::GameAlreadyStartedResponse, false);
        } else {
            player.state = PlayerState::Available;
            result = Result::respond(Message::OkResponse, false);
        }
    }

    result.add_update_lobby_except(lobby, username,
        Message::PlayerNotReadyUpdate { nickname: username.clone() });

    return result;
}

fn initialize_game(player1: &String, player2: &String) -> Rc<RefCell<Game>> {
    // Unwrapping is safe here since boards with no ships are always valid
    let first_board = Board::try_create(vec![], true).unwrap();
    let second_board = Board::try_create(vec![], true).unwrap();

    return Rc::new(RefCell::new(Game::new(first_board, second_board, (*player1).clone(), (*player2).clone())));
}

pub fn handle_challenge_player_request(challenged_player_name: String, challenger_name: &String,
        lobby: &mut HashMap<String, Player>, games: &mut Vec<Rc<RefCell<Game>>>) -> Result {
    // Is there a player called challenged_player_name?
    if let Some(ref mut challenged_player) = lobby.get_mut(&challenged_player_name) {
        if challenged_player.game.is_some() || challenged_player.state != PlayerState::Ready {
            // Challenged player is already in a game or not ready anymore
            return Result::respond(Message::NotWaitingResponse {
                nickname: challenged_player_name.clone() }, false);
        }

        // Challenged player is not in a game and Ready
        challenged_player.state = PlayerState::Playing;
    } else {
        return Result::respond(Message::NoSuchPlayerResponse {nickname:challenged_player_name}, false);
    }

    // Create and save new game
    let new_game = initialize_game(challenger_name, &challenged_player_name);
    lobby.get_mut(challenger_name).unwrap().state = PlayerState::Playing;
    // Set game reference for both players
    lobby.get_mut(challenger_name).unwrap().game = Some(new_game.clone());
    lobby.get_mut(&challenged_player_name).unwrap().game = Some(new_game.clone());
    games.push(new_game);
    // tell challenged player about the game
    let update_message = Message::GameStartUpdate {nickname: (*challenger_name).clone() };
    // OkResponse for player who issued challenge
    return Result::respond_and_update_single(Message::OkResponse, hashmap![challenged_player_name => vec![update_message]], false);
}

pub fn handle_surrender_request(username: &String, lobby: &mut HashMap<String, Player>,
        games: &mut Vec<Rc<RefCell<Game>>>) -> Result {
    let game;

    {
        let player = lobby.get(username).unwrap();
        if let Some(ref game_ref) = player.game {
            game = Some(game_ref.clone());
        } else {
            game = None;
        }
    }

    if let Some(ref game) = game {
        let updates = terminate_game(games, game.clone(), lobby, username, false,
                                     Reason::Surrendered);
        return Result::respond_and_update_single(Message::OkResponse, updates, false);
    } else {
        return Result::respond(Message::NotYourTurnResponse, false);
    }
}

pub fn handle_report_error_request(errormessage: String, player: &mut PlayerHandle, lobby: &mut HashMap<String, Player>, games: &mut Vec<Rc<RefCell<Game>>>) -> Result {
    let mut termination_result = Result {
        response: None,
        updates: HashMap::new(),
        terminate_connection: true,
    };

    println!("Client reported error: {}", errormessage);

    // For registered players we need to terminate a running game, if any
    if let Some(ref username) = player.nickname {
        termination_result.updates = terminate_player(username, lobby, games);
    }

    // Terminate connection to client reporting ErrorRequest
    return termination_result;
}

fn placement2ships(placement: [ShipPlacement; 5]) -> Vec<Ship> {
    let mut ships = vec![];
    let lengths_and_hp = vec![5, 4, 3, 2, 2];
    for (&ship_placement, &length_and_hp) in placement.iter().zip(lengths_and_hp.iter()) {
        let ShipPlacement { x, y, direction } = ship_placement;
        let ship = Ship {
            x: x as isize,
            y: y as isize,
            direction: direction,
            length: length_and_hp,
            health_points: length_and_hp,
        };
        ships.push(ship);
    }
    return ships;
}

pub fn handle_place_ships_request(placement: [ShipPlacement; 5], player_name: &String, lobby: &mut HashMap<String, Player>) -> Result {
    let player = lobby.get_mut(player_name).unwrap();

    if let Some(ref game) = player.game {
        if (*game).borrow().is_running() {
            return Result::respond(Message::InvalidRequestResponse, false);
        }

        let ships = placement2ships(placement);
        if Board::try_create(ships.clone(), false).is_none() {
            return Result::respond(Message::InvalidRequestResponse, false);
        }
        let opponent_ready;
        {
            println!("Computing initial placement for {}:", player_name);
            let mut game_ref = (*game).borrow_mut();
            *game_ref.get_board(player_name) = Board::try_create(ships, true).unwrap();
            opponent_ready = game_ref.get_opponent_board(player_name).has_ships();
        }

        let mut game_ref = (*game).borrow_mut();
        // opponent also done placing ships?
        if opponent_ready {
            game_ref.start();
            let mut result = Result::respond(Message::OkResponse, false);
            result.updates.insert(game_ref.get_active_player(), vec![Message::YourTurnUpdate]);
            result.updates.insert(game_ref.get_waiting_player(), vec![Message::EnemyTurnUpdate]);
            return result;
        } else {
            return Result::respond(Message::OkResponse, false);
        }
    }

    return Result::respond(Message::InvalidRequestResponse, false);
}

/**
 * Tries performing the requested movement
 * If the movement was invalid (invalid ship index, out of bounds etc.) None is returned
 * Otherwise Some is returned with a vector of visibility updates
 */
fn handle_move(game: &mut Game, player_name: &String, movement: (usize, Direction)) -> bool {
    let (ship_index, direction) = movement;
    if ship_index > 4 {
        println!("ship index out of bounds");
        return false;
    }

    let ref mut my_board = game.get_board(player_name);
    return my_board.move_ship(ship_index as u8, direction);
}

fn handle_shoot(games: &mut Vec<Rc<RefCell<Game>>>, game: Rc<RefCell<Game>>,
        lobby: &mut HashMap<String, Player>, player_name: &String, target_x: u8,
        target_y: u8) -> Result {
    let game_over;
    let hit_result;
    let response_msg;
    let mut updates = HashMap::new();
    let opponent_name;

    // evaluate shot
    {
        let mut game_ref = (*game).borrow_mut();
        opponent_name = game_ref.get_opponent_name(player_name).to_owned();

        // enemy visibility updates
        {
            let ref mut opponent_board = game_ref.get_opponent_board(player_name);
            println!("Shooting on {}'s board at {}:{}:", opponent_name, target_x, target_y);
            hit_result = opponent_board.hit(target_x as usize, target_y as usize);
            let opponent_updates = hashmap![player_name.clone() => opponent_board.pop_updates()];
            game_over = opponent_board.is_dead();
            merge_updates(&mut updates, opponent_updates);
        }

        // my visibility updates
        {
            let ref mut my_board = game_ref.get_board(player_name);
            let my_updates = hashmap![opponent_name.clone() => my_board.pop_updates()];
            merge_updates(&mut updates, my_updates);
        }

        // hit updates
        match hit_result {
            HitResult::Hit => {
                response_msg = Message::HitResponse { x: target_x, y: target_y };
                merge_updates(&mut updates, hashmap![opponent_name.clone() => vec![Message::EnemyHitUpdate { x: target_x, y: target_y }]]);
            },
            HitResult::Miss => {
                response_msg = Message::MissResponse { x: target_x, y: target_y };
                merge_updates(&mut updates, hashmap![opponent_name.clone() => vec![Message::EnemyMissUpdate { x: target_x, y: target_y }]]);
            },
            HitResult::Destroyed => {
                response_msg = Message::DestroyedResponse { x: target_x, y: target_y };
                if !game_over {
                    merge_updates(&mut updates, hashmap![opponent_name.clone() => vec![Message::EnemyHitUpdate { x: target_x, y: target_y }]]);
                }
            },
        }
    }

    if game_over {
        updates = terminate_game(games, game, lobby, player_name, true, Reason::Obliterated);
        return Result::respond_and_update_single(response_msg, updates, false);
    } else {
        let mut game_ref = (*game).borrow_mut();
        game_ref.switch_turns();
        return Result::respond_and_update_single(response_msg, updates, false);
    }
}

pub fn handle_move_shoot_request(target_coords: (u8, u8),
        ship_movement: Option<(usize, Direction)>, player_name: &String,
        lobby: &mut HashMap<String, Player>, games: &mut Vec<Rc<RefCell<Game>>>) -> Result {
    let game;

    {
        // get reference to active game, if there is any
        let ref active_game = lobby.get_mut(player_name).unwrap().game;

        if active_game.is_none() {
            return Result::respond(Message::NotYourTurnResponse, false);
        }

        game = active_game.as_ref().unwrap().clone();
    }

    {
        // check if in valid state and handle move, if requested
        let mut game_ref = (*game).borrow_mut();

        if !game_ref.is_running() {
            return Result::respond(Message::InvalidRequestResponse, false);
        }

        if !game_ref.my_turn(player_name) {
            return Result::respond(Message::NotYourTurnResponse, false);
        }

        // move if requested
        if let Some(movement) = ship_movement {
            if !handle_move(&mut game_ref, player_name, movement) {
                // Some(updates) => visibility_updates = updates,
                return Result::respond(Message::InvalidRequestResponse, false);
            }
        }
    }

    // handle shot
    return handle_shoot(games, game, lobby, player_name, target_coords.0, target_coords.1);
}

pub fn handle_afk(game: Rc<RefCell<Game>>, lobby: &mut HashMap<String, Player>,
        games: &mut Vec<Rc<RefCell<Game>>>) -> HashMap<String, Vec<Message>> {
    let active_player;

    {
        let mut game_ref = (*game).borrow_mut();
        active_player = game_ref.get_active_player();
        let strike_count = game_ref.get_active_player_afk_count();

        if strike_count > 1 {
            let opponent_name = game_ref.get_opponent_name(&game_ref.get_active_player()).clone();
            game_ref.dec_active_player_afk_count();
            game_ref.switch_turns();
            return hashmap![
                active_player => vec![Message::AfkWarningUpdate { strikes: strike_count - 1 }],
                opponent_name => vec![Message::EnemyAfkUpdate { strikes: strike_count - 1 }]
            ];
        }
    }

    return terminate_game(games, game, lobby, &active_player, false, Reason::Afk);
}
