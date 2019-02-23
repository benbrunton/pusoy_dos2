use crate::cards::{PlayedCard, Rank};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

macro_rules! build_fct {
    ($trick:ident, $cards:ident) => (Some(Hand::FiveCardTrick(
        Trick{
            trick_type: TrickType::$trick,
            cards:[
                $cards[0],
                $cards[1],
                $cards[2],
                $cards[3],
                $cards[4],
            ]
        }
    )))
}

#[derive(
    Clone,
    Debug,
    PartialEq,
    Copy,
    Serialize,
    Deserialize,
)]
/// Type of hand that can be played
pub enum Hand{
    /// No cards
    Pass,
    /// One card
    Single(PlayedCard),
    /// A pair of matching cards
    Pair(PlayedCard, PlayedCard),
    /// 3 of a kind
    Prial(PlayedCard, PlayedCard, PlayedCard),
    /// 5 card trick
    FiveCardTrick(Trick)
}

impl Hand {
    pub fn build(cards: Vec<PlayedCard>) -> Option<Hand> {
        match cards.len() {
                0 => Some(Hand::Pass),
                1 => Some(Hand::Single(cards[0])),
                2 => Self::check_valid_pair(cards),
                3 => Self::check_valid_prial(cards),
                5 => Self::check_valid_fct(cards),
                _ => None
        }
    }

    fn check_valid_pair(cards: Vec<PlayedCard>) -> Option<Hand> {
        if Self::get_counts(cards.clone()).len() == 1 {
            Some(Hand::Pair(cards[0], cards[1]))
        } else {
            None
        }
    }

    fn check_valid_prial(cards: Vec<PlayedCard>) -> Option<Hand> {
        if Self::get_counts(cards.clone()).len() == 1 {
            Some(Hand::Prial(cards[0], cards[1], cards[2]))
        } else {
            None
        }
    }

    fn check_valid_fct(c: Vec<PlayedCard>) -> Option<Hand> {
        let cards = Self::sort_cards(c);
        let rank_count = Self::get_counts(cards.clone());
        match rank_count.len() {
            1 => build_fct!(FiveOfAKind, cards),
            2 => {
                match *rank_count.values().last().unwrap() {
                    3 | 2   => build_fct!(FullHouse, cards),
                    4 | 1   => build_fct!(FourOfAKind, cards),
                    _       => None
                }
            },
            _ => {
                let fct_type = (
                    Self::is_straight(cards.clone()),
                    Self::is_flush(cards.clone())
                );
                match fct_type {
                    (true, true)    => build_fct!(
                        StraightFlush, cards
                    ),
                    (true, _)       => build_fct!(Straight, cards),
                    (_, true)       => build_fct!(Flush, cards),
                    _       => None
                }
            }
        }
    }

    fn is_straight(c: Vec<PlayedCard>) -> bool {
        c.iter().enumerate().all(|(i, &card)| {
            i == 0 || 
            card.previous_rank().is_some() 
            && c[i-1].get_rank() == card.previous_rank().unwrap()
        })
    }

    fn is_flush(c: Vec<PlayedCard>) -> bool {
        c.iter()
            .all(|&card| card.get_suit() == c[0].get_suit())
    }

    fn get_counts(cards: Vec<PlayedCard>) -> HashMap<Rank, usize> {
        cards.iter().fold(HashMap::new(), |mut acc, &card| {
            *acc.entry(card.get_rank()).or_insert(0) += 1;
            acc
        })
    }

    fn sort_cards(cards: Vec<PlayedCard>) -> Vec<PlayedCard> {
        let mut c = cards.clone();
        c.sort();
        c
    }
}


#[derive(
    Clone,
    Debug,
    PartialEq,
    PartialOrd,
    Eq,
    Ord,
    Copy,
    Serialize,
    Deserialize,
)]
/// Type of 5 card trick
pub enum TrickType{
    /// sequence
    Straight,
    /// same suit
    Flush,
    /// 3 over 2
    FullHouse,
    /// 4 of same, 1 different
    FourOfAKind,
    /// sequence of same suit
    StraightFlush,
    /// 5 of same
    FiveOfAKind,
}

#[derive(
    Clone,
    Debug,
    PartialEq,
    Copy,
    Serialize,
    Deserialize,
)]
pub struct Trick {
    pub trick_type: TrickType,
    pub cards: [PlayedCard; 5]
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::cards::*;

