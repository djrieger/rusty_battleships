extern crate time;
use game::{Game, GameState};
use board::{Board, Ship, CellState, HitResult, H, W};

/*****************************************************************
 * Package for adding client-specific methods Game instances.
 * TODO: Maybe create "naked" Game struct def and split up the
 *       implementations into commongame, servergame and clientgame.
 *****************************************************************/

impl Game {

    pub fn hello() {
        println!("Hello, World from the game!");
    }

    // pub fn empty() -> Game {
    //     Game {
    //         board1 : Board::empty(),
    //         board2 : Board::empty(),
    //         player1 : String::from("Nobody1"),
    //         player2 : String::from("Nobody2"),
    //         last_turn_started_at : None::<time::PreciseTime>,
    //         player1_active : true,
    //         player1_afk_count : 0,
    //         player2_afk_count : 0,
    //         state : GameState::Placing,
    //     }
    // }


}

impl Board {

    pub fn hello() {
        println!("Hello, World from the board!");
    }

    // pub fn empty() -> Board {
    //     Board {
    //         state: [[CellState { visible: false, ship_index: None }; H]; W],
    //         ships: Vec::<Ship>::new(),
    //     }
    // }

}
