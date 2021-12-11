use crate::card_game::american_cards::*;
use crate::card_game::klondike::*;
use std::io::{self, BufRead};
use ansi_term::Style;
use crate::card_game::klondike::ui::get_card_holder;

pub fn game() {
    let mut klondike = Klondike::new();

    let stdin = io::stdin();
    let mut iterator = stdin.lock().lines();
    
    loop {
        print_status(&klondike);

        let line = iterator.next().unwrap().unwrap();
        let mut part = line.split_whitespace();

        if let Some(cmd) = part.next() {
            match cmd {
                "x" | "X" => { 
                    break; 
                }
                "t" | "T" => { 
                    klondike.take(); 
                }
                "m" | "M" | /* these for the un*x gurus ;) */ "mv" | "MV" => {
                    if let Some(origin) = get_card_holder(part.next()) {
                        if let Some(destination) = get_card_holder(part.next()) {
                            if let Ok(number) = part.next().unwrap_or("1").parse::<u32>() {
                                
                                klondike.move_cards(origin, destination, number);
                            }
                        }
                    }
                }
                "p" | "P" => {
                    if let Some(origin) = get_card_holder(part.next()) {
                        klondike.to_pile(origin);
                    }
                }
                "u" | "U" => klondike.undo(),
                _ =>{}
            } 
        }
    }
}

fn print_status (klondike: &Klondike) {
    let status = klondike.get_status();

    println!("  P1    P2    P3    P4          D"); 
    println!("[{}] [{}] [{}] [{}]       [{}] [{:^3}]", 
        fmt_pile_card(status.piles[0].top_card),
        fmt_pile_card(status.piles[1].top_card),
        fmt_pile_card(status.piles[2].top_card),
        fmt_pile_card(status.piles[3].top_card),
        fmt_pile_card(status.deck.top_card_on_waste),
        status.deck.cards_on_stock
        );
    
    println!("");
    println!("  F1    F2    F3    F4    F5    F6    F7"); 
 
    let max_num_cards = status.foundations.iter()
        .map(|x| -> u32 {x.num_hidden + x.visible.len() as u32})
        .max().unwrap();

    for i in 0..max_num_cards {
        for j in 0..7 {
            print!("{} ", fmt_found_card(&status.foundations[j], i));
        }
        println!("");
    }

    println!("");

    let style = Style::new().bold();
    println!(
        "Commands: {}: Exit {}: Take from stock {}: move cards {}: move cards to pile {}: Undo",
        style.paint("X"),
        style.paint("T"),
        style.paint("M <origin> <destination> [number of cards]"),
        style.paint("P <origin>"),
        style.paint("U"),
        ); 
    println!("");

}

fn fmt_pile_card (card: Option<Card>) -> String {
    format!("{}", match card {
        None => "   ".to_string(),
        Some(card) => format!("{}", card) 
    })
}

fn fmt_found_card (foundation: &crate::card_game::klondike::foundation::FoundationStatus, index: u32) -> String {

    if index < foundation.num_hidden {
        "[ X ]".to_string()
    } else {
        format!("{}", match foundation.visible.get((index - foundation.num_hidden) as usize) {
            None => "     ".to_string(),
            Some(card) => format!("[{}]", card) 
        })
    }
}
