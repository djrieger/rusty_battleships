use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc;
use message::{Message, Direction};
use game::Game;
use ship::Ship;

extern crate time;

pub const W: usize = 16;
pub const H: usize = 10;

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
    pub from_child_endpoint: mpsc::Receiver<ToMainThreadCommand>,
    pub to_child_endpoint: mpsc::Sender<ToChildCommand>,
}

pub struct Player {
    pub state: PlayerState,
    pub game: Option<Rc<RefCell<Game>>>,
}

#[derive(PartialEq)]
pub enum PlayerState {
    Available,
    Ready,
    Playing
}

#[derive(Copy, Clone)]
pub struct CellState {
    pub visible: bool,
    pub ship_index: Option<u8>,
}

pub enum HitResult {
    Hit,
    Miss,
    Destroyed
}

impl CellState {
    pub fn new() -> CellState {
        CellState { visible: false, ship_index: None }
    }

    pub fn has_ship(&self) -> bool {
        self.ship_index.is_some()
    }

    pub fn set_ship(&mut self, ship_index: u8) {
        self.ship_index = Some(ship_index);
    }
}

pub struct Board {
    pub ships: Vec<Ship>,
    pub state: [[CellState; H]; W],
}

impl Board {
    pub fn new(ships: Vec<Ship>) -> Board {
        Board {
            state: [[CellState::new(); H]; W],
            ships: ships,
        }
    }

    fn clear(&mut self) -> () {
        self.state =  [[CellState::new(); H]; W];
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

    fn coords_valid(&self, x: usize, y: usize) -> bool {
        return x < (W as usize) && y < (H as usize));
    }

    fn get_ship_dest_coords(ship: &Ship, i: usize) -> (usize, usize) {
        let mut dest = (ship.x, ship.y);
        if ship.horizontal {
            dest.0 += i as isize;
        } else {
            dest.1 += i as isize;
        }
        return (dest.0 as usize, dest.1 as usize);
    }

    /**
     * Compute new board state.
     * @return.0 true if board state is valid, false otherwise (if ships overlap or are outside board
     * boarders)
     * @return.1 a list of visibility updates caused by recent movement
     */
    pub fn compute_state(&mut self) -> (bool, Vec<Message>) {
        let mut new_state = [[CellState::new(); H]; W];
        let mut visibility_updates = vec![];

        for (ship_index, ship) in self.ships.iter().filter(|ship| !ship.is_dead()).enumerate() {
            for i in 0..ship.length  {
                let (dest_x, dest_y) = Board::get_ship_dest_coords(ship, i);
                if !self.coords_valid(dest_x, dest_y) || new_state[dest_x][dest_y].has_ship() {
                    // coordinates are invalid or there is another ship at these coordinates
                    return (false, vec![]);
                } else {
                    let ref cell = self.state[dest_x][dest_y];
                    if cell.visible && cell.has_ship() {
                        // no ship was here before but now this ship occupies this cell
                        visibility_updates.push(Message::EnemyVisibleUpdate { x: dest_x as u8, y: dest_y as u8 });
                    }
                    new_state[dest_x as usize][dest_y as usize].set_ship((ship_index + 1) as u8);
                }
            }
        }

        // Find all cells that had ships in old state (self.state) but no longer in new_state ->
        // some ship moved out of some cell
        for x in 0..W {
            for y in 0..H {
                let ref old_cell = self.state[x][y];
                let ref mut new_cell = new_state[x][y];
                // copy visibility information to new state
                new_cell.visible = old_cell.visible;
                if old_cell.visible && old_cell.has_ship() && !new_cell.has_ship() {
                    visibility_updates.push(Message::EnemyInvisibleUpdate { x: x as u8, y: y as u8 });
                }
            }
        }

        self.state = new_state;
        return (true, visibility_updates);
    }

    pub fn is_dead(&self) -> bool {
        self.ships.iter().all(|ship| ship.is_dead())
    }
}
