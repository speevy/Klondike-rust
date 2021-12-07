use ansi_term::Colour::*;
use std::fmt;
use strum_macros::EnumIter;

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
            CardSuit::SPADES | CardSuit::CLUBS => Blue.paint(str),
        };

        write!(f, "{}", colored)
    }
}

impl Card {

    pub fn check_alternate_colors_and_descending_rank(first: Card, second: Card) -> bool {
        ((second.rank as i32) + 1) == (first.rank as i32)
            && match second.suit {
                CardSuit::DIAMONDS | CardSuit::HEARTS => {
                    first.suit == CardSuit::CLUBS || first.suit == CardSuit::SPADES
                }
                CardSuit::CLUBS | CardSuit::SPADES => {
                    first.suit == CardSuit::DIAMONDS || first.suit == CardSuit::HEARTS
                }
            }

    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn card_alternating_check() {
        card_alternating_check_case(CardSuit::DIAMONDS, CardRank::FIVE, CardSuit::CLUBS, CardRank::FOUR, true);
        card_alternating_check_case(CardSuit::DIAMONDS, CardRank::FIVE, CardSuit::HEARTS, CardRank::FOUR, false);
        card_alternating_check_case(CardSuit::DIAMONDS, CardRank::FIVE, CardSuit::SPADES, CardRank::FOUR, true);
        card_alternating_check_case(CardSuit::DIAMONDS, CardRank::FIVE, CardSuit::DIAMONDS, CardRank::FOUR, false);
        card_alternating_check_case(CardSuit::DIAMONDS, CardRank::FIVE, CardSuit::CLUBS, CardRank::THREE, false);
        card_alternating_check_case(CardSuit::DIAMONDS, CardRank::FIVE, CardSuit::HEARTS, CardRank::THREE, false);
        card_alternating_check_case(CardSuit::DIAMONDS, CardRank::FIVE, CardSuit::SPADES, CardRank::THREE, false);
        card_alternating_check_case(CardSuit::DIAMONDS, CardRank::FIVE, CardSuit::DIAMONDS, CardRank::THREE, false);
    }

    fn card_alternating_check_case (
        first_suit: CardSuit,
        first_rank: CardRank,
        second_suit: CardSuit,
        second_rank: CardRank,
        result: bool
    ) {
        assert_eq! (
            Card::check_alternate_colors_and_descending_rank(
                Card {suit:first_suit, rank: first_rank}, 
                Card {suit:second_suit, rank: second_rank})
            , result);
    }
}
