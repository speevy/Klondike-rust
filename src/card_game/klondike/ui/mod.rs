pub mod console;
pub mod web;

use crate::card_game::klondike::*;

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
