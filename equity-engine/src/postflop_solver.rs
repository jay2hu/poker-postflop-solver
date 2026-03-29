/// Postflop decision recommendation engine.
///
/// Combines equity computation, pot odds, SPR, and street-specific logic
/// to produce actionable recommendations.

use crate::{Card, Position, Street};
use crate::range::Range;
use crate::postflop::{hand_vs_range_equity, BoardTexture};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Recommended postflop action.
#[derive(Debug, Clone, PartialEq)]
pub enum PostflopAction {
    Fold,
    Check,
    Call,
    /// Bet size in big blinds
    Bet(f64),
    /// Raise to size in big blinds
    Raise(f64),
}

/// Full postflop decision recommendation.
#[derive(Debug, Clone)]
pub struct PostflopDecision {
    pub action: PostflopAction,
    /// Bet/raise size in bb (None for check/call/fold)
    pub sizing_bb: Option<f64>,
    /// Bet/raise size as fraction of pot (None for check/call/fold)
    pub sizing_pct_pot: Option<f64>,
    /// Hero's equity vs villain range (0.0–1.0)
    pub equity: f64,
    /// Pot odds to call (0.0–1.0), 0.0 if first to act
    pub pot_odds: f64,
    /// Estimated EV of recommended action (bb units, rough estimate)
    pub ev_estimate: f64,
    /// Human-readable reasoning
    pub reasoning: String,
}

// ---------------------------------------------------------------------------
// Core recommendation
// ---------------------------------------------------------------------------

/// Produce a postflop recommendation given the current game state.
///
/// # Arguments
/// - `hero_hand`          — Hero's two hole cards
/// - `hero_position`      — Hero's seat position
/// - `villain_range`      — Estimated villain range (call `Range::from_preflop_action` or custom)
/// - `board`              — Community cards (3–5)
/// - `pot_bb`             — Current pot in big blinds
/// - `hero_stack_bb`      — Hero's effective stack in big blinds
/// - `villain_stack_bb`   — Villain's effective stack in big blinds
/// - `to_call_bb`         — Outstanding bet hero must call (0 if first to act)
/// - `street`             — Current street
pub fn recommend(
    hero_hand: (Card, Card),
    hero_position: Position,
    villain_range: &Range,
    board: &[Card],
    pot_bb: f64,
    hero_stack_bb: f64,
    villain_stack_bb: f64,
    to_call_bb: f64,
    street: Street,
) -> PostflopDecision {
    let eff_stack = hero_stack_bb.min(villain_stack_bb);
    let spr = if pot_bb > 0.0 { eff_stack / pot_bb } else { 99.0 };

    // Compute hero equity vs villain range
    let iterations = match street {
        Street::Preflop => 1000,
        Street::Flop    => 3000,
        Street::Turn    => 5000,
        Street::River   => 7000,
    };
    let equity = hand_vs_range_equity(hero_hand, villain_range, board, iterations);
    let texture = BoardTexture::analyze(board);

    if to_call_bb > 0.0 {
        facing_bet(equity, spr, pot_bb, to_call_bb, eff_stack, texture, street)
    } else {
        first_to_act(equity, spr, pot_bb, eff_stack, hero_position, texture, street)
    }
}

// ---------------------------------------------------------------------------
// Facing a bet
// ---------------------------------------------------------------------------

fn facing_bet(
    equity: f64,
    spr: f64,
    pot_bb: f64,
    to_call_bb: f64,
    eff_stack: f64,
    texture: BoardTexture,
    _street: Street,
) -> PostflopDecision {
    let pot_odds = to_call_bb / (pot_bb + to_call_bb);

    // SPR adjustments: short stack polarizes thresholds
    let (fold_threshold, raise_threshold) = if spr < 3.0 {
        // Short stack: all-in or fold — tighter thresholds
        (pot_odds * 0.90, pot_odds * 1.15)
    } else {
        (pot_odds * 0.85, pot_odds * 1.30)
    };

    if equity < fold_threshold {
        let reason = format!(
            "Equity {:.1}% below pot odds {:.1}% (threshold {:.1}%). Fold.",
            equity * 100.0, pot_odds * 100.0, fold_threshold * 100.0
        );
        PostflopDecision {
            action: PostflopAction::Fold,
            sizing_bb: None,
            sizing_pct_pot: None,
            equity,
            pot_odds,
            ev_estimate: -(to_call_bb * fold_threshold),
            reasoning: reason,
        }
    } else if equity > raise_threshold {
        // Raise to 2.5× the bet (capped at eff stack)
        let raise_size = (to_call_bb * 2.5).min(eff_stack);
        let sizing_pct = raise_size / (pot_bb + to_call_bb);
        let reason = format!(
            "Equity {:.1}% well above pot odds {:.1}%. Raise to {:.1}bb ({:.0}% pot).",
            equity * 100.0, pot_odds * 100.0, raise_size, sizing_pct * 100.0
        );
        let ev = (pot_bb + to_call_bb) * (equity - pot_odds);
        PostflopDecision {
            action: PostflopAction::Raise(raise_size),
            sizing_bb: Some(raise_size),
            sizing_pct_pot: Some(sizing_pct),
            equity,
            pot_odds,
            ev_estimate: ev,
            reasoning: reason,
        }
    } else {
        // Call
        let ev = pot_bb * equity - to_call_bb * (1.0 - equity);
        let texture_note = if texture.flush_draw_possible || texture.straight_draw_possible {
            " Draw board — implied odds favour call."
        } else {
            ""
        };
        let reason = format!(
            "Equity {:.1}% vs pot odds {:.1}%.{} Call.",
            equity * 100.0, pot_odds * 100.0, texture_note
        );
        PostflopDecision {
            action: PostflopAction::Call,
            sizing_bb: None,
            sizing_pct_pot: None,
            equity,
            pot_odds,
            ev_estimate: ev,
            reasoning: reason,
        }
    }
}

