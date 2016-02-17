use std::collections::hash_map::Entry;
use std::collections::hash_map::OccupiedEntry;
use std::sync::mpsc;

extern crate time;

// extern crate rusty_battleships;
use message::{serialize_message, deserialize_message, Message, Direction};

const BLOCK: char = '\u{25AA}';

pub const W: usize = 16;
pub const H: usize = 10;
pub const SHIP_COUNT: usize = 2;

pub struct PlayerHandle {
    pub nickname: Option<String>,
    // Sending None to the main thread indicates that the client will be terminated and requests
    // cleanup operations such as terminating a running game for that client
    pub from_child_endpoint: mpsc::Receiver<Option<Message>>,
    // Sending None to a child indicates that the server wishes to terminate the (TCP) connection
    // with that child
    pub to_child_endpoint: mpsc::Sender<Option<Message>>,
}

pub struct Player<'a> {
    pub state: PlayerState,
    pub game: Option<&'a mut Game>,
}

impl <'a> Player<'a> {
    pub fn has_non_finished_game(&self) -> bool {
        if let Some(ref game) = self.game {
            match game.state {
                GameState::ShuttingDown => {},
                _ => return true,
            }
        }
        return false;
    }

//     pub fn set_available(&mut self, my_name: String, lobby: &mut HashMap<String, Player>, updates: &mut HashMap<String, Vec<Message>>) {
//         self.state = PlayerState::Available;
//         let mut updates = HashMap::new();
//         for (nickname, player) in &mut lobby {
//             if nickname != my_name {
//                 updates.insert(nickname, vec![Message::PlayerJoinedUpdate { nickname: nickname }]);
//             }
//         }
//
//         // TODO: PLAYER_READY
//     }
}

pub enum PlayerState {
    Available,
    Ready,
    Waiting,
    Playing
}

pub enum GameState {
    Placing,
    Running,
    ShuttingDown
}

pub struct Game {
    pub board1: Board,
    pub board2: Board,
    pub player1: String,
    pub player2: String,
    pub last_turn_started_at: Option<u64>,
    pub shutdown_started_at: Option<u64>,
    pub player1_active: bool,
    pub player1_afk_count: u8,
    pub player2_afk_count: u8,
    pub state: GameState,
}

impl PartialEq for Game {
    fn eq(&self, Rhs: &Game) -> bool {
        // TODO implement!
        return true;
    }

    fn ne(&self, Rhs: &Game) -> bool {
        // TODO implement!
        return true;
    }
}

impl Game {
    pub fn New(board1: Board, board2: Board, player1: String, player2: String) -> Game {
        Game {
            board1: board1,
            board2: board2,
            player1: player1,
            player2: player2,
            last_turn_started_at: None,
            shutdown_started_at: None,
            player1_active: true,
            player1_afk_count: 0,
            player2_afk_count: 0,
            state: GameState::Placing,
        }
    }

    pub fn get_opponent_name(&self, player_name: &String) -> &String {
        return if *self.player1 == *player_name { &self.player2 } else { &self.player1 };
    }

    pub fn get_board(&mut self, player_name: &String) -> &mut Board {
        return if *self.player1 == *player_name { &mut self.board2 } else { &mut self.board1 };
    }

    // TODO reason for shutdown
    pub fn shutdown(&mut self) {
        // shut down already initiated? 
        if let GameState::ShuttingDown = self.state {
            return;
        }
        self.state = GameState::ShuttingDown;
        self.shutdown_started_at = Some(time::precise_time_ns());
        // TODO inform opponent
    }

    pub fn my_turn(&self, player_name: &String) -> bool {
        return if *self.player1 == *player_name { self.player1_active } else { !self.player1_active };
    }
}

#[derive(Copy, Clone)]
pub struct Ship {
    pub x: isize,
    pub y: isize,
    pub length: usize,
    pub horizontal: bool,
    pub health_points: usize,
}

// #[derive(Copy, Clone)]
pub struct Board {
    pub ships: Vec<Ship>,
    pub state: [[usize; H]; W],
}

pub enum HitResult {
    Hit,
    Miss,
    Destroyed
}

impl Board {
    pub fn new(ships: Vec<Ship>) -> Board {
        Board {
            state: [[0; H]; W],
            ships: ships,
        }
    }

    fn clear(&mut self) -> () {
        self.state = [[0; H]; W];
    }

    pub fn hit(&mut self, x: usize, y: usize) -> HitResult {
        if x >= W || y >= H {
            return HitResult::Miss;
        }
        return match self.state[x][y] {
            0 => HitResult::Miss,
            ship_index => {
                let ref mut ship = self.ships[ship_index - 1];
                ship.health_points -= 1;
                match ship.health_points {
                    0 => HitResult::Destroyed,
                    _ => HitResult::Hit
                }
            }
        }
    }

    /**
     * Compute new board state.
     * @return true if board state is valid, false otherwise (if ships overlap or are outside board
     * boarders)
     */
    pub fn compute_state(&mut self) -> bool {
        self.clear();
        let mut dest;
        for (ship_index, ship) in self.ships.iter().enumerate() {
            for i in 0..ship.length  {
                if ship.health_points == 0 {
                    continue;
                }
                if ship.horizontal {
                    dest = (ship.x + (i as isize), ship.y);
                } else {
                    dest = (ship.x, ship.y + (i as isize));
                }
                if dest.0 < 0 || dest.1 < 0 || dest.0 >= (W as isize) - 1 || dest.1 >= (H as isize) - 1 || self.state[dest.0 as usize][dest.1 as usize] != 0 {
                    return false;
                } else {
                    self.state[dest.0 as usize][dest.1 as usize] = ship_index + 1;
                }
            }
        }
        return true;
    }

    pub fn is_dead(&self) -> bool {
        self.ships.iter().all(|ship| ship.is_dead())
    }
}

impl Ship {
    pub fn move_me(&mut self, direction: Direction) -> bool {
        // cannot move destroyed ship
        if self.health_points == 0 {
            return false;
        }
        match direction {
            Direction::North => self.y -= 1,
            Direction::East => self.x = 1,
            Direction::South => self.y = 1,
            Direction::West => self.x -= 1,
        }
        return true;
    }

    pub fn is_dead(&self) -> bool {
        self.health_points == 0
    }
}
