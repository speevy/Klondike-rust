use rand::seq::SliceRandom;
use rand::thread_rng;
use std::fmt;
use strum::IntoEnumIterator;
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
    fn try_peek(&self, number: usize) -> Option<Vec<Card>>;

    /// Peek an arbitrary number of cards. It should check the
    /// business logic for allowing this peek of cards. If everything
    /// is OK a vector containing the requested cards is returned, An empty
    /// one otherwise. The returned cards should be removed from the Card
    /// Origin.
    fn peek(&mut self, number: usize) -> Vec<Card>;
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

    fn can_peek(&self, number: usize) -> bool {
        number > 0 && number <= self.visible.len()
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

        ((cards[0].rank as i32) + 1) == (last_card.rank as i32)
            && match cards[0].suit {
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

pub trait CardMover {
    fn move_cards(
        &mut self,
        origin: &mut dyn CardOrigin,
        destination: &mut dyn CardDestination,
        number: usize,
    ) -> bool {
        if let Some(try_cards) = origin.try_peek(number) {
            if destination.try_poke(&try_cards) {
                destination.poke(&origin.peek(number));
                return true;
            }
        }

        return false;
    }
}

pub struct SimpleCardMover;
impl CardMover for SimpleCardMover {}

pub enum CardHolder {
    DECK,
    PILE(u32),
    FOUNDATION(u32),
}

pub struct Klondike<'a> {
    deck: Deck,
    piles: Vec<Pile>,
    foundations: Vec<Foundation>,
    mover: &'a mut (dyn CardMover + 'a),
}

