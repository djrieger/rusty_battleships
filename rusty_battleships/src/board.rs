use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc;
use message::{Message, Direction};
use game::Game;
use ship::Ship;

use ansi_term::Colour::{Green, Black, Red};

pub const W: usize = 10;
pub const H: usize = 10;

pub type BoardState = [[CellState; H]; W];

pub enum ToChildCommand {
    Message(Message),
    TerminateConnection
}

pub enum ToMainThreadCommand {
    Message(Message),
    Error(::std::io::Error),
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

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
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

type BoardArray = [[bool; H]; W];

#[derive(Copy, Clone)]
pub struct DumbBoard {
    ship_at: BoardArray,
    visible: BoardArray,
}

impl DumbBoard {
    pub fn new() -> DumbBoard {
        DumbBoard {
            ship_at: [[false; H]; W],
            visible: [[false; H]; W],
        }
    }

    fn handle(&mut self, x: usize, y: usize, ship_at_coords: bool) {
        self.visible[x][y] = true;
        self.ship_at[x][y] = ship_at_coords;
    }

    // former hit(), visible()
    pub fn set_ship(&mut self, x: u8, y: u8) {
        self.handle(x as usize, y as usize, true);
    }

    // former miss(), invisible(), destroyed()
    pub fn set_water(&mut self, x: u8, y: u8) {
        self.handle(x as usize, y as usize, false);
    }

    pub fn is_visible_at(&self, x: usize, y: usize) -> bool {
        self.visible[x][y]
    }

    pub fn has_ship_at(&self, x: usize, y: usize) -> bool {
        self.ship_at[x][y]
    }
}

#[derive(Debug, Clone)]
pub struct Board {
    ships: Vec<Ship>,
    state: BoardState,
    old_states: Vec<BoardState>,

    handle_visibility_updates: bool,
    visibility_updates: Vec<Message>,
}

impl Board {
    pub fn try_create(ships: Vec<Ship>, handle_visibility_updates: bool) -> Option<Board> {
        let mut board = Board {
            state: [[CellState::new(); H]; W],
            old_states: vec![],
            ships: ships,
            handle_visibility_updates: handle_visibility_updates,
            visibility_updates: vec![],
        };
        if let Some(state) = board.compute_state() {
            board.state = state;
            return Some(board);
        } else {
            return None;
        }
    }

    pub fn is_visible_at(&self, x: usize, y: usize) -> bool {
        self.state[x][y].visible
    }

    pub fn get_ship_index_at(&self, x: usize, y: usize) -> Option<u8> {
        self.state[x][y].ship_index
    }

    pub fn get_ships(&self) -> &Vec<Ship> {
        &self.ships
    }

    pub fn has_ships(&self) -> bool {
        self.ships.len() > 0
    }

    pub fn move_ship(&mut self, ship_index: u8, direction: Direction) -> bool {
        return self.ships[ship_index as usize].move_me(direction) && self.add_state();
    }

    pub fn hit(&mut self, x: usize, y: usize) -> HitResult {
        if x >= W || y >= H {
            return HitResult::Miss;
        }
        self.set_visible_at(x, y);
        let hit_result = match self.state[x][y].ship_index {
            // no ship
            None => HitResult::Miss,
            Some(ship_index) => {
                let ref mut ship = self.ships[ship_index as usize];
                ship.health_points -= 1;
                match ship.health_points {
                    0 => HitResult::Destroyed,
                    _ => HitResult::Hit
                }
            }
        };
        self.add_state();

        self.print_me(Some((x, y)));
        self.old_states.clear();
        return hit_result;
    }

    pub fn set_visible_at(&mut self, x: usize, y: usize) {
        self.state[x][y].visible = true;
    }

