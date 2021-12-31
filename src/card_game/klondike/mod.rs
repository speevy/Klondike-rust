pub mod deck;
pub mod pile;
pub mod foundation;
pub mod ui;
pub mod storage;

use rand::seq::SliceRandom;
use rand::thread_rng;
use strum::IntoEnumIterator;
use crate::card_game::american_cards::*;
use crate::card_game::card_containers::*;
use deck::*;
use pile::*;
use foundation::*;
use serde::{Serialize, Deserialize};

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum CardHolder {
    DECK,
    PILE(u32),
    FOUNDATION(u32),
}

#[derive(Clone)]
enum KlondikeAction {
    MOVE(CardHolder, CardHolder, u32),
    TAKE
}

#[derive(Clone)]
pub struct KlondikeMockable<T: CardMover> {
    deck: Box<Deck>,
    piles: Vec<Pile>,
    foundations: Vec<Foundation>,
    mover: T,
    history: Vec<KlondikeAction>,
}

pub type Klondike = KlondikeMockable<SimpleCardMover>;

impl Klondike {
    pub fn new() -> Self {
        let mover = SimpleCardMover {};
        KlondikeMockable::new_with_mover(mover)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct KlondikeStatus {
    pub deck: DeckStatus,
    pub piles: Vec<PileStatus>,
    pub foundations: Vec<FoundationStatus>
}

macro_rules! exec_move_cards {
    ($obj: expr, $origin: expr, $destination: expr, $number: expr, $is_undo: expr) => {
        if $is_undo {
            $obj.mover.undo_move_cards($origin, $destination, $number as usize);
            true
        } else {
           $obj.mover.move_cards($origin, $destination, $number as usize)
        }        
    };
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
            history: Vec::new()
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
        if self.do_move_cards(origin, destination, number, false) {
            self.history.push(KlondikeAction::MOVE(origin, destination, number));
            return true;
        }
        false
    }

    fn do_move_cards(&mut self, origin: CardHolder, destination: CardHolder, number: u32, is_undo: bool) -> bool {
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
                    return exec_move_cards!(self, origin, destination, number, is_undo);
                }
                CardHolder::PILE(origin_idx) => {
                    return exec_move_cards!(self,
                        &mut self.piles[origin_idx as usize],
                        &mut self.foundations[dest_idx as usize],
                        number, is_undo
                    );
                }
                CardHolder::DECK => {
                    return exec_move_cards!(self,
                        &mut *self.deck,
                        &mut self.foundations[dest_idx as usize],
                        number, is_undo
                    );
                }
            },
            CardHolder::PILE(dest_idx) => match origin {
                CardHolder::FOUNDATION(origin_idx) => {
                    return exec_move_cards!(self,
                        &mut self.foundations[origin_idx as usize],
                        &mut self.piles[dest_idx as usize],
                        number, is_undo
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
                    return exec_move_cards!(self,origin, destination, number, is_undo);
                }
                CardHolder::DECK => {
                    return exec_move_cards!(self,
                        &mut *self.deck,
                        &mut self.piles[dest_idx as usize],
                        number, is_undo
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
        self.history.push(KlondikeAction::TAKE);
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

    /// Move the top card of the given origin to the corresponding pile 
    /// (the first empty one in case is an Ace). return true if success
    pub fn to_pile(&mut self, origin: CardHolder) -> bool {
        for i in 0..self.piles.len() {
            if self.move_cards(origin, CardHolder::PILE(i as u32), 1) {
                return true;
            }
        }
        false
    }

    pub fn undo(&mut self) {
        if let Some(action) = self.history.pop() {
            match action {
                KlondikeAction::MOVE(origin, destination, number) => {
                    self.do_move_cards(origin, destination, number, true);
                },
                KlondikeAction::TAKE => {
                    self.deck.undo_take();
                }
            }
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
            history: Vec::new(),
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
    fn send_card_to_pile() {
        let (piles, foundations, deck) = prepare_card_movement_test();
        let origin_str = format!("{:p}", &*deck);
        let destination_str = format!("{:p}", &piles[1]);
        check_card_to_pile(
            piles,
            foundations,
            deck,
            origin_str,
            destination_str,
            CardHolder::DECK,
            true
        );
 
        let (piles, foundations, deck) = prepare_card_movement_test();
        let origin_str = format!("{:p}", &*deck);
        let destination_str = format!("{:p}", &piles[2]);
        check_card_to_pile(
            piles,
            foundations,
            deck,
            origin_str,
            destination_str,
            CardHolder::DECK,
            false
        );

        let (piles, foundations, deck) = prepare_card_movement_test();
        let origin_str = format!("{:p}", &foundations[0]);
        let destination_str = format!("{:p}", &piles[0]);
        check_card_to_pile(
            piles,
            foundations,
            deck,
            origin_str,
            destination_str,
            CardHolder::FOUNDATION(0),
            true
        );

        let (piles, foundations, deck) = prepare_card_movement_test();
        let origin_str = format!("{:p}", &foundations[2]);
        let destination_str = format!("{:p}", &piles[1]);
        check_card_to_pile(
            piles,
            foundations,
            deck,
            origin_str,
            destination_str,
            CardHolder::FOUNDATION(2),
            true
        );
    }

    fn check_card_to_pile(
        piles: Vec<Pile>,
        foundations: Vec<Foundation>,
        deck: Box<Deck>,
        origin_str: String,
        destination_str: String,
        origin: CardHolder,
        result: bool,
    ) {
        let mut klondike = KlondikeMockable {
            foundations,
            piles,
            deck,
            mover: TestPileCardMover::new(origin_str, destination_str, result),
            history: Vec::new()
        };

        let res = klondike.to_pile(origin);
        assert_eq!(res, result);
        assert_eq!(klondike.mover.success_count, 1);
    }

    struct TestPileCardMover {
        origin: String,
        destination: String,
        success_count: u32,
        result: bool
    }

    impl CardMover for TestPileCardMover {
        fn move_cards(
            &mut self,
            origin: &mut dyn CardOrigin,
            destination: &mut dyn CardDestination,
            number: usize,
        ) -> bool {
            assert_eq!(format!("{:p}", origin), self.origin);
            assert_eq!(1, number);
            if format!("{:p}", destination) != self.destination {
                return false;
            }
            self.success_count = self.success_count + 1;
            return self.result;
        }
    }

    impl TestPileCardMover {
        pub fn new(
            origin: String,
            destination: String,
            result: bool
        ) -> TestPileCardMover {
            TestPileCardMover {
                origin,
                destination,
                result,
                success_count: 0,
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

    use rand::distributions::{Distribution, Uniform};
    use mockall::*;
    use mockall::predicate::*;

    #[automock]
    trait CardMoverWrapper {
        fn move_cards(
            &mut self,
            origin: String,
            destination: String,
            number: usize,
        ) -> bool ;
    
        fn undo_move_cards(
            &mut self,
            origin: String,
            destination: String,
            number: usize,
        );
    }

    struct MockableCardMover <T: CardMoverWrapper> {
        wrapper: T 
    }

    impl<T: CardMoverWrapper> CardMover for MockableCardMover<T> {
        fn move_cards(
            &mut self,
            origin: &mut dyn CardOrigin,
            destination: &mut dyn CardDestination,
            number: usize,
        ) -> bool {
            let origin_str = format!("{:p}", origin);
            let destination_str = format!("{:p}", destination);
            print!("Move cards {} -> {} {}", origin_str, destination_str, number);
            let res = self.wrapper.move_cards(origin_str, destination_str, number);
            println!(" returned {}", res);
            res
        }
    
        fn undo_move_cards(
            &mut self,
            origin: &mut dyn CardOrigin,
            destination: &mut dyn CardDestination,
            number: usize,
        ) {
            let origin_str = format!("{:p}", origin);
            let destination_str = format!("{:p}", destination);
            println!("Undo Move cards {} -> {} {}", origin_str, destination_str, number);
            self.wrapper.undo_move_cards(origin_str, destination_str, number);
        }
    
    }

    /// Generates a random sequence of movements, and undoes them.
    /// If the number of moved cards is 5 or grater, the mover mock 
    /// returns false, simulating an invalid movement, so those movements
    /// should not be undone.
    #[test]
    fn klondike_undo_move() {
        let num_movements = 20;
        let mut mover_wrapper = MockCardMoverWrapper::new();
        mover_wrapper.expect_move_cards().returning(|_x, _y, number| number < 5);
        let (piles, foundations, deck) = prepare_card_movement_test();

        let mut movements: Vec<(CardHolder, CardHolder, usize)> = Vec::new();

        let mut rng_origin = rand::thread_rng();
        let dist_origin = Uniform::from(0..7);

        let mut rng_destination = rand::thread_rng();
        let dist_destination = Uniform::from(0..6);

        let mut rng_number = rand::thread_rng();
        let dist_number = Uniform::from(1..7);
        
        let mut seq = Sequence::new();

        println!("Deck: {:p}", &*deck);

        for _i in 1..num_movements {
            let origin_idx = dist_origin.sample(&mut rng_origin);
            let mut destination_idx = dist_destination.sample(&mut rng_destination);
            while destination_idx == origin_idx {
                destination_idx = dist_destination.sample(&mut rng_destination);
            }

            let (origin_str, origin_ch) = get_cardholder_ptr(&piles, &foundations, &deck, origin_idx);
            let (destination_str, destination_ch) = get_cardholder_ptr(&piles, &foundations, &deck, destination_idx);

            let number = dist_number.sample(&mut rng_number) as usize;

            println!("Prepare Move cards {} -> {} {}", origin_str, destination_str, number);

            movements.push((
                origin_ch, 
                destination_ch, 
                number
            ));

            if number < 5 {
                mover_wrapper.expect_undo_move_cards()
                    .with(eq(origin_str), eq(destination_str), eq(number as usize))
                    .times(1)
                    .in_sequence(&mut seq)
                    .returning(|_x, _y, _z| ());
            }
        }

        let mover = MockableCardMover {wrapper: mover_wrapper};
        let mut klondike = KlondikeMockable {
            deck,
            piles,
            foundations,
            mover,
            history: Vec::new(),
        };

        movements.reverse();

        for (origin, destination, number) in movements {
            klondike.move_cards(origin, destination, number as u32);
        }

        for _i in 1..20 {
            klondike.undo();
        }
    }

    fn get_cardholder_ptr (piles: &Vec<Pile>, foundations: &Vec<Foundation>, deck: &Box<Deck>, number: u32) -> (String, CardHolder) {
        match number {
            0..=2 => (format!("{:p}", &piles[number as usize]), CardHolder::PILE(number)),
            3..=5 => (format!("{:p}", &foundations[(number-3) as usize]), CardHolder::FOUNDATION(number - 3)),
            _ => (format!("{:p}", &**deck), CardHolder::DECK)
        }
    }

    #[test]
    fn klondike_undo_take() {
        let mut klondike = Klondike::new();
        let mut status_history: Vec<KlondikeStatus> = Vec::new();

        let status = klondike.get_status();
        log_status(&status);
        status_history.push(status);

        for i in 0..klondike.foundations.len() {
            if klondike.to_pile(CardHolder::FOUNDATION(i as u32)) {
                let status = klondike.get_status();
                log_status(&status);
                status_history.push(status);
            }
        }

        for _i in 0..30 {
            if klondike.to_pile(CardHolder::DECK) {
                let status = klondike.get_status();
                log_status(&status);
                status_history.push(status);    
            }
            
            klondike.take();
            let status = klondike.get_status();
            log_status(&status);
            status_history.push(status);
        }

        println!("------- Undoing ---------");

        status_history.pop();

        while let Some(expected_status) = status_history.pop() {
            klondike.undo();
            let status = klondike.get_status();
            log_status(&status);
            assert_eq!(status, expected_status);
        }
    }

    fn log_status(status: &KlondikeStatus) {
        print!("Deck: (waste: {} stock: {} ) Piles:", status.deck.cards_on_waste, status.deck.cards_on_stock);
        for i in &status.piles {
            print!(" {}", i.num_cards);
        }
        print!(" Found:");
        for i in &status.foundations {
            print!(" (hid: {}, vis: {})", i.num_hidden, i.visible.len());
        }
        println!("");
    }
}