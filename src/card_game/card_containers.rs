use super::american_cards::*;

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

pub mod test_common {
    use super::*;
    use rand::seq::SliceRandom;
    use rand::thread_rng;
    use strum::IntoEnumIterator;

    pub fn assert_peek_one_returns(origin: &mut dyn CardOrigin, card: Card) {
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

    pub fn generate_random_card_set(size: usize) -> Vec<Card> {
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
    
    pub fn generate_descending_alt_color_starting(start: usize, size: usize) -> Vec<Card> {
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

}

#[cfg(test)]
mod test {
    use super::*;
    use super::test_common::*;
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