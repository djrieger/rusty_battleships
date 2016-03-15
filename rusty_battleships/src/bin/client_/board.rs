// use rusty_battleships::board::{CellState};
// use rusty_battleships::message::{Direction};
// use rusty_battleships::ship::Ship;

/*
pub const W: usize = 10;
pub const H: usize = 10;

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct BoardOLD {
    pub mine: bool,
    pub ships: Vec<Ship>,
    pub state: [[CellState; H]; W],
}

impl BoardOLD {

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
                    let ref mut ship = self.ships[ship_index.unwrap() as usize];
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

        if self.mine { //Which of our precious ships has been destroyed?
            match self.state[x][y].ship_index {
                None => {
                    panic!("Server says DESTROYED, but there is no ship! X={}, Y={}", x, y);
                },
                Some(x) => {
                    let ref mut ship = self.ships[x as usize];
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

    pub fn compute_state(&mut self) -> bool {
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
                    return false;
                } else {
                    new_state[dest_x as usize][dest_y as usize].set_ship((ship_index) as u8);
                }
            }
        }

        self.state = new_state;
        true
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
}
*/