    #[test]
    fn an_empty_move_is_a_pass() {
        let hand = Hand::build(vec!());

        assert_eq!(hand.unwrap(), Hand::Pass);
    }

    #[test]
    fn a_single_card_is_a_single() {
        let suit_order = [
            Suit::Clubs,
            Suit::Hearts,
            Suit::Diamonds,
            Suit::Spades
        ];
        let clubs = SuitContext::new(Suit::Clubs, suit_order);

        let three_of_clubs = Card::new(Rank::Three, clubs, false);
        let card = PlayedCard::new(three_of_clubs, false);

        let cards = vec!(card);
        let hand = Hand::build(cards);

        assert_eq!(hand.unwrap(), Hand::Single(card));
    }

    #[test]
    fn a_pair_of_same_rank_cards_is_a_pair() {
        let suit_order = [
            Suit::Clubs,
            Suit::Hearts,
            Suit::Diamonds,
            Suit::Spades
        ];

        let clubs = SuitContext::new(Suit::Clubs, suit_order);
        let hearts = SuitContext::new(Suit::Hearts, suit_order);

        let three_of_clubs = Card::new(Rank::Three, clubs, false);
        let played_three_of_clubs = PlayedCard::new(three_of_clubs, false);
        let three_of_hearts = Card::new(Rank::Three, hearts, false);
        let played_three_of_hearts = PlayedCard::new(three_of_hearts, false);


        let cards = vec!(played_three_of_clubs, played_three_of_hearts);
        let hand = Hand::build(cards);

        assert_eq!(
            hand.unwrap(),
            Hand::Pair(played_three_of_clubs, played_three_of_hearts)
        );
    }

    #[test]
    fn a_pair_of_different_rank_cards_is_invalid() {
        let suit_order = [
            Suit::Clubs,
            Suit::Hearts,
            Suit::Diamonds,
            Suit::Spades
        ];
        let clubs = SuitContext::new(Suit::Clubs, suit_order);
        let hearts = SuitContext::new(Suit::Hearts, suit_order);

        let three_of_clubs = Card::new(Rank::Three, clubs, false);
        let played_three_of_clubs = PlayedCard::new(three_of_clubs, false);
        let four_of_hearts = Card::new(Rank::Four, hearts, false);
        let played_four_of_hearts = PlayedCard::new(four_of_hearts, false);


        let cards = vec!(played_three_of_clubs, played_four_of_hearts);
        let hand = Hand::build(cards);

        assert_eq!(
            hand,
            None
        );
    }

    #[test]
    fn three_cards_of_same_rank_is_a_prial() {
        let suit_order = [
            Suit::Clubs,
            Suit::Hearts,
            Suit::Diamonds,
            Suit::Spades
        ];
        let clubs = SuitContext::new(Suit::Clubs, suit_order);
        let hearts = SuitContext::new(Suit::Hearts, suit_order);
        let diamonds = SuitContext::new(
            Suit::Diamonds, suit_order
        );

        let three_of_clubs = Card::new(Rank::Three, clubs, false);
        let played_three_of_clubs = PlayedCard::new(
            three_of_clubs, false
        );
        let three_of_hearts = Card::new(
            Rank::Three, hearts, false
        );
        let played_three_of_hearts = PlayedCard::new(
            three_of_hearts, false
        );
        let three_of_diamonds = Card::new(
            Rank::Three, diamonds, false
        );
        let played_three_of_diamonds = PlayedCard::new(
            three_of_diamonds, false
        );

        let cards = vec!(
            played_three_of_clubs,
            played_three_of_hearts,
            played_three_of_diamonds
        );

        let hand = Hand::build(cards);

        assert_eq!(
            hand.unwrap(),
            Hand::Prial(
                played_three_of_clubs,
                played_three_of_hearts,
                played_three_of_diamonds
            )
        );
    }

