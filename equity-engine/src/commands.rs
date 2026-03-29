use crate::{engine, gto_strategy, preflop, ranges, EquityRequest, GameState, OpponentStats, PlayerArchetype, Recommendation, Street, Suggestion};

#[cfg(feature = "tauri")]
use crate::{RangeProfile};

// ---------------------------------------------------------------------------
// Non-tauri-gated pipeline helper — callable from integration tests
// ---------------------------------------------------------------------------

/// Returns the simulation count the pipeline will use for the given street.
/// Exposed for testing — allows asserting adaptive sim counts without timing.
pub fn sim_count_for_street(street: Street) -> u32 {
    match street {
        Street::Preflop => 5_000,
        Street::Flop    => 15_000,
        Street::Turn    => 20_000,
        Street::River   => 25_000,
    }
}

// ---------------------------------------------------------------------------
// Non-tauri-gated pipeline helper — callable from integration tests
// ---------------------------------------------------------------------------

/// Load range profile by name and run the full equity pipeline.
/// For Preflop, overrides recommendation/reasoning from the opening chart
/// while keeping the equity value from Monte Carlo.
/// Returns `Err` on invalid range name or simulation failure.
/// Derive villain range name from HandContext.
/// Preflop raise or 3bet seen → tighter villain range.
/// Villain was aggressor on current street → tight.
/// Derive villain range name from HandContext.
///
/// Single canonical mapping used by suggestion_pipeline — engine.rs no longer
/// does its own range selection (it uses the profile passed by the pipeline).
/// Logic matches engine.rs's previous internal mapping, now centralised here.
/// Select villain range name using the priority chain:
///   1. OpponentStats archetype (most reliable)
///   2. HandContext explicit VPIP/PFR
///   3. Action-based (preflop aggression)
pub fn range_from_archetype(archetype: &PlayerArchetype) -> &'static str {
    match archetype {
        PlayerArchetype::Fish | PlayerArchetype::CallingStation => "loose",
        PlayerArchetype::Nit  => "tight",
        PlayerArchetype::Tag | PlayerArchetype::Lag | PlayerArchetype::Unknown => "default",
    }
}

pub fn range_from_stats(vpip: u32, pfr: u32) -> &'static str {
    if vpip > 40 { "loose" }
    else if vpip < 18 { "tight" }
    else if pfr > 20 { "default" }
    else { "default" }
}

pub fn exploit_note(archetype: &PlayerArchetype) -> &'static str {
    match archetype {
        PlayerArchetype::Fish           => " Fish — bet for value, avoid bluffing.",
        PlayerArchetype::CallingStation => " Calling station — bet large for value, no bluffs.",
        PlayerArchetype::Nit            => " Tight player — respect aggression, fold marginal hands.",
        PlayerArchetype::Lag            => " Aggressive player — widen calling range, consider trapping.",
        PlayerArchetype::Tag            => " TAG — standard adjustments apply.",
        PlayerArchetype::Unknown        => "",
    }
}

fn best_villain_stats(stats: &[OpponentStats]) -> Option<&OpponentStats> {
    stats.iter().filter(|s| s.hands_seen > 0).max_by_key(|s| s.hands_seen)
}

fn range_from_context<'a>(game_state: &GameState, base: &'a str) -> &'a str {
    if base != "default" { return base; }
    // Priority 1: archetype from opponent_stats
    if let Some(best) = best_villain_stats(&game_state.opponent_stats) {
        let arch = best.archetype();
        if arch != PlayerArchetype::Unknown { return range_from_archetype(&arch); }
    }
    // Priority 2: action-based (villain_bet_count + existing signals)
    let ctx = &game_state.context;
    // 4-bet seen → AA/KK/AK-only range → use tight
    // (we don't have a "4bet" range profile; tight is the narrowest we ship)
    if ctx.preflop_4bet_seen {
        return "tight";
    }
    // Multi-street aggression: villain bet/raised on 2+ streets → tight
    let persistent_aggressor = ctx.villain_bet_count >= 2;
    // Turn/river with any preflop raise → tight
    let late_raise = ctx.streets_seen >= 2 && ctx.preflop_raise_seen;
    if ctx.preflop_3bet_seen
        || persistent_aggressor
        || late_raise
        || (ctx.villain_aggressor && ctx.streets_seen >= 1)
    {
        "tight"
    } else {
        "default"
    }
}