// ---------------------------------------------------------------------------
// First to act (no outstanding bet)
// ---------------------------------------------------------------------------

fn first_to_act(
    equity: f64,
    spr: f64,
    pot_bb: f64,
    eff_stack: f64,
    _position: Position,
    texture: BoardTexture,
    street: Street,
) -> PostflopDecision {
    // Bet sizing as fraction of pot
    let (bet_pct, threshold) = if spr < 3.0 {
        // Short stack: shove or check
        (1.0_f64, 0.60)
    } else if texture.is_monotone {
        (0.5, 0.62)   // smaller bet on monotone (polarised)
    } else if texture.is_dry() {
        (0.33, 0.52)  // block bet on dry boards
    } else {
        // Standard: 65% pot for value, 33% for thin value
        (0.65, 0.52)
    };

    if equity > 0.65 {
        // Strong hand → value bet
        let size = (pot_bb * if spr < 3.0 { 1.0 } else { 0.65 }).min(eff_stack);
        let sizing_pct = size / pot_bb;
        let ev = pot_bb * (equity - 0.5) + size * equity;
        let reason = format!(
            "Equity {:.1}% — strong hand{}.{}",
            equity * 100.0,
            if texture.is_dry() { " on dry board" } else { "" },
            format!(" Value bet {:.1}bb ({:.0}% pot).", size, sizing_pct * 100.0)
        );
        PostflopDecision {
            action: PostflopAction::Bet(size),
            sizing_bb: Some(size),
            sizing_pct_pot: Some(sizing_pct),
            equity,
            pot_odds: 0.0,
            ev_estimate: ev,
            reasoning: reason,
        }
    } else if equity > threshold {
        // Thin value / block bet
        let size = (pot_bb * bet_pct).min(eff_stack);
        let sizing_pct = size / pot_bb;
        let ev = pot_bb * (equity - 0.5) + size * equity;
        let reason = format!(
            "Equity {:.1}% — thin value/block bet {:.1}bb ({:.0}% pot).",
            equity * 100.0, size, sizing_pct * 100.0
        );
        PostflopDecision {
            action: PostflopAction::Bet(size),
            sizing_bb: Some(size),
            sizing_pct_pot: Some(sizing_pct),
            equity,
            pot_odds: 0.0,
            ev_estimate: ev,
            reasoning: reason,
        }
    } else {
        // Check
        let texture_note = if texture.flush_draw_possible || texture.straight_draw_possible {
            " Draws possible — check is safe."
        } else {
            ""
        };
        let reason = format!(
            "Equity {:.1}% — not strong enough to bet.{} Check.",
            equity * 100.0, texture_note
        );
        PostflopDecision {
            action: PostflopAction::Check,
            sizing_bb: None,
            sizing_pct_pot: None,
            equity,
            pot_odds: 0.0,
            ev_estimate: 0.0,
            reasoning: reason,
        }
    }
}

// Convenience
impl BoardTexture {
    fn is_dry(&self) -> bool {
        !self.flush_draw_possible && !self.straight_draw_possible && !self.is_paired
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Card, Position, Rank, Street, Suit};
    use crate::range::Range;

    fn card(rank: Rank, suit: Suit) -> Card { Card { rank, suit } }

    fn dry_board() -> Vec<Card> {
        vec![
            card(Rank::King, Suit::Diamonds),
            card(Rank::Seven, Suit::Clubs),
            card(Rank::Two, Suit::Hearts),
        ]
    }

