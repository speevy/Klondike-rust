pub mod card_game;

use card_game::klondike::*;
use card_game::card_containers::SimpleCardMover;

fn main() {
    let mut mover = SimpleCardMover {};
    let klondike = Klondike::new(&mut mover);

    println!("{:#?}", klondike.get_status());


}
