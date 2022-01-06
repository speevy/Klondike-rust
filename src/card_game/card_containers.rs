use super::american_cards::*;
use mockall::automock;

/// Anything where cards can be taken of
#[automock]
pub trait CardOrigin  {
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

    fn undo_peek(&mut self, cards: &Vec<Card>);
}

/// Anything where cards can be moved to
#[automock]
pub trait CardDestination {
    /// Try to poke an arbitrary number of cards. It should check the
    /// business logic for allowing this poke of cards. If everything
    /// is OK a true is returned.
    fn try_poke(&self, cards: &Vec<Card>) -> bool;

    /// Poke an arbitrary number of cards. It should check the
    /// business logic for allowing this poke of cards. If everything
    /// is OK a the cards should be added to the Card Destination.
    fn poke(&mut self, cards: &Vec<Card>);

    fn undo_poke(&mut self, number: usize) -> Vec<Card>;
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

    fn undo_move_cards(
        &mut self,
        origin: &mut dyn CardOrigin,
        destination: &mut dyn CardDestination,
        number: usize,
    ) {
        origin.undo_peek(&destination.undo_poke(number));
    }
}

#[derive(Clone, Debug, PartialEq)]
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
    use super::test_common::*;
    use super::*;
    use mockall::*;


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
        
        let cards = generate_random_card_set(number);
        let try_peek_result = match ret_peek {
            true => Some(cards.to_vec()),
            false => None,
        };

        let peek_result = match ret_peek {
            true => cards.to_vec(),
            false => Vec::new(),
        };

        let mut origin = MockCardOrigin::new();
        origin
            .expect_try_peek()
            .with(predicate::eq(number))
            .return_once(move |_x| try_peek_result);
        
        origin
        .expect_peek()
        .with(predicate::eq(number))
        .times(num_calls)
        .return_once(move |_x| peek_result);

        let mut destination = MockCardDestination::new();
        destination
            .expect_try_poke()
            .with(predicate::eq(cards.to_vec()))
            .return_once(move |_x| ret_poke);       

        destination
            .expect_poke()
            .with(predicate::eq(cards.to_vec()))
            .times(num_calls)
            .return_once(|_x| ());    

        let result = mover.move_cards(&mut origin, &mut destination, number);

        assert_eq!(result, expected);
    }

    #[test]
    fn undo_card_mover() {
        for i in 1..10 {
            undo_card_mover_case(i);
        }
    }

    fn undo_card_mover_case(number: usize) {
        let mut mover = SimpleCardMover {};
        let cards = generate_random_card_set(number);

        let mut origin = MockCardOrigin::new();
        origin
            .expect_undo_peek()
            .with(predicate::eq(cards.to_vec()))
            .times(1)
            .returning(|_x| ());

        let mut destination = MockCardDestination::new();
        destination
            .expect_undo_poke()
            .with(predicate::eq(number))
            .times(1)
            .return_once(|_x| cards);

        mover.undo_move_cards(&mut origin, &mut destination, number);
    }
}