    #[test]
    fn recommend_strong_hand_first_to_act_bets() {
        // Top set IP, first to act → should bet
        let hero = (card(Rank::King, Suit::Hearts), card(Rank::King, Suit::Spades));
        let villain = Range::from_str("AKs,AQs,AJs,KQs,QJs,JTs,TTs,99s,88s").expect("range");
        let decision = recommend(
            hero, Position::BTN, &villain, &dry_board(),
            10.0, 100.0, 100.0, 0.0, Street::Flop,
        );
        assert!(
            matches!(decision.action, PostflopAction::Bet(_)),
            "KK on K72 dry board first to act should Bet, got {:?}", decision.action
        );
        assert!(decision.equity > 0.70, "KK on K72 should have >70% equity");
    }

    #[test]
    fn recommend_weak_hand_facing_large_bet_folds() {
        // 72o facing a pot-sized bet on AKQ board → fold
        let hero = (card(Rank::Seven, Suit::Hearts), card(Rank::Two, Suit::Clubs));
        let villain = Range::from_str("AA,KK,QQ,AKs,AKo,AQs,KQs").expect("range");
        let board = vec![
            card(Rank::Ace, Suit::Diamonds),
            card(Rank::King, Suit::Clubs),
            card(Rank::Queen, Suit::Hearts),
        ];
        let decision = recommend(
            hero, Position::BB, &villain, &board,
            10.0, 100.0, 100.0, 10.0, // facing pot-sized bet
            Street::Flop,
        );
        assert_eq!(
            decision.action, PostflopAction::Fold,
            "72o facing PSB on AKQ vs strong range should Fold, got {:?}", decision.action
        );
        assert!(decision.equity < 0.10);
    }

    #[test]
    fn recommend_medium_hand_facing_small_bet_calls() {
        // Middle pair facing a small bet (not strong enough to raise but above pot odds)
        let hero = (card(Rank::Seven, Suit::Hearts), card(Rank::Seven, Suit::Diamonds));
        let villain = Range::from_str("AKo,AQo,AKs,AQs,KQs,TT,99,88,77,66").expect("range");
        let board = vec![
            card(Rank::King, Suit::Diamonds),
            card(Rank::Seven, Suit::Clubs),
            card(Rank::Two, Suit::Hearts),
        ];
        // Facing a small bet (1/3 pot)
        let decision = recommend(
            hero, Position::BB, &villain, &board,
            10.0, 100.0, 100.0, 3.3, Street::Flop,
        );
        // 77 has middle set, should call or raise (not fold)
        assert!(
            !matches!(decision.action, PostflopAction::Fold),
            "77 with set on K72 facing small bet should not Fold, got {:?}", decision.action
        );
    }

    #[test]
    fn recommend_check_with_weak_equity() {
        // Very weak hand IP on a scary board, no bet facing → check
        let hero = (card(Rank::Three, Suit::Hearts), card(Rank::Four, Suit::Clubs));
        let villain = Range::from_str("AA,KK,AKs,AQs").expect("range");
        let board = vec![
            card(Rank::Ace, Suit::Spades),
            card(Rank::Ace, Suit::Hearts),
            card(Rank::King, Suit::Diamonds),
        ];
        let decision = recommend(
            hero, Position::BTN, &villain, &board,
            10.0, 100.0, 100.0, 0.0, Street::Flop,
        );
        assert_eq!(
            decision.action, PostflopAction::Check,
            "34o on AAK vs AA/KK range should Check, got {:?}", decision.action
        );
    }

    #[test]
    fn recommend_spr_short_stack_polarizes() {
        // Short stack SPR < 3 — raise threshold tightens
        let hero = (card(Rank::Ace, Suit::Hearts), card(Rank::King, Suit::Diamonds));
        let villain = Range::from_str("QQ,JJ,TT,AQs,AJs").expect("range");
        let board = vec![
            card(Rank::Ace, Suit::Clubs),
            card(Rank::Seven, Suit::Hearts),
            card(Rank::Two, Suit::Spades),
        ];
        // SPR = 6/10 = 0.6 (very short)
        let decision = recommend(
            hero, Position::BTN, &villain, &board,
            10.0, 6.0, 6.0, 0.0, Street::Flop,
        );
        // Top pair top kicker with short stack should still bet/shove
        assert!(
            !matches!(decision.action, PostflopAction::Check),
            "AK with TPTK short-stacked should bet, got {:?}", decision.action
        );
    }

    #[test]
    fn postflop_decision_has_reasoning() {
        let hero = (card(Rank::Ace, Suit::Hearts), card(Rank::King, Suit::Diamonds));
        let villain = Range::from_str("AA,KK,QQ").expect("range");
        let board = vec![
            card(Rank::King, Suit::Clubs),
            card(Rank::Seven, Suit::Diamonds),
            card(Rank::Two, Suit::Hearts),
        ];
        let d = recommend(hero, Position::BTN, &villain, &board,
                          10.0, 100.0, 100.0, 0.0, Street::Flop);
        assert!(!d.reasoning.is_empty(), "reasoning should not be empty");
        assert!(d.equity > 0.0 && d.equity < 1.0, "equity should be in (0,1)");
    }
}
