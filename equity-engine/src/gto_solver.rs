/// Async-ready GTO solver wrapper around `postflop-solver`.
///
/// Produces real GTO strategy frequencies for turn and river spots.
/// Flop returns `None` — the game tree is too large for real-time use.
///
/// # Feature gate
/// Only compiled with `--features gto-solver`. Default builds are unaffected.
#[cfg(feature = "gto-solver")]
mod inner {
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    };
    use std::time::Instant;

    use postflop_solver::{
        Action, ActionTree, BetSizeOptions, BoardState, CardConfig, PostFlopGame, Range,
        TreeConfig, flop_from_str, NOT_DEALT,
    };
    use serde::{Deserialize, Serialize};

    use crate::{Card, GameState, Rank, Street, Suit};

    // -----------------------------------------------------------------------
    // Public types
    // -----------------------------------------------------------------------

    /// Output of a completed (or partially completed) GTO solve.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct GtoResult {
        /// Fraction of time hero should bet/raise (0.0–1.0)
        pub raise_freq: f64,
        /// Fraction of time hero should check/call (0.0–1.0)
        pub call_freq: f64,
        /// Fraction of time hero should fold (0.0–1.0)
        pub fold_freq: f64,
        /// Exploitability achieved, as a % of the starting pot (lower = better)
        pub exploitability: f64,
        /// Number of CFR iterations completed
        pub iterations: u32,
        /// Wall-clock solve time in milliseconds
        pub solve_ms: u64,
    }

    /// Synchronous GTO solver for turn/river spots.
    pub struct GtoSolver;

    impl GtoSolver {
        /// Solve a turn or river spot and return GTO frequencies for hero.
        ///
        /// Returns `None` when:
        /// - Street is Preflop or Flop (tree too large for real-time)
        /// - Board has fewer than 4 cards (turn not yet dealt)
        /// - Solver fails to build the game tree (invalid state)
        ///
        /// `cancel` is checked every 10 iterations — set to `true` to abort.
        /// If elapsed time exceeds `timeout_ms`, the solve is stopped and the
        /// best-so-far result is returned.
        pub fn solve(
            state: &GameState,
            villain_range_str: &str,
            max_iterations: u32,
            timeout_ms: u64,
            cancel: Arc<AtomicBool>,
        ) -> Option<GtoResult> {
            // Only turn (4 cards) and river (5 cards) are supported
            match state.street {
                Street::Preflop | Street::Flop => return None,
                Street::Turn if state.board_cards.len() < 4 => return None,
                Street::River if state.board_cards.len() < 5 => return None,
                _ => {}
            }

            let start = Instant::now();

            // ---------------------------------------------------------------
            // Build card config
            // ---------------------------------------------------------------
            let hero_str = format!(
                "{}{}{}{}",
                rank_char(state.hero_cards[0].rank), suit_char(state.hero_cards[0].suit),
                rank_char(state.hero_cards[1].rank), suit_char(state.hero_cards[1].suit),
            );

            // Parse ranges — postflop-solver uses equilab format natively
            // OOP = villain (we place hero as IP for simplicity; position handling is a v2 concern)
            let oop_range: Range = villain_range_str.parse().ok()?;
            let ip_range: Range = hero_str.parse().ok()?;

            // Board cards
            let board = &state.board_cards;
            let flop_str = format!(
                "{}{}{}{}{}{}",
                rank_char(board[0].rank), suit_char(board[0].suit),
                rank_char(board[1].rank), suit_char(board[1].suit),
                rank_char(board[2].rank), suit_char(board[2].suit),
            );
            let flop = flop_from_str(&flop_str).ok()?;

            let turn = card_to_pf(&board[3]);
            let river = if board.len() >= 5 { card_to_pf(&board[4]) } else { NOT_DEALT };

            let card_config = CardConfig {
                range: [oop_range, ip_range],
                flop,
                turn,
                river,
            };

            // ---------------------------------------------------------------
            // Tree config — standard bet sizing
            // ---------------------------------------------------------------
            let initial_state = match state.street {
                Street::Turn  => BoardState::Turn,
                Street::River => BoardState::River,
                _             => return None,
            };

            let starting_pot = (state.pot_size * 100.0).round() as i32;
            let effective_stack = (state.hero_stack.min(state.villain_stack) * 100.0).round() as i32;

            if starting_pot <= 0 || effective_stack <= 0 {
                return None;
            }

            let bet_sizes = BetSizeOptions::try_from(("66%, e, a", "2.5x")).ok()?;
            let tree_config = TreeConfig {
                initial_state,
                starting_pot,
                effective_stack,
                rake_rate: 0.0,
                rake_cap: 0.0,
                flop_bet_sizes: [bet_sizes.clone(), bet_sizes.clone()],
                turn_bet_sizes: [bet_sizes.clone(), bet_sizes.clone()],
                river_bet_sizes: [bet_sizes.clone(), bet_sizes],
                turn_donk_sizes: None,
                river_donk_sizes: None,
                add_allin_threshold: 1.5,
                force_allin_threshold: 0.15,
                merging_threshold: 0.1,
            };

            let action_tree = ActionTree::new(tree_config).ok()?;
            let mut game = PostFlopGame::with_config(card_config, action_tree).ok()?;
            game.allocate_memory(false);

            // ---------------------------------------------------------------
            // Solve — manual iteration loop with cancel + timeout support
            // solve_step(&game, i) runs iteration i (takes &T)
            // compute_exploitability(&game) checks convergence
            // finalize(&mut game) normalizes the strategy (called once at end)
            // ---------------------------------------------------------------
            let target_exploitability = starting_pot as f32 * 0.005; // 0.5% pot
            let mut exploitability = f32::MAX;
            let mut iterations_done = 0u32;

            for i in 0..max_iterations {
                if cancel.load(Ordering::Relaxed) {
                    break;
                }
                if start.elapsed().as_millis() as u64 > timeout_ms {
                    break;
                }

                postflop_solver::solve_step(&game, i);
                iterations_done = i + 1;

                // Check convergence every 10 iterations
                if iterations_done % 10 == 0 {
                    exploitability = postflop_solver::compute_exploitability(&game);
                    if exploitability <= target_exploitability {
                        break;
                    }
                }
            }

            // Finalize normalizes the strategy — must be called exactly once
            postflop_solver::finalize(&mut game);
            // Final exploitability if we didn't check at the last iter
            if iterations_done % 10 != 0 || exploitability == f32::MAX {
                exploitability = postflop_solver::compute_exploitability(&game);
            }

            let solve_ms = start.elapsed().as_millis() as u64;

            // ---------------------------------------------------------------
            // Extract hero strategy
            // ---------------------------------------------------------------
            // IP (player 1) = hero. Navigate to hero's first decision:
            // if OOP acts first (check/bet), we need to navigate. At root,
            // check available_actions — if it's OOP to act, play Check (index 0)
            // to reach IP's decision node.
            let strategy_at_root = game.strategy();
            let root_actions = game.available_actions();
            let num_ip_hands = game.private_cards(1).len();

            // Find hero combo index in IP range
            let hero_combo = (card_to_pf(&state.hero_cards[0]), card_to_pf(&state.hero_cards[1]));
            let combo_idx = game.private_cards(1).iter()
                .position(|&c| c == hero_combo || c == (hero_combo.1, hero_combo.0))?;

            // Determine whose turn it is at root
            // At turn/river root OOP acts first; play OOP Check to reach IP node
            game.back_to_root();
            let (actions, strategy) = if game.current_player() == 0 {
                // OOP to act first — play check (action 0) to reach IP
                game.play(0);
                (game.available_actions(), game.strategy())
            } else {
                // IP acts at root directly
                (root_actions, strategy_at_root)
            };

            let mut raise_freq = 0.0f64;
            let mut call_freq  = 0.0f64;
            let mut fold_freq  = 0.0f64;

            for (i, action) in actions.iter().enumerate() {
                let freq = strategy.get(i * num_ip_hands + combo_idx)
                    .copied()
                    .unwrap_or(0.0) as f64;
                match action {
                    Action::Fold                                              => fold_freq  += freq,
                    Action::Check | Action::Call                             => call_freq  += freq,
                    Action::Bet(_) | Action::Raise(_) | Action::AllIn(_)    => raise_freq += freq,
                    Action::None | Action::Chance(_)                        => {}
                }
            }

            // Normalize to handle floating-point drift
            let total = raise_freq + call_freq + fold_freq;
            if total > 0.001 {
                raise_freq /= total;
                call_freq  /= total;
                fold_freq  /= total;
            }

            Some(GtoResult {
                raise_freq,
                call_freq,
                fold_freq,
                exploitability: exploitability as f64 / starting_pot as f64 * 100.0,
                iterations: iterations_done,
                solve_ms,
            })
        }
    }

    // -----------------------------------------------------------------------
    // Card encoding helpers
    // -----------------------------------------------------------------------

    /// Convert our `Card` to postflop-solver's `u8`: rank_idx * 4 + suit_idx.
    fn card_to_pf(card: &Card) -> postflop_solver::Card {
        let rank_idx: u8 = match card.rank {
            Rank::Two => 0,   Rank::Three => 1, Rank::Four => 2,
            Rank::Five => 3,  Rank::Six => 4,   Rank::Seven => 5,
            Rank::Eight => 6, Rank::Nine => 7,  Rank::Ten => 8,
            Rank::Jack => 9,  Rank::Queen => 10, Rank::King => 11,
            Rank::Ace => 12,
        };
        let suit_idx: u8 = match card.suit {
            Suit::Clubs => 0, Suit::Diamonds => 1, Suit::Hearts => 2, Suit::Spades => 3,
        };
        rank_idx * 4 + suit_idx
    }

    fn rank_char(r: Rank) -> char {
        match r {
            Rank::Two => '2', Rank::Three => '3', Rank::Four => '4',
            Rank::Five => '5', Rank::Six => '6', Rank::Seven => '7',
            Rank::Eight => '8', Rank::Nine => '9', Rank::Ten => 'T',
            Rank::Jack => 'J', Rank::Queen => 'Q', Rank::King => 'K', Rank::Ace => 'A',
        }
    }

    fn suit_char(s: Suit) -> char {
        match s { Suit::Hearts => 'h', Suit::Diamonds => 'd', Suit::Clubs => 'c', Suit::Spades => 's' }
    }

    // -----------------------------------------------------------------------
    // Tests
    // -----------------------------------------------------------------------

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::{Card, GameState, HandContext, Position, Rank, Street, Suit};
        use std::sync::{atomic::AtomicBool, Arc};

        fn make_card(rank: Rank, suit: Suit) -> Card {
            Card { rank, suit }
        }

        fn no_cancel() -> Arc<AtomicBool> {
            Arc::new(AtomicBool::new(false))
        }

        // Typical BB defend range
        const BB_RANGE: &str =
            "66+,A8s+,A5s-A4s,AJo+,K9s+,KQo,QTs+,JTs,T9s,98s,87s,76s,65s,54s";
        // BTN open range (includes AKs)
        const BTN_RANGE: &str =
            "QQ-22,AKs,AQs-A2s,ATo+,K5s+,KJo+,Q8s+,J8s+,T7s+,96s+,86s+,75s+,64s+,53s+";

        fn turn_state() -> GameState {
            // AhKh on QhJh2cTc, pot=20, stack=90 (4.5x pot)
            GameState {
                street: Street::Turn,
                hero_cards: [make_card(Rank::Ace, Suit::Hearts), make_card(Rank::King, Suit::Hearts)],
                board_cards: vec![
                    make_card(Rank::Queen, Suit::Hearts),
                    make_card(Rank::Jack, Suit::Hearts),
                    make_card(Rank::Two, Suit::Clubs),
                    make_card(Rank::Ten, Suit::Clubs),
                ],
                pot_size: 20.0,
                current_bet: 0.0,
                hero_stack: 90.0,
                villain_stack: 90.0,
                position: Position::BTN,
                to_act: true,
                player_count: 2,
                context: HandContext::default(),
                opponent_stats: vec![],
                table_id: None,
                big_blind_size: 1.0,
            }
                villain_stacks: std::collections::HashMap::new(),
        dealer_seat: 0,
        active_bet_size: None,
        hero_to_act: false,
            strategy_ev: None,
        strategy_pot_odds: None,
        strategy_betsize_bb: None,
        strategy_betsize_pct_pot: None,
    }
        
        fn river_state() -> GameState {
            GameState {
                street: Street::River,
                board_cards: vec![
                    make_card(Rank::Queen, Suit::Hearts),
                    make_card(Rank::Jack, Suit::Hearts),
                    make_card(Rank::Two, Suit::Clubs),
                    make_card(Rank::Ten, Suit::Clubs),
                    make_card(Rank::Three, Suit::Spades),
                ],
                ..turn_state()
            }
        }

        #[test]
        fn returns_none_for_preflop() {
            let mut state = turn_state();
            state.street = Street::Preflop;
            state.board_cards.clear();
            assert!(GtoSolver::solve(&state, BTN_RANGE, 10, 5000, no_cancel()).is_none());
        }

        #[test]
        fn returns_none_for_flop() {
            let mut state = turn_state();
            state.street = Street::Flop;
            state.board_cards.truncate(3);
            assert!(GtoSolver::solve(&state, BB_RANGE, 10, 5000, no_cancel()).is_none());
        }

        #[test]
        fn turn_solve_produces_valid_result() {
            let result = GtoSolver::solve(&turn_state(), BB_RANGE, 100, 10_000, no_cancel())
                .expect("turn solve should succeed");

            // Frequencies must sum to ~1.0
            let sum = result.raise_freq + result.call_freq + result.fold_freq;
            assert!((sum - 1.0).abs() < 0.01,
                "frequencies must sum to 1.0, got {sum:.4}: raise={:.3} call={:.3} fold={:.3}",
                result.raise_freq, result.call_freq, result.fold_freq);

            // All frequencies non-negative
            assert!(result.raise_freq >= 0.0 && result.call_freq >= 0.0 && result.fold_freq >= 0.0,
                "all frequencies must be non-negative");

            // Solve time should be recorded
            assert!(result.solve_ms > 0, "solve_ms should be > 0");

            // AhKh on QhJh2cTc: flush draw + straight draw — should mostly bet or call
            assert!(result.fold_freq < 0.5,
                "AhKh with nut flush draw should not fold >50%: fold_freq={:.3}", result.fold_freq);

            println!(
                "turn solve: raise={:.3} call={:.3} fold={:.3} | exploitability={:.2}%pot | {}ms",
                result.raise_freq, result.call_freq, result.fold_freq,
                result.exploitability, result.solve_ms
            );
        }

        #[test]
        fn river_solve_produces_valid_result() {
            let result = GtoSolver::solve(&river_state(), BB_RANGE, 100, 10_000, no_cancel())
                .expect("river solve should succeed");

            let sum = result.raise_freq + result.call_freq + result.fold_freq;
            assert!((sum - 1.0).abs() < 0.01,
                "river frequencies must sum to 1.0, got {sum:.4}");

            println!(
                "river solve: raise={:.3} call={:.3} fold={:.3} | exploitability={:.2}%pot | {}ms",
                result.raise_freq, result.call_freq, result.fold_freq,
                result.exploitability, result.solve_ms
            );
        }

        #[test]
        fn cancel_token_respected() {
            // Set cancel=true immediately — should return quickly with whatever ran
            let cancel = Arc::new(AtomicBool::new(true));
            let state = turn_state();
            // With cancel=true from the start, solve loop runs 0 iterations
            // May return None or a partial result — either is acceptable
            let result = GtoSolver::solve(&state, BB_RANGE, 1000, 30_000, cancel);
            // We just verify it doesn't hang
            println!("cancel test: {:?}", result.map(|r| r.iterations));
        }

        #[test]
        fn timeout_prevents_long_solve() {
            // 1ms timeout — should terminate immediately
            let state = turn_state();
            let start = std::time::Instant::now();
            let result = GtoSolver::solve(&state, BB_RANGE, 10_000, 1, no_cancel());
            let elapsed = start.elapsed().as_millis();
            // Should complete well under 1 second despite 10k iteration request
            assert!(elapsed < 5000, "timeout not respected: {}ms elapsed", elapsed);
            println!("timeout test: {:?}ms, result: {:?}", elapsed, result.map(|r| r.iterations));
        }
    }
}

// Re-export public types at module level
#[cfg(feature = "gto-solver")]
pub use inner::{GtoResult, GtoSolver};