    #[test]
    fn three_cards_of_different_rank_is_a_invalid() {
        let suit_order = [
            Suit::Clubs,
            Suit::Hearts,
            Suit::Diamonds,
            Suit::Spades
        ];
        let clubs = SuitContext::new(Suit::Clubs, suit_order);
        let hearts = SuitContext::new(Suit::Hearts, suit_order);
        let diamonds = SuitContext::new(Suit::Diamonds, suit_order);

        let three_of_clubs = Card::new(Rank::Three, clubs, false);
        let played_three_of_clubs = PlayedCard::new(three_of_clubs, false);
        let four_of_hearts = Card::new(Rank::Four, hearts, false);
        let played_four_of_hearts = PlayedCard::new(four_of_hearts, false);
        let three_of_diamonds = Card::new(Rank::Three, diamonds, false);
        let played_three_of_diamonds = PlayedCard::new(three_of_diamonds, false);

        let cards = vec!(
            played_three_of_clubs,
            played_four_of_hearts,
            played_three_of_diamonds
        );

        let hand = Hand::build(cards);

        assert_eq!(
            hand,
            None
        );
    }

    #[test]
    fn five_of_a_kind_is_five_of_a_kind() {
        let suit_order = [
            Suit::Clubs,
            Suit::Hearts,
            Suit::Diamonds,
            Suit::Spades
        ];
        let clubs = SuitContext::new(Suit::Clubs, suit_order);

        let three_of_clubs = Card::new(Rank::Three, clubs, false);
        let played_three_of_clubs = PlayedCard::new(three_of_clubs, false);

        let cards = vec!(
            played_three_of_clubs,
            played_three_of_clubs,
            played_three_of_clubs,
            played_three_of_clubs,
            played_three_of_clubs,
        );

        let hand = Hand::build(cards);
        let expected_cards = [
            played_three_of_clubs,
            played_three_of_clubs,
            played_three_of_clubs,
            played_three_of_clubs,
            played_three_of_clubs,
        ];

        assert_eq!(
            hand.unwrap(),
            build_fct!(FiveOfAKind, expected_cards).unwrap()
        );
    }

    #[test]
    fn four_of_a_kind_is_four_of_a_kind() {
        let suit_order = [
            Suit::Clubs,
            Suit::Hearts,
            Suit::Diamonds,
            Suit::Spades
        ];
        let clubs = SuitContext::new(Suit::Clubs, suit_order);

        let three_of_clubs = Card::new(Rank::Three, clubs, false);
        let four_of_clubs = Card::new(Rank::Four, clubs, false);
        let played_three_of_clubs = PlayedCard::new(three_of_clubs, false);
        let played_four_of_clubs = PlayedCard::new(four_of_clubs, false);

        let cards = vec!(
            played_three_of_clubs,
            played_three_of_clubs,
            played_three_of_clubs,
            played_three_of_clubs,
            played_four_of_clubs,
        );

        let hand = Hand::build(cards);
        let expected_cards = [
            played_three_of_clubs,
            played_three_of_clubs,
            played_three_of_clubs,
            played_three_of_clubs,
            played_four_of_clubs,
        ];

        assert_eq!(
            hand.unwrap(),
            build_fct!(FourOfAKind, expected_cards).unwrap()
        );
    }

    #[test]
    fn full_house_is_a_full_house() {
        let suit_order = [
            Suit::Clubs,
            Suit::Hearts,
            Suit::Diamonds,
            Suit::Spades
        ];
        let clubs = SuitContext::new(Suit::Clubs, suit_order);

        let three_of_clubs = Card::new(Rank::Three, clubs, false);
        let four_of_clubs = Card::new(Rank::Four, clubs, false);
        let played_three_of_clubs = PlayedCard::new(three_of_clubs, false);
        let played_four_of_clubs = PlayedCard::new(four_of_clubs, false);

        let cards = vec!(
            played_three_of_clubs,
            played_three_of_clubs,
            played_three_of_clubs,
            played_four_of_clubs,
            played_four_of_clubs,
        );

        let hand = Hand::build(cards);
        let expected_cards = [
            played_three_of_clubs,
            played_three_of_clubs,
            played_three_of_clubs,
            played_four_of_clubs,
            played_four_of_clubs,
        ];

        assert_eq!(
            hand.unwrap(),
            build_fct!(FullHouse, expected_cards).unwrap()
        );
    }

