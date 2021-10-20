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
        write!(
            f,
            "{}{}",
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
        )
    }
}

//---------------

/// Anything where cards can be taken of
pub trait CardOrigin {
    /// Try to peek an arbitrary number of cards. It should check the
    /// business logic for allowing this peek of cards. If everything
    /// is OK a vector containing the requested cards is returned, None
    /// otherwise.  
    fn try_peek(&self, number: u32) -> Option<Vec<Card>>;

    /// Peek an arbitrary number of cards. It should check the
    /// business logic for allowing this peek of cards. If everything
    /// is OK a vector containing the requested cards is returned, An empty
    /// one otherwise. The returned cards should be removed from the Card
    /// Origin.
    fn peek(&mut self, number: u32) -> Vec<Card>;
}

/// Anything where cards can be moved to
pub trait CardDestination {
    /// Try to poke an arbitrary number of cards. It should check the
    /// business logic for allowing this poke of cards. If everything
    /// is OK a true is returned.
    fn try_poke(&self, cards: &Vec<Card>) -> bool;

    /// Poke an arbitrary number of cards. It should check the
    /// business logic for allowing this poke of cards. If everything
    /// is OK a the cards should be added to the Card Destination.
    fn poke(&mut self, cards: &Vec<Card>);
}

/// The deck of the game, consisting in two piles: the stock and the waste.
/// The waste also acts as a CardOrigin.
pub struct Deck {
    stock: Vec<Card>,
    waste: Vec<Card>,
}

impl CardOrigin for Deck {
    fn peek(&mut self, number: u32) -> Vec<Card> {
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

    fn try_peek(&self, number: u32) -> Option<Vec<Card>> {
        if number == 1 && self.waste.len() > 0 {
            return Some(self.waste[self.waste.len() - 1..].to_vec());
        }
        return None;
    }
}

impl Deck {
    ///Creates a deck containing the given cards. One of the cards goes to
    ///the waste, the others to the pile.
    pub fn init(cards: &Vec<Card>) -> Deck {
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
}

pub struct Pile {
    cards: Vec<Card>,
}

impl Pile {
    pub fn new() -> Pile {
        Pile { cards: vec![] }
    }
}

impl CardOrigin for Pile {
    fn try_peek(&self, number: u32) -> Option<Vec<Card>> {
        if number == 1 && !self.cards.is_empty() {
            return Some(self.cards[self.cards.len() - 1..].to_vec());
        }
        return None;
    }

    fn peek(&mut self, number: u32) -> Vec<Card> {
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
}

pub struct Foundation {
    hidden: Vec<Card>,
    visible: Vec<Card>,
}

impl Foundation {
    pub fn new(cards: Vec<Card>) -> Foundation {
        Foundation {
            hidden: cards[..cards.len() - 1].to_vec(),
            visible: cards[cards.len() - 1..].to_vec(),
        }
    }

    fn can_peek(&self, number: u32) -> bool {
        number > 0 && (number as usize) <= self.visible.len()
    }
}

impl CardOrigin for Foundation {
    fn try_peek(&self, number: u32) -> Option<Vec<Card>> {
        if self.can_peek(number) {
            return Some(self.visible[self.visible.len() - (number as usize)..].to_vec());
        }
        return None;
    }

    fn peek(&mut self, number: u32) -> Vec<Card> {
        if self.can_peek(number) {
            let res: Vec<Card> = self
                .visible
                .drain(self.visible.len() - (number as usize)..)
                .collect();
            if self.visible.is_empty() {
                match self.hidden.pop() {
                    Some(card) => {
                        self.visible.push(card);
                    }
                    None => {}
                }
            }

            return res;
        }
        return Vec::new();
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

        let last_card = self.visible[self.visible.len() - 1];

        ((cards[0].rank as i32) + 1) == (last_card.rank as i32) &&
        match cards[0].suit {
            CardSuit::DIAMONDS | CardSuit::HEARTS => { 
                last_card.suit == CardSuit::CLUBS || last_card.suit == CardSuit::SPADES
            }
            CardSuit::CLUBS | CardSuit::SPADES => { 
                last_card.suit == CardSuit::DIAMONDS || last_card.suit == CardSuit::HEARTS
            }
        }
    }

