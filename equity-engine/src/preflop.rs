/// Preflop opening chart for 6-max Texas Hold'em.
///
/// Provides fast O(1) preflop action decisions based on:
///   - hero cards (hand category from 169 canonical hands)
///   - position (BTN/CO/MP/UTG/SB/BB)
///   - player_count (active players, affects tightening)
///   - current_bet (0 = first to act / open, >0 = facing a raise)
///
/// This bypasses Monte Carlo simulation for preflop decisions where
/// position-based opening charts are more accurate than equity alone.
use crate::{Card, Position, Rank};

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Preflop action recommendation from the opening chart.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreflopDecision {
    /// Open raise (first to act)
    Open,
    /// 3-bet (facing a single raise)
    ThreeBet,
    /// Call a single raise (float / set-mine / speculative)
    Call,
    /// Fold
    Fold,
}

#[derive(Debug, Clone)]
pub struct PreflopAction {
    pub decision: PreflopDecision,
    pub reasoning: String,
}

// ---------------------------------------------------------------------------
// Hand category — 169 canonical hands ranked by strength
// ---------------------------------------------------------------------------

/// A canonical hand category: pair / suited / offsuit, with rank(s).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandCategory {
    Pair(Rank),
    Suited(Rank, Rank),   // higher rank first
    Offsuit(Rank, Rank),  // higher rank first
}

/// Classify two cards into one of 169 canonical hand categories.
pub fn classify(c1: &Card, c2: &Card) -> HandCategory {
    let (hi, lo) = if rank_val(c1.rank) >= rank_val(c2.rank) {
        (c1, c2)
    } else {
        (c2, c1)
    };
    if hi.rank == lo.rank {
        HandCategory::Pair(hi.rank)
    } else if hi.suit == lo.suit {
        HandCategory::Suited(hi.rank, lo.rank)
    } else {
        HandCategory::Offsuit(hi.rank, lo.rank)
    }
}

/// Numeric value for rank (2=2 … A=14)
fn rank_val(r: Rank) -> u8 {
    match r {
        Rank::Two => 2,   Rank::Three => 3, Rank::Four => 4,
        Rank::Five => 5,  Rank::Six => 6,   Rank::Seven => 7,
        Rank::Eight => 8, Rank::Nine => 9,  Rank::Ten => 10,
        Rank::Jack => 11, Rank::Queen => 12, Rank::King => 13,
        Rank::Ace => 14,
    }
}