pub fn suggestion_pipeline(game_state: GameState, range_name: &str) -> Result<Suggestion, String> {
    // Derive actual range from HandContext when using "default"
    let effective_range = range_from_context(&game_state, range_name);
    let profile = ranges::load_default_profile(game_state.street, effective_range)?;

    // Adaptive simulation count by street
    let simulations = match game_state.street {
        Street::Preflop => 5_000,
        Street::Flop    => 15_000,
        Street::Turn    => 20_000,
        Street::River   => 25_000,
    };

    let request = EquityRequest {
        game_state: game_state.clone(),
        range_profile: profile,
        simulations,
    };
    let mut suggestion = engine::calculate_equity(&request)?;

    // Preflop override: GTO Wizard strategy (if available) or static chart
    if matches!(game_state.street, Street::Preflop) {
        // Try GTO Wizard strategy files first (more accurate)
        let facing_open = if game_state.current_bet > 0.0 { None } else { None }; // TODO: derive villain position from context
        let bb = game_state.big_blind_size.max(0.01);
        let hero_stack_bb = game_state.hero_stack / bb;
        let pf = gto_strategy::default_strategies_dir()
            .and_then(|dir| gto_strategy::gto_preflop_action(
                &game_state.hero_cards[0],
                &game_state.hero_cards[1],
                game_state.position,
                facing_open,
                &dir,
                hero_stack_bb,
            ))
            // Fall back to static percentile chart
            .unwrap_or_else(|| preflop::preflop_action(
                &game_state.hero_cards[0],
                &game_state.hero_cards[1],
                game_state.position,
                game_state.player_count,
                game_state.current_bet,
            ));

        let rec = match pf.decision {
            preflop::PreflopDecision::Open | preflop::PreflopDecision::ThreeBet => Recommendation::Raise,
            preflop::PreflopDecision::Call => Recommendation::Call,
            preflop::PreflopDecision::Fold => Recommendation::Fold,
        };
        suggestion.recommendation = rec;
        suggestion.reasoning = format!(
            "{} [equity {:.1}%]",
            pf.reasoning,
            suggestion.equity * 100.0
        );
        suggestion.suggested_bet_size =
            crate::suggestion::suggested_bet_size(&suggestion.recommendation, &game_state);
    }

    // Append exploit note based on villain archetype (post-preflop reasoning too)
    if let Some(villain) = best_villain_stats(&game_state.opponent_stats) {
        let arch = villain.archetype();
        let note = exploit_note(&arch);
        if !note.is_empty() {
            suggestion.reasoning.push_str(note);
        }
    }

    Ok(suggestion)
}

// ---------------------------------------------------------------------------
// Tauri commands — delegate to suggestion_pipeline
// ---------------------------------------------------------------------------

/// Primary Tauri command — accepts full game state, uses default range profile.
#[cfg(feature = "tauri")]
#[tauri::command]
pub fn calculate_suggestion(game_state: GameState) -> Result<Suggestion, String> {
    suggestion_pipeline(game_state, "default")
}

/// Tauri command — calculate with an explicit range profile name.
#[cfg(feature = "tauri")]
#[tauri::command]
pub fn calculate_suggestion_with_range(
    game_state: GameState,
    range_name: String,
) -> Result<Suggestion, String> {
    suggestion_pipeline(game_state, &range_name)
}

/// Tauri command — load a range profile for display in the UI.
#[cfg(feature = "tauri")]
#[tauri::command]
pub fn load_range_profile(street: Street, name: String) -> Result<RangeProfile, String> {
    ranges::load_default_profile(street, &name)
}

/// Tauri command — list available built-in range profile names.
#[cfg(feature = "tauri")]
#[tauri::command]
pub fn list_range_profiles() -> Vec<&'static str> {
    ranges::available_profiles()
}
