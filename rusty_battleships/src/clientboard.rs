use board::{Ship, CellState};

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
}
