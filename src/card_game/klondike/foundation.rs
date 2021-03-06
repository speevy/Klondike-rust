use crate::card_game::american_cards::*;
use crate::card_game::card_containers::*;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq)]
pub struct Foundation {
    hidden: Vec<Card>,
    visible: Vec<Card>,
    peek_caused_flip: Vec<bool>
}

/// Value object used by UI for representing the status of a Foundation
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FoundationStatus {
    pub num_hidden: u32,
    pub visible: Vec<Card>,
}

/// Each of the foundations of the game. Acts as a Card Origin an Card Destination.
/// Multiple cards can be peek or poke at a time.
/// When poking one or various cards, it has to alternate suit colors and 
/// the next rank has to be the immediate previous value of the rank of the 
/// preceding card. If the foundation is empty a KING of any suit is allowed.
impl Foundation {
    pub fn new(cards: Vec<Card>) -> Foundation {
        Foundation {
            hidden: cards[..cards.len() - 1].to_vec(),
            visible: cards[cards.len() - 1..].to_vec(),
            peek_caused_flip: Vec::new(),
        }
    }

    fn can_peek(&self, number: usize) -> bool {
        number > 0 && number <= self.visible.len()
    }

    pub fn get_status(&self) -> FoundationStatus {
        FoundationStatus {
            num_hidden: self.hidden.len() as u32,
            visible: self.visible[..].to_vec(),
        }
    }
}

impl CardOrigin for Foundation {
    fn try_peek(&self, number: usize) -> Option<Vec<Card>> {
        if self.can_peek(number) {
            return Some(self.visible[self.visible.len() - number..].to_vec());
        }
        return None;
    }

    fn peek(&mut self, number: usize) -> Vec<Card> {
        if self.can_peek(number) {
            let res: Vec<Card> = self.visible.drain(self.visible.len() - number..).collect();

            let mut flipped = false;
            if self.visible.is_empty() {
                if let Some(card) = self.hidden.pop() {
                    self.visible.push(card);
                    flipped = true;
                }
            }

            self.peek_caused_flip.push(flipped);
            return res;
        }
        return Vec::new();
    }

    fn undo_peek(&mut self, cards: &Vec<Card>) {
        // If the peek we are undoing caused to show a hidden card, we have to
        // hide it again
        let flipped = self.peek_caused_flip.pop().unwrap_or(false);
        if flipped && self.visible.len() == 1 && cards.len() > 0 {
            self.hidden.push(self.visible.pop().unwrap());
        }
        self.visible.append(&mut cards.to_vec());
    }
}

impl CardDestination for Foundation {
    fn try_poke(&self, cards: &Vec<Card>) -> bool {
        if cards.is_empty() {
            return false;
        }

        if self.visible.is_empty() {
            return cards[0].rank == CardRank::KING;
        }

        Card::check_alternate_colors_and_descending_rank(
            self.visible[self.visible.len() - 1], 
            cards[0])
    }

    fn poke(&mut self, cards: &Vec<Card>) {
        if self.try_poke(cards) {
            self.visible.append(&mut cards.to_vec());
        }
    }