impl<'a> Klondike<'a> {
    pub fn new(mover: &'a mut dyn CardMover) -> Klondike<'a> {
        let cards = Klondike::generate_randomized_card_deck();
        let mut card_idx = 0;

        let mut piles: Vec<Pile> = Vec::new();
        for _i in 0..4 {
            piles.push(Pile::new());
        }

        let mut foundations: Vec<Foundation> = Vec::new();

        for i in 1..8 {
            foundations.push(Foundation::new(cards[card_idx..card_idx + i].to_vec()));
            card_idx += i;
        }

        Klondike {
            piles,
            foundations,
            deck: Deck::init(&cards[card_idx..].to_vec()),
            mover,
        }
    }

    fn generate_randomized_card_deck() -> Vec<Card> {
        let mut cards: Vec<Card> = Vec::new();
        for suit in CardSuit::iter() {
            for rank in CardRank::iter() {
                cards.push(Card {
                    rank: rank,
                    suit: suit,
                });
            }
        }
        let mut rng = thread_rng();
        cards.shuffle(&mut rng);
        return cards;
    }

    pub fn move_cards(&mut self, origin: CardHolder, destination: CardHolder, number: u32) -> bool {
        match destination {
            CardHolder::FOUNDATION(dest_idx) => match origin {
                CardHolder::FOUNDATION(origin_idx) => {
                    let elements = self.foundations.len() as u32;
                    if origin_idx == dest_idx || elements < origin_idx || elements < dest_idx {
                        return false;
                    }
                    let (origin, destination) = extract_two_mutable_elements(
                        &mut self.foundations,
                        origin_idx as usize,
                        dest_idx as usize,
                    );

                    return self.mover.move_cards(origin, destination, number as usize);
                }
                CardHolder::PILE(origin_idx) => {
                    return self.mover.move_cards(
                        &mut self.piles[origin_idx as usize],
                        &mut self.foundations[dest_idx as usize],
                        number as usize,
                    );
                }
                CardHolder::DECK => {
                    return self.mover.move_cards(
                        &mut self.deck,
                        &mut self.foundations[dest_idx as usize],
                        number as usize,
                    );
                }
            },
            CardHolder::PILE(dest_idx) => match origin {
                CardHolder::FOUNDATION(origin_idx) => {
                    return self.mover.move_cards(
                        &mut self.foundations[origin_idx as usize],
                        &mut self.piles[dest_idx as usize],
                        number as usize,
                    );
                }
                CardHolder::PILE(origin_idx) => {
                    let elements = self.piles.len() as u32;
                    if origin_idx == dest_idx || elements < origin_idx || elements < dest_idx {
                        return false;
                    }
                    let (origin, destination) = extract_two_mutable_elements(
                        &mut self.piles,
                        origin_idx as usize,
                        dest_idx as usize,
                    );
                    return self.mover.move_cards(origin, destination, number as usize);
                }
                CardHolder::DECK => {
                    return self.mover.move_cards(
                        &mut self.deck,
                        &mut self.piles[dest_idx as usize],
                        number as usize,
                    );
                }
            },
            CardHolder::DECK => {
                return false;
            }
        }
    }
}

pub fn extract_two_mutable_elements<T>(
    vector: &mut Vec<T>,
    first_idx: usize,
    second_idx: usize,
) -> (&mut T, &mut T) {
    if first_idx < second_idx {
        let (first_vector, second_vector) = vector.split_at_mut(first_idx + 1);
        return (
            &mut first_vector[first_idx],
            &mut second_vector[second_idx - first_idx - 1],
        );
    }

    if first_idx > second_idx {
        let (first_vector, second_vector) = vector.split_at_mut(second_idx + 1);
        return (
            &mut second_vector[first_idx - second_idx - 1],
            &mut first_vector[second_idx],
        );
    }

    panic!("indexes cannot be equal");
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
    fn klondike_new() {
        let mut mover = SimpleCardMover {};
        let mut klondike = Klondike::new(&mut mover);
        // Four empty piles
        assert_eq!(klondike.piles.len(), 4);
        for pile in klondike.piles {
            assert_eq!(pile.try_peek(1), None);
        }

        // Seven foundations: the first one with one card, the second one with two ...
        assert_eq!(klondike.foundations.len(), 7);
        for i in 0..7 {
            assert_eq!(
                get_card_origin_number_of_cards(&mut klondike.foundations[i]),
                i as u32 + 1
            );
        }

        // The remaining cards are on the deck
        assert_eq!(get_deck_number_of_cards(&mut klondike.deck), 24); // 52 - 1 - 2 - 3 - 4 - 5 - 6 - 7

        //TODO: check cards not repeated and randomized (if possible)
    }
    fn get_card_origin_number_of_cards(origin: &mut dyn CardOrigin) -> u32 {
        let mut count = 0;
        while origin.peek(1).len() == 1 {
            count = count + 1;
        }
        return count;
    }

    fn get_deck_number_of_cards(deck: &mut Deck) -> u32 {
        let mut count = 0;
        while deck.peek(1).len() == 1 {
            count = count + 1;
            deck.take();
        }
        return count;
    }

    #[test]
    fn klondike_impossible_card_movements() {
        let mut mover = SimpleCardMover {};
        let mut klondike = Klondike::new(&mut mover);
        // Can't move cards to the deck
        assert_eq!(
            klondike.move_cards(CardHolder::DECK, CardHolder::DECK, 1),
            false
        );
        assert_eq!(
            klondike.move_cards(CardHolder::FOUNDATION(1), CardHolder::DECK, 1),
            false
        );
        assert_eq!(
            klondike.move_cards(CardHolder::PILE(0), CardHolder::DECK, 1),
            false
        );

        // Can't move more than one card to the pile
        assert_eq!(
            klondike.move_cards(CardHolder::FOUNDATION(1), CardHolder::PILE(0), 2),
            false
        );

        // Can't move more than one card from the pile
        assert_eq!(
            klondike.move_cards(CardHolder::FOUNDATION(1), CardHolder::PILE(0), 2),
            false
        );
    }

    #[test]
    fn test_extract_two_mutables() {
        test_extract_two_mutables_case(0,1);
        test_extract_two_mutables_case(1,0);
        test_extract_two_mutables_case(2,3);
        test_extract_two_mutables_case(3,2);
        test_extract_two_mutables_case(4,7);
        test_extract_two_mutables_case(7,4);
        test_extract_two_mutables_case(7,9);
        test_extract_two_mutables_case(9,7);
        test_extract_two_mutables_case(8,9);
        test_extract_two_mutables_case(9,8);
    }

    #[test]
    #[should_panic]
    fn test_extract_two_mutables_same() {
        test_extract_two_mutables_case(5,5);
    }

    fn test_extract_two_mutables_case(first:u32, second: u32) {
        let mut vec_test = vec![0,1,2,3,4,5,6,7,8,9];

        let (a, b) = extract_two_mutable_elements(&mut vec_test, first as usize, second as usize);

        assert_eq!(*a, first);
        assert_eq!(*b, second);
    }

    #[test]
    fn klondike_card_movements() {
        let mut piles = vec![Pile::new(), Pile::new(), Pile::new()];
        let mut mover = TestCardMover::new(1, true);

        unsafe {
            let ptr = &mut piles as *mut Vec<Pile>;
            let piles2 = &*ptr;
            mover.origin = Some(&piles2[0]);
            mover.destination = Some(&piles2[1]);
        }

        let mut klondike = Klondike {
            foundations: Vec::new(),
            piles,
            deck:Deck::init(&Vec::new()),
            mover :&mut mover
        };

        let res = klondike.move_cards(CardHolder::PILE(0), CardHolder::PILE(1), 1);
        assert_eq!(res, true);
        assert_eq!(mover.call_count, 1);
    }

    struct TestCardMover<'a, 'b> {
        origin: Option<&'a (dyn CardOrigin + 'a)>,
        destination: Option<&'b (dyn CardDestination + 'b)>,
        card_number: usize,
        to_return: bool,
        call_count: u32,
    }

    impl<'a, 'b> CardMover for TestCardMover<'a, 'b> {
        fn move_cards(
            &mut self,
            origin: &mut dyn CardOrigin,
            destination: &mut dyn CardDestination,
            number: usize,
        ) -> bool {
            self.call_count = self.call_count + 1;
            if let Some(expected) = self.origin {
                // convert pointer to text, as direct compare does not work :(
                assert_eq!(format!("{:p}", origin), format!("{:p}", expected));
            }
            if let Some(expected) = self.destination {
                assert_eq!(format!("{:p}", destination), format!("{:p}", expected));
            }
            assert_eq!(self.card_number, number);
            self.to_return
        }
    }

    impl<'a, 'b> TestCardMover<'a, 'b> {
        pub fn new(card_number: usize, to_return: bool) -> TestCardMover<'a, 'b> {
            TestCardMover {
                origin: None,
                destination: None,
                card_number,
                to_return,
                call_count: 0,
            }
        }
    }

    struct TestCardOrigin {
        to_return: Option<Vec<Card>>,
        peek_parameters: Vec<usize>,
    }

    impl CardOrigin for TestCardOrigin {
        fn try_peek(&self, number: usize) -> Option<Vec<Card>> {
            if let Some(cards) = &self.to_return {
                if number <= cards.len() {
                    return Some(cards[..number].to_vec());
                }
            }
            None
        }

        fn peek(&mut self, number: usize) -> Vec<Card> {
            self.peek_parameters.push(number);
            self.try_peek(number).unwrap_or(Vec::new())
        }
    }

    struct TestCardDestination {
        to_return: bool,
        poke_parameters: Vec<Vec<Card>>,
    }

    impl CardDestination for TestCardDestination {
        fn try_poke(&self, _cards: &Vec<Card>) -> bool {
            self.to_return
        }

        fn poke(&mut self, cards: &Vec<Card>) {
            self.poke_parameters.push(cards.to_vec());
        }
    }

    #[test]
    fn card_mover() {
        for i in 1..5 {
            card_mover_check(false, false, i, false);
            card_mover_check(false, true, i, false);
            card_mover_check(true, false, i, false);
            card_mover_check(true, true, i, true);
        }
    }

    fn card_mover_check(ret_peek: bool, ret_poke: bool, number: usize, expected: bool) {
        let mut mover = SimpleCardMover {};
        let num_calls = match expected {
            true => 1,
            false => 0,
        };

        let mut origin = TestCardOrigin {
            to_return: match ret_peek {
                true => Some(generate_random_card_set(number)),
                false => None,
            },
            peek_parameters: Vec::new(),
        };
        let mut destination = TestCardDestination {
            to_return: ret_poke,
            poke_parameters: Vec::new(),
        };

        let result = mover.move_cards(&mut origin, &mut destination, number);

        assert_eq!(result, expected);
        assert_eq!(origin.peek_parameters.len(), num_calls);
        assert_eq!(destination.poke_parameters.len(), num_calls);

        if expected {
            assert_eq!(origin.peek_parameters[0], number);
            if let Some(cards) = origin.to_return {
                assert_eq!(destination.poke_parameters[0], cards);
            }
        }
    }
}
