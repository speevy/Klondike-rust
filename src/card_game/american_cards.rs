use strum_macros::EnumIter;
use std::fmt;
use ansi_term::Colour::*;

#[derive(Debug, EnumIter, Copy, Clone, PartialEq)]
pub enum CardSuit {
    CLUBS,
    DIAMONDS,
    HEARTS,
    SPADES,
}

#[derive(Debug, EnumIter, Copy, Clone, PartialEq)]
pub enum CardRank {
    ACE = 1,
    TWO,
    THREE,
    FOUR,
    FIVE,
    SIX,
    SEVEN,
    EIGHT,
    NINE,
    TEN,
    JACK,
    QUEEN,
    KING,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Card {
    pub suit: CardSuit,
    pub rank: CardRank,
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let str = format!(
            "{:>2}{}",
            match self.rank {
                CardRank::ACE => "A",
                CardRank::TWO => "2",
                CardRank::THREE => "3",
                CardRank::FOUR => "4",
                CardRank::FIVE => "5",
                CardRank::SIX => "6",
                CardRank::SEVEN => "7",
                CardRank::EIGHT => "8",
                CardRank::NINE => "9",
                CardRank::TEN => "10",
                CardRank::JACK => "J",
                CardRank::QUEEN => "Q",
                CardRank::KING => "K",
            },
            match self.suit {
                CardSuit::CLUBS => "♣",
                CardSuit::DIAMONDS => "♦",
                CardSuit::HEARTS => "♥",
                CardSuit::SPADES => "♤",
            }
        );

        let colored = match self.suit {
            CardSuit::DIAMONDS | CardSuit::HEARTS => Red.paint(str),
            CardSuit::SPADES | CardSuit::CLUBS =>  Blue.paint(str),
        };
        write!(f, "{}", colored)
    }
}
