use board::{Board};

use rand::{thread_rng, Rng};

use time;

#[derive(PartialEq)]
pub enum GameState {
    Placing,
    Running,
}

static LIMIT_SECONDS: i64 = 60;

pub struct Game {
    board1: Board,
    board2: Board,
    player1: String,
    player2: String,
    last_turn_started_at: Option<time::PreciseTime>,
    player1_active: bool,
    player1_afk_count: u8,
    player2_afk_count: u8,
    state: GameState,
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
            player1_active: thread_rng().gen(),
            player1_afk_count: 3,
            player2_afk_count: 3,
            state: GameState::Placing,
        }
    }

    pub fn get_opponent_name(&self, player_name: &String) -> &String {
        return if *self.player1 == *player_name { &self.player2 } else { &self.player1 };
    }

    pub fn get_board(&mut self, player_name: &String) -> &mut Board {
        return if *self.player1 == *player_name { &mut self.board1 } else { &mut self.board2 };
    }

    pub fn get_opponent_board(&mut self, player_name: &String) -> &mut Board {
        return if *self.player1 == *player_name { &mut self.board2 } else { &mut self.board1 };
    }

    pub fn my_turn(&self, player_name: &String) -> bool {
        return if *self.player1 == *player_name { self.player1_active } else { !self.player1_active };
    }

    pub fn get_active_player(&self) -> String {
        return if self.player1_active { self.player1.clone() } else { self.player2.clone() };
    }

    pub fn get_waiting_player(&self) -> String {
        return if self.player1_active { self.player2.clone() } else { self.player1.clone() };
    }

    pub fn start(&mut self) {
        self.state = GameState::Running;
        self.last_turn_started_at = Some(time::PreciseTime::now());
    }

    pub fn switch_turns(&mut self) {
        self.player1_active = !self.player1_active;
        self.last_turn_started_at = Some(time::PreciseTime::now());
    }

    pub fn turn_time_exceeded(&self) -> bool {
        match self.last_turn_started_at {
            None => false,
            Some(start_time) => start_time.to(time::PreciseTime::now()) > time::Duration::seconds(LIMIT_SECONDS),
        }
    }

    pub fn is_running(&self) -> bool {
        self.state == GameState::Running
    }

    pub fn is_player1_active(&self) -> bool {
        self.player1_active
    }

    pub fn get_active_player_afk_count(&self) -> u8 {
        if self.player1_active { self.player1_afk_count } else { self.player2_afk_count }
    }

    pub fn dec_active_player_afk_count(&mut self) -> () {
        if self.player1_active {
            self.player1_afk_count -= 1;
        } else {
            self.player2_afk_count -= 1;
        }
    }
}
