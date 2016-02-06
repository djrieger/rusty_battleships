use std::collections::hash_map::Entry;
use std::collections::hash_map::OccupiedEntry;
use std::sync::mpsc;

// extern crate rusty_battleships;
use message::{serialize_message, deserialize_message, Message};

const BLOCK: char = '\u{25AA}';

pub const W: usize = 16;
pub const H: usize = 10;
pub const SHIP_COUNT: usize = 2;

pub struct PlayerHandle {
    pub nickname: Option<String>,
    pub from_child_endpoint: mpsc::Receiver<Message>,
    pub to_child_endpoint: mpsc::Sender<Message>,
}

pub struct Player<'a> {
    pub state: PlayerState,
    pub game: Option<&'a Game>,
}

pub enum PlayerState {
    Available,
    Ready,
    Waiting,
}

pub struct Game {
    pub boards: (Board, Board),
    pub players: (String, String),
    // time elapsed / round
    // active player
}

// list of games
// hashmap nickname -> Player


#[derive(Copy, Clone)]
pub struct Ship {
    pub x: isize,
    pub y: isize,
    pub length: usize,
    pub horizontal: bool,
    pub health_points: usize,
}

#[derive(Copy, Clone)]
pub struct Board {
    pub top: usize,
    pub left: usize,
    pub ships: [Ship; SHIP_COUNT],
    pub state: [[usize; H]; W],
}

impl Board {
    pub fn new(top: usize, left: usize, ships: [Ship; SHIP_COUNT]) -> Board {
        Board {
            top: top,
            left: left,
            state: [[0; H]; W],
            ships: ships,
        }
    }

    fn clear(&mut self) -> () {
        self.state = [[0; H]; W];
    }

    pub fn hit(&mut self, x: usize, y: usize) -> bool {
       match self.state[x][y] {
           0 => {},
           ship_index => self.ships[ship_index - 1].health_points -= 1,
       }
       true
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

}

impl Ship {
    fn move_me(val: isize, offset: isize, limit: usize) -> isize {
        return val + offset;
    }

    pub fn move_left(& mut self) -> () {
        self.x = Ship::move_me(self.x , -1, 0) ;
    }

    pub fn move_right(& mut self) -> () {
        let mut max = W - 1;
        if self.horizontal {
            max -= self.length;
        }
        self.x = Ship::move_me(self.x , 1, max) ;
    }

    pub fn move_up(& mut self) -> () {
        self.y = Ship::move_me(self.y , -1, 0) ;
    }

    pub fn move_down(& mut self) -> () {
        let mut max = H - 1;
        if !self.horizontal {
            max -= self.length;
        }
        self.y = Ship::move_me(self.y , 1, max) ;
    }
}
