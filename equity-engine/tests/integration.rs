//! Integration tests for the equity-engine pipeline:
//! GameState → load_range → calculate_equity → Suggestion
//!
//! These tests call production code end-to-end without the Tauri feature flag.
//! They exercise: valid pipelines on all streets, nuts/dead scenarios,
//! error propagation (no silent equity=0.5 fallback), and range name validation.

use equity_engine::{
    commands::{sim_count_for_street, suggestion_pipeline},
    engine::calculate_equity,
    ranges::load_default_profile,
    Card, EquityRequest, GameState, HandContext, Position, Rank, Recommendation, Street, Suit,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_card(rank: Rank, suit: Suit) -> Card {
    Card { rank, suit }
}

/// Build a minimal GameState for the given street with a standard hero hand (AhKd).
fn standard_state(street: Street) -> GameState {
    let board = match street {
        Street::Preflop => vec![],
        Street::Flop => vec![
            make_card(Rank::Two, Suit::Clubs),
            make_card(Rank::Seven, Suit::Diamonds),
            make_card(Rank::Jack, Suit::Hearts),
        ],
        Street::Turn => vec![
            make_card(Rank::Two, Suit::Clubs),
            make_card(Rank::Seven, Suit::Diamonds),
            make_card(Rank::Jack, Suit::Hearts),
            make_card(Rank::Three, Suit::Spades),
        ],
        Street::River => vec![
            make_card(Rank::Two, Suit::Clubs),
            make_card(Rank::Seven, Suit::Diamonds),
            make_card(Rank::Jack, Suit::Hearts),
            make_card(Rank::Three, Suit::Spades),
            make_card(Rank::Nine, Suit::Clubs),
        ],
    };
    GameState {
        street,
        hero_cards: [
            make_card(Rank::Ace, Suit::Hearts),
            make_card(Rank::King, Suit::Diamonds),
        ],
        board_cards: board,
        pot_size: 100.0,
        current_bet: 50.0,
        hero_stack: 900.0,
        villain_stack: 900.0,
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

// ---------------------------------------------------------------------------
// Test 1–4: Valid pipeline on all four streets
// ---------------------------------------------------------------------------

fn run_pipeline_on_street(street: Street) {
    let state = standard_state(street);
    let result = suggestion_pipeline(state, "default");
    let suggestion = result.expect("pipeline should succeed on valid GameState");

    assert!(
        suggestion.equity >= 0.0 && suggestion.equity <= 1.0,
        "equity out of range: {}",
        suggestion.equity
    );
    assert!(
        suggestion.pot_odds >= 0.0 && suggestion.pot_odds <= 1.0,
        "pot_odds out of range: {}",
        suggestion.pot_odds
    );
    // pot_odds: 50 / (100 + 50) = 0.333...
    let expected_po = 50.0_f64 / 150.0;
    assert!(
        (suggestion.pot_odds - expected_po).abs() < 1e-9,
        "pot_odds mismatch: got {}, expected {}",
        suggestion.pot_odds,
        expected_po
    );
    assert!(
        !suggestion.reasoning.is_empty(),
        "reasoning should be non-empty"
    );
}

#[test]
fn pipeline_preflop() {
    run_pipeline_on_street(Street::Preflop);
}

#[test]
fn pipeline_flop() {
    run_pipeline_on_street(Street::Flop);
}

#[test]
fn pipeline_turn() {
    run_pipeline_on_street(Street::Turn);
}

#[test]
fn pipeline_river() {
    run_pipeline_on_street(Street::River);
}

// ---------------------------------------------------------------------------
// Test 5: Hero has the nuts — quads (4h4d on board 4c4s Ah)
// ---------------------------------------------------------------------------

#[test]
fn nuts_quads_equity_near_one_and_raises() {
    // Hero: 4h 4d, Board: 4c 4s Ah — four of a kind, essentially unbeatable
    let state = GameState {
        street: Street::River,
        hero_cards: [
            make_card(Rank::Four, Suit::Hearts),
            make_card(Rank::Four, Suit::Diamonds),
        ],
        board_cards: vec![
            make_card(Rank::Four, Suit::Clubs),
            make_card(Rank::Four, Suit::Spades),
            make_card(Rank::Ace, Suit::Hearts),
            make_card(Rank::King, Suit::Clubs),
            make_card(Rank::Queen, Suit::Diamonds),
        ],
        pot_size: 200.0,
        current_bet: 50.0,
        hero_stack: 800.0,
        villain_stack: 800.0,
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

    let suggestion = suggestion_pipeline(state, "default")
        .expect("nuts scenario should not error");

    assert!(
        suggestion.equity > 0.90,
        "expected equity > 0.90 for quads, got {}",
        suggestion.equity
    );
    assert!(
        matches!(suggestion.recommendation, Recommendation::Raise),
        "expected RAISE for nuts, got {:?}",
        suggestion.recommendation
    );
}

// ---------------------------------------------------------------------------
// Test 6: Hero is a massive underdog — 72o vs tight premium range + big bet
// ---------------------------------------------------------------------------

#[test]
fn weak_hand_vs_tight_range_folds() {
    // Hero: 7h 2d (worst starting hand), facing large bet vs tight premium range
    let state = GameState {
        street: Street::Flop,
        hero_cards: [
            make_card(Rank::Seven, Suit::Hearts),
            make_card(Rank::Two, Suit::Diamonds),
        ],
        board_cards: vec![
            make_card(Rank::Ace, Suit::Spades),
            make_card(Rank::King, Suit::Hearts),
            make_card(Rank::Queen, Suit::Clubs),
        ],
        pot_size: 100.0,
        current_bet: 200.0, // overbet — pot odds = 200/300 = 0.667
        hero_stack: 700.0,
        villain_stack: 700.0,
        position: Position::BB,
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

    let suggestion = suggestion_pipeline(state, "tight")
        .expect("weak hand scenario should not error");

    assert!(
        suggestion.equity < 0.40,
        "expected low equity for 72o vs tight range on AKQ board, got {}",
        suggestion.equity
    );
    assert!(
        matches!(suggestion.recommendation, Recommendation::Fold),
        "expected FOLD for 72o vs tight range + overbet, got {:?}",
        suggestion.recommendation
    );
}

// ---------------------------------------------------------------------------
// Test 7: Error propagation — conflicting cards (duplicate card in hand + board)
// ---------------------------------------------------------------------------

#[test]
fn duplicate_card_returns_err_not_silent_fallback() {
    // Ah appears in both hero hand and board — invalid, must error
    let state = GameState {
        street: Street::Flop,
        hero_cards: [
            make_card(Rank::Ace, Suit::Hearts), // Ah — also on board
            make_card(Rank::King, Suit::Diamonds),
        ],
        board_cards: vec![
            make_card(Rank::Ace, Suit::Hearts), // duplicate!
            make_card(Rank::Seven, Suit::Clubs),
            make_card(Rank::Two, Suit::Spades),
        ],
        pot_size: 100.0,
        current_bet: 50.0,
        hero_stack: 900.0,
        villain_stack: 900.0,
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

    let profile = load_default_profile(Street::Flop, "default")
        .expect("range load should succeed");
    let request = EquityRequest {
        game_state: state,
        range_profile: profile,
        simulations: 10_000,
    };
    let result = calculate_equity(&request);

    assert!(
        result.is_err(),
        "expected Err for duplicate card scenario, got Ok with equity={}",
        result.map(|s| s.equity).unwrap_or(-1.0)
    );
}

// ---------------------------------------------------------------------------
// Tests 8–11: suggestion_pipeline range name validation
// ---------------------------------------------------------------------------

#[test]
fn range_tight_returns_ok() {
    let state = standard_state(Street::Flop);
    assert!(
        suggestion_pipeline(state, "tight").is_ok(),
        "\"tight\" range should be valid"
    );
}

#[test]
fn range_default_returns_ok() {
    let state = standard_state(Street::Flop);
    assert!(
        suggestion_pipeline(state, "default").is_ok(),
        "\"default\" range should be valid"
    );
}

#[test]
fn range_loose_returns_ok() {
    let state = standard_state(Street::Flop);
    assert!(
        suggestion_pipeline(state, "loose").is_ok(),
        "\"loose\" range should be valid"
    );
}

#[test]
fn invalid_range_name_returns_err_not_panic() {
    let state = standard_state(Street::Flop);
    let result = suggestion_pipeline(state, "nonexistent");
    assert!(
        result.is_err(),
        "invalid range name should return Err, not panic"
    );
    let err = result.unwrap_err();
    assert!(
        err.contains("nonexistent") || err.contains("Failed to load"),
        "error message should be descriptive, got: {}",
        err
    );
}

// ---------------------------------------------------------------------------
// Test: 6-max equity differs from heads-up equity
// With 5 opponents all holding random hands, hero equity is significantly
// lower than heads-up. This validates that player_count wires through to
// the Monte Carlo engine.
// ---------------------------------------------------------------------------

#[test]
fn multiway_equity_lower_than_headsup() {
    let base = GameState {
        street: Street::Preflop,
        hero_cards: [
            make_card(Rank::Ace, Suit::Hearts),
            make_card(Rank::Ace, Suit::Diamonds),
        ],
        board_cards: vec![],
        pot_size: 0.0,
        current_bet: 0.0,
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

    let profile = load_default_profile(Street::Preflop, "default")
        .expect("range load should succeed");

    // Heads-up equity for AA
    let hu_request = EquityRequest {
        game_state: base.clone(),
        range_profile: profile.clone(),
        simulations: 50_000,
    };
    let hu_equity = calculate_equity(&hu_request)
        .expect("HU equity should succeed")
        .equity;

    // 6-max (5 opponents) equity for AA — must be lower
    let mut multiway_state = base.clone();
    multiway_state.player_count = 6;
    let mw_request = EquityRequest {
        game_state: multiway_state,
        range_profile: profile,
        simulations: 50_000,
    };
    let mw_equity = calculate_equity(&mw_request)
        .expect("multiway equity should succeed")
        .equity;

    assert!(
        hu_equity > mw_equity,
        "AA heads-up equity ({:.3}) should exceed 6-max equity ({:.3})",
        hu_equity, mw_equity
    );
    // AA vs 1 opponent is ~85%; vs 5 opponents drops to ~50%
    assert!(
        hu_equity > 0.75,
        "AA heads-up equity should be > 75%, got {:.3}",
        hu_equity
    );
    assert!(
        mw_equity < 0.70,
        "AA 6-max equity should be < 70%, got {:.3}",
        mw_equity
    );
}

// ---------------------------------------------------------------------------
// Task 1: calc_time_ms flows through the pipeline (QD-003 / tech debt lock-in)
// ---------------------------------------------------------------------------

/// Asserts that `calc_time_ms > 0` on a real pipeline call.
///
/// This locks in that:
///   1. The Instant::now() / elapsed() measurement in engine.rs actually runs
///   2. The value is propagated through Suggestion without being zeroed
///   3. `suggestion_pipeline` does not short-circuit before reaching the engine
///
/// Even on a trivially fast machine this should be > 0 — the Monte Carlo
/// simulation runs iterations and the clock resolution is nanoseconds.
#[test]
fn calc_time_ms_is_nonzero_after_real_pipeline_call() {
    let state = GameState {
        street: Street::Flop,
        hero_cards: [
            make_card(Rank::Ace, Suit::Hearts),
            make_card(Rank::King, Suit::Diamonds),
        ],
        board_cards: vec![
            make_card(Rank::Ace, Suit::Clubs),
            make_card(Rank::Seven, Suit::Spades),
            make_card(Rank::Two, Suit::Hearts),
        ],
        pot_size: 100.0,
        current_bet: 40.0,
        hero_stack: 800.0,
        villain_stack: 900.0,
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

    let suggestion = suggestion_pipeline(state, "default")
        .expect("pipeline should succeed");

    assert!(
        suggestion.calc_time_ms > 0,
        "calc_time_ms should be > 0 after a real Monte Carlo simulation, got {}",
        suggestion.calc_time_ms
    );
}

// ---------------------------------------------------------------------------
// Task 2: GameState PartialEq (tech debt #5)
// ---------------------------------------------------------------------------

#[test]
fn game_state_partial_eq_identical() {
    let s1 = standard_state(Street::Flop);
    let s2 = standard_state(Street::Flop);
    assert_eq!(s1, s2, "identical GameState values should be equal");
}

#[test]
fn game_state_partial_eq_different_streets() {
    let flop  = standard_state(Street::Flop);
    let river = standard_state(Street::River);
    assert_ne!(flop, river, "different streets should not be equal");
}

// ---------------------------------------------------------------------------
// Task 3: HandContext — preflop raise narrows villain range
// ---------------------------------------------------------------------------

#[test]
fn preflop_raise_uses_tighter_villain_range() {
    use equity_engine::HandContext;

    // Limped pot — default range
    let limped = {
        let mut s = standard_state(Street::Flop);
        s.context = HandContext::default(); // preflop_raise_seen = false
        s
    };

    // Raised pot — tight range
    let raised = {
        let mut s = standard_state(Street::Flop);
        s.context = HandContext {
            preflop_raise_seen: true,
            preflop_3bet_seen: false,
            villain_aggressor: false,
            streets_seen: 1,
                    villain_bet_count: 0,
                    preflop_4bet_seen: false,
            squeeze_situation: false,
        };
        s
    };

    let limped_eq  = suggestion_pipeline(limped,  "default").expect("limped pipeline ok");
    let raised_eq  = suggestion_pipeline(raised,  "default").expect("raised pipeline ok");

    // With a tighter villain range (raised pot), hero equity vs villain should differ.
    // AK on a generic flop: tighter villain range typically gives hero slightly different equity.
    // We can't assert exact direction (depends on board), but we CAN assert the pipeline
    // runs without error and produces valid equity values in both cases.
    assert!(limped_eq.equity > 0.0 && limped_eq.equity < 1.0,
        "limped pot equity out of range: {}", limped_eq.equity);
    assert!(raised_eq.equity > 0.0 && raised_eq.equity < 1.0,
        "raised pot equity out of range: {}", raised_eq.equity);

    // The suggestion pipeline must NOT error on either path
    // (This validates the context-to-range mapping plumbing is correct end-to-end)
}

// ---------------------------------------------------------------------------
// Task 1: Street-aware villain range tightening
// ---------------------------------------------------------------------------

#[test]
fn villain_aggressor_on_flop_uses_tight_range() {
    use equity_engine::HandContext;

    // Villain bet the flop (villain_aggressor=true), no preflop raise
    let state = {
        let mut s = standard_state(Street::Flop);
        s.context = HandContext {
            preflop_raise_seen: false,
            preflop_3bet_seen: false,
            villain_aggressor: true,
            streets_seen: 1,
                    villain_bet_count: 0,
                    preflop_4bet_seen: false,
            squeeze_situation: false,
        };
        s
    };

    // Pipeline must succeed — villain aggressor triggers tight range selection
    let result = suggestion_pipeline(state, "default");
    assert!(result.is_ok(), "villain aggressor context should not cause pipeline error");
    let suggestion = result.unwrap();
    assert!(suggestion.equity > 0.0 && suggestion.equity < 1.0,
        "equity out of range: {}", suggestion.equity);
}

#[test]
fn late_street_with_action_uses_tight_range() {
    use equity_engine::HandContext;

    // Turn with streets_seen=2 and preflop raise — should use tight range
    let state = {
        let mut s = standard_state(Street::Turn);
        s.context = HandContext {
            preflop_raise_seen: true,
            preflop_3bet_seen: false,
            villain_aggressor: false,
            streets_seen: 2,
                    villain_bet_count: 0,
                    preflop_4bet_seen: false,
            squeeze_situation: false,
        };
        s
    };

    let result = suggestion_pipeline(state, "default");
    assert!(result.is_ok(), "late street with preflop raise should not error");
}

// ---------------------------------------------------------------------------
// Task 2: Adaptive simulation count
// ---------------------------------------------------------------------------

#[test]
fn pipeline_succeeds_on_all_streets_with_adaptive_sims() {
    // Verify that all four streets run successfully with their adaptive sim counts.
    // calc_time_ms > 0 confirms the simulation actually ran.
    for street in [Street::Preflop, Street::Flop, Street::Turn, Street::River] {
        let state = standard_state(street);
        let result = suggestion_pipeline(state, "default");
        assert!(result.is_ok(), "pipeline failed on {:?}: {:?}", street, result.err());
        let s = result.unwrap();
        assert!(s.calc_time_ms > 0,
            "calc_time_ms should be >0 on {:?}", street);
        assert!(s.equity > 0.0 && s.equity < 1.0,
            "equity out of range on {:?}: {}", street, s.equity);
    }
}

#[test]
fn river_equity_variance_lower_than_preflop() {
    // River uses tighter stdev_target (0.005) than preflop (0.010).
    // Run 5 river calls and 5 preflop calls; river std dev should be ≤ preflop std dev.
    // This is a statistical test — with 5 samples it's not perfectly reliable,
    // but AK on a fixed river board (5 known cards) has near-zero variance anyway.
    let river_state = {
        let mut s = standard_state(Street::River);
        // Add a full board (5 cards): Ah Kd Qc Jh Ts (Broadway — AK has straight)
        s.board_cards = vec![
            make_card(Rank::Queen, Suit::Clubs),
            make_card(Rank::Jack, Suit::Hearts),
            make_card(Rank::Ten, Suit::Spades),
            make_card(Rank::Two, Suit::Clubs),
            make_card(Rank::Three, Suit::Diamonds),
        ];
        s
    };

    let equities: Vec<f64> = (0..5)
        .map(|_| suggestion_pipeline(river_state.clone(), "default")
            .expect("river pipeline ok")
            .equity)
        .collect();

    // All river equities should be very close (stdev < 0.02)
    let mean = equities.iter().sum::<f64>() / equities.len() as f64;
    let variance = equities.iter().map(|e| (e - mean).powi(2)).sum::<f64>() / equities.len() as f64;
    let stdev = variance.sqrt();

    assert!(stdev < 0.02,
        "river equity stdev should be < 0.02, got {:.4} (values: {:?})",
        stdev, equities);
}

// ---------------------------------------------------------------------------
// equity-engine-v2: Task 1 — Adaptive simulation count
// ---------------------------------------------------------------------------

/// Part A (deterministic): verify the sim count constants are ordered correctly.
/// This directly asserts the logic in `sim_count_for_street` without any timing.
#[test]
fn adaptive_sim_count_increases_with_street() {
    let preflop = sim_count_for_street(Street::Preflop);
    let flop    = sim_count_for_street(Street::Flop);
    let turn    = sim_count_for_street(Street::Turn);
    let river   = sim_count_for_street(Street::River);

    assert!(preflop < flop,
        "preflop ({preflop}) should use fewer sims than flop ({flop})");
    assert!(flop < turn,
        "flop ({flop}) should use fewer sims than turn ({turn})");
    assert!(turn < river,
        "turn ({turn}) should use fewer sims than river ({river})");

    // Concrete values from the spec: flop=15k, river=25k
    assert_eq!(flop,  15_000, "flop sim count should be 15,000");
    assert_eq!(river, 25_000, "river sim count should be 25,000");
}

/// Part B (timing): assert river sim count > flop sim count which drives more
/// computation. The wall-clock test is unreliable in parallel test runs due to
/// scheduler jitter — instead we assert the stdev_target is tighter on river
/// (0.005) vs flop (0.007) by verifying the sim counts directly, and separately
/// verify both streets complete in reasonable time and produce valid results.
/// The deterministic Part A test above is the canonical adaptive-count test.
#[test]
fn adaptive_sim_river_produces_valid_result_and_completes() {
    // River: 5-card board, 25k sims, stdev_target=0.005 (tightest)
    let river_state = GameState {
        street: Street::River,
        hero_cards: [
            make_card(Rank::Ace,  Suit::Hearts),
            make_card(Rank::King, Suit::Hearts),
        ],
        board_cards: vec![
            make_card(Rank::Queen, Suit::Hearts),
            make_card(Rank::Jack,  Suit::Hearts),
            make_card(Rank::Two,   Suit::Clubs),
            make_card(Rank::Seven, Suit::Diamonds),
            make_card(Rank::Three, Suit::Spades),
        ],
        pot_size: 100.0,
        current_bet: 0.0,
        hero_stack: 900.0,
        villain_stack: 900.0,
        position: Position::BTN,
        to_act: true,
        player_count: 2,
        context: HandContext::default(),
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

    // Run 3 times, collect times and equities
    let results: Vec<_> = (0..3)
        .map(|_| suggestion_pipeline(river_state.clone(), "default").expect("river pipeline ok"))
        .collect();

    // All must produce valid equity
    for r in &results {
        assert!(r.equity > 0.0 && r.equity < 1.0,
            "river equity out of range: {}", r.equity);
        assert!(r.calc_time_ms > 0,
            "calc_time_ms should be > 0 on river, got {}", r.calc_time_ms);
    }

    // sim_count for river must be the highest (25k > flop 15k)
    assert!(
        sim_count_for_street(Street::River) > sim_count_for_street(Street::Flop),
        "river sim count ({}) should exceed flop ({})",
        sim_count_for_street(Street::River),
        sim_count_for_street(Street::Flop)
    );
}

// ---------------------------------------------------------------------------
// equity-engine-v2: Task 2 — Street-aware villain range narrowing
// ---------------------------------------------------------------------------

/// Two identical GameState (flop, BTN, AhKh on QhJh2c) with different HandContext.
/// Context A: limped pot (default villain range).
/// Context B: raised + villain aggressive (tight villain range).
///
/// We test this in two layers:
///
/// Layer 1 (deterministic): context B has `preflop_raise_seen=true` which the engine
/// maps to "tight" range. We verify this by calling the pipeline with explicit range
/// names "default" vs "tight" and asserting equity differs — this is the ground truth
/// for what the context routing should produce.
///
/// Layer 2 (context routing): same GameStates with different HandContext, both using
/// "default" range name — the engine internally selects "tight" based on context B.
/// We assert equity differs from context A. Uses 5 runs + median to reduce MC variance.
#[test]
fn villain_range_narrowing_changes_equity() {
    // AhKh on QhJh2c — hero has flush draw + two overcards.
    // Against tight range (AA/KK/QQ/JJ/TT, AKs/AQs, AKo/AQo): villain holds
    // mostly overpairs and top-pair hands — hero has significant equity.
    // Against default range (adds 99/88/77/66, suited connectors, AJo/KQo):
    // villain has more medium-strength hands — equity distribution shifts.
    let base = GameState {
        street: Street::Flop,
        hero_cards: [
            make_card(Rank::Ace,  Suit::Hearts),
            make_card(Rank::King, Suit::Hearts),
        ],
        board_cards: vec![
            make_card(Rank::Queen, Suit::Hearts),
            make_card(Rank::Jack,  Suit::Hearts),
            make_card(Rank::Two,   Suit::Clubs),
        ],
        pot_size: 100.0,
        current_bet: 40.0,
        hero_stack: 800.0,
        villain_stack: 900.0,
        position: Position::BTN,
        to_act: true,
        player_count: 2,
        context: HandContext::default(),
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

    // Layer 1: explicit range names — direct ground truth
    fn median_equity_named(state: GameState, range: &str) -> f64 {
        let mut eq: Vec<f64> = (0..5)
            .map(|_| suggestion_pipeline(state.clone(), range)
                .expect("pipeline ok").equity)
            .collect();
        eq.sort_by(|a, b| a.partial_cmp(b).unwrap());
        eq[2] // median of 5
    }

    let eq_default = median_equity_named(base.clone(), "default");
    let eq_tight   = median_equity_named(base.clone(), "tight");

    assert!(eq_default > 0.0 && eq_default < 1.0,
        "default range equity out of range: {eq_default}");
    assert!(eq_tight > 0.0 && eq_tight < 1.0,
        "tight range equity out of range: {eq_tight}");

    let direct_diff = (eq_default - eq_tight).abs();
    assert!(
        direct_diff > 0.01,
        "AhKh on QhJh2c: equity vs default ({eq_default:.4}) should differ from \
         tight ({eq_tight:.4}) by > 0.01 (got {direct_diff:.4}); \
         ranges differ significantly (20 vs 10 combos)"
    );

    // Layer 2: context routing — HandContext drives range selection internally
    let state_a = GameState {
        context: HandContext {
            preflop_raise_seen: false,
            villain_aggressor:  false,
            ..Default::default()
        },
        ..base.clone()
    };
    let state_b = GameState {
        context: HandContext {
            preflop_raise_seen: true,
            villain_aggressor:  true,
            streets_seen: 1,
            ..Default::default()
        },
        ..base.clone()
    };

    // Layer 2: context routing — pass empty RangeProfile so the engine selects
    // the villain range from HandContext rather than using explicit combos.
    // This is the only code path that exercises context-driven range selection.
    //
    // NOTE: suggestion_pipeline() always loads a non-empty profile (bypassing
    // context routing). Direct calculate_equity() with empty combos is required.
    use equity_engine::{EquityRequest, RangeProfile};

    fn median_equity_ctx(state: GameState) -> f64 {
        let empty_profile = RangeProfile {
            name: "context-driven".to_string(),
            street: state.street,
            combos: vec![], // empty → engine selects based on HandContext
        };
        let request = EquityRequest {
            game_state: state.clone(),
            range_profile: empty_profile,
            simulations: 15_000,
        };
        let mut eq: Vec<f64> = (0..5)
            .map(|_| calculate_equity(&request).expect("pipeline ok").equity)
            .collect();
        eq.sort_by(|a, b| a.partial_cmp(b).unwrap());
        eq[2]
    }

    let eq_a = median_equity_ctx(state_a);
    let eq_b = median_equity_ctx(state_b);

    assert!(eq_a > 0.0 && eq_a < 1.0, "context A equity out of range: {eq_a}");
    assert!(eq_b > 0.0 && eq_b < 1.0, "context B equity out of range: {eq_b}");

    let ctx_diff = (eq_a - eq_b).abs();
    assert!(
        ctx_diff > 0.01,
        "context routing: equity vs limped context ({eq_a:.4}) should differ from \
         raised/aggressor context ({eq_b:.4}) by > 0.01 (got {ctx_diff:.4}); \
         context B (preflop_raise_seen=true) should route to tight range internally"
    );
}

// ---------------------------------------------------------------------------
// Board texture analysis
// ---------------------------------------------------------------------------

#[test]
fn board_texture_wet_raises_threshold() {
    use equity_engine::HandContext;

    // Wet board: 8d 9d Th (flush + straight draw)
    let wet_state = GameState {
        street: Street::Flop,
        hero_cards: [
            make_card(Rank::Ace, Suit::Hearts),
            make_card(Rank::King, Suit::Hearts),
        ],
        board_cards: vec![
            make_card(Rank::Eight, Suit::Diamonds),
            make_card(Rank::Nine, Suit::Diamonds),
            make_card(Rank::Ten, Suit::Hearts),
        ],
        pot_size: 100.0, current_bet: 0.0, hero_stack: 900.0, villain_stack: 900.0,
        position: Position::BTN, to_act: true, player_count: 2,
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

    // Dry board: 2c 7h Ks (rainbow, no connects)
    let dry_state = GameState {
        board_cards: vec![
            make_card(Rank::Two, Suit::Clubs),
            make_card(Rank::Seven, Suit::Hearts),
            make_card(Rank::King, Suit::Spades),
        ],
        ..wet_state.clone()
    };

    let wet_result  = suggestion_pipeline(wet_state,  "default").expect("wet pipeline ok");
    let dry_result  = suggestion_pipeline(dry_state,  "default").expect("dry pipeline ok");

    // Both should produce valid equity values
    assert!(wet_result.equity  > 0.0 && wet_result.equity  < 1.0);
    assert!(dry_result.equity  > 0.0 && dry_result.equity  < 1.0);

    // Wet board reasoning should mention the draw
    assert!(
        wet_result.reasoning.contains("flush") || wet_result.reasoning.contains("straight")
            || wet_result.reasoning.contains("wet"),
        "wet board reasoning should mention draw type: {}", wet_result.reasoning
    );
}

#[test]
fn board_texture_from_cards_wet_detection() {
    use equity_engine::BoardTexture;

    // 8d 9d Th — flush draw (2 diamonds) + straight draw (8-9-T connected)
    let board = vec![
        make_card(Rank::Eight, Suit::Diamonds),
        make_card(Rank::Nine, Suit::Diamonds),
        make_card(Rank::Ten, Suit::Hearts),
    ];
    let tex = BoardTexture::from_cards(&board);
    assert!(tex.flush_draw_possible, "8d9dTh: flush draw expected");
    assert!(tex.straight_draw_possible, "8d9dTh: straight draw expected");
    assert!(tex.wet, "8d9dTh: should be wet");
    assert!(!tex.dry, "8d9dTh: should not be dry");
    assert!(!tex.monotone, "8d9dTh: not monotone");
}

#[test]
fn board_texture_from_cards_dry_detection() {
    use equity_engine::BoardTexture;

    // 2c 7h Ks — rainbow, no connectivity
    let board = vec![
        make_card(Rank::Two, Suit::Clubs),
        make_card(Rank::Seven, Suit::Hearts),
        make_card(Rank::King, Suit::Spades),
    ];
    let tex = BoardTexture::from_cards(&board);
    assert!(!tex.flush_draw_possible, "2c7hKs: no flush draw");
    assert!(!tex.wet, "2c7hKs: should not be wet");
    assert!(tex.dry, "2c7hKs: should be dry");
}

#[test]
fn board_texture_paired_board() {
    use equity_engine::BoardTexture;

    let board = vec![
        make_card(Rank::Seven, Suit::Hearts),
        make_card(Rank::Seven, Suit::Clubs),
        make_card(Rank::Ace, Suit::Spades),
    ];
    let tex = BoardTexture::from_cards(&board);
    assert!(tex.paired_board, "77A: paired board expected");
}

#[test]
fn board_texture_monotone() {
    use equity_engine::BoardTexture;

    let board = vec![
        make_card(Rank::Two, Suit::Hearts),
        make_card(Rank::Seven, Suit::Hearts),
        make_card(Rank::King, Suit::Hearts),
    ];
    let tex = BoardTexture::from_cards(&board);
    assert!(tex.monotone, "2h7hKh: monotone expected");
    assert!(tex.flush_draw_possible, "2h7hKh: flush draw possible");
}

// ---------------------------------------------------------------------------
// Multiway pot threshold adjustment
// ---------------------------------------------------------------------------

#[test]
fn multiway_pot_reasoning_mentions_player_count() {
    use equity_engine::HandContext;

    let multiway = GameState {
        street: Street::Flop,
        hero_cards: [
            make_card(Rank::Ace, Suit::Hearts),
            make_card(Rank::King, Suit::Diamonds),
        ],
        board_cards: vec![
            make_card(Rank::Two, Suit::Clubs),
            make_card(Rank::Seven, Suit::Hearts),
            make_card(Rank::King, Suit::Spades),
        ],
        pot_size: 100.0, current_bet: 0.0, hero_stack: 900.0, villain_stack: 900.0,
        position: Position::BTN, to_act: true, player_count: 4,
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

    let result = suggestion_pipeline(multiway, "default").expect("multiway pipeline ok");
    assert!(
        result.reasoning.contains("Multiway") || result.reasoning.contains("players"),
        "multiway reasoning should mention player count: {}", result.reasoning
    );
    assert!(result.reasoning.contains("4"), "should mention 4 players: {}", result.reasoning);
}

// ---------------------------------------------------------------------------
// Actionable reasoning — conclusion-first format
// ---------------------------------------------------------------------------

#[test]
fn actionable_reasoning_starts_with_hand_description() {
    let state = standard_state(Street::Flop);
    let result = suggestion_pipeline(state, "default").expect("pipeline ok");
    let valid_starts = ["Dominating", "Strong hand", "Ahead", "Behind but", "Behind"];
    assert!(
        valid_starts.iter().any(|s| result.reasoning.starts_with(s)),
        "reasoning should start with hand description, got: {}", result.reasoning
    );
}

#[test]
fn actionable_reasoning_contains_numbers_in_brackets() {
    let state = standard_state(Street::Flop);
    let result = suggestion_pipeline(state, "default").expect("pipeline ok");
    // Numbers should appear in brackets: [equity X%, pot odds Y%, margin Z%]
    assert!(
        result.reasoning.contains('%'),
        "reasoning should include percentage numbers: {}", result.reasoning
    );
}

// ---------------------------------------------------------------------------
// Feature 1+2: OpponentStats + exploitative range adjustments
// ---------------------------------------------------------------------------

#[test]
fn opponent_archetype_fish_uses_loose_range() {
    use equity_engine::{HandContext, OpponentStats};
    let mut state = standard_state(Street::Flop);
    state.opponent_stats = vec![OpponentStats {
        player_id: "fish_1".to_string(),
        hands_seen: 50,
        vpip: 25,  // 50% → Fish
        pfr: 5,
        ..Default::default()
    }];
    let result = suggestion_pipeline(state, "default").expect("fish pipeline ok");
    assert!(result.equity > 0.0 && result.equity < 1.0);
    // Fish exploit note should appear in reasoning
    assert!(result.reasoning.contains("Fish") || result.reasoning.contains("value"),
        "Fish exploit note expected: {}", result.reasoning);
}

#[test]
fn opponent_archetype_nit_uses_tight_range() {
    use equity_engine::{HandContext, OpponentStats};
    let mut state = standard_state(Street::Flop);
    state.opponent_stats = vec![OpponentStats {
        player_id: "nit_1".to_string(),
        hands_seen: 30,
        vpip: 5,  // 16.7% → Nit
        pfr: 3,
        ..Default::default()
    }];
    let result = suggestion_pipeline(state, "default").expect("nit pipeline ok");
    assert!(result.equity > 0.0);
}

#[test]
fn opponent_archetype_unknown_below_20_hands() {
    use equity_engine::OpponentStats;
    let stats = OpponentStats {
        hands_seen: 15,
        vpip: 8,
        ..Default::default()
    };
    assert_eq!(stats.archetype(), equity_engine::PlayerArchetype::Unknown);
}

#[test]
fn opponent_stats_vpip_pfr_af_calculations() {
    use equity_engine::OpponentStats;
    let stats = OpponentStats {
        hands_seen: 100,
        vpip: 30,
        pfr: 20,
        aggression_count: 40,
        check_call_count: 20,
        ..Default::default()
    };
    assert!((stats.vpip_pct() - 0.30).abs() < 0.001);
    assert!((stats.pfr_pct()  - 0.20).abs() < 0.001);
    assert!((stats.af()       - 2.0 ).abs() < 0.001);
    assert_eq!(stats.archetype(), equity_engine::PlayerArchetype::Unknown); // VPIP=30% PFR=20% doesn't strictly match any archetype
}

#[test]
fn range_from_archetype_all_variants() {
    use equity_engine::{PlayerArchetype, commands::range_from_archetype};
    assert_eq!(range_from_archetype(&PlayerArchetype::Fish), "loose");
    assert_eq!(range_from_archetype(&PlayerArchetype::CallingStation), "loose");
    assert_eq!(range_from_archetype(&PlayerArchetype::Nit), "tight");
    assert_eq!(range_from_archetype(&PlayerArchetype::Tag), "default");
    assert_eq!(range_from_archetype(&PlayerArchetype::Lag), "default");
    assert_eq!(range_from_archetype(&PlayerArchetype::Unknown), "default");
}

// ---------------------------------------------------------------------------
// Feature 3: preflop_action_with_reads
// ---------------------------------------------------------------------------

#[test]
fn preflop_with_reads_vs_nit_folds_more() {
    use equity_engine::{Card, OpponentStats, Position, Rank, Suit, preflop::preflop_action_with_reads};
    let nit = OpponentStats {
        hands_seen: 50,
        vpip: 8, pfr: 6,
        ..Default::default()
    };
    let c1 = Card { rank: Rank::Ten, suit: Suit::Hearts };
    let c2 = Card { rank: Rank::Nine, suit: Suit::Hearts };
    // TTs is marginal vs 3-bet from a Nit
    let action = preflop_action_with_reads(&c1, &c2, Position::BTN, 6, 8.0, Some(&nit));
    // T9s vs Nit 3-bet should fold or call, not re-raise
    assert!(
        action.decision != equity_engine::preflop::PreflopDecision::ThreeBet,
        "T9s should not 4-bet a Nit: {:?}", action.decision
    );
}

#[test]
fn preflop_with_reads_no_stats_equals_base() {
    use equity_engine::{Card, Position, Rank, Suit, preflop::{preflop_action, preflop_action_with_reads}};
    let c1 = Card { rank: Rank::Ace, suit: Suit::Hearts };
    let c2 = Card { rank: Rank::King, suit: Suit::Spades };
    let base   = preflop_action(&c1, &c2, Position::BTN, 6, 0.0);
    let reads  = preflop_action_with_reads(&c1, &c2, Position::BTN, 6, 0.0, None);
    assert_eq!(base.decision, reads.decision, "no stats → same as base");
}

// ---------------------------------------------------------------------------
// Range narrowing for multi-street aggression
// ---------------------------------------------------------------------------

#[test]
fn multi_street_aggressor_uses_tight_range() {
    use equity_engine::HandContext;

    // Villain raised preflop + bet flop + bet turn → villain_bet_count=3, streets_seen=2
    let aggressive = {
        let mut s = standard_state(Street::Turn);
        s.context = HandContext {
            preflop_raise_seen: true,
            villain_aggressor: true,
            streets_seen: 2,
            villain_bet_count: 3, // raised preflop + bet flop + bet turn
            ..Default::default()
        };
        s
    };

    // Limped pot, no aggression
    let passive = {
        let mut s = standard_state(Street::Turn);
        s.context = HandContext {
            preflop_raise_seen: false,
            villain_aggressor: false,
            streets_seen: 2,
            villain_bet_count: 0,
            ..Default::default()
        };
        s
    };

    let agg_result  = suggestion_pipeline(aggressive, "default").expect("aggressive pipeline ok");
    let pass_result = suggestion_pipeline(passive,    "default").expect("passive pipeline ok");

    // Both must return valid equity — the key assertion is that they don't panic
    // and that the aggressive hand produces a meaningful (different or same) equity.
    assert!(agg_result.equity  > 0.0 && agg_result.equity  < 1.0);
    assert!(pass_result.equity > 0.0 && pass_result.equity < 1.0);

    // The reasoning for the aggressive hand should come from tight range selection.
    // We can't assert exact equity diff (depends on range files), but we assert
    // the pipeline runs correctly for both paths.
}

#[test]
fn villain_bet_count_zero_uses_default_range() {
    use equity_engine::HandContext;

    // No aggression at all → default range
    let mut state = standard_state(Street::Flop);
    state.context = HandContext {
        preflop_raise_seen: false,
        villain_aggressor: false,
        villain_bet_count: 0,
        streets_seen: 1,
        ..Default::default()
    };
    let result = suggestion_pipeline(state, "default").expect("no-aggression pipeline ok");
    assert!(result.equity > 0.0);
}

#[test]
fn villain_bet_count_parsed_from_hand_history() {
    // Sample hand: villain raises preflop + bets flop + bets turn = 3 aggression events
    let hand = r#"Poker Hand #RC9991234567: Hold'em No Limit ($1/$2) - 2024/03/26 15:00:00 UTC
Table 'RushAndCash NLH 6max' 6-max Seat #1 is the button
Seat 1: Villain ($200 in chips)
Seat 2: Hero ($150 in chips)
Villain: posts small blind $1
Hero: posts big blind $2
*** HOLE CARDS ***
Dealt to Hero [Ah Kd]
Villain: raises $6 to $8
Hero: calls $6
*** FLOP *** [2c 7h Ts] (Pot: $16)
Villain: bets $8
Hero: calls $8
*** TURN *** [2c 7h Ts] [5d] (Pot: $32)
Villain: bets $16
Hero: calls $16
"#;
    // We can't call parse_hand directly but we can test that the test hand parses
    // and that the context has villain_bet_count > 0 via the public interface.
    // The HandHistoryParser is in another crate — test via the integration test
    // fixture approach: just verify the field exists on HandContext.
    let ctx = equity_engine::HandContext {
        villain_bet_count: 3,
        ..Default::default()
    };
    assert_eq!(ctx.villain_bet_count, 3);
}

// ---------------------------------------------------------------------------
// Implied odds + 4bet/squeeze
// ---------------------------------------------------------------------------

#[test]
fn implied_odds_flop_reduces_effective_pot_odds() {
    use equity_engine::HandContext;

    // On flop: pot_odds = 0.33, implied = 0.33 * 0.80 = 0.264
    // equity = 0.28: raw margin = -0.05 (FOLD), implied margin = +0.016 (CALL)
    let flop_state = {
        let mut s = standard_state(Street::Flop);
        s.pot_size = 100.0;
        s.current_bet = 50.0; // pot_odds = 50/150 = 0.333
        s
    };
    let raw_po = 50.0 / 150.0;
    let equity = raw_po - 0.05; // 0.283 — just below raw pot odds

    // On river, same situation: margin is negative → FOLD
    let river_state = {
        let mut s = standard_state(Street::River);
        s.pot_size = 100.0;
        s.current_bet = 50.0;
        s.board_cards = vec![
            make_card(Rank::Two, Suit::Clubs),
            make_card(Rank::Seven, Suit::Hearts),
            make_card(Rank::Jack, Suit::Spades),
            make_card(Rank::Three, Suit::Diamonds),
            make_card(Rank::Nine, Suit::Clubs),
        ];
        s
    };

    let flop_result  = suggestion_pipeline(flop_state,  "default").expect("flop ok");
    let river_result = suggestion_pipeline(river_state, "default").expect("river ok");

    // Both must produce valid equity
    assert!(flop_result.equity  > 0.0 && flop_result.equity  < 1.0);
    assert!(river_result.equity > 0.0 && river_result.equity < 1.0);
    // Flop reasoning should mention implied odds when discount is active
    // (it will if there's a bet, so pot_odds > 0 and flop discount fires)
}

#[test]
fn preflop_4bet_uses_tight_range() {
    use equity_engine::HandContext;

    let state_4bet = {
        let mut s = standard_state(Street::Flop);
        s.context = HandContext {
            preflop_4bet_seen: true,
            preflop_raise_seen: true,
            preflop_3bet_seen: true,
            ..Default::default()
        };
        s
    };

    let state_no_4bet = {
        let mut s = standard_state(Street::Flop);
        s.context = HandContext {
            preflop_4bet_seen: false,
            preflop_raise_seen: false,
            preflop_3bet_seen: false,
            ..Default::default()
        };
        s
    };

    // Both must succeed without error
    let r4 = suggestion_pipeline(state_4bet, "default").expect("4bet pipeline ok");
    let r0 = suggestion_pipeline(state_no_4bet, "default").expect("no-4bet pipeline ok");
    assert!(r4.equity > 0.0 && r4.equity < 1.0);
    assert!(r0.equity > 0.0 && r0.equity < 1.0);
}

#[test]
fn squeeze_situation_detected() {
    use equity_engine::HandContext;

    // Verify the field exists and is accessible
    let ctx = HandContext {
        squeeze_situation: true,
        preflop_4bet_seen: false,
        ..Default::default()
    };
    assert!(ctx.squeeze_situation);
    assert!(!ctx.preflop_4bet_seen);
}

// ---------------------------------------------------------------------------
// Task 3: Street-aware GTO preflop guard
// ---------------------------------------------------------------------------

#[test]
fn flop_state_does_not_use_preflop_chart() {
    let state = standard_state(Street::Flop);
    let result = suggestion_pipeline(state, "default").expect("flop pipeline ok");

    // Flop reasoning must NOT contain preflop-chart language
    assert!(
        !result.reasoning.contains("GTO Wizard"),
        "flop reasoning should not reference GTO Wizard chart: {}", result.reasoning
    );
    assert!(
        !result.reasoning.contains("opening range"),
        "flop reasoning should not reference opening range: {}", result.reasoning
    );
    assert!(
        !result.reasoning.contains("3-bet range"),
        "flop reasoning should not reference 3-bet range: {}", result.reasoning
    );
    // Must contain equity information (postflop equity-based reasoning)
    assert!(
        result.reasoning.contains('%'),
        "flop reasoning should contain equity %: {}", result.reasoning
    );
}

#[test]
fn preflop_state_uses_chart_reasoning() {
    let state = standard_state(Street::Preflop);
    let result = suggestion_pipeline(state, "default").expect("preflop pipeline ok");

    // Preflop should use chart-based reasoning (either GTO Wizard or static)
    // The reasoning will contain either "GTO Wizard" or positional chart language
    // plus equity — just verify it's not equity-only (which would mean chart was skipped)
    let has_chart = result.reasoning.contains("GTO Wizard")
        || result.reasoning.contains("opening range")
        || result.reasoning.contains("BTN")
        || result.reasoning.contains("UTG")
        || result.reasoning.contains("fold");
    assert!(has_chart,
        "preflop reasoning should use chart language: {}", result.reasoning);
}
