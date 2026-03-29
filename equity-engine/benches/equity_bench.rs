use criterion::{black_box, criterion_group, criterion_main, Criterion};
use equity_engine::{
    preflop::preflop_action,
    Card, GameState, Position, Rank, Street, Suit,
};
use equity_engine::commands::suggestion_pipeline;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn card(rank: Rank, suit: Suit) -> Card {
    Card { rank, suit }
}

fn flop_game_state() -> GameState {
    // AhKh hero vs Qh Jh 2c board — standard flush-draw scenario
    GameState {
        street: Street::Flop,
        hero_cards: [card(Rank::Ace, Suit::Hearts), card(Rank::King, Suit::Hearts)],
        board_cards: vec![
            card(Rank::Queen, Suit::Hearts),
            card(Rank::Jack, Suit::Hearts),
            card(Rank::Two, Suit::Clubs),
        ],
        position: Position::BTN,
        pot_size: 40.0,
        current_bet: 0.0,
        hero_stack: 200.0,
        villain_stack: 200.0,
        to_act: true,
        player_count: 2,
        context: Default::default(),
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

// ---------------------------------------------------------------------------
// Benchmarks
// ---------------------------------------------------------------------------

fn bench_calculate_suggestion_flop(c: &mut Criterion) {
    let state = flop_game_state();
    c.bench_function("calculate_suggestion/flop_AhKh_QhJh2c", |b| {
        b.iter(|| suggestion_pipeline(black_box(state.clone()), black_box("default")))
    });
}

fn bench_preflop_action(c: &mut Criterion) {
    // Pure table-lookup — should be sub-microsecond
    let c1 = card(Rank::Ace, Suit::Spades);
    let c2 = card(Rank::King, Suit::Hearts);
    c.bench_function("preflop_action/AKo_BTN_open", |b| {
        b.iter(|| {
            preflop_action(
                black_box(&c1),
                black_box(&c2),
                black_box(Position::BTN),
                black_box(6),
                black_box(0.0),
            )
        })
    });
}

criterion_group!(
    benches,
    bench_calculate_suggestion_flop,
    bench_preflop_action,
);
criterion_main!(benches);
