use std::sync::{atomic::AtomicBool, Arc};
use std::time::Instant;
use log::error;
use pokers::{HandRange, get_card_mask, approx_equity};
use crate::{Card, EquityRequest, Rank, Suggestion, Suit};

/// Convert our Card to pokers card string notation (e.g. "Ah", "Td")
fn card_to_str(card: &Card) -> String {
    let rank = match card.rank {
        Rank::Two => "2", Rank::Three => "3", Rank::Four => "4",
        Rank::Five => "5", Rank::Six => "6", Rank::Seven => "7",
        Rank::Eight => "8", Rank::Nine => "9", Rank::Ten => "T",
        Rank::Jack => "J", Rank::Queen => "Q", Rank::King => "K",
        Rank::Ace => "A",
    };
    let suit = match card.suit {
        Suit::Hearts => "h", Suit::Diamonds => "d",
        Suit::Clubs => "c", Suit::Spades => "s",
    };
    format!("{}{}", rank, suit)
}

/// Build a board mask string from board cards (e.g. "AhKdQc")
fn board_mask_str(board: &[Card]) -> String {
    board.iter().map(card_to_str).collect()
}

/// Calculate equity via pokers Monte Carlo simulation.
///
/// Returns `Err` if the simulation fails (e.g. conflicting cards, invalid hand).
/// Callers must not swallow this — surface it to the user or log it explicitly.
pub fn calculate_equity(request: &EquityRequest) -> Result<Suggestion, String> {
    let start = Instant::now();

    // Hero hand as a single-combo range string e.g. "AhKd"
    let hero_str = format!(
        "{}{}",
        card_to_str(&request.game_state.hero_cards[0]),
        card_to_str(&request.game_state.hero_cards[1])
    );

    // Villain range: use the profile passed by the caller.
    // commands.rs selects the appropriate range via range_from_context before calling here.
    // When profile_combos is empty (e.g. direct test calls), fall back to
    // context-driven range selection so direct callers still get sensible behaviour.
    let villain_str = {
        let profile_combos = &request.range_profile.combos;
        if profile_combos.is_empty() {
            // Direct caller with empty profile — derive range from HandContext
            let ctx = &request.game_state.context;
            let range_name = if ctx.preflop_3bet_seen
                || ctx.preflop_raise_seen
                || ctx.villain_aggressor
                || ctx.streets_seen >= 2
            {
                "tight"
            } else {
                "default"
            };
            match crate::ranges::load_default_profile(request.game_state.street, range_name) {
                Ok(profile) => match crate::notation::expand_range(&profile.combos) {
                    Ok(expanded) if !expanded.is_empty() => expanded.join(","),
                    _ => crate::notation::to_pokers_range_string(&profile.combos),
                },
                Err(_) => "random".to_string(),
            }
        } else {
            match crate::notation::expand_range(profile_combos) {
                Ok(expanded) => expanded.join(","),
                Err(_) => crate::notation::to_pokers_range_string(profile_combos),
            }
        }
    };

    // Build hand ranges: hero + (player_count - 1) villains, all with the same range profile.
    // player_count is clamped to [2, 9] — 1 hero + up to 8 villains.
    let opponent_count = (request.game_state.player_count.clamp(2, 9) - 1) as usize;
    let mut ranges = Vec::with_capacity(1 + opponent_count);
    ranges.push(hero_str);
    for _ in 0..opponent_count {
        ranges.push(villain_str.clone());
    }
    let hand_ranges = HandRange::from_strings(ranges);

    let board_str = board_mask_str(&request.game_state.board_cards);
    let board_mask = get_card_mask(&board_str);
    let dead_mask = get_card_mask("");
    let cancel_token = Arc::new(AtomicBool::new(false));

    // Adaptive stdev_target: tighter (more simulations) on later streets where
    // precision matters more. Derived from request.simulations hint set by the pipeline:
    //   preflop 5k → 0.010  (chart is primary; fast MC for equity display)
    //   flop   15k → 0.007
    //   turn   20k → 0.006
    //   river  25k → 0.005  (no more cards — best precision)
    let stdev_target = match request.simulations {
        0..=7_500   => 0.010_f64,
        7_501..=17_500  => 0.007_f64,
        17_501..=22_500 => 0.006_f64,
        _               => 0.005_f64,
    };

    // stdev_target of 0.005 balances accuracy vs speed; typically <50ms
    let result = approx_equity(
        &hand_ranges,
        board_mask,
        dead_mask,
        4,            // threads
        stdev_target,
        cancel_token,
        |_| {},       // no-op progress callback
    );

    let calc_time_ms = start.elapsed().as_millis() as u64;

    let equity = match result {
        Ok(sim) => sim.equities[0] as f64,
        Err(e) => {
            // Log and propagate — no silent 0.5 fallback (tech debt #1 resolved)
            error!("[equity-engine] Monte Carlo simulation failed: {:?}", e);
            return Err(format!("Equity simulation failed: {:?}", e));
        }
    };

    // Pot odds: call_amount / (pot + call_amount)
    let pot_odds = if request.game_state.current_bet > 0.0 {
        request.game_state.current_bet
            / (request.game_state.pot_size + request.game_state.current_bet)
    } else {
        0.0
    };

    let (recommendation, confidence, reasoning) =
        crate::suggestion::make_suggestion(equity, pot_odds, &request.game_state);

    let suggested_bet_size =
        crate::suggestion::suggested_bet_size(&recommendation, &request.game_state);

    let hand_category = crate::hand_eval::hand_category(
        &request.game_state.hero_cards,
        &request.game_state.board_cards,
        request.game_state.street,
    );

    let suggestion = Suggestion {
        equity,
        pot_odds,
        recommendation,
        confidence,
        reasoning,
        calc_time_ms,
        suggested_bet_size,
        hand_category,
        gto_hints: crate::gto::GtoHints { // placeholder — filled below
            mdf: 0.0, raise_freq: 0.0, call_freq: 0.0, fold_freq: 0.0, bluff_ratio: None,
        },
    };
    let gto_hints = crate::gto::GtoHints::calculate(&request.game_state, &suggestion);
    Ok(Suggestion { gto_hints, ..suggestion })
}
