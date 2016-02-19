extern crate time;

use board::{Board};
use message::{Message, Reason};

pub enum GameState {
    Placing,
    Running,
    ShuttingDown
}

pub struct Game {
    pub board1: Board,
    pub board2: Board,
    pub player1: String,
    pub player2: String,
    last_turn_started_at: Option<time::PreciseTime>,
    pub shutdown_started_at: Option<time::PreciseTime>,
    player1_active: bool,
    pub player1_afk_count: u8,
    pub player2_afk_count: u8,
    pub state: GameState,
}

impl PartialEq for Game {
    fn eq(&self, Rhs: &Game) -> bool {
        // TODO implement!
        return true;
    }

    fn ne(&self, Rhs: &Game) -> bool {
        // TODO implement!
        return true;
    }
}

impl Game {
    pub fn New(board1: Board, board2: Board, player1: String, player2: String) -> Game {
        Game {
            board1: board1,
            board2: board2,
            player1: player1,
            player2: player2,
            last_turn_started_at: None,
            shutdown_started_at: None,
            player1_active: true,
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
        // shut down already initiated? 
        if let GameState::ShuttingDown = self.state {
            return None;
        }
        self.state = GameState::ShuttingDown;
        self.shutdown_started_at = Some(time::PreciseTime::now());

        // inform opponent
        return Some(Message::GameOverUpdate { victorious: !victorious, reason: reason });
    }

    pub fn my_turn(&self, player_name: &String) -> bool {
        return if *self.player1 == *player_name { self.player1_active } else { !self.player1_active };
    }

    pub fn switch_turns(&mut self) {
        self.player1_active = !self.player1_active;
        self.last_turn_started_at = Some(time::PreciseTime::now());
    }

    fn time_exceeded(time: Option<time::PreciseTime>, limit_seconds: u64) -> bool {
        return match time {
            None => false,
            Some(start_time) => start_time.to(time::PreciseTime::now()) < time::Duration::seconds(limit_seconds as i64),
        }
    }

    pub fn turn_time_exceeded(&self) -> bool {
        return Game::time_exceeded(self.last_turn_started_at, 60);
    }

    pub fn shutdown_time_exceeded(&self) -> bool {
        return Game::time_exceeded(self.last_turn_started_at, 1);
    }

    pub fn is_running(&self) -> bool {
        if let GameState::Running = self.state {
            return true;
        }
        false
    }

    pub fn is_shutting_down(&self) -> bool {
        if let GameState::ShuttingDown = self.state {
            return true;
        }
        false
    }
}
