extern crate rand;
extern crate termion;

use crate::game::models::{Action, Block};
use rand::Rng;
use std::io::{Stdout, Write, stdin, stdout};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::{clear, color, cursor};

pub fn start_game() {
    // Enter raw mode
    let mut stdout = stdout().into_raw_mode().unwrap();

    let fall_time: u64 = 1700; // the time it takes for a block to fall one row in milliseconds

    let visible_rows: u16 = 20;
    let hidden_rows: u16 = 7;
    let rows: u16 = visible_rows + hidden_rows;
    let cols: u16 = 10;

    // Initialize the 2D board
    let mut board: Vec<Vec<u8>> = vec![vec![0; cols as usize]; rows as usize];

    // Board cell representation:
    // 0 = empty cell
    // 1 = stationary cell

    // Track the coordinates of the current block using a vector with size 4 of vectors of size 2
    // Since we know every tetris block has 4 cells, and each cell has a 2D coordinate, we can track
    // the coordinates of the current block using a vector with size 4 of vectors of size 2.
    let mut falling_cells_coords: Vec<(usize, usize)> = generate_block();

    // Create a channel (MPSC)
    let (tx1, rx) = mpsc::channel();
    let tx2 = tx1.clone();

    // Spawn threads to handle game logic and user input
    spawn_game_logic_thread(tx1, fall_time);
    spawn_user_input_thread(tx2);

    for received in rx {
        // Perform game action, if possible
        match received {
            Action::MoveLeft => {
                // Check if the block can move left (not board edge and no stationary cells)
                if falling_cells_coords
                    .iter()
                    .all(|&(row, col)| col > 0 && board[row][col-1] == 0) {
                    // Move the block left
                    for (_row, col) in &mut falling_cells_coords {
                        *col -= 1;
                    }
                }
            }
            Action::MoveRight => {
                // Check if the block can move right (not board edge and no stationary cells)
                if falling_cells_coords
                    .iter()
                    .all(|&(row, col)| col < board[0].len()-1 && board[row][col+1] == 0) {
                    // Move the block right
                    for (_row, col) in &mut falling_cells_coords {
                        *col += 1;
                    }
                }
            }
            Action::MoveDown => {
                // Check if the block can move down (not board bottom and no stationary cells)
                if falling_cells_coords
                    .iter()
                    .all(|&(row, col)| row < board.len()-1 && board[row+1][col] == 0) {
                    // Move the block down
                    for (row, _col) in &mut falling_cells_coords {
                        *row += 1;
                    }
                }
            }
            Action::UserQuit => {
                break;
            }
        }

        // If there is a block collision (with edges of the board or stationary cells)
        if block_collided(&board, &falling_cells_coords) {
            // Mark the block cells as stationary
            for &(row, col) in &falling_cells_coords {
                board[row][col] = 1;
            }

            // Remove any completed rows
            remove_complete_rows(&mut board);

            // Generate a new block
            falling_cells_coords = generate_block();
        }

        // Print updated game board to terminal
        print_board(&board, &falling_cells_coords, &mut stdout);

        // Check if game is over
        if game_is_over(&board) {
            println!("Game Over!");
            break;
        }
    }
}

fn spawn_game_logic_thread(tx: mpsc::Sender<Action>, fall_time: u64) {
    thread::Builder::new()
        .name("game_logic_thread".to_string())
        .spawn(move || {
            loop {
                // Wait for duration of fall time
                thread::sleep(Duration::from_millis(fall_time));

                // Move the current block one row down
                // If transmission fails -> break loop -> kill thread
                if tx.send(Action::MoveDown).is_err() {
                    break;
                }
            }
        })
        .unwrap();
}