/// Hand strength percentile (0–100, higher = stronger).
/// Based on standard 6-max preflop hand ranking of all 169 categories.
/// Pairs > suited Broadway > suited connectors > offsuit Broadway > etc.
pub fn hand_strength_percentile(h: HandCategory) -> u8 {
    match h {
        // --- Premium pairs (top 2%) ---
        HandCategory::Pair(Rank::Ace)   => 100,
        HandCategory::Pair(Rank::King)  => 99,
        HandCategory::Pair(Rank::Queen) => 98,
        HandCategory::Pair(Rank::Jack)  => 97,

        // --- Strong suited Broadway (top 3–6%) ---
        HandCategory::Suited(Rank::Ace, Rank::King)   => 96,
        HandCategory::Suited(Rank::Ace, Rank::Queen)  => 95,
        HandCategory::Suited(Rank::Ace, Rank::Jack)   => 94,
        HandCategory::Suited(Rank::King, Rank::Queen) => 93,

        // --- Pairs TT–77 (top 7–12%) ---
        HandCategory::Pair(Rank::Ten)   => 92,
        HandCategory::Pair(Rank::Nine)  => 90,
        HandCategory::Pair(Rank::Eight) => 88,
        HandCategory::Pair(Rank::Seven) => 86,

        // --- Strong suited/offsuit (top 8–15%) ---
        HandCategory::Suited(Rank::Ace, Rank::Ten)    => 91,
        HandCategory::Offsuit(Rank::Ace, Rank::King)  => 89,
        HandCategory::Offsuit(Rank::Ace, Rank::Queen) => 87,
        HandCategory::Suited(Rank::King, Rank::Jack)  => 85,
        HandCategory::Suited(Rank::King, Rank::Ten)   => 84,
        HandCategory::Offsuit(Rank::Ace, Rank::Jack)  => 83,
        HandCategory::Suited(Rank::Queen, Rank::Jack) => 82,
        HandCategory::Offsuit(Rank::King, Rank::Queen)=> 81,
        HandCategory::Suited(Rank::Queen, Rank::Ten)  => 80,
        HandCategory::Suited(Rank::Jack, Rank::Ten)   => 79,

        // --- Medium pairs (top 13–18%) ---
        HandCategory::Pair(Rank::Six)  => 78,
        HandCategory::Pair(Rank::Five) => 76,
        HandCategory::Pair(Rank::Four) => 74,
        HandCategory::Pair(Rank::Three)=> 72,
        HandCategory::Pair(Rank::Two)  => 70,

        // --- Suited Aces, suited connectors (top 18–35%) ---
        HandCategory::Suited(Rank::Ace, Rank::Nine)  => 75,
        HandCategory::Suited(Rank::Ace, Rank::Eight) => 73,
        HandCategory::Suited(Rank::Ace, Rank::Seven) => 71,
        HandCategory::Suited(Rank::Ace, Rank::Six)   => 69,
        HandCategory::Suited(Rank::Ace, Rank::Five)  => 68,
        HandCategory::Suited(Rank::Ace, Rank::Four)  => 67,
        HandCategory::Suited(Rank::Ace, Rank::Three) => 66,
        HandCategory::Suited(Rank::Ace, Rank::Two)   => 65,
        HandCategory::Offsuit(Rank::Ace, Rank::Ten)  => 77,
        HandCategory::Offsuit(Rank::King, Rank::Jack) => 64,
        HandCategory::Offsuit(Rank::King, Rank::Ten)  => 63,
        HandCategory::Suited(Rank::King, Rank::Nine)  => 62,
        HandCategory::Offsuit(Rank::Queen, Rank::Jack)=> 61,
        HandCategory::Suited(Rank::Ten, Rank::Nine)   => 60,
        HandCategory::Offsuit(Rank::Queen, Rank::Ten) => 59,
        HandCategory::Suited(Rank::Nine, Rank::Eight) => 58,
        HandCategory::Offsuit(Rank::Jack, Rank::Ten)  => 57,
        HandCategory::Suited(Rank::Eight, Rank::Seven)=> 56,
        HandCategory::Suited(Rank::Jack, Rank::Nine)  => 55,
        HandCategory::Suited(Rank::Seven, Rank::Six)  => 54,
        HandCategory::Suited(Rank::King, Rank::Eight) => 53,
        HandCategory::Suited(Rank::Queen, Rank::Nine) => 52,
        HandCategory::Suited(Rank::Six, Rank::Five)   => 51,
        HandCategory::Suited(Rank::Five, Rank::Four)  => 50,
        HandCategory::Offsuit(Rank::Ace, Rank::Nine)  => 49,
        HandCategory::Suited(Rank::Ten, Rank::Eight)  => 48,
        HandCategory::Offsuit(Rank::King, Rank::Nine)  => 47,
        HandCategory::Suited(Rank::Nine, Rank::Seven) => 46,
        HandCategory::Suited(Rank::Eight, Rank::Six)  => 45,
        HandCategory::Suited(Rank::Jack, Rank::Eight) => 44,
        HandCategory::Suited(Rank::Seven, Rank::Five) => 43,
        HandCategory::Offsuit(Rank::Ace, Rank::Eight) => 42,
        HandCategory::Offsuit(Rank::Queen, Rank::Nine)=> 41,
        HandCategory::Suited(Rank::Four, Rank::Three) => 40,
        HandCategory::Offsuit(Rank::Ace, Rank::Seven) => 39,
        HandCategory::Suited(Rank::Six, Rank::Four)   => 38,
        HandCategory::Suited(Rank::Five, Rank::Three) => 37,
        HandCategory::Offsuit(Rank::Jack, Rank::Nine) => 36,
        HandCategory::Offsuit(Rank::Ace, Rank::Six)   => 35,
        HandCategory::Suited(Rank::Three, Rank::Two)  => 34,
        HandCategory::Offsuit(Rank::Ten, Rank::Nine)  => 33,
        HandCategory::Offsuit(Rank::Ace, Rank::Five)  => 32,
        HandCategory::Offsuit(Rank::King, Rank::Eight)=> 31,
        HandCategory::Suited(Rank::King, Rank::Seven) => 30,
        HandCategory::Offsuit(Rank::Ace, Rank::Four)  => 29,
        HandCategory::Suited(Rank::Nine, Rank::Six)   => 28,
        HandCategory::Offsuit(Rank::Jack, Rank::Eight)=> 27,
        HandCategory::Offsuit(Rank::Nine, Rank::Eight)=> 26,
        HandCategory::Offsuit(Rank::Ace, Rank::Three) => 25,
        HandCategory::Suited(Rank::King, Rank::Six)   => 24,
        HandCategory::Suited(Rank::Eight, Rank::Five) => 23,
        HandCategory::Offsuit(Rank::Ace, Rank::Two)   => 22,
        HandCategory::Offsuit(Rank::Queen, Rank::Eight)=>21,
        HandCategory::Offsuit(Rank::King, Rank::Seven)=> 20,
        HandCategory::Offsuit(Rank::Ten, Rank::Eight) => 19,
        HandCategory::Suited(Rank::Seven, Rank::Four) => 18,
        HandCategory::Offsuit(Rank::Nine, Rank::Seven)=> 17,
        HandCategory::Suited(Rank::Six, Rank::Three)  => 16,
        HandCategory::Offsuit(Rank::Eight, Rank::Seven)=>15,
        HandCategory::Suited(Rank::Five, Rank::Two)   => 14,
        HandCategory::Suited(Rank::King, Rank::Five)  => 13,
        HandCategory::Offsuit(Rank::Jack, Rank::Seven)=> 12,
        HandCategory::Offsuit(Rank::Ten, Rank::Seven) => 11,
        // Everything else — bottom of range
        _ => 5,
    }
}

