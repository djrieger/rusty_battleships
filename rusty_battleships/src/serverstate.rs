use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

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

fn add_update_all(lobby: &HashMap<String, Player>, updates: &mut HashMap<String, Vec<Message>>,
        new_update: Message) {
    for player in lobby.keys() {
        let player_updates = updates.entry(player.clone()).or_insert(vec![]);
        player_updates.push(new_update.clone());
    }
}

fn merge_updates(updates: &mut HashMap<String, Vec<Message>>, mut additional_updates: HashMap<String, Vec<Message>>) {
    for (receiver_name, update_vec) in additional_updates.drain() {
        if !updates.contains_key(&receiver_name) {
            updates.insert(receiver_name, update_vec);
        } else {
            updates.get_mut(&receiver_name).as_mut().unwrap().extend(update_vec);
        }
    }
}

pub struct Result {
    pub response: Option<Message>,
    pub updates: HashMap<String, Vec<Message>>,
    pub terminate_connection: bool,
}

impl Result {
    pub fn add_update_all(&mut self, lobby: &HashMap<String, Player>, new_update: Message) {
        add_update_all(lobby, &mut self.updates, new_update);
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

    pub fn update(updates: HashMap<String, Vec<Message>>) -> Result {
        return Result {
            response: None,
            updates: updates,
            terminate_connection: false,
        }
    }
    pub fn empty() -> Result {
        return Result { response: None, updates: hashmap![], terminate_connection: false };
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
        updates.insert(active_player_name.clone(), vec![Message::GameOverUpdate {
            reason: reason,
            victorious: victorious
        }]);
    }

    // delete game for opponent
    {
        let p2 = lobby.get_mut(opponent_name).unwrap();
        p2.game = None;
        p2.state = PlayerState::Available;
    }
    updates.insert(opponent_name.clone(), vec![Message::GameOverUpdate {
        reason: reason,
        victorious: !victorious
    }]);

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

    add_update_all(lobby, &mut result, Message::PlayerLeftUpdate{ nickname: name.clone() });

    return result;
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
        let mut result = Result::respond(Message::OkResponse, false);

        result.add_update_all(lobby, Message::PlayerJoinedUpdate { nickname: username.clone() });

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
    let mut player = lobby.get_mut(username).unwrap();

    if player.game.is_some() {
        return Result::respond(Message::InvalidRequestResponse, false);
    } else {
        player.state = PlayerState::Ready;
        return Result::respond(Message::OkResponse, false);
    }
}

pub fn handle_not_ready_request(username: &String, lobby: &mut HashMap<String, Player>) -> Result {
    let mut player = lobby.get_mut(username).unwrap();

    match player.game {
        None => {
            player.state = PlayerState::Available;
            return Result::respond(Message::OkResponse, false);
        },
        Some(_) => return Result::respond(Message::GameAlreadyStartedResponse, false),
    }
    panic!("Invalid state or request!");
}

fn initialize_game(player1: &String, player2: &String) -> Rc<RefCell<Game>> {
    let first_board = Board::new(vec![]);
    let second_board = Board::new(vec![]);

    return Rc::new(RefCell::new(Game::new(first_board, second_board, (*player1).clone(), (*player1).clone())));
}

pub fn handle_challenge_player_request(challenged_player_name: String, challenger_name: &String, lobby: &mut HashMap<String, Player>, games: &mut Vec<Rc<RefCell<Game>>>) -> Result {
    let mut launch_game = false;

    let not_waiting_result = Result::respond(Message::NotWaitingResponse {nickname: challenged_player_name.clone() }, false);

    // Is there a player called challenged_player_name?
    if let Some(ref mut challenged_player) = lobby.get_mut(&challenged_player_name) {
        if challenged_player.game.is_some() {
            // Challenged player is already in a game -> NotWaiting
            return not_waiting_result;
        }
        if PlayerState::Ready == challenged_player.state  {
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
        // Set game reference for both players
        lobby.get_mut(challenger_name).unwrap().game = Some(new_game.clone());
        lobby.get_mut(&challenged_player_name).unwrap().game = Some(new_game.clone());
        games.push(new_game);
        // tell challenged player about the game
        let update_message = Message::GameStartUpdate {nickname: (*challenger_name).clone() };
        // OkResponse for player who issued challenge
        return Result::respond_and_update_single(Message::OkResponse, hashmap![challenged_player_name => vec![update_message]], false);
    }
    panic!("Invalid state or request!");
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
        terminate_game(games, game.clone(), lobby, username, false, Reason::Surrendered);
        return Result::respond(Message::OkResponse, false);
    } else {
        return Result::respond(Message::NotYourTurnResponse, false);
    }
}

pub fn handle_report_error_request(errormessage: String, player: &mut PlayerHandle, lobby: &mut HashMap<String, Player>, games: &mut Vec<Rc<RefCell<Game>>>) -> Result {
    let mut termination_result: Result = return Result {
        response: None,
        updates: HashMap::new(),
        terminate_connection: true,
    };

    // For registered players we need to terminate a running game, if any
    if let Some(ref username) = player.nickname {
        termination_result.updates = terminate_player(username, lobby, games);
    }

    // Terminate connection to client reporting ErrorRequest
    return termination_result;
}

pub fn handle_place_ships_request(placement: [ShipPlacement; 5], player_name: &String, lobby: &mut HashMap<String, Player>) -> Result {
    let player = lobby.get_mut(player_name).unwrap();

    if let Some(ref game) = player.game {
        if let GameState::Running = (*game).borrow().state {
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

        let new_board_valid;
        let opponent_ready;
        {
            let mut game_ref = (*game).borrow_mut();
            if game_ref.player1 == *player_name {
                game_ref.board1.ships = ships;
                new_board_valid = game_ref.board1.compute_state();
                opponent_ready = game_ref.board2.ships.len() > 0;
            } else {
                game_ref.board2.ships = ships;
                new_board_valid = game_ref.board2.compute_state();
                opponent_ready = game_ref.board1.ships.len() > 0;
            };
        }

        // Check if new state is valid
        if !new_board_valid {
            return Result::respond(Message::InvalidRequestResponse, false);
        } else {
            let mut game_ref = (*game).borrow_mut();
            // opponent also done placing ships?
            if opponent_ready {
                game_ref.state = GameState::Running;
            }
            return Result::respond_and_update_single(Message::OkResponse, game_ref.switch_turns(), false);
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

fn handle_shoot(games: &mut Vec<Rc<RefCell<Game>>>, game: Rc<RefCell<Game>>,
        lobby: &mut HashMap<String, Player>, player_name: &String, target_x: u8,
        target_y: u8) -> Result {
    let game_over;
    let hit_result;
    let response_msg;
    let mut updates = hashmap![];

    // evaluate shot
    {
        let mut game_ref = (*game).borrow_mut();
        let opponent_name = game_ref.get_opponent_name(player_name).to_owned();

        {
            let ref mut opponent_board = if game_ref.player1 != *player_name { &mut game_ref.board2 } else { &mut game_ref.board1 };
            hit_result = opponent_board.hit(target_x as usize, target_y as usize);
            game_over = opponent_board.is_dead();
        }

        match hit_result {
            HitResult::Hit => {
                response_msg = Message::HitResponse { x: target_x, y: target_y };
                updates = hashmap![opponent_name.clone() => vec![Message::EnemyHitUpdate { x: target_x, y: target_y }]];

                let visibility_updates = hashmap![(*player_name).clone() => game_ref.get_board(player_name).get_visibility_updates()];
                merge_updates(&mut updates, visibility_updates);
            },
            HitResult::Miss => {
                response_msg = Message::MissResponse { x: target_x, y: target_y };
                updates = hashmap![opponent_name.clone() => vec![Message::EnemyMissUpdate { x: target_x, y: target_y }]];

                let visibility_updates = hashmap![(*player_name).clone() => game_ref.get_board(player_name).get_visibility_updates()];
                merge_updates(&mut updates, visibility_updates);
            },
            HitResult::Destroyed => {
                response_msg = Message::DestroyedResponse { x: target_x, y: target_y };
                if !game_over {
                    // TODO which update for enemy?
                }
            },
        }
    }

    if game_over {
        updates = terminate_game(games, game, lobby, player_name, true, Reason::Obliterated);
        return Result::respond_and_update_single(response_msg, updates, false);
    } else {
        let mut game_ref = (*game).borrow_mut();
        merge_updates(&mut updates, game_ref.switch_turns());
        return Result::respond_and_update_single(response_msg, updates, false);
    }
}

pub fn handle_move_shoot_request(target_coords: (u8, u8),
        ship_movement: Option<(usize, Direction)>, player_name: &String,
        lobby: &mut HashMap<String, Player>, games: &mut Vec<Rc<RefCell<Game>>>) -> Result {
    if let Some(ref game) = lobby.get_mut(player_name).unwrap().game.clone() {
        // check if in valid state and handle move, if requested
        let mut game_ref = (*game).borrow_mut();
        if let GameState::Running = game_ref.state {
            if game_ref.my_turn(player_name) {
                // move if requested
                if let Some(movement) = ship_movement {
                    if let Some(result) = handle_move(&mut game_ref, player_name, movement) {
                        return result;
                    }
                }

                // handle shot
                return handle_shoot(games, game.clone(), lobby, player_name, target_coords.0, target_coords.1);
            } else {
                return Result::respond(Message::NotYourTurnResponse, false);
            }
        } else {
            return Result::respond(Message::InvalidRequestResponse, false);
        }
    } else {
        return Result::respond(Message::NotYourTurnResponse, false);
    }
}

pub fn handle_afk(game: Rc<RefCell<Game>>, lobby: &mut HashMap<String, Player>,
        games: &mut Vec<Rc<RefCell<Game>>>) -> HashMap<String, Vec<Message>> {
    let active_player;

    {
        let mut game_ref = (*game).borrow_mut();
        active_player = game_ref.get_active_player();
        let ref mut exceeded_count = if game_ref.is_player1_active() { game_ref.player1_afk_count } else { game_ref.player2_afk_count };

        if *exceeded_count < 2 {
            *exceeded_count += 1;
            let opponent_name = game_ref.get_opponent_name(&game_ref.get_active_player()).clone();
            let mut afk_updates = hashmap![
                active_player => vec![Message::AfkWarningUpdate { strikes: *exceeded_count }],
                opponent_name => vec![Message::EnemyAfkUpdate { strikes: *exceeded_count }]
            ];
            merge_updates(&mut afk_updates, game_ref.switch_turns());
            return afk_updates;
        }
    }

    return terminate_game(games, game, lobby, &active_player, false, Reason::Afk);
}