    /**
     * Compute new board state.
     * @return true if board state is valid, false otherwise (if ships overlap or are outside board
     * boarders)
     */
    pub fn compute_state(&mut self) -> Option<BoardState> {
        let mut new_state = [[CellState::new(); H]; W];

        for (ship_index, ship) in self.ships.iter().enumerate() {
            if ship.is_dead() {
                continue;
            }
            for i in 0..ship.length  {
                let (dest_x, dest_y) = Board::get_ship_dest_coords(ship, i);
                if !self.coords_valid(dest_x, dest_y) || new_state[dest_x as usize][dest_y as usize].has_ship() {
                    // coordinates are invalid or there is another ship at these coordinates
                    println!("Coords invalid or collision detected at {}:{}, new ship index {}", dest_x, dest_y, ship_index);
                    self.print_me(None);
                    return None;
                } else {
                    new_state[dest_x as usize][dest_y as usize].set_ship((ship_index) as u8);
                }
            }
        }

        return Some(new_state);
    }

    pub fn pop_updates(&mut self) -> Vec<Message> {
        if !self.handle_visibility_updates {
            panic!("This board does not compute visibility updates");
        }
        let foo = self.visibility_updates.clone();
        self.visibility_updates = vec![];
        return foo;
    }

    fn add_state(&mut self) -> bool {
        if let Some(new_state) = self.compute_state() {
            self.old_states.push(self.state.clone());
            self.state = new_state;
            self.compute_visibility_updates();
            true
        } else {
            false
        }
    }

    fn compute_visibility_updates(&mut self) {
        // Find all cells that had ships in old state (self.state) but no longer in new_state and
        // vice versa -> some ship moved out of some cell
        for x in 0..W {
            for y in 0..H {
                let ref old_cell = self.old_states.last().unwrap()[x][y];
                let ref mut new_cell = self.state[x][y];
                // copy visibility information to new state
                new_cell.visible = new_cell.visible || old_cell.visible;
                if self.handle_visibility_updates && new_cell.visible {
                    let ship_entered_cell = !old_cell.has_ship() && new_cell.has_ship();
                    let ship_left_cell = old_cell.has_ship() && !new_cell.has_ship();
                    if ship_left_cell {
                        self.visibility_updates.push(Message::EnemyInvisibleUpdate { x: x as u8, y: y as u8 });
                    } else if ship_entered_cell {
                        self.visibility_updates.push(Message::EnemyVisibleUpdate { x: x as u8, y: y as u8 });
                    }
                }
            }
        }
    }

    fn coords_valid(&self, x: isize, y: isize) -> bool {
        return x >= 0 && y >= 0 && x < (W as isize) && y < (H as isize);
    }

    fn get_ship_dest_coords(ship: &Ship, i: usize) -> (isize, isize) {
        let mut dest = (ship.x, ship.y);
        match ship.direction {
            Direction::East => dest.0 += i as isize,
            Direction::South => dest.1 += i as isize,
            Direction::West => dest.0 -= i as isize,
            Direction::North => dest.1 -= i as isize,
        }
        return (dest.0, dest.1);
    }

    fn print_state(state: &BoardState, target_coords: Option<(usize, usize)>) -> Vec<String> {
        let mut lines = vec![];
        for y in 0..H {
            let mut line = String::new();
            for x in 0..W {
                let character = match state[x][y].ship_index {
                    Some(index) => String::from(index.to_string()),
                    None => String::from("-"),
                };

                if let Some((target_x, target_y)) = target_coords {
                    if x == target_x && y == target_y {
                        line.push_str(&format!("{}", Black.on(Red).paint(character)));
                        continue;
                    }
                }

                if state[x][y].visible {
                    line.push_str(&format!("{}", Black.on(Green).paint(character)));
                } else {
                    line.push_str(&character);
                }
            }
            lines.push(line);
        }
        return lines;
    }

    fn print_me(&self, target_coords: Option<(usize, usize)>) {
        println!("Printing state");
        let mut printed_boards: Vec<Vec<String>> = self.old_states.iter().map(|state| Board::print_state(&state, None)).collect();
        printed_boards.push(Board::print_state(&self.state, target_coords));

        for i in 0..10 {
            for board in &printed_boards {
                print!("{}  |  ", board.get(i).unwrap());
            }
            print!("\n");
        }
        println!("");
    }

    pub fn is_dead(&self) -> bool {
        self.ships.iter().all(|ship| ship.is_dead())
    }

}
