/*extern crate rustty;*/
extern crate rusty_battleships;

/*use rustty::{
    Terminal,
    Event,
    HasSize,
    CellAccessor
};

use rustty::ui::{
    Painter,
    Dialog,
    Widget,
    Alignable,
    HorizontalAlign,
    VerticalAlign
};*/

use rusty_battleships::board::{
    Board,
    Ship
};

/*fn create_optiondlg() -> Dialog {
    let mut optiondlg = Dialog::new(50, 6);
    let inc_label = "+ -> Increase Radius";
    let dec_label = "+ -> Decrease Radius";
    let q_label = "q -> Quit";
    let inc_pos = optiondlg.window().halign_line(inc_label, HorizontalAlign::Left, 1);
    let dec_pos = optiondlg.window().halign_line(dec_label, HorizontalAlign::Left, 1);
    let q_pos = optiondlg.window().halign_line(q_label, HorizontalAlign::Left, 1);
    optiondlg.window_mut().printline(inc_pos, 1, inc_label);
    optiondlg.window_mut().printline(dec_pos, 2, dec_label);
    optiondlg.window_mut().printline(q_pos, 3, q_label);
    optiondlg.window_mut().draw_box();
    optiondlg
}*/

fn main() {
    /*// Create our terminal, dialog window and main canvas
    let mut term = Terminal::new().unwrap();
    let mut optiondlg = create_optiondlg();
    let mut canvas = Widget::new(term.size().0, term.size().1 - 4);

    // Align canvas to top left, and dialog to bottom right
    optiondlg.window_mut().align(&term, HorizontalAlign::Right, VerticalAlign::Bottom, 0);
    canvas.align(&term, HorizontalAlign::Left, VerticalAlign::Top, 0);

    let first_ship = Ship { x: 1, y: 2, length: 4, horizontal: true, health_points: 4 };
    let second_ship = Ship { x: 5, y: 2, length: 2, horizontal: false, health_points: 2 };
    let current_ship_index = 1;

    let mut first_board = Board::new(5, 8, [ first_ship, second_ship ]);
    first_board.compute_state();
    let mut saved_board;

    'main: loop {
        // make board copy here
        saved_board = first_board.clone();

        while let Some(Event::Key(ch)) = term.get_event(0).unwrap() {
            match ch {
                'q' => break 'main,
                'l' => first_board.ships[current_ship_index].move_right(),
                'h' => first_board.ships[current_ship_index].move_left(),
                'k' => first_board.ships[current_ship_index].move_up(),
                'j' => first_board.ships[current_ship_index].move_down(),
                _ => {},
            }
        }
        // Grab the size of the canvas
        let (cols, rows) = canvas.size();
        let (cols, rows) = (cols as isize, rows as isize);

        // if board not valid after move, restore previous state
        if !first_board.compute_state() {
            first_board = saved_board.clone();
            // We should have a valid state now as we assume the previous state to be valid
            if !first_board.compute_state() {
                panic!("This should not happen!");
            }
        }

        for y in 0..battleship::H - 1  {
            for x in 0..battleship::W - 1 {
                let dest_x = x + first_board.left;
                let dest_y = y + first_board.top;
                let mut cell = canvas.get_mut(dest_x + 1 as usize, dest_y + 1 as usize).unwrap();
                let mut output = ' ';
                match first_board.state[x][y] {
                    0 => output = ' ',
                    _ => output = format!("{}", first_board.state[x][y]).pop().unwrap(),
                      //  format!("{}", first_board.state[x][y]).pop().unwrap(),
                }
                cell.set_ch(output);
            }
        }

        for i in 0..battleship::W + 1 {
            let dest_i = i + first_board.left;
            canvas.get_mut(dest_i as usize, first_board.top).unwrap().set_ch('-');
            canvas.get_mut(dest_i as usize, first_board.top + battleship::H  as usize).unwrap().set_ch('-');
        }

        for i in 0..battleship::H + 1 {
            let dest_i = i + first_board.top;
            canvas.get_mut(first_board.left, dest_i as usize).unwrap().set_ch('|');
            canvas.get_mut(first_board.left + battleship::W  as usize, dest_i as usize).unwrap().set_ch('|');
        }

        canvas.draw_into(&mut term);
        optiondlg.window().draw_into(&mut term);
        term.swap_buffers().unwrap();
    }*/
}