// ---------------------------------------------------------------------------
// Opening ranges per position (percentile thresholds)
// ---------------------------------------------------------------------------

/// Thresholds: (open_min_pct, threebet_min_pct)
/// Hands at or above open_min_pct → open (first to act)
/// Hands at or above threebet_min_pct → 3-bet (facing a raise)
fn position_thresholds(pos: Position) -> (u8, u8) {
    match pos {
        Position::BTN => (55, 92),  // open top 45%, 3bet top 8%
        Position::CO  => (65, 94),  // open top 35%, 3bet top 6%
        Position::MP  => (78, 95),  // open top 22%, 3bet top 5%
        Position::UTG => (85, 96),  // open top 15%, 3bet top 4%
        Position::SB  => (60, 93),  // open top 40%, 3bet top 7%
        Position::BB  => (45, 94),  // defend top 55% vs open, 3bet top 6%
        // 7-9 handed positions: map to closest 6-max equivalent
        Position::HJ  => (75, 95),
        Position::LJ  => (80, 96),
        Position::UTG1 | Position::UTG2 => (87, 97),
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Look up the preflop action for the given cards, position, and context.
///
/// `current_bet > 0` means hero is facing a raise (3-bet decision).
/// `current_bet == 0` means hero is first to act (open decision).
pub fn preflop_action(
    c1: &Card,
    c2: &Card,
    position: Position,
    _player_count: u8,
    current_bet: f64,
) -> PreflopAction {
    let hand = classify(c1, c2);
    let pct = hand_strength_percentile(hand);
    let (open_thresh, threebet_thresh) = position_thresholds(position);
    let hand_desc = hand_description(&hand);
    let pos_name = position_name(position);

    if current_bet > 0.0 {
        // Facing a raise — 3-bet or call or fold
        if pct >= threebet_thresh {
            PreflopAction {
                decision: PreflopDecision::ThreeBet,
                reasoning: format!(
                    "{} — {pos_name} 3-bet range (top {}%). Strength justifies 3-bet.",
                    hand_desc,
                    100 - threebet_thresh
                ),
            }
        } else if pct >= open_thresh {
            // In range but not 3-bet quality — call
            PreflopAction {
                decision: PreflopDecision::Call,
                reasoning: format!(
                    "{} — {pos_name} call range. Not strong enough to 3-bet, but worth calling.",
                    hand_desc
                ),
            }
        } else {
            PreflopAction {
                decision: PreflopDecision::Fold,
                reasoning: format!(
                    "{} — below {pos_name} call range (pct={}). Fold facing raise.",
                    hand_desc, pct
                ),
            }
        }
    } else {
        // First to act — open or fold
        if pct >= open_thresh {
            PreflopAction {
                decision: PreflopDecision::Open,
                reasoning: format!(
                    "{} — {pos_name} opening range (top {}%). Standard open.",
                    hand_desc,
                    100 - open_thresh
                ),
            }
        } else {
            PreflopAction {
                decision: PreflopDecision::Fold,
                reasoning: format!(
                    "{} — below {pos_name} opening range (pct={}). Fold.",
                    hand_desc, pct
                ),
            }
        }
    }
}

/// Preflop action with opponent reads.
///
/// Adjustments from opponent stats:
/// - vs Nit 3-bet: fold more (threebet_thresh raised by 5 percentile points)
/// - vs Fish 3-bet: call/4bet wider (threebet_thresh lowered by 5 points)
/// - vs low fold-to-3bet (<30%): 3-bet less, flat more
pub fn preflop_action_with_reads(
    c1: &Card,
    c2: &Card,
    position: Position,
    player_count: u8,
    current_bet: f64,
    opponent: Option<&crate::OpponentStats>,
) -> PreflopAction {
    let base = preflop_action(c1, c2, position, player_count, current_bet);

    let opp = match opponent {
        Some(o) if o.hands_seen >= 20 => o,
        _ => return base, // insufficient data → use base
    };

    let arch = opp.archetype();
    // Only adjust when facing a raise — opening decisions unchanged
    if current_bet <= 0.0 { return base; }

    let hand = classify(c1, c2);
    let pct = hand_strength_percentile(hand);
    let (open_thresh, threebet_thresh) = position_thresholds(position);
    let hand_desc = hand_description(&hand);
    let pos_name = position_name(position);

    // Adjust 3-bet threshold based on opponent archetype
    let adj_3bet: i32 = match arch {
        crate::PlayerArchetype::Nit => 5,   // tighten: fold more vs nit
        crate::PlayerArchetype::Fish
        | crate::PlayerArchetype::CallingStation => -5, // loosen: widen vs fish
        _ => 0,
    };

    // Fold-to-3bet adjustment: if opp rarely folds to 3bets, flat more
    let fold_to_3bet_pct = if opp.hands_seen > 0 {
        opp.fold_to_3bet as f64 / opp.hands_seen as f64
    } else { 0.5 };
    let flat_bias = fold_to_3bet_pct < 0.30; // opp rarely folds → reduce 3bets

    let adj_thresh = (threebet_thresh as i32 + adj_3bet).clamp(0, 99) as u8;

    if pct >= adj_thresh && !flat_bias {
        PreflopAction {
            decision: PreflopDecision::ThreeBet,
            reasoning: format!(
                "{} — {pos_name} 3-bet vs {} (reads-adjusted, thresh={}%).",
                hand_desc, pos_name, 100 - adj_thresh
            ),
        }
    } else if pct >= open_thresh {
        let note = if flat_bias { " (3bet-less vs low fold-to-3bet)" } else { "" };
        PreflopAction {
            decision: PreflopDecision::Call,
            reasoning: format!("{} — {pos_name} call range{}.", hand_desc, note),
        }
    } else {
        base // hand not in range at all — base fold applies
    }
}

pub fn hand_description(h: &HandCategory) -> String {
    match h {
        HandCategory::Pair(r) => format!("{}{}",  rank_char(*r), rank_char(*r)),
        HandCategory::Suited(hi, lo) => format!("{}{}s", rank_char(*hi), rank_char(*lo)),
        HandCategory::Offsuit(hi, lo) => format!("{}{}o", rank_char(*hi), rank_char(*lo)),
    }
}

pub fn rank_char(r: Rank) -> char {
    match r {
        Rank::Two => '2', Rank::Three => '3', Rank::Four => '4',
        Rank::Five => '5', Rank::Six => '6', Rank::Seven => '7',
        Rank::Eight => '8', Rank::Nine => '9', Rank::Ten => 'T',
        Rank::Jack => 'J', Rank::Queen => 'Q', Rank::King => 'K',
        Rank::Ace => 'A',
    }
}

fn position_name(p: Position) -> &'static str {
    match p {
        Position::BTN => "BTN",
        Position::CO  => "CO",
        Position::MP  => "MP",
        Position::UTG => "UTG",
        Position::SB  => "SB",
        Position::BB  => "BB",
        Position::HJ  => "HJ",
        Position::LJ  => "LJ",
        Position::UTG1 => "UTG+1",
        Position::UTG2 => "UTG+2",
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Rank, Suit};

    fn card(rank: Rank, suit: Suit) -> Card {
        Card { rank, suit }
    }

    // --- Hand classification ---

    #[test]
    fn classify_pair() {
        let h = classify(&card(Rank::Ace, Suit::Hearts), &card(Rank::Ace, Suit::Spades));
        assert_eq!(h, HandCategory::Pair(Rank::Ace));
    }

    #[test]
    fn classify_suited() {
        let h = classify(&card(Rank::King, Suit::Hearts), &card(Rank::Queen, Suit::Hearts));
        assert_eq!(h, HandCategory::Suited(Rank::King, Rank::Queen));
    }

    #[test]
    fn classify_offsuit_orders_high_first() {
        let h = classify(&card(Rank::Two, Suit::Hearts), &card(Rank::Seven, Suit::Clubs));
        assert_eq!(h, HandCategory::Offsuit(Rank::Seven, Rank::Two));
    }

    // --- Opening decisions ---

    #[test]
    fn btn_opens_ak_offsuit() {
        let action = preflop_action(
            &card(Rank::Ace, Suit::Hearts),
            &card(Rank::King, Suit::Clubs),
            Position::BTN, 6, 0.0,
        );
        assert_eq!(action.decision, PreflopDecision::Open,
            "AKo should open from BTN, got: {:?} — {}", action.decision, action.reasoning);
    }

    #[test]
    fn utg_folds_72o() {
        let action = preflop_action(
            &card(Rank::Seven, Suit::Hearts),
            &card(Rank::Two, Suit::Clubs),
            Position::UTG, 6, 0.0,
        );
        assert_eq!(action.decision, PreflopDecision::Fold,
            "72o should fold from UTG, got: {:?} — {}", action.decision, action.reasoning);
    }

    #[test]
    fn btn_threebets_qq_facing_raise() {
        let action = preflop_action(
            &card(Rank::Queen, Suit::Hearts),
            &card(Rank::Queen, Suit::Clubs),
            Position::BTN, 6, 8.0, // facing a raise
        );
        // QQ is top 2% — well above BTN 3-bet threshold (top 8%)
        assert_eq!(action.decision, PreflopDecision::ThreeBet,
            "QQ should 3-bet from BTN, got: {:?} — {}", action.decision, action.reasoning);
    }

    #[test]
    fn utg_opens_aa() {
        let action = preflop_action(
            &card(Rank::Ace, Suit::Hearts),
            &card(Rank::Ace, Suit::Clubs),
            Position::UTG, 6, 0.0,
        );
        assert_eq!(action.decision, PreflopDecision::Open,
            "AA should open from UTG");
    }

    #[test]
    fn co_folds_weak_hand_first_to_act() {
        // 72o — percentile 5, well below CO open threshold (65)
        let action = preflop_action(
            &card(Rank::Seven, Suit::Hearts),
            &card(Rank::Two, Suit::Diamonds),
            Position::CO, 6, 0.0,
        );
        assert_eq!(action.decision, PreflopDecision::Fold);
    }

    #[test]
    fn bb_defends_medium_hand_facing_open() {
        // T9s — percentile 60, BB defend threshold 45 → defend (Call)
        let action = preflop_action(
            &card(Rank::Ten, Suit::Hearts),
            &card(Rank::Nine, Suit::Hearts),
            Position::BB, 6, 4.0, // facing an open
        );
        // pct=60, bb call thresh=45 → Call or 3bet
        assert!(
            action.decision == PreflopDecision::Call || action.decision == PreflopDecision::ThreeBet,
            "T9s should defend from BB, got: {:?}", action.decision
        );
    }

    #[test]
    fn reasoning_contains_position_and_hand() {
        let action = preflop_action(
            &card(Rank::Ace, Suit::Hearts),
            &card(Rank::King, Suit::Hearts),
            Position::BTN, 6, 0.0,
        );
        assert!(action.reasoning.contains("BTN"), "reasoning should mention BTN");
        assert!(action.reasoning.contains("AK"), "reasoning should mention hand");
    }

    // -----------------------------------------------------------------------
    // Task: preflop chart tests
    // -----------------------------------------------------------------------

    /// Task 4b-i: BTN + AKs → Open (not fold)
    #[test]
    fn btn_aks_opens() {
        let action = preflop_action(
            &card(Rank::Ace, Suit::Diamonds),
            &card(Rank::King, Suit::Diamonds), // suited
            Position::BTN, 6, 0.0,
        );
        assert_eq!(
            action.decision, PreflopDecision::Open,
            "AKs should open from BTN, got: {:?} — {}", action.decision, action.reasoning
        );
    }

    /// Task 4b-ii: UTG + 72o → Fold (confirmed — also verified by existing test)
    #[test]
    fn utg_72o_folds_first_to_act() {
        let action = preflop_action(
            &card(Rank::Seven, Suit::Spades),
            &card(Rank::Two, Suit::Hearts),
            Position::UTG, 6, 0.0,
        );
        assert_eq!(
            action.decision, PreflopDecision::Fold,
            "72o must fold from UTG, got: {:?} — {}", action.decision, action.reasoning
        );
    }

    /// Task 4b-iii: BB + QQ facing open → 3bet or call, NOT fold
    ///
    /// QQ is top 2% (percentile=98). BB threebet_thresh=94.
    /// 98 >= 94 → ThreeBet from BB.
    #[test]
    fn bb_qq_facing_open_threebets_not_folds() {
        let action = preflop_action(
            &card(Rank::Queen, Suit::Hearts),
            &card(Rank::Queen, Suit::Clubs),
            Position::BB, 6, 4.0, // facing an open raise
        );
        assert_ne!(
            action.decision, PreflopDecision::Fold,
            "QQ from BB facing open must not fold, got: {:?} — {}", action.decision, action.reasoning
        );
        assert!(
            action.decision == PreflopDecision::ThreeBet || action.decision == PreflopDecision::Call,
            "QQ from BB should 3bet or call, got: {:?} — {}", action.decision, action.reasoning
        );
    }

    /// Task 4b-iv: Preflop reasoning string differs from postflop reasoning string
    ///
    /// Preflop uses the opening chart (PreflopAction.reasoning).
    /// Postflop uses make_suggestion (equity/pot-odds/SPR reasoning).
    /// These should not contain the same content — preflop should mention hand/position,
    /// postflop should mention equity percentages.
    #[test]
    fn preflop_reasoning_differs_from_postflop_reasoning() {
        use crate::{Card as C, GameState, Street, Suit as S};
        use crate::suggestion::make_suggestion;

        // Preflop chart reasoning
        let preflop_action = preflop_action(
            &card(Rank::Ace, Suit::Hearts),
            &card(Rank::King, Suit::Spades),
            Position::BTN, 6, 0.0,
        );
        let preflop_reason = &preflop_action.reasoning;

        // Postflop make_suggestion reasoning
        let state = GameState {
            street: Street::Flop,
            hero_cards: [
                C { rank: Rank::Ace, suit: S::Hearts },
                C { rank: Rank::King, suit: S::Spades },
            ],
            board_cards: vec![],
            pot_size: 100.0,
            current_bet: 40.0,
            hero_stack: 1000.0,
            villain_stack: 1000.0,
            position: Position::BTN,
            to_act: true,
            player_count: 2,
            context: Default::default(),
        
        opponent_stats: vec![],
        table_id: None,
            big_blind_size: 1.0,
            villain_stacks: std::collections::HashMap::new(),
        dealer_seat: 0,
        active_bet_size: None,
        hero_to_act: false,
            strategy_ev: None,
        strategy_pot_odds: None,
        strategy_betsize_bb: None,
        strategy_betsize_pct_pot: None,
    };
        let (_, _, postflop_reason) = make_suggestion(0.65, 0.286, &state);
        let postflop_reason: String = postflop_reason;

        // Preflop reasoning: mentions hand name + position + "opening range" or similar
        assert!(
            preflop_reason.contains("BTN") || preflop_reason.contains("AK"),
            "preflop reasoning should mention position/hand, got: {preflop_reason}"
        );
        // Postflop reasoning: contains equity percentage (new format: numbers in brackets)
        assert!(
            postflop_reason.contains('%'),
            "postflop reasoning should contain %, got: {postflop_reason}"
        );
        // They should not be identical
        assert_ne!(
            preflop_reason, &postflop_reason,
            "preflop and postflop reasoning must differ"
        );
    }
}