    fn poke(&mut self, cards: &Vec<Card>) {
        if self.try_poke(cards) {
            self.visible.append(&mut cards.to_vec());
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use rand::seq::SliceRandom;
    use rand::thread_rng;
    use strum::IntoEnumIterator;

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
    fn assert_peek_one_returns(origin: &mut dyn CardOrigin, card: Card) {
        match origin.try_peek(1) {
            None => {
                panic!("No card returned for peek(1) having cards in deck");
            }
            Some(result) => {
                assert_eq!(result.len(), 1);
                assert_eq!(result[0], card);
            }
        }

        let result = origin.peek(1);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], card);
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

        let deck = Deck::init(&cards);

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
    #[should_panic]
    fn foundation_new_no_cards() {
        Foundation::new(vec![]);
    }

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
            found.try_peek(peek as u32),
            Some(generate_descending_alt_color_starting(visible - peek, peek))
        );
        assert_eq!(
            found.peek(peek as u32),
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
        }
    }

    fn generate_random_card_set(size: usize) -> Vec<Card> {
        let mut cards: Vec<Card> = Vec::new();
        for suit in CardSuit::iter() {
            for rank in vec![
                CardRank::ACE,
                CardRank::TWO,
                CardRank::THREE,
                CardRank::FOUR,
            ] {
                cards.push(Card { rank, suit });
            }
        }

        let mut rng = thread_rng();
        cards.shuffle(&mut rng);
        return cards[..size].to_vec();
    }

    fn generate_descending_alt_color_starting(start: usize, size: usize) -> Vec<Card> {
        vec![
            Card {
                suit: CardSuit::HEARTS,
                rank: CardRank::KING,
            },
            Card {
                suit: CardSuit::SPADES,
                rank: CardRank::QUEEN,
            },
            Card {
                suit: CardSuit::DIAMONDS,
                rank: CardRank::JACK,
            },
            Card {
                suit: CardSuit::CLUBS,
                rank: CardRank::TEN,
            },
            Card {
                suit: CardSuit::DIAMONDS,
                rank: CardRank::NINE,
            },
            Card {
                suit: CardSuit::CLUBS,
                rank: CardRank::EIGHT,
            },
            Card {
                suit: CardSuit::HEARTS,
                rank: CardRank::SEVEN,
            },
            Card {
                suit: CardSuit::SPADES,
                rank: CardRank::SIX,
            },
            Card {
                suit: CardSuit::HEARTS,
                rank: CardRank::FIVE,
            },
        ][start..start + size]
            .to_vec()
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
            4,
            vec![Card {
                suit: CardSuit::DIAMONDS,
                rank: CardRank::SIX,
            }],
        );
        foundation_poke_case_ko(
            0,
            4,
            vec![Card {
                suit: CardSuit::HEARTS,
                rank: CardRank::SIX,
            }],
        );
        foundation_poke_case_ko(
            0,
            4,
            vec![Card {
                suit: CardSuit::CLUBS,
                rank: CardRank::NINE,
            }],
        );
        foundation_poke_case_ko(
            0,
            4,
            vec![Card {
                suit: CardSuit::SPADES,
                rank: CardRank::NINE,
            }],
        );
        foundation_poke_case_ko(
            0,
            5,
            vec![Card {
                suit: CardSuit::CLUBS,
                rank: CardRank::SIX,
            }],
        );
        foundation_poke_case_ko(
            0,
            4,
            vec![Card {
                suit: CardSuit::SPADES,
                rank: CardRank::SIX,
            }],
        );
        foundation_poke_case_ko(
            0,
            4,
            vec![Card {
                suit: CardSuit::HEARTS,
                rank: CardRank::EIGHT,
            }],
        );
        foundation_poke_case_ko(
            0,
            4,
            vec![Card {
                suit: CardSuit::DIAMONDS,
                rank: CardRank::EIGHT,
            }],
        );
        foundation_poke_case_ko(
            0,
            4,
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
}
