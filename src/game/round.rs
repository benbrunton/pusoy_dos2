use super::{
    compare_hands,
    Hand,
    Player,
    Trick,
    TrickType,
    Ruleset,
};
use crate::cards::{Card, PlayedCard, Rank, Suit};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum SubmitError {
    FirstRoundPass,
    FirstHandMustContainLowestCard,
    HandNotHighEnough,
    NotCurrentPlayer,
    InvalidHand,
    PlayerDoesntHaveCard,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Round {
    players: Vec<Player>,
    next_player: Option<String>,
    last_move: Option<Hand>,
    last_player: Option<String>,
    suit_order: [Suit; 4],
    rank_order: [Rank; 13],
    ruleset: Ruleset
}

impl Round {
    pub fn new(
        players: Vec<Player>,
        next_player: Option<String>,
        last_move: Option<Hand>,
        last_player: Option<String>,
        suit_order: [Suit; 4],
        rank_order: [Rank; 13],
        ruleset: Ruleset
    ) -> Round {
        Round {
            players,
            next_player,
            last_move,
            last_player,
            suit_order,
            rank_order,
            ruleset,
        }
    }

    pub fn get_next_player(&self) -> Option<String> {
        match &self.next_player {
            None => {
                if self.get_players_still_in(&self.players).len() > 1 {
                    self.get_starting_player()
                } else {
                    None
                }
            },
            Some(x) => Some(x.to_string()),
        }
    }

    pub fn submit_move(
        &self,
        user_id: &str,
        cards: Vec<PlayedCard>
    ) -> Result<Round, SubmitError> {
        if user_id != self.get_next_player()
            .expect("invalid_player") {
            return Err(SubmitError::NotCurrentPlayer);
        }

        let hand = Hand::build(cards.clone());
        if hand.is_none() {
            return Err(SubmitError::InvalidHand);
        }

        if self.last_move == None {

            let starting_move_error = self.check_starting_move(
                &cards
            );

            if starting_move_error.is_some() {
                return Err(starting_move_error.unwrap());
            }

        } else if self.last_move != Some(Hand::Pass)
            && hand != Some(Hand::Pass) 
            && !self.hand_beats_last_move(hand.unwrap()) {
                return Err(SubmitError::HandNotHighEnough);
        }

        let mut player = self.get_player(user_id)
            .expect("invalid player!");

        match player.play_move(cards) {
            Ok(p) => player = p,
            _ => return Err(SubmitError::PlayerDoesntHaveCard)
        }

        let players = self.get_updated_players(&player);
        let new_last_player = if hand == Some(Hand::Pass) {
            self.last_player.to_owned()
        } else {
            Some(user_id.to_string())
        };

        let ( 
            new_last_move, next_player
        ) = self.get_last_move_and_new_player(
            user_id,
            hand,
            &new_last_player
        );

        let output_next_player = if self.get_players_still_in(&players)
            .len() > 1 {
            Some(next_player)
        } else {
            None
        };

        let (
            suit_order, rank_order
        ) = self.get_updated_suit_and_rank_order(hand);

        Ok(Self::new(
            players,
            output_next_player,
            new_last_move,
            new_last_player,
            suit_order,
            rank_order,
            self.ruleset
        ))
    }

    pub fn get_player(&self, user_id: &str) -> Option<Player> {
        for player in self.players.iter() {
            if player.get_id() == user_id {
                return Some(player.clone());
            }
        }

        None
    }

    pub fn get_last_move(&self) -> Option<Hand> {
        self.last_move
    }

    pub fn get_last_player(&self) -> Option<String> {
        match &self.last_player {
            None => None,
            Some(x) => Some(x.to_string())
        }
    }

    pub fn get_suit_order(&self) -> [Suit; 4] {
        self.suit_order
    }

    pub fn get_rank_order(&self) -> [Rank; 13] {
        self.rank_order
    }

    fn check_starting_move(
        &self,
        cards:&[PlayedCard]) -> Option<SubmitError> {
            if cards.is_empty() {
                return Some(SubmitError::FirstRoundPass);
            }

            if !self.contains_lowest_card(cards.to_vec()) {
                return Some(
                    SubmitError::FirstHandMustContainLowestCard
                );
            }

            None
    }

    fn get_starting_player(&self) -> Option<String> {
        let lowest_card = Card::Standard {
            deck_id: 0,
            suit: self.suit_order[0],
            rank: self.rank_order[0],
        };
        for player in self.players.iter() {
            if player.has_card(lowest_card) {
                return Some(player.get_id().to_string());
            }
        }
        None
    }

    fn get_updated_players(
        &self,
        player: &Player) -> Vec<Player> {
        self.players.iter().map(|p| {
            if p.get_id() == player.get_id() {
                player.clone()
            } else {
                p.clone()
            }
        }).collect()
    }

    fn hand_beats_last_move(&self, cards: Hand) -> bool {
        compare_hands(
            self.last_move
                .expect("cannot compare when no last_move"),
            cards,
            self.ruleset.flush_precedence,
            self.suit_order,
            self.rank_order,
        )
    }

    fn contains_lowest_card(&self, cards: Vec<PlayedCard>) -> bool {
        for &card in cards.iter() {
            if card.get_rank() == self.rank_order[0] && card.get_suit() == self.suit_order[0] {
                return true;
            }
        }

        false
    }

    fn get_next_player_in_rotation(&self, user_id: &str) -> String {
        if self.players.last()
            .unwrap().get_id() == user_id {
            return self.players.first()
                .unwrap().get_id().to_string();
        }
        let mut index = 0;
        for (i, player) in self.players.iter().enumerate() {
            if player.get_id() == user_id {
                index = i + 1;
            }
        }

        self.players[index].get_id().to_string()
    }

    fn get_players_still_in(&self, players: &[Player]) -> Vec<Player> {
        players.iter()
            .filter(|p| !p.get_hand().is_empty())
            .cloned()
            .collect()
    }

    fn get_last_move_and_new_player(&self,
            user_id: &str,
            hand: Option<Hand>,
            new_last_player: &Option<String>
    ) -> (Option<Hand>, String) {

        let mut new_last_move = hand;
        let mut next_player = self.get_next_player_in_rotation(
            user_id
        );

        if hand == Some(Hand::Pass) {
            new_last_move = self.last_move;
        }

        if next_player == new_last_player.clone()
            .unwrap_or_else(|| "invalid_player".to_string()) {
            new_last_move = Some(Hand::Pass);
        }

        while self.get_player(&next_player)
            .unwrap().get_hand().is_empty() {

            next_player = self.get_next_player_in_rotation(&next_player);
            if next_player == new_last_player.clone()
                .unwrap_or_else(|| "invalid_player".to_string()) {
                new_last_move = Some(Hand::Pass);
            }
        }

        (new_last_move, next_player)
    }

    fn get_updated_suit_and_rank_order(
        &self,
        hand:Option<Hand>
    ) -> ([Suit;4], [Rank;13]) {
        let mut suit_order = self.suit_order;
        let mut rank_order = self.rank_order;

        if self.ruleset.reversals_enabled {
            if let Hand::FiveCardTrick(Trick{
                    trick_type: TrickType::FourOfAKind,
                    ..
                }) = hand.unwrap_or(Hand::Pass) {
                suit_order.reverse();
                rank_order.reverse();
            } 
        }

        (suit_order, rank_order)
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::*;
    use crate::game::FlushPrecedence;

    static DEFAULT_SUIT_ORDER: [Suit; 4] =
        [Suit::Clubs, Suit::Hearts, Suit::Diamonds, Suit::Spades];

    static DEFAULT_RANK_ORDER: [Rank; 13] = [
        Rank::Three,
        Rank::Four,
        Rank::Five,
        Rank::Six,
        Rank::Seven,
        Rank::Eight,
        Rank::Nine,
        Rank::Ten,
        Rank::Jack,
        Rank::Queen,
        Rank::King,
        Rank::Ace,
        Rank::Two,
    ];

    const DEFAULT_RULESET: Ruleset = Ruleset{
        reversals_enabled: true,
        flush_precedence: FlushPrecedence::Rank,
    };

    #[test]
    fn when_game_hasnt_started_player_with_3clubs_starts() {
        let a_cards = vec![Card::Standard {
            deck_id: 0,
            rank: Rank::Three,
            suit: Suit::Clubs,
        }];
        let b_cards = vec![Card::Standard {
            deck_id: 0,
            rank: Rank::Four,
            suit: Suit::Clubs,
        }];
        let player_a = Player::new("a".to_string(), a_cards);
        let player_b = Player::new("b".to_string(), b_cards);

        let players = vec![player_a, player_b];

        let round = Round::new(
            players,
            None,
            None,
            None,
            DEFAULT_SUIT_ORDER,
            DEFAULT_RANK_ORDER,
            DEFAULT_RULESET,
        );

        assert_eq!(round.get_next_player(), Some("a".to_string()));
    }

    #[test]
    fn when_game_has_started_there_will_be_a_current_player() {
        let a_cards = vec![Card::Standard {
            deck_id: 0,
            rank: Rank::Three,
            suit: Suit::Clubs,
        }];
        let b_cards = vec![Card::Standard {
            deck_id: 0,
            rank: Rank::Four,
            suit: Suit::Clubs,
        }];
        let player_a = Player::new("a".to_string(), a_cards);
        let player_b = Player::new("b".to_string(), b_cards);
        let players = vec![player_a, player_b];
        let round = Round::new(
            players,
            Some("b".to_string()),
            None,
            None,
            DEFAULT_SUIT_ORDER,
            DEFAULT_RANK_ORDER,
            DEFAULT_RULESET,
        );

        assert_eq!(round.get_next_player(), Some("b".to_string()));
    }

    #[test]
    fn player_cannot_start_a_game_with_a_pass() {
        let a_cards = vec![Card::Standard {
            deck_id: 0,
            rank: Rank::Three,
            suit: Suit::Clubs,
        }];
        let b_cards = vec![Card::Standard {
            deck_id: 0,
            rank: Rank::Four,
            suit: Suit::Clubs,
        }];
        let player_a = Player::new("a".to_string(), a_cards);
        let player_b = Player::new("b".to_string(), b_cards);
        let players = vec![player_a, player_b];
        let round = Round::new(
            players,
            None,
            None,
            None,
            DEFAULT_SUIT_ORDER,
            DEFAULT_RANK_ORDER,
            DEFAULT_RULESET,
        );

        let err = round.submit_move("a", vec![]).err().unwrap();

        assert_eq!(err, SubmitError::FirstRoundPass);
    }

    #[test]
    fn player_must_start_a_game_with_three_clubs() {
        let a_cards = vec![
            Card::Standard {
                deck_id: 0,
                rank: Rank::Three,
                suit: Suit::Clubs,
            },
            Card::Standard {
                deck_id: 0,
                rank: Rank::Six,
                suit: Suit::Clubs,
            },
        ];
        let b_cards = vec![Card::Standard {
            deck_id: 0,
            rank: Rank::Four,
            suit: Suit::Clubs,
        }];
        let player_a = Player::new("a".to_string(), a_cards);
        let player_b = Player::new("b".to_string(), b_cards);
        let players = vec![player_a, player_b];
        let round = Round::new(
            players,
            None,
            None,
            None,
            DEFAULT_SUIT_ORDER,
            DEFAULT_RANK_ORDER,
            DEFAULT_RULESET,
        );
        let played_hand = vec![
            PlayedCard::new(Rank::Six, Suit::Clubs, false)
        ];
        let err = round.submit_move("a", played_hand)
            .err().unwrap();
        assert_eq!(
            err,
            SubmitError::FirstHandMustContainLowestCard
        );
    }

    #[test]
    fn playing_a_valid_hand_returns_the_new_round() {
        let a_cards = vec![
            Card::Standard {
                deck_id: 0,
                rank: Rank::Three,
                suit: Suit::Clubs,
            },
            Card::Standard {
                deck_id: 0,
                rank: Rank::Six,
                suit: Suit::Clubs,
            },
        ];
        let b_cards = vec![Card::Standard {
            deck_id: 0,
            rank: Rank::Four,
            suit: Suit::Clubs,
        }];
        let player_a = Player::new("a".to_string(), a_cards);
        let player_b = Player::new("b".to_string(), b_cards);
        let players = vec![player_a, player_b];
        let round = Round::new(
            players,
            None,
            None,
            None,
            DEFAULT_SUIT_ORDER,
            DEFAULT_RANK_ORDER,
            DEFAULT_RULESET
        );
        let played_hand = vec![
            PlayedCard::new(Rank::Three, Suit::Clubs, false)
        ];
        assert!(round.submit_move("a", played_hand).is_ok());
    }

    #[test]
    fn lower_hand_cannot_beat_last_move() {
        let a_cards = vec![
            Card::Standard {
                deck_id: 0,
                rank: Rank::Three,
                suit: Suit::Clubs,
            },
            Card::Standard {
                deck_id: 0,
                rank: Rank::Six,
                suit: Suit::Clubs,
            },
        ];
        let b_cards = vec![Card::Standard {
            deck_id: 0,
            rank: Rank::Four,
            suit: Suit::Clubs,
        }];
        let player_a = Player::new("a".to_string(), a_cards);
        let player_b = Player::new("b".to_string(), b_cards);
        let players = vec![player_a, player_b];
        let last_move = Some(Hand::Single(PlayedCard::new(
            Rank::Three,
            Suit::Clubs,
            false,
        )));
        let round = Round::new(
            players,
            Some("a".to_string()),
            last_move,
            None,
            DEFAULT_SUIT_ORDER,
            DEFAULT_RANK_ORDER,
            DEFAULT_RULESET
        );
        let played_hand = vec![
            PlayedCard::new(Rank::Three, Suit::Clubs, false)
        ];

        let err = round.submit_move("a", played_hand).err().unwrap();
        assert_eq!(err, SubmitError::HandNotHighEnough);
    }

    #[test]
    fn higher_hand_can_beat_last_move() {
        let a_cards = vec![
            Card::Standard {
                deck_id: 0,
                rank: Rank::Three,
                suit: Suit::Clubs,
            },
            Card::Standard {
                deck_id: 0,
                rank: Rank::Six,
                suit: Suit::Clubs,
            },
        ];
        let b_cards = vec![Card::Standard {
            deck_id: 0,
            rank: Rank::Four,
            suit: Suit::Clubs,
        }];
        let player_a = Player::new("a".to_string(), a_cards);
        let player_b = Player::new("b".to_string(), b_cards);
        let players = vec![player_a, player_b];
        let last_move = Some(Hand::Single(PlayedCard::new(
            Rank::Three,
            Suit::Clubs,
            false,
        )));
        let round = Round::new(
            players,
            Some("a".to_string()),
            last_move,
            None,
            DEFAULT_SUIT_ORDER,
            DEFAULT_RANK_ORDER,
            DEFAULT_RULESET
        );
        let played_hand = vec![
            PlayedCard::new(Rank::Six, Suit::Clubs, false)
        ];

        assert!(round.submit_move("a", played_hand).is_ok());
    }

    #[test]
    fn invalid_player_cannot_make_a_move() {
        let a_cards = vec![
            Card::Standard {
                deck_id: 0,
                rank: Rank::Three,
                suit: Suit::Clubs,
            },
            Card::Standard {
                deck_id: 0,
                rank: Rank::Six,
                suit: Suit::Clubs,
            },
        ];
        let b_cards = vec![Card::Standard {
            deck_id: 0,
            rank: Rank::Four,
            suit: Suit::Clubs,
        }];
        let player_a = Player::new("a".to_string(), a_cards);
        let player_b = Player::new("b".to_string(), b_cards);
        let players = vec![player_a, player_b];
        let last_move = Some(Hand::Single(PlayedCard::new(
            Rank::Three,
            Suit::Clubs,
            false,
        )));
        let round = Round::new(
            players,
            Some("a".to_string()),
            last_move,
            None,
            DEFAULT_SUIT_ORDER,
            DEFAULT_RANK_ORDER,
            DEFAULT_RULESET
        );
        let played_hand = vec![
            PlayedCard::new(Rank::Six, Suit::Clubs, false)
        ];

        let err = round.submit_move("b", played_hand)
            .err().unwrap();
        assert_eq!(err, SubmitError::NotCurrentPlayer);
    }

    #[test]
    fn it_should_be_a_valid_hand() {
        let a_cards = vec![
            Card::Standard {
                deck_id: 0,
                rank: Rank::Three,
                suit: Suit::Clubs,
            },
            Card::Standard {
                deck_id: 0,
                rank: Rank::Six,
                suit: Suit::Clubs,
            },
        ];
        let b_cards = vec![Card::Standard {
            deck_id: 0,
            rank: Rank::Four,
            suit: Suit::Clubs,
        }];
        let player_a = Player::new("a".to_string(), a_cards);
        let player_b = Player::new("b".to_string(), b_cards);
        let players = vec![player_a, player_b];
        let last_move = Some(Hand::Single(PlayedCard::new(
            Rank::Three,
            Suit::Clubs,
            false,
        )));
        let round = Round::new(
            players,
            Some("a".to_string()),
            last_move,
            None,
            DEFAULT_SUIT_ORDER,
            DEFAULT_RANK_ORDER,
            DEFAULT_RULESET
        );
        let played_hand = vec![
            PlayedCard::new(Rank::Six, Suit::Clubs, false),
            PlayedCard::new(Rank::Three, Suit::Clubs, false),
        ];

        let err = round.submit_move("a", played_hand)
            .err().unwrap();
        assert_eq!(err, SubmitError::InvalidHand);
    }

    #[test]
    fn it_should_be_a_valid_hand_even_at_start() {
        let a_cards = vec![
            Card::Standard {
                deck_id: 0,
                rank: Rank::Three,
                suit: Suit::Clubs,
            },
            Card::Standard {
                deck_id: 0,
                rank: Rank::Six,
                suit: Suit::Clubs,
            },
        ];
        let b_cards = vec![Card::Standard {
            deck_id: 0,
            rank: Rank::Four,
            suit: Suit::Clubs,
        }];
        let player_a = Player::new("a".to_string(), a_cards);
        let player_b = Player::new("b".to_string(), b_cards);
        let players = vec![player_a, player_b];
        let round = Round::new(
            players,
            None,
            None,
            None,
            DEFAULT_SUIT_ORDER,
            DEFAULT_RANK_ORDER,
            DEFAULT_RULESET
        );
        let played_hand = vec![
            PlayedCard::new(Rank::Six, Suit::Clubs, false),
            PlayedCard::new(Rank::Three, Suit::Clubs, false),
        ];

        let err = round.submit_move("a", played_hand)
            .err().unwrap();
        assert_eq!(err, SubmitError::InvalidHand);
    }

    #[test]
    fn player_cannot_play_cards_it_doesnt_hold() {
        let a_cards = vec![
            Card::Standard {
                deck_id: 0,
                rank: Rank::Three,
                suit: Suit::Clubs,
            },
            Card::Standard {
                deck_id: 0,
                rank: Rank::Six,
                suit: Suit::Clubs,
            },
        ];
        let b_cards = vec![Card::Standard {
            deck_id: 0,
            rank: Rank::Four,
            suit: Suit::Clubs,
        }];
        let player_a = Player::new("a".to_string(), a_cards);
        let player_b = Player::new("b".to_string(), b_cards);
        let players = vec![player_a, player_b];
        let last_move = Some(Hand::Single(PlayedCard::new(
            Rank::Three,
            Suit::Clubs,
            false,
        )));
        let round = Round::new(
            players,
            Some("a".to_string()),
            last_move,
            None,
            DEFAULT_SUIT_ORDER,
            DEFAULT_RANK_ORDER,
            DEFAULT_RULESET
        );
        let played_hand = vec![
            PlayedCard::new(Rank::Four, Suit::Clubs, false),
        ];

        let err = round.submit_move("a", played_hand)
            .err().unwrap();

        assert_eq!(err, SubmitError::PlayerDoesntHaveCard);

    }

    #[test]
    fn player_can_only_play_a_card_once() {
        let a_cards = vec![
            Card::Standard {
                deck_id: 0,
                rank: Rank::Three,
                suit: Suit::Clubs,
            },
            Card::Standard {
                deck_id: 0,
                rank: Rank::Six,
                suit: Suit::Clubs,
            },
        ];
        let b_cards = vec![Card::Standard {
            deck_id: 0,
            rank: Rank::Four,
            suit: Suit::Clubs,
        }];
        let player_a = Player::new("a".to_string(), a_cards);
        let player_b = Player::new("b".to_string(), b_cards);
        let players = vec![player_a, player_b];
        let last_move = Some(
            Hand::Pair(
                PlayedCard::new(
                    Rank::Three,
                    Suit::Clubs,
                    false,
                ),
                PlayedCard::new(
                    Rank::Three,
                    Suit::Clubs,
                    false,
                )
            )
        );
        let round = Round::new(
            players,
            Some("a".to_string()),
            last_move,
            None,
            DEFAULT_SUIT_ORDER,
            DEFAULT_RANK_ORDER,
            DEFAULT_RULESET
        );
        let played_hand = vec![
            PlayedCard::new(Rank::Six, Suit::Clubs, false),
            PlayedCard::new(Rank::Six, Suit::Clubs, false),
        ];

        let err = round.submit_move("a", played_hand)
            .err().unwrap();

        assert_eq!(err, SubmitError::PlayerDoesntHaveCard);

    }

    #[test]
    fn playing_a_valid_card_removes_from_players_hand() {
        let a_cards = vec![
            Card::Standard {
                deck_id: 0,
                rank: Rank::Three,
                suit: Suit::Clubs,
            },
            Card::Standard {
                deck_id: 0,
                rank: Rank::Six,
                suit: Suit::Clubs,
            },
        ];
        let b_cards = vec![Card::Standard {
            deck_id: 0,
            rank: Rank::Four,
            suit: Suit::Clubs,
        }];
        let player_a = Player::new("a".to_string(), a_cards);
        let player_b = Player::new("b".to_string(), b_cards);
        let players = vec![player_a, player_b];
        let round = Round::new(
            players,
            None,
            None,
            None,
            DEFAULT_SUIT_ORDER,
            DEFAULT_RANK_ORDER,
            DEFAULT_RULESET
        );
        let played_hand = vec![
            PlayedCard::new(Rank::Three, Suit::Clubs, false)
        ];

        let new_round = round.submit_move("a", played_hand)
            .unwrap();

        let new_player_a = new_round.get_player("a").unwrap();

        assert_eq!(new_player_a.get_hand().len(), 1);
    }

    #[test]
    fn a_valid_move_is_set_as_last_move() {
        let a_cards = vec![
            Card::Standard {
                deck_id: 0,
                rank: Rank::Three,
                suit: Suit::Clubs,
            },
            Card::Standard {
                deck_id: 0,
                rank: Rank::Six,
                suit: Suit::Clubs,
            },
        ];
        let b_cards = vec![Card::Standard {
            deck_id: 0,
            rank: Rank::Four,
            suit: Suit::Clubs,
        }];
        let player_a = Player::new("a".to_string(), a_cards);
        let player_b = Player::new("b".to_string(), b_cards);
        let players = vec![player_a, player_b];
        let round = Round::new(
            players,
            None,
            None,
            None,
            DEFAULT_SUIT_ORDER,
            DEFAULT_RANK_ORDER,
            DEFAULT_RULESET
        );
        let played_hand = vec![
            PlayedCard::new(Rank::Three, Suit::Clubs, false)
        ];

        let new_round = round.submit_move("a", played_hand)
            .unwrap();

        assert_eq!(
            new_round.get_last_move(),
            Some(Hand::Single(PlayedCard::new(
                Rank::Three, Suit::Clubs, false
            )))
        );
    }

    #[test]
    fn when_a_valid_move_is_made_the_next_player_rotates() {
        let a_cards = vec![
            Card::Standard {
                deck_id: 0,
                rank: Rank::Three,
                suit: Suit::Clubs,
            },
            Card::Standard {
                deck_id: 0,
                rank: Rank::Six,
                suit: Suit::Clubs,
            },
        ];
        let b_cards = vec![Card::Standard {
            deck_id: 0,
            rank: Rank::Four,
            suit: Suit::Clubs,
        }];
        let player_a = Player::new("a".to_string(), a_cards);
        let player_b = Player::new("b".to_string(), b_cards);
        let players = vec![player_a, player_b];
        let round = Round::new(
            players,
            None,
            None,
            None,
            DEFAULT_SUIT_ORDER,
            DEFAULT_RANK_ORDER,
            DEFAULT_RULESET
        );
        let played_hand = vec![
            PlayedCard::new(Rank::Three, Suit::Clubs, false)
        ];

        let new_round = round.submit_move("a", played_hand)
            .unwrap();

        assert_eq!(
            new_round.get_next_player(),
            Some("b".to_string())
        );

    }

    #[test]
    fn player_rotation_comes_back_round() {
        let a_cards = vec![
            Card::Standard {
                deck_id: 0,
                rank: Rank::Four,
                suit: Suit::Clubs,
            },
            Card::Standard {
                deck_id: 0,
                rank: Rank::Six,
                suit: Suit::Clubs,
            },
        ];
        let b_cards = vec![
            Card::Standard {
                deck_id: 0,
                rank: Rank::Three,
                suit: Suit::Clubs,
            },
            Card::Standard {
                deck_id: 0,
                rank: Rank::Six,
                suit: Suit::Clubs,
            }
        ];
        let player_a = Player::new("a".to_string(), a_cards);
        let player_b = Player::new("b".to_string(), b_cards);
        let players = vec![player_a, player_b];
        let round = Round::new(
            players,
            Some("b".to_string()),
            None,
            None,
            DEFAULT_SUIT_ORDER,
            DEFAULT_RANK_ORDER,
            DEFAULT_RULESET
        );
        let played_hand = vec![
            PlayedCard::new(Rank::Three, Suit::Clubs, false)
        ];

        let new_round = round.submit_move("b", played_hand)
            .unwrap();

        assert_eq!(
            new_round.get_next_player(),
            Some("a".to_string())
        );

    }

    #[test]
    fn passing_moves_without_changing_the_last_move() {
        let a_cards = vec![
            Card::Standard {
                deck_id: 0,
                rank: Rank::Four,
                suit: Suit::Clubs,
            },
            Card::Standard {
                deck_id: 0,
                rank: Rank::Six,
                suit: Suit::Clubs,
            },
        ];
        let b_cards = vec![
            Card::Standard {
                deck_id: 0,
                rank: Rank::Three,
                suit: Suit::Clubs,
            },
            Card::Standard {
                deck_id: 0,
                rank: Rank::Six,
                suit: Suit::Clubs,
            }
        ];
        let c_cards = vec![
            Card::Standard {
                deck_id: 0,
                rank: Rank::Three,
                suit: Suit::Clubs,
            },
            Card::Standard {
                deck_id: 0,
                rank: Rank::Six,
                suit: Suit::Clubs,
            }
        ];
        let player_a = Player::new("a".to_string(), a_cards);
        let player_b = Player::new("b".to_string(), b_cards);
        let player_c = Player::new("c".to_string(), c_cards);
        let players = vec![player_a, player_b, player_c];
        let last_move = Some(
            Hand::Pair(
                PlayedCard::new(
                    Rank::Three,
                    Suit::Clubs,
                    false,
                ),
                PlayedCard::new(
                    Rank::Three,
                    Suit::Clubs,
                    false,
                )
            )
        );

        let round = Round::new(
            players,
            Some("b".to_string()),
            last_move,
            None,
            DEFAULT_SUIT_ORDER,
            DEFAULT_RANK_ORDER,
            DEFAULT_RULESET
        );
        let played_hand = vec![];

        let new_round = round.submit_move("b", played_hand)
            .unwrap();

        assert_eq!(
            new_round.get_next_player(),
            Some("c".to_string())
        );

        assert_eq!(
            new_round.get_last_move(),
            last_move
        );
    }

    #[test]
    fn a_valid_move_switches_the_last_player() {
        let a_cards = vec![
            Card::Standard {
                deck_id: 0,
                rank: Rank::Four,
                suit: Suit::Clubs,
            },
            Card::Standard {
                deck_id: 0,
                rank: Rank::Six,
                suit: Suit::Clubs,
            },
        ];
        let b_cards = vec![
            Card::Standard {
                deck_id: 0,
                rank: Rank::Three,
                suit: Suit::Clubs,
            },
            Card::Standard {
                deck_id: 0,
                rank: Rank::Six,
                suit: Suit::Clubs,
            },
            Card::Standard {
                deck_id: 0,
                rank: Rank::Three,
                suit: Suit::Spades,
            }
        ];
        let c_cards = vec![
            Card::Standard {
                deck_id: 0,
                rank: Rank::Three,
                suit: Suit::Clubs,
            },
            Card::Standard {
                deck_id: 0,
                rank: Rank::Six,
                suit: Suit::Clubs,
            }
        ];
        let player_a = Player::new("a".to_string(), a_cards);
        let player_b = Player::new("b".to_string(), b_cards);
        let player_c = Player::new("c".to_string(), c_cards);
        let players = vec![player_a, player_b, player_c];
        let last_move = Some(
            Hand::Pair(
                PlayedCard::new(
                    Rank::Three,
                    Suit::Clubs,
                    false,
                ),
                PlayedCard::new(
                    Rank::Three,
                    Suit::Clubs,
                    false,
                )
            )
        );

        let round = Round::new(
            players,
            Some("b".to_string()),
            last_move,
            None,
            DEFAULT_SUIT_ORDER,
            DEFAULT_RANK_ORDER,
            DEFAULT_RULESET
        );
        let played_hand = vec![
            PlayedCard::new(
                Rank::Three,
                Suit::Clubs,
                false,
            ),
            PlayedCard::new(
                Rank::Three,
                Suit::Spades,
                false,
            )
        ];

        let new_round = round.submit_move("b", played_hand)
            .unwrap();

        assert_eq!(
            new_round.get_last_player(),
            Some("b".to_string())
        );
    }

    #[test]
    fn if_last_and_next_player_are_same_the_table_is_cleared() {
        let a_cards = vec![
            Card::Standard {
                deck_id: 0,
                rank: Rank::Four,
                suit: Suit::Clubs,
            },
            Card::Standard {
                deck_id: 0,
                rank: Rank::Six,
                suit: Suit::Clubs,
            },
        ];
        let b_cards = vec![
            Card::Standard {
                deck_id: 0,
                rank: Rank::Three,
                suit: Suit::Clubs,
            },
            Card::Standard {
                deck_id: 0,
                rank: Rank::Six,
                suit: Suit::Clubs,
            },
            Card::Standard {
                deck_id: 0,
                rank: Rank::Three,
                suit: Suit::Spades,
            }
        ];
        let c_cards = vec![
            Card::Standard {
                deck_id: 0,
                rank: Rank::Three,
                suit: Suit::Clubs,
            },
            Card::Standard {
                deck_id: 0,
                rank: Rank::Six,
                suit: Suit::Clubs,
            }
        ];
        let player_a = Player::new("a".to_string(), a_cards);
        let player_b = Player::new("b".to_string(), b_cards);
        let player_c = Player::new("c".to_string(), c_cards);
        let players = vec![player_a, player_b, player_c];
        let last_move = Some(
            Hand::Pair(
                PlayedCard::new(
                    Rank::Three,
                    Suit::Clubs,
                    false,
                ),
                PlayedCard::new(
                    Rank::Three,
                    Suit::Clubs,
                    false,
                )
            )
        );

        let round = Round::new(
            players,
            Some("b".to_string()),
            last_move,
            Some("c".to_string()),
            DEFAULT_SUIT_ORDER,
            DEFAULT_RANK_ORDER,
            DEFAULT_RULESET
        );
        let played_hand = vec![];

        let new_round = round.submit_move("b", played_hand)
            .unwrap();

        assert_eq!(
            new_round.get_last_move(),
            Some(Hand::Pass)
        );

    }

    #[test]
    fn any_card_beats_a_pass() {
        let a_cards = vec![
            Card::Standard {
                deck_id: 0,
                rank: Rank::Three,
                suit: Suit::Clubs,
            },
            Card::Standard {
                deck_id: 0,
                rank: Rank::Three,
                suit: Suit::Clubs,
            }
        ];
        let b_cards = vec![Card::Standard {
            deck_id: 0,
            rank: Rank::Four,
            suit: Suit::Clubs,
        }];
        let player_a = Player::new("a".to_string(), a_cards);
        let player_b = Player::new("b".to_string(), b_cards);

        let players = vec![player_a, player_b];

        let round = Round::new(
            players,
            Some("a".to_string()),
            Some(Hand::Pass),
            Some("a".to_string()),
            DEFAULT_SUIT_ORDER,
            DEFAULT_RANK_ORDER,
            DEFAULT_RULESET
        );

        let played_hand = vec![
            PlayedCard::new(
                Rank::Three,
                Suit::Clubs,
                false,
            ),
        ];

        let new_round = round.submit_move("a", played_hand);

        assert!(new_round.is_ok());
    }

    #[test]
    fn players_with_no_cards_are_skipped() {
        let a_cards = vec![
            Card::Standard {
                deck_id: 0,
                rank: Rank::Three,
                suit: Suit::Clubs,
            },
            Card::Standard {
                deck_id: 0,
                rank: Rank::Three,
                suit: Suit::Clubs,
            }
        ];
        let b_cards = vec![];
        let c_cards = vec![Card::Standard{
            deck_id: 0,
            rank: Rank::Three,
            suit: Suit::Clubs,
        }];
        let player_a = Player::new("a".to_string(), a_cards);
        let player_b = Player::new("b".to_string(), b_cards);
        let player_c = Player::new("c".to_string(), c_cards);

        let players = vec![player_a, player_b, player_c];

        let last_move = Some(
            Hand::Pair(
                PlayedCard::new(
                    Rank::Three,
                    Suit::Clubs,
                    false,
                ),
                PlayedCard::new(
                    Rank::Three,
                    Suit::Clubs,
                    false,
                )
            )
        );

        let round = Round::new(
            players,
            Some("a".to_string()),
            last_move,
            Some("c".to_string()),
            DEFAULT_SUIT_ORDER,
            DEFAULT_RANK_ORDER,
            DEFAULT_RULESET
        );

        let played_hand = vec![];

        let new_round = round.submit_move(
            "a",
            played_hand
        ).unwrap();

        assert_eq!(
            new_round.get_next_player().unwrap(),
            "c".to_string()
        );
    }

    #[test]
    fn once_the_game_is_over_the_next_player_is_none() {
        let a_cards = vec![
            Card::Standard {
                deck_id: 0,
                rank: Rank::Four,
                suit: Suit::Spades,
            },
            Card::Standard {
                deck_id: 0,
                rank: Rank::Four,
                suit: Suit::Clubs,
            }
        ];
        let b_cards = vec![];
        let c_cards = vec![Card::Standard{
            deck_id: 0,
            rank: Rank::Three,
            suit: Suit::Clubs,
        }];
        let player_a = Player::new("a".to_string(), a_cards);
        let player_b = Player::new("b".to_string(), b_cards);
        let player_c = Player::new("c".to_string(), c_cards);

        let players = vec![player_a, player_b, player_c];

        let last_move = Some(
            Hand::Pair(
                PlayedCard::new(
                    Rank::Three,
                    Suit::Clubs,
                    false,
                ),
                PlayedCard::new(
                    Rank::Three,
                    Suit::Clubs,
                    false,
                )
            )
        );

        let round = Round::new(
            players,
            Some("a".to_string()),
            last_move,
            Some("c".to_string()),
            DEFAULT_SUIT_ORDER,
            DEFAULT_RANK_ORDER,
            DEFAULT_RULESET
        );

        let played_hand = vec![
            PlayedCard::new(
                Rank::Four,
                Suit::Spades,
                false,
            ),
            PlayedCard::new(
                Rank::Four,
                Suit::Clubs,
                false,
            )
        ];

        let new_round = round.submit_move(
            "a",
            played_hand
        ).unwrap();

        assert!(
            new_round.get_next_player().is_none(),
        );

    }

    #[test]
    fn when_player_wins_next_player_starts() {
        let a_cards = vec![
            Card::Standard {
                deck_id: 0,
                rank: Rank::Three,
                suit: Suit::Clubs,
            },
            Card::Standard {
                deck_id: 0,
                rank: Rank::Three,
                suit: Suit::Clubs,
            }
        ];
        let b_cards = vec![];
        let c_cards = vec![Card::Standard{
            deck_id: 0,
            rank: Rank::Three,
            suit: Suit::Clubs,
        }];
        let player_a = Player::new("a".to_string(), a_cards);
        let player_b = Player::new("b".to_string(), b_cards);
        let player_c = Player::new("c".to_string(), c_cards);

        let players = vec![player_a, player_b, player_c];

        let last_move = Some(
            Hand::Pair(
                PlayedCard::new(
                    Rank::Three,
                    Suit::Clubs,
                    false,
                ),
                PlayedCard::new(
                    Rank::Three,
                    Suit::Clubs,
                    false,
                )
            )
        );

        let round = Round::new(
            players,
            Some("a".to_string()),
            last_move,
            Some("b".to_string()),
            DEFAULT_SUIT_ORDER,
            DEFAULT_RANK_ORDER,
            DEFAULT_RULESET
        );

        let played_hand = vec![];

        let new_round = round.submit_move(
            "a",
            played_hand
        ).unwrap();

        assert_eq!(
            new_round.get_last_move().unwrap(),
            Hand::Pass
        );
    }

    #[test]
    fn playing_on_pass_sets_player_as_last_move() {
        let a_cards = vec![
            Card::Standard {
                deck_id: 0,
                rank: Rank::Three,
                suit: Suit::Clubs,
            },
            Card::Standard {
                deck_id: 0,
                rank: Rank::Three,
                suit: Suit::Clubs,
            }
        ];
        let b_cards = vec![];
        let c_cards = vec![Card::Standard{
            deck_id: 0,
            rank: Rank::Three,
            suit: Suit::Clubs,
        }];
        let player_a = Player::new("a".to_string(), a_cards);
        let player_b = Player::new("b".to_string(), b_cards);
        let player_c = Player::new("c".to_string(), c_cards);

        let players = vec![player_a, player_b, player_c];

        let last_move = Some(
            Hand::Pass
        );

        let round = Round::new(
            players,
            Some("a".to_string()),
            last_move,
            Some("c".to_string()),
            DEFAULT_SUIT_ORDER,
            DEFAULT_RANK_ORDER,
            DEFAULT_RULESET
        );

        let played_hand = vec![
            PlayedCard::new(Rank::Three, Suit::Clubs, false)
        ];

        let new_round = round.submit_move(
            "a",
            played_hand
        ).unwrap();

        assert_eq!(
            new_round.get_last_player().unwrap(),
            "a".to_string()
        );
    }

    #[test]
    fn when_there_are_two_players_left_pass_clears_the_table() {
        let a_cards = vec![
            Card::Standard {
                deck_id: 0,
                rank: Rank::Three,
                suit: Suit::Clubs,
            },
            Card::Standard {
                deck_id: 0,
                rank: Rank::Three,
                suit: Suit::Clubs,
            }
        ];
        let b_cards = vec![];
        let c_cards = vec![];
        let d_cards = vec![
             Card::Standard {
                deck_id: 0,
                rank: Rank::Three,
                suit: Suit::Clubs,
            },
        ];
        let player_a = Player::new("a".to_string(), a_cards);
        let player_b = Player::new("b".to_string(), b_cards);
        let player_c = Player::new("c".to_string(), c_cards);
        let player_d = Player::new("d".to_string(), d_cards);

        let players = vec![
            player_a,
            player_b,
            player_c,
            player_d
        ];

        let last_move = Some(
            Hand::Pair(
                PlayedCard::new(
                    Rank::Three,
                    Suit::Clubs,
                    false,
                ),
                PlayedCard::new(
                    Rank::Three,
                    Suit::Clubs,
                    false,
                )
            )
        );

        let round = Round::new(
            players,
            Some("a".to_string()),
            last_move,
            Some("d".to_string()),
            DEFAULT_SUIT_ORDER,
            DEFAULT_RANK_ORDER,
            DEFAULT_RULESET
        );

        let played_hand = vec![];

        let new_round = round.submit_move(
            "a",
            played_hand
        ).unwrap();

        assert_eq!(
            new_round.get_next_player().unwrap(),
            "d".to_string()
        );

        assert_eq!(
            new_round.get_last_move().unwrap(),
            Hand::Pass
        );
    }

    #[test]
    fn when_reversals_are_enabled_4ofakind_reverses_orders() {
        let a_cards = vec![
            Card::Standard {
                deck_id: 0,
                rank: Rank::Three,
                suit: Suit::Clubs,
            },
            Card::Standard {
                deck_id: 0,
                rank: Rank::Three,
                suit: Suit::Clubs,
            },
            Card::Standard {
                deck_id: 0,
                rank: Rank::Three,
                suit: Suit::Clubs,
            },
            Card::Standard {
                deck_id: 0,
                rank: Rank::Three,
                suit: Suit::Clubs,
            },
            Card::Standard {
                deck_id: 0,
                rank: Rank::Four,
                suit: Suit::Clubs,
            }
        ];
        let b_cards = vec![
            Card::Standard {
                deck_id: 0,
                rank: Rank::Three,
                suit: Suit::Clubs,
            },
        ];
        let c_cards = vec![Card::Standard{
            deck_id: 0,
            rank: Rank::Three,
            suit: Suit::Clubs,
        }];
        let player_a = Player::new("a".to_string(), a_cards);
        let player_b = Player::new("b".to_string(), b_cards);
        let player_c = Player::new("c".to_string(), c_cards);

        let players = vec![player_a, player_b, player_c];
        let last_move = Some(Hand::Pass);

        let round = Round::new(
            players,
            Some("a".to_string()),
            last_move,
            Some("b".to_string()),
            DEFAULT_SUIT_ORDER,
            DEFAULT_RANK_ORDER,
            DEFAULT_RULESET
        );

        let played_hand = vec![
            PlayedCard::new(Rank::Three, Suit::Clubs, false),
            PlayedCard::new(Rank::Three, Suit::Clubs, false),
            PlayedCard::new(Rank::Three, Suit::Clubs, false),
            PlayedCard::new(Rank::Three, Suit::Clubs, false),
            PlayedCard::new(Rank::Four, Suit::Clubs, false),
        ];

        let new_round = round.submit_move(
            "a",
            played_hand
        ).unwrap();

        let mut expected_suit_order = DEFAULT_SUIT_ORDER;
        let mut expected_rank_order = DEFAULT_RANK_ORDER;
        expected_suit_order.reverse();
        expected_rank_order.reverse();

        assert_eq!(
            new_round.get_suit_order(),
            expected_suit_order
        );

        assert_eq!(
            new_round.get_rank_order(),
            expected_rank_order
        );
    }

    #[test]
    fn when_reversals_are_enabled_only_a_4or5ofakind_reverses() {
        let a_cards = vec![
            Card::Standard {
                deck_id: 0,
                rank: Rank::Three,
                suit: Suit::Clubs,
            },
            Card::Standard {
                deck_id: 0,
                rank: Rank::Three,
                suit: Suit::Clubs,
            },
            Card::Standard {
                deck_id: 0,
                rank: Rank::Three,
                suit: Suit::Clubs,
            },
            Card::Standard {
                deck_id: 0,
                rank: Rank::Four,
                suit: Suit::Clubs,
            },
            Card::Standard {
                deck_id: 0,
                rank: Rank::Four,
                suit: Suit::Clubs,
            }
        ];
        let b_cards = vec![
            Card::Standard {
                deck_id: 0,
                rank: Rank::Three,
                suit: Suit::Clubs,
            },
        ];
        let c_cards = vec![Card::Standard{
            deck_id: 0,
            rank: Rank::Three,
            suit: Suit::Clubs,
        }];
        let player_a = Player::new("a".to_string(), a_cards);
        let player_b = Player::new("b".to_string(), b_cards);
        let player_c = Player::new("c".to_string(), c_cards);

        let players = vec![player_a, player_b, player_c];
        let last_move = Some(Hand::Pass);

        let round = Round::new(
            players,
            Some("a".to_string()),
            last_move,
            Some("b".to_string()),
            DEFAULT_SUIT_ORDER,
            DEFAULT_RANK_ORDER,
            DEFAULT_RULESET
        );

        let played_hand = vec![
            PlayedCard::new(Rank::Three, Suit::Clubs, false),
            PlayedCard::new(Rank::Three, Suit::Clubs, false),
            PlayedCard::new(Rank::Three, Suit::Clubs, false),
            PlayedCard::new(Rank::Four, Suit::Clubs, false),
            PlayedCard::new(Rank::Four, Suit::Clubs, false),
        ];

        let new_round = round.submit_move(
            "a",
            played_hand
        ).unwrap();

        let expected_suit_order = DEFAULT_SUIT_ORDER;
        let expected_rank_order = DEFAULT_RANK_ORDER;

        assert_eq!(
            new_round.get_suit_order(),
            expected_suit_order
        );

        assert_eq!(
            new_round.get_rank_order(),
            expected_rank_order
        );
    }

    #[test]
    fn when_reversals_are_not_enabled_no_reversals() {
        let a_cards = vec![
            Card::Standard {
                deck_id: 0,
                rank: Rank::Three,
                suit: Suit::Clubs,
            },
            Card::Standard {
                deck_id: 0,
                rank: Rank::Three,
                suit: Suit::Clubs,
            },
            Card::Standard {
                deck_id: 0,
                rank: Rank::Three,
                suit: Suit::Clubs,
            },
            Card::Standard {
                deck_id: 0,
                rank: Rank::Three,
                suit: Suit::Clubs,
            },
            Card::Standard {
                deck_id: 0,
                rank: Rank::Four,
                suit: Suit::Clubs,
            }
        ];
        let b_cards = vec![
            Card::Standard {
                deck_id: 0,
                rank: Rank::Three,
                suit: Suit::Clubs,
            },
        ];
        let c_cards = vec![Card::Standard{
            deck_id: 0,
            rank: Rank::Three,
            suit: Suit::Clubs,
        }];
        let player_a = Player::new("a".to_string(), a_cards);
        let player_b = Player::new("b".to_string(), b_cards);
        let player_c = Player::new("c".to_string(), c_cards);

        let players = vec![player_a, player_b, player_c];
        let last_move = Some(Hand::Pass);

        let ruleset = Ruleset {
            reversals_enabled: false,
            flush_precedence: FlushPrecedence::Rank
        };

        let round = Round::new(
            players,
            Some("a".to_string()),
            last_move,
            Some("b".to_string()),
            DEFAULT_SUIT_ORDER,
            DEFAULT_RANK_ORDER,
            ruleset
        );

        let played_hand = vec![
            PlayedCard::new(Rank::Three, Suit::Clubs, false),
            PlayedCard::new(Rank::Three, Suit::Clubs, false),
            PlayedCard::new(Rank::Three, Suit::Clubs, false),
            PlayedCard::new(Rank::Three, Suit::Clubs, false),
            PlayedCard::new(Rank::Four, Suit::Clubs, false),
        ];

        let new_round = round.submit_move(
            "a",
            played_hand
        ).unwrap();

        let expected_suit_order = DEFAULT_SUIT_ORDER;
        let expected_rank_order = DEFAULT_RANK_ORDER;

        assert_eq!(
            new_round.get_suit_order(),
            expected_suit_order
        );

        assert_eq!(
            new_round.get_rank_order(),
            expected_rank_order
        );
    }

    #[test]
    fn deck_id_is_not_checked_when_move_played() {
        let a_cards = vec![
            Card::Standard {
                deck_id: 1,
                rank: Rank::Three,
                suit: Suit::Clubs,
            },
            Card::Standard {
                deck_id: 0,
                rank: Rank::Four,
                suit: Suit::Clubs,
            },
        ];
        let b_cards = vec![
            Card::Standard {
                deck_id: 0,
                rank: Rank::Three,
                suit: Suit::Clubs,
            },
        ];
        let c_cards = vec![Card::Standard{
            deck_id: 0,
            rank: Rank::Three,
            suit: Suit::Clubs,
        }];
        let player_a = Player::new("a".to_string(), a_cards);
        let player_b = Player::new("b".to_string(), b_cards);
        let player_c = Player::new("c".to_string(), c_cards);

        let players = vec![player_a, player_b, player_c];
        let last_move = Some(Hand::Pass);
        let ruleset = Ruleset {
            reversals_enabled: false,
            flush_precedence: FlushPrecedence::Rank
        };

        let round = Round::new(
            players,
            Some("a".to_string()),
            last_move,
            Some("b".to_string()),
            DEFAULT_SUIT_ORDER,
            DEFAULT_RANK_ORDER,
            ruleset
        );

        let played_hand = vec![
            PlayedCard::new(Rank::Three, Suit::Clubs, false),
        ];

        let new_round = round.submit_move(
            "a",
            played_hand
        );

        assert!(new_round.is_ok());
    }

}
