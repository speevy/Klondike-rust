pub mod card_game;

#[macro_use] extern crate rocket;
#[macro_use] extern crate lazy_static;

use card_game::klondike::ui::*;
use std::env;

fn main() {
    if env::args().any(|arg| -> bool {"-web".eq(&arg)}) {
        if let Err(e) = web::main_rocket() {
            println!("Whoops! Rocket didn't launch!");
            // We drop the error to get a Rocket-formatted panic.
            drop(e);
        }
    } else {
        console::game();
    }
}

