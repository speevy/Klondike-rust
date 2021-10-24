pub mod card_game;

use card_game::american_cards::*;
use card_game::card_containers::CardMover;
use card_game::klondike::*;
use std::io::{self, BufRead};
use ansi_term::Style;

fn main() {
    let mut klondike = card_game::klondike::new();

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
                _ =>{}
            } 
        }
    }
}

fn get_card_holder (str: Option<&str>) -> Option<CardHolder> {
    match str {
        Some("d") | Some("D") => Some(CardHolder::DECK),

        Some("p1") | Some("P1") => Some(CardHolder::PILE(0)),
        Some("p2") | Some("P2") => Some(CardHolder::PILE(1)),
        Some("p3") | Some("P3") => Some(CardHolder::PILE(2)),
        Some("p4") | Some("P4") => Some(CardHolder::PILE(3)),

        Some("f1") | Some("F1")  => Some(CardHolder::FOUNDATION(0)),
        Some("f2") | Some("F2")  => Some(CardHolder::FOUNDATION(1)),
        Some("f3") | Some("F3")  => Some(CardHolder::FOUNDATION(2)),
        Some("f4") | Some("F4")  => Some(CardHolder::FOUNDATION(3)),
        Some("f5") | Some("F5")  => Some(CardHolder::FOUNDATION(4)),
        Some("f6") | Some("F6")  => Some(CardHolder::FOUNDATION(5)),
        Some("f7") | Some("F7")  => Some(CardHolder::FOUNDATION(6)),

        _ => None
    }
}

fn print_status<T:CardMover> (klondike: &Klondike<T>) {
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
        "Commands: {}: Exit {}: Take from stock {}: move cards",
        style.paint("X"),
        style.paint("T"),
        style.paint("M <origin> <destination> [number of cards]"),

        ); 
    println!("");

}

fn fmt_pile_card (card: Option<Card>) -> String {
    format!("{}", match card {
        None => "   ".to_string(),
        Some(card) => format!("{}", card) 
    })
}

fn fmt_found_card (foundation: &card_game::klondike::foundation::FoundationStatus, index: u32) -> String {

    if index < foundation.num_hidden {
        "[ X ]".to_string()
    } else {
        format!("{}", match foundation.visible.get((index - foundation.num_hidden) as usize) {
            None => "     ".to_string(),
            Some(card) => format!("[{}]", card) 
        })
    }
}