fn spawn_user_input_thread(tx: mpsc::Sender<Action>) {
    thread::Builder::new()
        .name("user_input_thread".to_string())
        .spawn(move || {
            let stdin = stdin();

            for res_key in stdin.keys() {
                match res_key {
                    Ok(key) => {
                        let game_action: Action = match key {
                            Key::Left => Action::MoveLeft,
                            Key::Right => Action::MoveRight,
                            Key::Down => Action::MoveDown,
                            Key::Esc => Action::UserQuit,
                            _ => continue,
                        };

                        // Transmit game action to receiver
                        // If transmission fails -> break loop -> kill thread
                        if tx.send(game_action).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        })
        .unwrap();
}

fn generate_block() -> Vec<(usize, usize)> {
    let mut rng = rand::rng();

    // Generate a random Block
    let spawn_block: Block = Block::random();
    let block_matrix: Vec<Vec<u8>> = spawn_block.get_matrix();
    let block_width: usize = block_matrix[0].len();
    let block_height: usize = block_matrix.len();

    let spawn_row: usize = 0;
    let spawn_col: usize = rng.random_range(0..(10 - block_width));

    // Board coordinates of the cells of the new block. There are 4 cells in a block.
    let mut new_block_cell_coords: Vec<(usize, usize)> = Vec::with_capacity(4);

    for (row_idx, row) in block_matrix.iter().enumerate().take(block_height) {
        for (col_idx, cell) in row.iter().enumerate() {
            // Check if the cell in the matrix is a block cell (=1) or an empty cell (=0)
            if *cell == 1 {
                // Calculate the relative coordinates of the cell in the board
                new_block_cell_coords.push((spawn_row + row_idx, spawn_col + col_idx));
            }
        }
    }

    new_block_cell_coords
}

fn game_is_over(board: &Vec<Vec<u8>>) -> bool {
    // Check if the 6th row (the last hidden row) has any stationary cells, then the game is over.
    board[6].iter().any(|&cell| cell == 1)
}

fn block_collided(board: &Vec<Vec<u8>>, falling_cells_coords: &Vec<(usize, usize)>) -> bool {
    // Check if any falling cell reached the bottom of the board or is touching a stationary cell
    falling_cells_coords
        .iter()
        .any(|&(row, col)| row == board.len()-1 || board[row+1][col] == 1)
}

fn remove_complete_rows(board: &mut Vec<Vec<u8>>) {
    // first visible row is 7, so start at row 7 and check till row 27 (inclusive).
    for row in 7..board.len() {
        if board[row].iter().all(|&cell| cell == 1) {
            // Remove the completed row
            board.remove(row);
            // Add a new row at the top of the board
            board.insert(0, vec![0; board[row].len()]);
        }
    }
}

fn print_board(
    board: &Vec<Vec<u8>>,
    falling_cells_coords: &Vec<(usize, usize)>,
    stdout: &mut RawTerminal<Stdout>
) {
    // This variable is used to determine the first row that will be visible on the terminal
    let first_visible_row: usize = 7;

    // Clear the screen and reset the cursor position
    write!(stdout, "{}{}", clear::All, cursor::Goto(1, 1)).unwrap();

    let mut board_repr: String = String::new();

    // Slice the board from the first visible row index to the end
    for (row_idx, row) in board.iter().enumerate() {
        board_repr.push('|');
        for (col_idx, cell) in row.iter().enumerate() {
            if falling_cells_coords.contains(&(row_idx, col_idx)) {
                board_repr.push_str(
                    &format!("{}[]{}", color::Fg(color::Yellow), color::Fg(color::Reset))
                );
            } else {
                match cell {
                    1 => board_repr.push_str(
                        &format!("{}[]{}", color::Fg(color::Green), color::Fg(color::Reset))
                    ),
                    _ => board_repr.push_str("  ")
                }
            }
        }
        board_repr.push_str("|\r\n");
    }

    // Print the bottom line of the board
    board_repr.push_str(
        &format!("⎣{}⎦", std::iter::repeat_n("==", 10).collect::<String>())
    );

    // Print and flush the output to ensure it is displayed
    writeln!(stdout, "{}", board_repr).unwrap();
    stdout.flush().unwrap();
}
