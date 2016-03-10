use message::{Message, Direction};

#[derive(Copy, Clone,Debug)]
pub struct Ship {
    pub x: isize,
    pub y: isize,
    pub length: usize,
    pub direction: Direction,
    pub health_points: usize,
}

impl Ship {
    pub fn move_me(&mut self, direction: Direction) -> bool {
        // cannot move destroyed ship
        if self.health_points == 0 {
            return false;
        }
        println!("Moving ship len {} from current position {}:{} to {:?}", self.length, self.x, self.y, direction);
        match direction {
            Direction::North => self.y -= 1,
            Direction::East => self.x += 1,
            Direction::South => self.y += 1,
            Direction::West => self.x -= 1,
        }
        println!("Finished moving ship len {}, new ship pos {}:{}", self.length, self.x, self.y);
        return true;
    }

    pub fn is_dead(&self) -> bool {
        self.health_points == 0
    }

    pub fn is_horizontal(&self) -> bool {
        self.direction == Direction::East || self.direction == Direction::West
    }

    pub fn is_reverse(&self) -> bool {
        self.direction == Direction::West || self.direction == Direction::North
    }
}
