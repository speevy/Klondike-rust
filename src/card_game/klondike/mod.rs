pub mod deck;
pub mod pile;
pub mod foundation;
pub mod ui;

use rand::seq::SliceRandom;
use rand::thread_rng;
use strum::IntoEnumIterator;
use crate::card_game::american_cards::*;
use crate::card_game::card_containers::*;
use deck::*;
use pile::*;
use foundation::*;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum CardHolder {
    DECK,
    PILE(u32),
    FOUNDATION(u32),
}

pub struct KlondikeMockable<T: CardMover> {
    deck: Box<Deck>,
    piles: Vec<Pile>,
    foundations: Vec<Foundation>,
    mover: T,
}

pub type Klondike = KlondikeMockable<SimpleCardMover>;

impl Klondike {
    pub fn new() -> Self {
        let mover = SimpleCardMover {};
        KlondikeMockable::new_with_mover(mover)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KlondikeStatus {
    pub deck: DeckStatus,
    pub piles: Vec<PileStatus>,
    pub foundations: Vec<FoundationStatus>
}

impl<T: CardMover> KlondikeMockable<T> {

    fn new_with_mover(mover: T) -> Self {
        let cards = KlondikeMockable::<T>::generate_randomized_card_deck();
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

        KlondikeMockable {
            piles,
            foundations,
            deck: Box::new(Deck::new(&cards[card_idx..].to_vec())),
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
                    // Both Origin and Destination are Foundations
                    // First do some sanitary checks
                    let elements = self.foundations.len() as u32;
                    if origin_idx == dest_idx || elements < origin_idx || elements < dest_idx {
                        return false;
                    }

                    // Now whe have to split the vector in order to extract 
                    // the two mutable elements safely
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
                        &mut *self.deck,
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
                    // Both Origin and Destination are Piles
                    // First do some sanitary checks
                    let elements = self.piles.len() as u32;
                    if origin_idx == dest_idx || elements < origin_idx || elements < dest_idx {
                        return false;
                    }

                    // Now whe have to split the vector in order to extract the 
                    // two mutable elements safely
                    let (origin, destination) = extract_two_mutable_elements(
                        &mut self.piles,
                        origin_idx as usize,
                        dest_idx as usize,
                    );
                    return self.mover.move_cards(origin, destination, number as usize);
                }
                CardHolder::DECK => {
                    return self.mover.move_cards(
                        &mut *self.deck,
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

    pub fn take(&mut self) {
        (*(self.deck)).take();
    }

    pub fn get_status(&self) -> KlondikeStatus {
        KlondikeStatus {
            deck: self.deck.get_status(),
            piles: self.piles.iter()
                .map(|x| -> PileStatus {return x.get_status();}).collect(),
            foundations: self.foundations.iter()
                .map(|x| -> FoundationStatus {return x.get_status();}).collect()
        }
    }
}

fn extract_two_mutable_elements<T>(
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

    use crate::card_game::card_containers::test_common::*;

    #[test]
    #[should_panic]
    fn foundation_new_no_cards() {
        Foundation::new(vec![]);
    }




    #[test]
    fn klondike_new() {
        let mut klondike = Klondike::new();
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
        let mut klondike = Klondike::new();
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
        test_extract_two_mutables_case(0, 1);
        test_extract_two_mutables_case(1, 0);
        test_extract_two_mutables_case(2, 3);
        test_extract_two_mutables_case(3, 2);
        test_extract_two_mutables_case(4, 7);
        test_extract_two_mutables_case(7, 4);
        test_extract_two_mutables_case(7, 9);
        test_extract_two_mutables_case(9, 7);
        test_extract_two_mutables_case(8, 9);
        test_extract_two_mutables_case(9, 8);
    }

    #[test]
    #[should_panic]
    fn test_extract_two_mutables_same() {
        test_extract_two_mutables_case(5, 5);
    }

    fn test_extract_two_mutables_case(first: u32, second: u32) {
        let mut vec_test = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];

        let (a, b) = extract_two_mutable_elements(&mut vec_test, first as usize, second as usize);

        assert_eq!(*a, first);
        assert_eq!(*b, second);
    }

    #[test]
    fn klondike_card_movements() {
        let (piles, foundations, deck) = prepare_card_movement_test();
        let origin_str = format!("{:p}", &foundations[0]);
        let destination_str = format!("{:p}", &foundations[1]);
        check_card_movement(
            piles,
            foundations,
            deck,
            origin_str,
            destination_str,
            CardHolder::FOUNDATION(0),
            CardHolder::FOUNDATION(1),
            1,
            true,
        );

        let (piles, foundations, deck) = prepare_card_movement_test();
        let origin_str = format!("{:p}", &foundations[0]);
        let destination_str = format!("{:p}", &piles[1]);
        check_card_movement(
            piles,
            foundations,
            deck,
            origin_str,
            destination_str,
            CardHolder::FOUNDATION(0),
            CardHolder::PILE(1),
            1,
            false,
        );

        let (piles, foundations, deck) = prepare_card_movement_test();
        let origin_str = format!("{:p}", &piles[2]);
        let destination_str = format!("{:p}", &piles[1]);
        check_card_movement(
            piles,
            foundations,
            deck,
            origin_str,
            destination_str,
            CardHolder::PILE(2),
            CardHolder::PILE(1),
            5,
            false,
        );

        let (piles, foundations, deck) = prepare_card_movement_test();
        let origin_str = format!("{:p}", &piles[2]);
        let destination_str = format!("{:p}", &foundations[1]);
        check_card_movement(
            piles,
            foundations,
            deck,
            origin_str,
            destination_str,
            CardHolder::PILE(2),
            CardHolder::FOUNDATION(1),
            5,
            true,
        );

        let (piles, foundations, deck) = prepare_card_movement_test();
        let origin_str = format!("{:p}", &*deck);
        let destination_str = format!("{:p}", &foundations[1]);
        check_card_movement(
            piles,
            foundations,
            deck,
            origin_str,
            destination_str,
            CardHolder::DECK,
            CardHolder::FOUNDATION(1),
            5,
            true,
        );

        let (piles, foundations, deck) = prepare_card_movement_test();
        let origin_str = format!("{:p}", &*deck);
        let destination_str = format!("{:p}", &piles[1]);
        check_card_movement(
            piles,
            foundations,
            deck,
            origin_str,
            destination_str,
            CardHolder::DECK,
            CardHolder::PILE(1),
            5,
            true,
        );
    }

    fn prepare_card_movement_test() -> (Vec<Pile>, Vec<Foundation>, Box<Deck>) {
        (
            vec![Pile::new(), Pile::new(), Pile::new()],
            vec![
                Foundation::new(generate_descending_alt_color_starting(0, 1)),
                Foundation::new(generate_descending_alt_color_starting(1, 1)),
                Foundation::new(generate_descending_alt_color_starting(2, 1)),
            ],
            Box::new(Deck::new(&Vec::new())),
        )
    }

    fn check_card_movement(
        piles: Vec<Pile>,
        foundations: Vec<Foundation>,
        deck: Box<Deck>,
        origin_str: String,
        destination_str: String,
        origin: CardHolder,
        destination: CardHolder,
        number: u32,
        result: bool,
    ) {
        let mut klondike = KlondikeMockable {
            foundations,
            piles,
            deck,
            mover: TestCardMover::new(number as usize, result, origin_str, destination_str),
        };

        let res = klondike.move_cards(origin, destination, number);
        assert_eq!(res, result);
        assert_eq!(klondike.mover.call_count, 1);
    }

    struct TestCardMover {
        origin: String,
        destination: String,
        card_number: usize,
        to_return: bool,
        call_count: u32,
    }

    impl CardMover for TestCardMover {
        fn move_cards(
            &mut self,
            origin: &mut dyn CardOrigin,
            destination: &mut dyn CardDestination,
            number: usize,
        ) -> bool {
            self.call_count = self.call_count + 1;
            assert_eq!(format!("{:p}", origin), self.origin);
            assert_eq!(format!("{:p}", destination), self.destination);
            assert_eq!(self.card_number, number);
            self.to_return
        }
    }

    impl TestCardMover {
        pub fn new(
            card_number: usize,
            to_return: bool,
            origin: String,
            destination: String,
        ) -> TestCardMover {
            TestCardMover {
                origin,
                destination,
                card_number,
                to_return,
                call_count: 0,
            }
        }
    }


    #[test]
    fn check_klondike_status() {
        let klondike = Klondike::new();
        let status = klondike.get_status();
        assert_eq!(status.deck, klondike.deck.get_status());

        for i in 0..klondike.foundations.len() {
            assert_eq!(
                status.foundations[i], 
                klondike.foundations[i].get_status()
            );
        }

        for i in 0..klondike.piles.len() {
            assert_eq!(
                status.piles[i], 
                klondike.piles[i].get_status()
            );
        }
    }
}