use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc;

extern crate time;

// extern crate rusty_battleships;
use message::{Message, Direction};
use game::Game;

//const BLOCK: char = '\u{25AA}';

pub const W: usize = 16;
pub const H: usize = 10;
pub const SHIP_COUNT: usize = 2;

pub enum ToChildCommand {
    Message(Message),
    TerminateConnection
}

pub enum ToMainThreadCommand {
    Message(Message),
    TerminatePlayer,
}

pub struct PlayerHandle {
    pub nickname: Option<String>,
    // Sending None to the main thread indicates that the client will be terminated and requests
    // cleanup operations such as terminating a running game for that client
    pub from_child_endpoint: mpsc::Receiver<ToMainThreadCommand>,
    pub to_child_endpoint: mpsc::Sender<ToChildCommand>,
}

pub struct Player {
    pub state: PlayerState,
    pub game: Option<Rc<RefCell<Game>>>,
}

impl Player {
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


#[derive(Copy, Clone)]
pub struct Ship {
    pub x: isize,
    pub y: isize,
    pub length: usize,
    pub horizontal: bool,
    pub health_points: usize,
}

#[derive(Copy, Clone)]
pub struct CellState {
    pub visible: bool,
    pub ship_index: Option<u8>,
}

// #[derive(Copy, Clone)]
pub struct Board {
    pub ships: Vec<Ship>,
    pub state: [[CellState; H]; W],
}

pub enum HitResult {
    Hit,
    Miss,
    Destroyed
}

impl Board {
    pub fn new(ships: Vec<Ship>) -> Board {
        Board {
            state: [[CellState { visible: false, ship_index: None }; H]; W],
            ships: ships,
        }
    }

    fn clear(&mut self) -> () {
        self.state =  [[CellState { visible: false, ship_index: None }; H]; W];
    }

    pub fn hit(&mut self, x: usize, y: usize) -> HitResult {
        if x >= W || y >= H {
            return HitResult::Miss;
        }
        self.state[x][y].visible = true;
        return match self.state[x][y].ship_index {
            // no ship
            None => HitResult::Miss,
            Some(ship_index) => {
                let ref mut ship = self.ships[ship_index as usize - 1];
                ship.health_points -= 1;
                match ship.health_points {
                    0 => HitResult::Destroyed,
                    _ => HitResult::Hit
                }
            }
        }
    }

    pub fn get_visibility_updates(&self) -> Vec<Message> {
        let mut visibility_updates = vec![];
        for x in 0..W - 1 {
            for y in 0..H - 1 {
                let ref cell = self.state[x][y];
                if cell.visible {
                    let ship_index = cell.ship_index;
                    // if there is a ship that is not dead -> send visible update
                    if ship_index.is_some() && self.ships[ship_index.unwrap() as usize - 1].health_points > 0 {
                        visibility_updates.push(Message::EnemyVisibleUpdate { x: x as u8, y: y as u8 });
                    } else {
                        visibility_updates.push(Message::EnemyInvisibleUpdate { x: x as u8, y: y as u8 });
                    }
                }
            }
        }
        visibility_updates
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
                if dest.0 < 0 || dest.1 < 0 || dest.0 >= (W as isize) - 1 || dest.1 >= (H as isize) - 1 || self.state[dest.0 as usize][dest.1 as usize].ship_index.is_some() {
                    return false;
                } else {
                    self.state[dest.0 as usize][dest.1 as usize].ship_index = Some((ship_index + 1) as u8);
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
