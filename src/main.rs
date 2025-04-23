mod game;

extern crate termion;

pub use game::logic::start_game;

use std::io::{self, Write};
use std::process::exit;
use termion::{clear, cursor};


macro_rules! prompt {
    ($msg:expr, $input:expr) => {
        // Print the prompt message to terminal
        print!("{}", $msg);
        io::stdout().flush().unwrap();

        io::stdin().read_line(&mut $input).unwrap();
    };
}

macro_rules! clear_screen {
    () => {
        print!("{}{}", clear::All, cursor::Goto(1, 1));
        io::stdout().flush().unwrap();
    };
}



fn main() {
    // Clear the terminal screen
    clear_screen!();
    println!("Welcome to the Tetris game!");

    let mut menu_choice: String = String::new();
    prompt!(
        "\
    [S] To start the game \n\
    [Q] To quit the game \n\
    > ",
        menu_choice
    );

    match menu_choice.trim().to_uppercase().as_str() {
        "S" => start_game(),
        _ => exit(0),
    }
}
