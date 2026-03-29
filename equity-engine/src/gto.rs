use crate::{GameState, Recommendation, Suggestion};

/// GTO approximation hints derived from equity margin and bet sizing.
///
/// These are heuristic approximations — not a solver — but give actionable
/// mixed-strategy guidance that aligns with GTO principles.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GtoHints {
    /// Minimum Defense Frequency: how often hero must defend vs this bet to deny
    /// villain a profitable pure bluff. `pot / (pot + bet)`. Zero when no bet.
    pub mdf: f64,

    /// Approximate raise frequency in this spot (0.0–1.0).
    pub raise_freq: f64,
    /// Approximate call frequency (0.0–1.0).
    pub call_freq: f64,
    /// Approximate fold frequency (0.0–1.0). raise + call + fold == 1.0.
    pub fold_freq: f64,

    /// GTO bluff-to-value ratio for raise spots.
    /// Formula: `bet / (pot + 2 * bet)`. None when not raising or no bet size.
    pub bluff_ratio: Option<f64>,
}

use serde::{Deserialize, Serialize};

impl GtoHints {
    /// Compute GTO hints from the current game state and suggestion.
    pub fn calculate(state: &GameState, suggestion: &Suggestion) -> Self {
        // -----------------------------------------------------------------------
        // MDF: pot / (pot + bet). 0.0 when no bet to call.
        // -----------------------------------------------------------------------
        let mdf = if state.current_bet > 0.0 && state.pot_size > 0.0 {
            state.pot_size / (state.pot_size + state.current_bet)
        } else {
            0.0
        };

        // -----------------------------------------------------------------------
        // Action frequencies — approximated from equity margin vs thresholds.
        // Using the same street thresholds as make_suggestion for consistency.
        // -----------------------------------------------------------------------
        let margin = suggestion.equity - suggestion.pot_odds;

        // Street thresholds (duplicated to keep gto.rs standalone)
        let (raise_high, raise_med, call_low) = match state.street {
            crate::Street::Preflop => (0.15, 0.08, -0.03),
            crate::Street::Flop    => (0.12, 0.06, -0.04),
            crate::Street::Turn    => (0.10, 0.05, -0.05),
            crate::Street::River   => (0.08, 0.04, -0.05),
        };

        let (raise_freq, call_freq, fold_freq) = if margin > raise_high {
            (0.85, 0.10, 0.05)
        } else if margin > raise_med {
            (0.60, 0.30, 0.10)
        } else if margin > 0.0 {
            (0.20, 0.70, 0.10)
        } else if margin > call_low {
            (0.05, 0.55, 0.40)
        } else {
            (0.00, 0.10, 0.90)
        };

        // -----------------------------------------------------------------------
        // Bluff ratio for raise spots.
        // GTO: bluffs / (bluffs + value) = bet / (pot + 2 * bet).
        // Only meaningful when recommendation is RAISE and we have a bet size.
        // -----------------------------------------------------------------------
        let bluff_ratio = if matches!(suggestion.recommendation, Recommendation::Raise) {
            suggestion.suggested_bet_size.map(|bet| {
                let pot = state.pot_size.max(0.01);
                (bet / (pot + 2.0 * bet)).clamp(0.0, 1.0)
            })
        } else {
            None
        };

        GtoHints {
            mdf,
            raise_freq,
            call_freq,
            fold_freq,
            bluff_ratio,
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Card, Confidence, GameState, Position, Rank, Recommendation, Street, Suit};

    fn make_state(pot: f64, bet: f64) -> GameState {
        GameState {
            street: Street::Flop,
            hero_cards: [
                Card { rank: Rank::Ace, suit: Suit::Hearts },
                Card { rank: Rank::King, suit: Suit::Diamonds },
            ],
            board_cards: vec![],
            pot_size: pot,
            current_bet: bet,
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
    }
    }

    fn make_suggestion_stub(equity: f64, pot_odds: f64, rec: Recommendation, bet_size: Option<f64>) -> Suggestion {
        Suggestion {
            equity,
            pot_odds,
            recommendation: rec,
            confidence: Confidence::High,
            reasoning: String::new(),
            calc_time_ms: 0,
            suggested_bet_size: bet_size,
            hand_category: None,
            gto_hints: GtoHints {
                mdf: 0.0, raise_freq: 0.0, call_freq: 0.0,
                fold_freq: 0.0, bluff_ratio: None,
            },
        }
    }

    // MDF tests

    #[test]
    fn mdf_half_pot_bet() {
        // Pot=100, bet=50 → MDF = 100/(100+50) = 0.667
        let state = make_state(100.0, 50.0);
        let suggestion = make_suggestion_stub(0.5, 50.0 / 150.0, Recommendation::Call, None);
        let hints = GtoHints::calculate(&state, &suggestion);
        assert!((hints.mdf - 2.0/3.0).abs() < 0.001,
            "half-pot MDF should be ~0.667, got {}", hints.mdf);
    }

    #[test]
    fn mdf_full_pot_bet() {
        // Pot=100, bet=100 → MDF = 100/200 = 0.5
        let state = make_state(100.0, 100.0);
        let suggestion = make_suggestion_stub(0.5, 0.5, Recommendation::Call, None);
        let hints = GtoHints::calculate(&state, &suggestion);
        assert!((hints.mdf - 0.5).abs() < 0.001,
            "full-pot MDF should be 0.5, got {}", hints.mdf);
    }

    #[test]
    fn mdf_zero_when_no_bet() {
        let state = make_state(100.0, 0.0);
        let suggestion = make_suggestion_stub(0.6, 0.0, Recommendation::Raise, None);
        let hints = GtoHints::calculate(&state, &suggestion);
        assert_eq!(hints.mdf, 0.0, "MDF should be 0 when no bet");
    }

    // Frequency tests

    #[test]
    fn frequencies_sum_to_one() {
        let cases = [
            (0.70, 0.30), // strong edge — high raise freq
            (0.45, 0.33), // moderate edge
            (0.38, 0.35), // slight edge
            (0.32, 0.35), // slight disadvantage
            (0.15, 0.40), // large deficit
        ];
        for (equity, po) in cases {
            let state = make_state(100.0, 50.0);
            let suggestion = make_suggestion_stub(equity, po, Recommendation::Call, None);
            let h = GtoHints::calculate(&state, &suggestion);
            let sum = h.raise_freq + h.call_freq + h.fold_freq;
            assert!((sum - 1.0).abs() < 0.001,
                "frequencies must sum to 1.0 for equity={equity}, got {sum}");
        }
    }

    #[test]
    fn strong_edge_has_high_raise_freq() {
        // margin > raise_high (0.12 on flop): raise_freq = 0.85
        let state = make_state(100.0, 0.0);
        let suggestion = make_suggestion_stub(0.60, 0.0, Recommendation::Raise, None);
        let h = GtoHints::calculate(&state, &suggestion);
        assert!(h.raise_freq > 0.80,
            "strong edge should have raise_freq > 0.80, got {}", h.raise_freq);
    }

    #[test]
    fn weak_spot_has_high_fold_freq() {
        // margin well below call_low: fold_freq = 0.90
        let state = make_state(100.0, 100.0);
        let suggestion = make_suggestion_stub(0.10, 0.5, Recommendation::Fold, None);
        let h = GtoHints::calculate(&state, &suggestion);
        assert!(h.fold_freq > 0.80,
            "weak spot should have fold_freq > 0.80, got {}", h.fold_freq);
    }

    // Bluff ratio tests

    #[test]
    fn bluff_ratio_half_pot_raise() {
        // bet=66 (2/3 pot), pot=100 → bluff_ratio = 66 / (100 + 132) = 66/232 ≈ 0.284
        let state = make_state(100.0, 0.0);
        let suggestion = make_suggestion_stub(0.70, 0.0, Recommendation::Raise, Some(66.0));
        let h = GtoHints::calculate(&state, &suggestion);
        let expected = 66.0 / (100.0 + 132.0);
        assert!(h.bluff_ratio.is_some(), "raise should have bluff_ratio");
        let ratio = h.bluff_ratio.unwrap();
        assert!((ratio - expected).abs() < 0.01,
            "bluff_ratio should be ~{expected:.3}, got {ratio:.3}");
    }

    #[test]
    fn bluff_ratio_none_for_non_raise() {
        let state = make_state(100.0, 50.0);
        let suggestion = make_suggestion_stub(0.40, 0.33, Recommendation::Call, Some(50.0));
        let h = GtoHints::calculate(&state, &suggestion);
        assert!(h.bluff_ratio.is_none(), "CALL should have no bluff_ratio");
    }

    /// Task 8: bluff_ratio ≈ 0.33 for half-pot raise (bet = pot/2).
    ///
    /// Formula: bet / (pot + 2*bet)
    /// pot=100, bet=50 → 50 / (100 + 100) = 50/200 = 0.25
    ///
    /// Note: half-pot raise gives ratio = 0.25, not 0.33.
    /// A third-pot raise (bet = pot/3 ≈ 33) gives ratio ≈ 0.33:
    ///   33 / (100 + 66) = 33/166 ≈ 0.199
    /// The ≈0.33 target comes from the GTO bluff ratio for a pot-sized bet:
    ///   pot=100, bet=100 → 100/(100+200) = 100/300 = 0.333
    ///
    /// We test both: half-pot (≈0.25 ±0.05) and pot-sized (≈0.33 ±0.05).
    #[test]
    fn bluff_ratio_half_pot_raise_is_point_25() {
        // bet=50, pot=100 → 50 / (100 + 100) = 0.25
        let state = make_state(100.0, 0.0);
        let suggestion = make_suggestion_stub(0.70, 0.0, Recommendation::Raise, Some(50.0));
        let h = GtoHints::calculate(&state, &suggestion);
        assert!(h.bluff_ratio.is_some(), "raise should have bluff_ratio");
        let ratio = h.bluff_ratio.unwrap();
        assert!((ratio - 0.25).abs() < 0.01,
            "half-pot raise bluff_ratio should be ~0.25, got {ratio:.4}");
    }

    /// Pot-sized raise → bluff_ratio ≈ 0.333 (the "≈0.33" target from the spec).
    #[test]
    fn bluff_ratio_approx_third_for_pot_sized_raise() {
        // bet=100, pot=100 → 100/(100+200) = 0.333
        let state = make_state(100.0, 0.0);
        let suggestion = make_suggestion_stub(0.70, 0.0, Recommendation::Raise, Some(100.0));
        let h = GtoHints::calculate(&state, &suggestion);
        assert!(h.bluff_ratio.is_some(), "raise should have bluff_ratio");
        let ratio = h.bluff_ratio.unwrap();
        assert!((ratio - 1.0/3.0).abs() < 0.05,
            "pot-sized raise bluff_ratio should be ~0.333 ±0.05, got {ratio:.4}");
    }
}
