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
        if self.mine {
            match self.state[x][y].ship_index {
                None => ship_index = None,
                Some(x) => {
                    ship_index = Some(x);
                    self.ships[ship_index.unwrap() as usize].health_points -= 1;
                    // if self.ships.ship_index.health_points == 0 {
                    // TODO: What if it's destroyed?
                    // }
                },
            }
        } else {
            ship_index = Some(9);
        }
        self.state[x][y] = CellState {visible: true, ship_index: ship_index};
    }
}