    #[test]
    fn flush_is_a_flush() {
        let suit_order = [
            Suit::Clubs,
            Suit::Hearts,
            Suit::Diamonds,
            Suit::Spades
        ];
        let clubs = SuitContext::new(Suit::Clubs, suit_order);

        let three_of_clubs = Card::new(Rank::Three, clubs, false);
        let four_of_clubs = Card::new(Rank::Four, clubs, false);
        let five_of_clubs = Card::new(Rank::Five, clubs, false);
        let played_three_of_clubs = PlayedCard::new(three_of_clubs, false);
        let played_four_of_clubs = PlayedCard::new(four_of_clubs, false);
        let played_five_of_clubs = PlayedCard::new(five_of_clubs, false);

        let cards = vec!(
            played_five_of_clubs,
            played_three_of_clubs,
            played_three_of_clubs,
            played_four_of_clubs,
            played_four_of_clubs,
        );

        let hand = Hand::build(cards);
        let expected_cards = [
            played_three_of_clubs,
            played_three_of_clubs,
            played_four_of_clubs,
            played_four_of_clubs,
            played_five_of_clubs,
        ];

        assert_eq!(
            hand.unwrap(),
            build_fct!(Flush, expected_cards).unwrap()
        );
    }

    #[test]
    fn straight_is_a_straight() {
        let suit_order = [
            Suit::Clubs,
            Suit::Hearts,
            Suit::Diamonds,
            Suit::Spades
        ];
        let clubs = SuitContext::new(Suit::Clubs, suit_order);
        let hearts = SuitContext::new(Suit::Hearts, suit_order);

        let three_of_clubs = Card::new(Rank::Three, clubs, false);
        let four_of_clubs = Card::new(Rank::Four, clubs, false);
        let five_of_clubs = Card::new(Rank::Five, clubs, false);
        let six_of_hearts = Card::new(Rank::Six, hearts, false);
        let seven_of_hearts = Card::new(
            Rank::Seven, hearts, false
        );
        let played_three_of_clubs = PlayedCard::new(
            three_of_clubs, false
        );
        let played_four_of_clubs = PlayedCard::new(
            four_of_clubs, false
        );
        let played_five_of_clubs = PlayedCard::new(
            five_of_clubs, false
        );
        let played_six_of_hearts = PlayedCard::new(
            six_of_hearts, false
        );
        let played_seven_of_hearts = PlayedCard::new(
            seven_of_hearts, false
        );

        let cards = vec!(
            played_five_of_clubs,
            played_three_of_clubs,
            played_six_of_hearts,
            played_four_of_clubs,
            played_seven_of_hearts,
        );

        let hand = Hand::build(cards);
        let expected_cards = [
            played_three_of_clubs,
            played_four_of_clubs,
            played_five_of_clubs,
            played_six_of_hearts,
            played_seven_of_hearts,
        ];

        assert_eq!(
            hand.unwrap(),
            build_fct!(Straight, expected_cards).unwrap()
        );
    }

    #[test]
    fn straight_flush_is_a_straight_flush() {
        let suit_order = [
            Suit::Clubs,
            Suit::Hearts,
            Suit::Diamonds,
            Suit::Spades
        ];
        let clubs = SuitContext::new(Suit::Clubs, suit_order);

        let three_of_clubs = Card::new(Rank::Three, clubs, false);
        let four_of_clubs = Card::new(Rank::Four, clubs, false);
        let five_of_clubs = Card::new(Rank::Five, clubs, false);
        let six_of_clubs = Card::new(Rank::Six, clubs, false);
        let seven_of_clubs = Card::new(
            Rank::Seven, clubs, false
        );
        let played_three_of_clubs = PlayedCard::new(
            three_of_clubs, false
        );
        let played_four_of_clubs = PlayedCard::new(
            four_of_clubs, false
        );
        let played_five_of_clubs = PlayedCard::new(
            five_of_clubs, false
        );
        let played_six_of_clubs = PlayedCard::new(
            six_of_clubs, false
        );
        let played_seven_of_clubs = PlayedCard::new(
            seven_of_clubs, false
        );

        let cards = vec!(
            played_five_of_clubs,
            played_three_of_clubs,
            played_six_of_clubs,
            played_four_of_clubs,
            played_seven_of_clubs,
        );

        let hand = Hand::build(cards);
        let expected_cards = [
            played_three_of_clubs,
            played_four_of_clubs,
            played_five_of_clubs,
            played_six_of_clubs,
            played_seven_of_clubs,
        ];

        assert_eq!(
            hand.unwrap(),
            build_fct!(StraightFlush, expected_cards).unwrap()
        );
    }

}
