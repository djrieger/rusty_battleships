use board::{CellState};
use ship::Ship;
use message::{Message, Direction};

pub const W: usize = 16;
pub const H: usize = 10;

pub struct Board {
    pub mine: bool,
    pub ships: Vec<Ship>,
    pub state: [[CellState; H]; W],
}

impl Board {

    pub fn new(ships: Vec<Ship>, mine: bool) -> Board {
        Board {
            mine: mine,
            state: [[CellState { visible: false, ship_index: None }; H]; W],
            ships: ships,
        }
    }

    pub fn hit(&mut self, x: usize, y: usize) -> () {
        if x >= W && y >= H {
            panic!("Hit out of bounds! X={}, Y={}", x, y);
        }
        let ship_index: Option<u8>;
        if self.mine { //Which of our precious ships has been destroyed?
            match self.state[x][y].ship_index {
                None => ship_index = None,
                Some(x) => {
                    ship_index = Some(x);
                    let mut ship = self.ships[ship_index.unwrap() as usize];
                    ship.health_points -= 1;
                    if ship.health_points == 0 {
                        println!("Ship destroyed: {:?}", ship)// TODO: What if it's destroyed?
                    }
                },
            }
        } else { // We cannot know which ship we hit.
            ship_index = Some(9);
        }
        self.state[x][y] = CellState {visible: true, ship_index: ship_index};
    }

    pub fn miss(&mut self, x: usize, y: usize) -> () {
        if x >= W && y >= H {
            panic!("Miss out of bounds! X={}, Y={}", x, y);
        }
        self.state[x][y] = CellState {visible: true, ship_index: None};
    }

    pub fn destroyed(&mut self, x: usize, y: usize) -> () {
        if x >= W && y >= H {
            panic!("Destroyed out of bounds! X={}, Y={}", x, y);
        }
        let ship_index: Option<u8>;
        if self.mine { //Which of our precious ships has been destroyed?
            match self.state[x][y].ship_index {
                None => {
                    ship_index = None;
                    panic!("Server says DESTROYED, but there is no ship! X={}, Y={}", x, y);
                },
                Some(x) => {
                    ship_index = Some(x);
                    let mut ship = self.ships[ship_index.unwrap() as usize];
                    if ship.health_points != 1 {
                        panic!("Server says DESTROYED, but the ship had {} hp before the hit!", ship.health_points);
                    }
                    ship.health_points = 0;
                },
            }
        } else {
            //Nothing. We cannot know which ship we hit.
        }
        self.state[x][y] = CellState {visible: true, ship_index: None};
    }

    pub fn visible(&mut self, x: usize, y: usize) {
        if x >= W && y >= H {
            panic!("Visible out of bounds! X={}, Y={}", x, y);
        }
        if self.mine {
            // Irrelevant - We know the visibility conditions through enemy hits.
        } else {
            self.state[x][y] = CellState {visible: true, ship_index: Some(9)};
        }
    }

    pub fn invisible(&mut self, x: usize, y: usize) {
        if x >= W && y >= H {
            panic!("Invisible out of bounds! X={}, Y={}", x, y);
        }
        if self.mine {
            // Irrelevant - We know the visibility conditions through enemy hits.
        } else {
            self.state[x][y] = CellState {visible: true, ship_index: None};
        }
    }

    pub fn move_ship(&mut self, ship_id: usize, dir: Direction) {
        if ship_id >= 5 {
            panic!("ship_id out of bounds: {}", ship_id);
        }
        if let Some(ref mut ship) = self.ships.get_mut(ship_id) {
            ship.move_me(dir);
        }
        self.compute_state();
    }

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

    fn coords_valid(&self, x: usize, y: usize) -> bool {
        return !(x < 0 || y < 0 || x >= (W as usize) - 1 || y >= (H as usize) - 1);
    }

    fn get_ship_dest_coords(ship: &Ship, i: usize) -> (usize, usize) {
        let mut dest = (ship.x, ship.y);
        match ship.direction {
            Direction::East => dest.0 += i as isize,
            Direction::South => dest.1 += i as isize,
            Direction::West => dest.0 -= i as isize,
            Direction::North => dest.1 -= i as isize,
        }
        return (dest.0 as usize, dest.1 as usize);
    }
}