    fn undo_poke(&mut self, number: usize) -> Vec<Card> {
        if self.visible.len() >= number {
            return self.visible.drain(self.visible.len() - number..).collect();
        }
        Vec::new()
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card_game::card_containers::test_common::*;
    #[test]
    fn foundation_new() {
        let cards = vec![
            Card {
                suit: CardSuit::DIAMONDS,
                rank: CardRank::ACE,
            },
            Card {
                suit: CardSuit::DIAMONDS,
                rank: CardRank::TWO,
            },
            Card {
                suit: CardSuit::DIAMONDS,
                rank: CardRank::THREE,
            },
        ];
        let found = Foundation::new(cards);

        assert_eq!(found.hidden.len(), 2);
        assert_eq!(
            found.hidden[0],
            Card {
                suit: CardSuit::DIAMONDS,
                rank: CardRank::ACE,
            }
        );
        assert_eq!(
            found.hidden[1],
            Card {
                suit: CardSuit::DIAMONDS,
                rank: CardRank::TWO,
            }
        );
        assert_eq!(found.visible.len(), 1);
        assert_eq!(
            found.visible[0],
            Card {
                suit: CardSuit::DIAMONDS,
                rank: CardRank::THREE,
            }
        );
    }

    #[test]
    fn foundation_new_one() {
        let cards = vec![Card {
            suit: CardSuit::DIAMONDS,
            rank: CardRank::ACE,
        }];
        let found = Foundation::new(cards);

        assert_eq!(found.hidden.len(), 0);
        assert_eq!(found.visible.len(), 1);
        assert_eq!(
            found.visible[0],
            Card {
                suit: CardSuit::DIAMONDS,
                rank: CardRank::ACE,
            }
        );
    }

    #[test]
    fn foundation_peek() {
        foundation_peek_test(3, 1, 1, 2, 1);
        foundation_peek_test(3, 2, 1, 3, 1);
        foundation_peek_test(3, 2, 2, 2, 1);
        foundation_peek_test(1, 1, 1, 0, 1);
        foundation_peek_test(0, 1, 1, 0, 0);
        foundation_peek_test(0, 2, 1, 0, 1);
        foundation_peek_test(0, 2, 2, 0, 0);
    }

    fn foundation_peek_test(
        hidden: usize,
        visible: usize,
        peek: usize,
        remaining_hidden: usize,
        remaining_visible: usize,
    ) {
        let mut found = create_test_foundation(hidden, 0, visible);

        assert_eq!(
            found.try_peek(peek),
            Some(generate_descending_alt_color_starting(visible - peek, peek))
        );
        assert_eq!(
            found.peek(peek),
            generate_descending_alt_color_starting(visible - peek, peek)
        );
        assert_eq!(found.hidden.len(), remaining_hidden);
        assert_eq!(found.visible.len(), remaining_visible);
    }

    #[test]
    fn foundation_peek_overflow() {
        let mut found = create_test_foundation(3, 0, 1);
        assert_eq!(found.try_peek(2), None);
        assert_eq!(found.peek(2).is_empty(), true);
    }

    fn create_test_foundation(
        hidden: usize,
        visible_start: usize,
        visible_number: usize,
    ) -> Foundation {
        Foundation {
            hidden: generate_random_card_set(hidden),
            visible: generate_descending_alt_color_starting(visible_start, visible_number),
            peek_caused_flip: Vec::new(),
        }
    }

    #[test]
    fn foundation_poke() {
        foundation_poke_case_ok(0, 1, 1);
        foundation_poke_case_ok(0, 2, 1);
        foundation_poke_case_ok(0, 1, 2);
        foundation_poke_case_ok(1, 1, 1);
        foundation_poke_case_ok(2, 2, 2);
    }

    fn foundation_poke_case_ok(visible_start: usize, visible_size: usize, to_add: usize) {
        let mut foun = create_test_foundation(1, visible_start, visible_size);
        let cards = generate_descending_alt_color_starting(visible_start + visible_size, to_add);
        assert_eq!(foun.try_poke(&cards), true);
        foun.poke(&cards);
        assert_eq!(
            foun.visible,
            generate_descending_alt_color_starting(visible_start, visible_size + to_add)
        );
    }

    #[test]
    fn foundation_poke_ko() {
        foundation_poke_case_ko(
            0,
            4, //last card is 10 CLUBS
            vec![Card {
                suit: CardSuit::DIAMONDS,
                rank: CardRank::SIX,
            }],
        );
        foundation_poke_case_ko(
            0,
            4, //last card is 10 CLUBS
            vec![Card {
                suit: CardSuit::HEARTS,
                rank: CardRank::SIX,
            }],
        );
        foundation_poke_case_ko(
            0,
            4, //last card is 10 CLUBS
            vec![Card {
                suit: CardSuit::CLUBS,
                rank: CardRank::NINE,
            }],
        );
        foundation_poke_case_ko(
            0,
            4, //last card is 10 CLUBS
            vec![Card {
                suit: CardSuit::SPADES,
                rank: CardRank::NINE,
            }],
        );
        foundation_poke_case_ko(
            0,
            5, //last card is 9 DIAMONDS
            vec![Card {
                suit: CardSuit::CLUBS,
                rank: CardRank::SIX,
            }],
        );
        foundation_poke_case_ko(
            0,
            4, //last card is 10 CLUBS
            vec![Card {
                suit: CardSuit::SPADES,
                rank: CardRank::SIX,
            }],
        );
        foundation_poke_case_ko(
            0,
            4, //last card is 10 CLUBS
            vec![Card {
                suit: CardSuit::HEARTS,
                rank: CardRank::EIGHT,
            }],
        );
        foundation_poke_case_ko(
            0,
            4, //last card is 10 CLUBS
            vec![Card {
                suit: CardSuit::DIAMONDS,
                rank: CardRank::EIGHT,
            }],
        );
        foundation_poke_case_ko(
            0,
            4, //last card is 10 CLUBS
            vec![Card {
                suit: CardSuit::DIAMONDS,
                rank: CardRank::FIVE,
            }],
        );
    }

    fn foundation_poke_case_ko(visible_start: usize, visible_size: usize, to_add: Vec<Card>) {
        let mut foun = create_test_foundation(1, visible_start, visible_size);
        assert_eq!(foun.try_poke(&to_add), false);
        foun.poke(&to_add);
        assert_eq!(
            foun.visible,
            generate_descending_alt_color_starting(visible_start, visible_size)
        );
    }

    #[test]
    fn foundation_status() {
        let mut foun = create_test_foundation(5, 0, 2);
        let status = foun.get_status();
        assert_eq!(status.num_hidden, 5);
        assert_eq!(status.visible.len(), 2);

        foun.hidden.clear();
        let status = foun.get_status();
        assert_eq!(status.num_hidden, 0);
        assert_eq!(status.visible.len(), 2);

        foun.visible.pop();
        let status = foun.get_status();
        assert_eq!(status.num_hidden, 0);
        assert_eq!(status.visible.len(), 1);

        foun.visible.clear();
        let status = foun.get_status();
        assert_eq!(status.num_hidden, 0);
        assert_eq!(status.visible.len(), 0);
    }

    #[test]
    fn foundation_undo_peek() {
        foundation_undo_peek_case(3, 1, 1);
        foundation_undo_peek_case(3, 2, 1);
        foundation_undo_peek_case(3, 2, 2);
        foundation_undo_peek_case(1, 1, 1);
        foundation_undo_peek_case(0, 1, 1);
        foundation_undo_peek_case(0, 2, 1);
        foundation_undo_peek_case(0, 2, 2);
    }

    fn foundation_undo_peek_case(hidden: usize, visible: usize, peek: usize) {
        let mut found = create_test_foundation(hidden, 0, visible);
        let status = found.get_status();
        let cards = found.peek(peek);

        found.undo_peek(&cards);

        assert_eq!(status, found.get_status());
    }

    #[test]
    fn foundation_undo_poke() {
        foundation_undo_poke_case(0, 1, 1);
        foundation_undo_poke_case(0, 2, 1);
        foundation_undo_poke_case(0, 1, 2);
        foundation_undo_poke_case(1, 1, 1);
        foundation_undo_poke_case(2, 2, 2);
    }

    fn foundation_undo_poke_case(visible_start: usize, visible_size: usize, to_add: usize) {
        let mut foun = create_test_foundation(1, visible_start, visible_size);
        let cards = generate_descending_alt_color_starting(visible_start + visible_size, to_add);

        let status = foun.get_status();
        foun.poke(&cards);

        assert_eq!(foun.undo_poke(cards.len()), cards);
        assert_eq!(foun.get_status(), status);
    }

    #[test]
    fn foundation_undo_peek_unhidden() {

        let visible = vec![Card {rank:CardRank::SEVEN, suit: CardSuit::CLUBS}];
        let hidden = vec![Card {rank:CardRank::EIGHT, suit: CardSuit::HEARTS}];

        let mut found = Foundation { visible, hidden, peek_caused_flip: Vec::new() };

        let status = found.get_status();

        let cards = found.peek(1);
        found.undo_peek(&cards);

        assert_eq! (found.get_status(), status);
    }
}
