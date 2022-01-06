use crate::card_game::american_cards::*;
use crate::card_game::card_containers::*;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq)]
pub struct Pile {
    cards: Vec<Card>,
}

/// Value object used by UI for representing the status of a Pile
#[derive(Debug, Copy, Clone, PartialEq, Serialize)]
pub struct PileStatus {
    pub top_card: Option<Card>,
    pub num_cards: u32
}

impl Pile {
    pub fn new() -> Pile {
        Pile { cards: vec![] }
    }

    pub fn get_status(&self) -> PileStatus {
        let mut top_card = None;
        if !self.cards.is_empty() {
            top_card = Some(self.cards[self.cards.len() - 1]);
        }

        PileStatus {
            top_card, num_cards: self.cards.len() as u32
        }
    }
}

impl CardOrigin for Pile {
    fn try_peek(&self, number: usize) -> Option<Vec<Card>> {
        if number == 1 && !self.cards.is_empty() {
            return Some(self.cards[self.cards.len() - 1..].to_vec());
        }
        return None;
    }

    fn peek(&mut self, number: usize) -> Vec<Card> {
        if number == 1 {
            match self.cards.pop() {
                Some(card) => {
                    let r = vec![card];
                    return r;
                }
                None => {}
            }
        }
        return Vec::new();
    }

    fn undo_peek(&mut self, cards: &Vec<Card>) {
        if cards.len() == 1 {
            self.cards.push(cards[0]);
        }
    }
}

impl CardDestination for Pile {
    fn try_poke(&self, cards: &Vec<Card>) -> bool {
        if cards.len() != 1 {
            return false;
        }
        let card = cards[0];

        if self.cards.is_empty() {
            return cards[0].rank == CardRank::ACE;
        }
        let last_card = self.cards[self.cards.len() - 1];

        return last_card.suit == card.suit && card.rank as i32 == last_card.rank as i32 + 1;
    }

    fn poke(&mut self, cards: &Vec<Card>) {
        if self.try_poke(cards) {
            self.cards.push(cards[0]);
        }
    }

