use std::collections::HashMap;

extern crate rand;
use self::rand::{thread_rng, Rng};

extern crate time;

use board::{Board};
use message::{Message, Reason};

#[derive(PartialEq)]
pub enum GameState {
    Placing,
    Running,
}

pub struct Game {
    pub board1: Board,
    pub board2: Board,
    pub player1: String,
    pub player2: String,
    last_turn_started_at: Option<time::PreciseTime>,
    player1_active: bool,
    player1_afk_count: u8,
    player2_afk_count: u8,
    pub state: GameState,
}

impl PartialEq for Game {
    fn eq(&self, other: &Game) -> bool {
        self.player1 == other.player1 && self.player2 == other.player2
    }
}

impl Game {
    pub fn new(board1: Board, board2: Board, player1: String, player2: String) -> Game {
        Game {
            board1: board1,
            board2: board2,
            player1: player1,
            player2: player2,
            last_turn_started_at: None,
            player1_active: rand::thread_rng().gen(),
            player1_afk_count: 0,
            player2_afk_count: 0,
            state: GameState::Placing,
        }
    }

    pub fn get_opponent_name(&self, player_name: &String) -> &String {
        return if *self.player1 == *player_name { &self.player2 } else { &self.player1 };
    }

    pub fn get_board(&mut self, player_name: &String) -> &mut Board {
        return if *self.player1 == *player_name { &mut self.board2 } else { &mut self.board1 };
    }

    // returns GameOverUpdate message to be sent to opponent
    pub fn shutdown(&mut self /*, initiator_name: String, */, victorious: bool, reason: Reason) -> Option<Message> {
        // inform opponent
        return Some(Message::GameOverUpdate { victorious: !victorious, reason: reason });
    }

    pub fn my_turn(&self, player_name: &String) -> bool {
        return if *self.player1 == *player_name { self.player1_active } else { !self.player1_active };
    }

    pub fn get_active_player(&self) -> String {
        return if self.player1_active { self.player1.clone() } else { self.player2.clone() };
    }

    pub fn switch_turns(&mut self) -> HashMap<String, Vec<Message>> {
        self.player1_active = !self.player1_active;
        self.last_turn_started_at = Some(time::PreciseTime::now());

        let (active_player, waiting_player) = if self.player1_active { (self.player1.clone(), self.player2.clone()) } else { (self.player2.clone(), self.player1.clone()) };
        let mut updates = HashMap::new();
        updates.insert(active_player, vec![Message::YourTurnUpdate]);
        updates.insert(waiting_player, vec![Message::EnemyTurnUpdate]);
        return updates;
    }

    fn time_exceeded(time: Option<time::PreciseTime>, limit_seconds: u64) -> bool {
        return match time {
            None => false,
            Some(start_time) => start_time.to(time::PreciseTime::now()) > time::Duration::seconds(limit_seconds as i64),
        }
    }

    pub fn turn_time_exceeded(&self) -> bool {
        return Game::time_exceeded(self.last_turn_started_at, 60);
    }

    pub fn is_running(&self) -> bool {
        if let GameState::Running = self.state {
            return true;
        }
        false
    }

    pub fn is_player1_active(&self) -> bool {
        self.player1_active
    }

    pub fn get_active_player_afk_count(&self) -> u8 {
        if self.player1_active { self.player1_afk_count } else { self.player2_afk_count }
    }

    pub fn inc_active_player_afk_count(&mut self) -> () {
        if self.player1_active {
            self.player1_afk_count += 1;
        } else {
            self.player2_afk_count += 1;
        }
    }
}
