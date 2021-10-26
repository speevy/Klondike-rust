use crate::card_game::american_cards::*;
use crate::card_game::card_containers::*;
/// The deck of the game, consisting in two piles: the stock and the waste.
/// The waste also acts as a CardOrigin.
pub struct Deck {
    stock: Vec<Card>,
    waste: Vec<Card>,
}

/// Value object used by UI for representing the status of a Deck
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct DeckStatus {
    pub cards_on_waste: u32,
    pub cards_on_stock: u32,
    pub top_card_on_waste: Option<Card>
}

impl CardOrigin for Deck {
    fn peek(&mut self, number: usize) -> Vec<Card> {
        if number == 1 {
            match self.waste.pop() {
                Some(card) => {
                    let r = vec![card];
                    return r;
                }
                None => {}
            }
        }
        return Vec::new();
    }

    fn try_peek(&self, number: usize) -> Option<Vec<Card>> {
        if number == 1 && self.waste.len() > 0 {
            return Some(self.waste[self.waste.len() - 1..].to_vec());
        }
        return None;
    }
}

impl Deck {
    ///Creates a deck containing the given cards. One of the cards goes to
    ///the waste, the others to the pile.
    pub fn new(cards: &Vec<Card>) -> Deck {
        let mut deck = Deck {
            stock: cards.to_vec(),
            waste: Vec::new(),
        };

        deck.take();
        return deck;
    }

    ///Moves one card from the pile to the waste.
    ///If the pile is empty, all the waste cards are moved to the pile.
    ///If both the pile and the waste are empty, nothing is done.
    pub fn take(&mut self) {
        if self.stock.is_empty() && !self.waste.is_empty() {
            self.waste.reverse();
            self.stock.append(&mut self.waste);
        }

        match self.stock.pop() {
            Some(card) => {
                self.waste.push(card);
            }
            None => {}
        }
    }

    pub fn get_status(&self) -> DeckStatus {
        let mut top_card_on_waste = None;
        if !self.waste.is_empty() {
            top_card_on_waste = Some(self.waste[self.waste.len() - 1]);
        }

        DeckStatus {
            cards_on_waste: self.waste.len() as u32,
            cards_on_stock: self.stock.len() as u32,
            top_card_on_waste
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card_game::card_containers::test_common::*;

    #[test]
    fn deck_peek_one() {
        let mut deck = create_test_deck();

        assert_peek_one_returns(
            &mut deck,
            Card {
                suit: CardSuit::CLUBS,
                rank: CardRank::THREE,
            },
        );

        assert_peek_one_returns(
            &mut deck,
            Card {
                suit: CardSuit::CLUBS,
                rank: CardRank::TWO,
            },
        );

        assert_peek_one_returns(
            &mut deck,
            Card {
                suit: CardSuit::CLUBS,
                rank: CardRank::ACE,
            },
        );

        assert_eq!(deck.peek(1), Vec::new());
    }

    #[test]
    fn deck_peek_one_empty() {
        let mut deck = create_test_deck();

        deck.waste.clear();

        assert_eq!(deck.try_peek(1), None);

        assert_eq!(deck.peek(1).len(), 0);
    }
    #[test]
    fn deck_peek_not_one() {
        let mut deck = create_test_deck();

        assert_eq!(deck.try_peek(2), None);
        assert_eq!(deck.peek(2).len(), 0);

        assert_eq!(deck.try_peek(0), None);
        assert_eq!(deck.peek(0).len(), 0);
    }

    fn create_test_deck() -> Deck {
        Deck {
            stock: vec![
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
            waste: vec![
                Card {
                    suit: CardSuit::CLUBS,
                    rank: CardRank::ACE,
                },
                Card {
                    suit: CardSuit::CLUBS,
                    rank: CardRank::TWO,
                },
                Card {
                    suit: CardSuit::CLUBS,
                    rank: CardRank::THREE,
                },
            ],
        }
    }

    #[test]
    fn deck_init() {
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
            Card {
                suit: CardSuit::CLUBS,
                rank: CardRank::ACE,
            },
            Card {
                suit: CardSuit::CLUBS,
                rank: CardRank::TWO,
            },
            Card {
                suit: CardSuit::CLUBS,
                rank: CardRank::THREE,
            },
        ];

        let deck = Deck::new(&cards);

        assert_eq!(deck.stock.len(), 5);
        assert_eq!(deck.waste.len(), 1);
        assert_eq!(
            deck.waste[0],
            Card {
                suit: CardSuit::CLUBS,
                rank: CardRank::THREE
            }
        );
    }

    #[test]
    fn deck_take() {
        let mut deck = create_test_deck();

        deck.take();
        assert_deck(&deck, 2, 4, CardSuit::DIAMONDS, CardRank::THREE);

        deck.take();
        assert_deck(&deck, 1, 5, CardSuit::DIAMONDS, CardRank::TWO);

        deck.take();
        assert_deck(&deck, 0, 6, CardSuit::DIAMONDS, CardRank::ACE);

        deck.take();
        assert_deck(&deck, 5, 1, CardSuit::CLUBS, CardRank::ACE);

        deck.take();
        assert_deck(&deck, 4, 2, CardSuit::CLUBS, CardRank::TWO);

        deck.take();
        assert_deck(&deck, 3, 3, CardSuit::CLUBS, CardRank::THREE);

        deck.take();
        assert_deck(&deck, 2, 4, CardSuit::DIAMONDS, CardRank::THREE);
    }

    fn assert_deck(
        deck: &Deck,
        stock_len: usize,
        waste_len: usize,
        suit: CardSuit,
        rank: CardRank,
    ) {
        assert_eq!(deck.stock.len(), stock_len);
        assert_eq!(deck.waste.len(), waste_len);
        assert_eq!(deck.try_peek(1), Some(vec![Card { suit, rank }]));
    }

    #[test]
    fn deck_take_empty() {
        let mut deck = Deck {
            stock: Vec::new(),
            waste: Vec::new(),
        };
        deck.take();

        assert_eq!(deck.stock.len(), 0);
        assert_eq!(deck.waste.len(), 0);
    }

    #[test]
    fn deck_satus() {
        let mut deck = create_test_deck();
        let status = deck.get_status();
        assert_eq!(status.cards_on_stock, 3);
        assert_eq!(status.cards_on_waste, 3);
        assert_eq!(status.top_card_on_waste, Some(Card {
            suit: CardSuit::CLUBS,
            rank: CardRank::THREE,
        }));

        deck.waste.pop();

        let status = deck.get_status();
        assert_eq!(status.cards_on_stock, 3);
        assert_eq!(status.cards_on_waste, 2);
        assert_eq!(status.top_card_on_waste, Some(Card {
            suit: CardSuit::CLUBS,
            rank: CardRank::TWO,
        }));

        deck.waste.clear();

        let status = deck.get_status();
        assert_eq!(status.cards_on_stock, 3);
        assert_eq!(status.cards_on_waste, 0);
        assert_eq!(status.top_card_on_waste, None);

        deck.stock.clear();

        let status = deck.get_status();
        assert_eq!(status.cards_on_stock, 0);
        assert_eq!(status.cards_on_waste, 0);
        assert_eq!(status.top_card_on_waste, None);

        let mut deck = create_test_deck();
        deck.stock.clear();
        let status = deck.get_status();
        assert_eq!(status.cards_on_stock, 0);
        assert_eq!(status.cards_on_waste, 3);
        assert_eq!(status.top_card_on_waste, Some(Card {
            suit: CardSuit::CLUBS,
            rank: CardRank::THREE,
        }));
    }
}