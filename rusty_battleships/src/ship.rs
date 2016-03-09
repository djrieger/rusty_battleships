use message::{Message, Direction};

#[derive(Copy, Clone,Debug)]
pub struct Ship {
    pub x: isize,
    pub y: isize,
    pub length: usize,
    pub horizontal: bool,
    pub health_points: usize,
}


impl Ship {
    pub fn move_me(&mut self, direction: Direction) -> bool {
        // cannot move destroyed ship
        if self.health_points == 0 {
            return false;
        }
        match direction {
            Direction::North => self.y -= 1,
            Direction::East => self.x += 1,
            Direction::South => self.y += 1,
            Direction::West => self.x -= 1,
        }
        return true;
    }

    pub fn is_dead(&self) -> bool {
        self.health_points == 0
    }
}