    fn undo_poke(&mut self, number: usize) -> Vec<Card> {
        let mut res: Vec<Card> = Vec::new();
        
        if number == 1 {
            if let Some(card) = self.cards.pop() {
                res.push(card);
            }
        }

        res
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::card_game::card_containers::test_common::*;


    #[test]
    fn pile_new() {
        let pile = Pile::new();
        assert_eq!(pile.cards.is_empty(), true);
    }

    #[test]
    fn pile_peek_one() {
        let mut pile = create_test_pile();

        assert_peek_one_returns(
            &mut pile,
            Card {
                suit: CardSuit::DIAMONDS,
                rank: CardRank::THREE,
            },
        );

        assert_peek_one_returns(
            &mut pile,
            Card {
                suit: CardSuit::DIAMONDS,
                rank: CardRank::TWO,
            },
        );

        assert_peek_one_returns(
            &mut pile,
            Card {
                suit: CardSuit::DIAMONDS,
                rank: CardRank::ACE,
            },
        );

        assert_eq!(pile.peek(1), Vec::new());
    }

    fn create_test_pile() -> Pile {
        Pile {
            cards: vec![
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
            ],
        }
    }

    #[test]
    fn pile_peek_one_empty() {
        let mut pile = Pile { cards: vec![] };

        assert_eq!(pile.try_peek(1), None);

        assert_eq!(pile.peek(1).len(), 0);
    }

    #[test]
    fn pile_peek_not_one() {
        let mut pile = create_test_pile();

        assert_eq!(pile.try_peek(2), None);
        assert_eq!(pile.peek(2).len(), 0);

        assert_eq!(pile.try_peek(0), None);
        assert_eq!(pile.peek(0).len(), 0);
    }
    #[test]
    fn pile_empty_poke_one_ace() {
        pile_empty_poke_one_ace_of_suit(CardSuit::CLUBS);
        pile_empty_poke_one_ace_of_suit(CardSuit::DIAMONDS);
        pile_empty_poke_one_ace_of_suit(CardSuit::HEARTS);
        pile_empty_poke_one_ace_of_suit(CardSuit::SPADES);
    }

    fn pile_empty_poke_one_ace_of_suit(suit: CardSuit) {
        let mut pile = Pile { cards: vec![] };

        pile_poke_card_expect_size(&mut pile, suit, CardRank::ACE, 1, true);
    }

    #[test]
    fn pile_empty_poke_one_not_ace() {
        let mut pile = Pile { cards: vec![] };

        pile_poke_card_expect_size(&mut pile, CardSuit::DIAMONDS, CardRank::FOUR, 0, false);
    }

    #[test]
    fn pile_poke_next_rank_same_suit() {
        let mut pile = create_test_pile();

        pile_poke_card_expect_size(&mut pile, CardSuit::DIAMONDS, CardRank::FOUR, 4, true);
    }

    #[test]
    fn pile_poke_next_rank_other_suit() {
        let mut pile = create_test_pile();

        pile_poke_card_expect_size(&mut pile, CardSuit::CLUBS, CardRank::FOUR, 3, false);
        pile_poke_card_expect_size(&mut pile, CardSuit::HEARTS, CardRank::FOUR, 3, false);
        pile_poke_card_expect_size(&mut pile, CardSuit::SPADES, CardRank::FOUR, 3, false);
    }

    #[test]
    fn pile_poke_not_next_rank() {
        let mut pile = create_test_pile();

        pile_poke_card_expect_size(&mut pile, CardSuit::CLUBS, CardRank::FIVE, 3, false);
        pile_poke_card_expect_size(&mut pile, CardSuit::DIAMONDS, CardRank::FIVE, 3, false);
        pile_poke_card_expect_size(&mut pile, CardSuit::HEARTS, CardRank::FIVE, 3, false);
        pile_poke_card_expect_size(&mut pile, CardSuit::SPADES, CardRank::FIVE, 3, false);
    }

    fn pile_poke_card_expect_size(
        pile: &mut Pile,
        suit: CardSuit,
        rank: CardRank,
        size: usize,
        try_result: bool,
    ) {
        let card = Card { suit, rank };

        assert_eq!(pile.try_poke(&vec![card]), try_result);

        pile.poke(&vec![card]);

        assert_eq!(pile.cards.len(), size);
        if try_result {
            assert_eq!(pile.cards[size - 1], card);
        }
    }

    #[test]
    fn pile_status() {
        let mut pile = create_test_pile();
        let status = pile.get_status();
        assert_eq!(status.num_cards, 3);
        assert_eq!(status.top_card,
            Some(Card {
                suit: CardSuit::DIAMONDS,
                rank: CardRank::THREE,
            }));

        pile.cards.pop();
        let status = pile.get_status();
        assert_eq!(status.num_cards, 2);
        assert_eq!(status.top_card,
            Some(Card {
                suit: CardSuit::DIAMONDS,
                rank: CardRank::TWO,
            }));

        pile.cards.clear();
        let status = pile.get_status();
        assert_eq!(status.num_cards, 0);
        assert_eq!(status.top_card, None);
    }

    #[test]
    fn pile_undo_peek() {
        const NUMBER_OF_UNDOS:u32 = 10;
        let mut pile = create_test_pile();
        let mut history_status:Vec<PileStatus> = Vec::new();
        let mut history_cards:Vec<Vec<Card>> = Vec::new();

        for _i in 0..NUMBER_OF_UNDOS {
            history_status.push(pile.get_status());
            history_cards.push(pile.peek(1));
        }

        for _i in 0..NUMBER_OF_UNDOS {
            pile.undo_peek(&history_cards.pop().unwrap());
            assert_eq!(history_status.pop().unwrap(), pile.get_status())
        }
    }

    #[test]
    fn pile_undo_poke() {
        let mut pile = Pile::new();
        let first: Vec<Card> = vec![Card {suit:CardSuit::HEARTS, rank: CardRank::ACE}];
        let second: Vec<Card> = vec![Card {suit:CardSuit::HEARTS, rank: CardRank::TWO}];
        let third: Vec<Card> = vec![Card {suit:CardSuit::HEARTS, rank: CardRank::THREE}];

        pile.poke(&first);
        pile.poke(&second);
        pile.poke(&third);

        assert_eq!(pile.undo_poke(1), third);
        assert_eq!(pile.cards.len(), 2);

        assert_eq!(pile.undo_poke(1), second);
        assert_eq!(pile.cards.len(), 1);

        assert_eq!(pile.undo_poke(1), first);
        assert_eq!(pile.cards.len(), 0);

    }

}