/// Strategy v2 lookup — reads consolidated GTO Wizard v2 JSON files and
/// returns EV, pot odds, and recommended action for the current game state.
///
/// File location: `{Documents}/poker-strategy-viewer/strategies/v2/mmt_8p_{stack_bb}bb.json`
/// Default stack: 100bb (configurable via `STRATEGY_STACK_BB` env var).

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

use crate::{Card, Position, Rank, Street, Suit};

// ---------------------------------------------------------------------------
// v2 JSON structures (minimal — we only parse what we need)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
struct V2Action {
    total_freq: f64,
    betsize_bb: Option<f64>,
    betsize_pct_pot: Option<f64>,
    #[allow(dead_code)]
    group: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct V2Scenario {
    #[allow(dead_code)]
    description: Option<String>,
    #[allow(dead_code)]
    hero: Option<String>,
    #[allow(dead_code)]
    facing: Option<String>,
    pot_odds: Option<f64>,
    actions: HashMap<String, V2Action>,
    hands: HashMap<String, HashMap<String, f64>>,
}

#[derive(Debug, Clone, Deserialize)]
struct V2Meta {
    #[allow(dead_code)]
    stack_bb: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
struct V2File {
    #[allow(dead_code)]
    meta: V2Meta,
    scenarios: HashMap<String, V2Scenario>,
}

// ---------------------------------------------------------------------------
// Cache
// ---------------------------------------------------------------------------

static STRATEGY_CACHE: OnceLock<std::sync::Mutex<HashMap<u32, V2File>>> =
    OnceLock::new();

fn cache() -> &'static std::sync::Mutex<HashMap<u32, V2File>> {
    STRATEGY_CACHE.get_or_init(|| std::sync::Mutex::new(HashMap::new()))
}

fn strategy_file_path(stack_bb: u32) -> Option<PathBuf> {
    // Primary: Documents/poker-strategy-viewer/strategies/v2/
    let docs = dirs::document_dir()?
        .join("poker-strategy-viewer")
        .join("strategies")
        .join("v2")
        .join(format!("mmt_8p_{stack_bb}bb.json"));
    if docs.exists() { return Some(docs); }

    // Fallback: same dir as table_config.json
    let config_dir = dirs::config_dir()?
        .join("texas-holdem-rta")
        .join("strategies")
        .join("v2")
        .join(format!("mmt_8p_{stack_bb}bb.json"));
    if config_dir.exists() { return Some(config_dir); }

    None
}

fn load_strategy(stack_bb: u32) -> Option<()> {
    {
        let guard = cache().lock().unwrap();
        if guard.contains_key(&stack_bb) {
            return Some(());
        }
    }
    let path = strategy_file_path(stack_bb)?;
    let content = std::fs::read_to_string(&path).ok()?;
    let file: V2File = serde_json::from_str(&content).ok()?;
    {
        let mut guard = cache().lock().unwrap();
        guard.insert(stack_bb, file);
    }
    log::info!("[strategy-v2] loaded {stack_bb}bb from {path:?}");
    Some(())
}

// ---------------------------------------------------------------------------
// Position → scenario key derivation
// ---------------------------------------------------------------------------

fn pos_to_key(pos: Position) -> &'static str {
    match pos {
        Position::UTG  => "UTG",
        Position::UTG1 => "UTG1",
        Position::UTG2 => "UTG2",
        Position::LJ   => "LJ",
        Position::HJ   => "HJ",
        Position::MP   => "HJ",   // MP ≈ HJ in this context
        Position::CO   => "CO",
        Position::BTN  => "BTN",
        Position::SB   => "SB",
        Position::BB   => "BB",
    }
}

fn scenario_key_for_position(pos: Position) -> String {
    format!("{}_rfi", pos_to_key(pos))
}

// ---------------------------------------------------------------------------
// Hand → canonical key
// ---------------------------------------------------------------------------

fn hand_key(c1: &Card, c2: &Card) -> String {
    fn rank_char(r: Rank) -> char {
        match r {
            Rank::Two => '2', Rank::Three => '3', Rank::Four => '4',
            Rank::Five => '5', Rank::Six => '6', Rank::Seven => '7',
            Rank::Eight => '8', Rank::Nine => '9', Rank::Ten => 'T',
            Rank::Jack => 'J', Rank::Queen => 'Q', Rank::King => 'K',
            Rank::Ace => 'A',
        }
    }
    fn rank_val(r: Rank) -> u8 {
        match r {
            Rank::Two => 2, Rank::Three => 3, Rank::Four => 4,
            Rank::Five => 5, Rank::Six => 6, Rank::Seven => 7,
            Rank::Eight => 8, Rank::Nine => 9, Rank::Ten => 10,
            Rank::Jack => 11, Rank::Queen => 12, Rank::King => 13,
            Rank::Ace => 14,
        }
    }

    let (hi, lo) = if rank_val(c1.rank) >= rank_val(c2.rank) { (c1, c2) } else { (c2, c1) };

    if hi.rank == lo.rank {
        format!("{}{}", rank_char(hi.rank), rank_char(lo.rank))
    } else if hi.suit == lo.suit {
        format!("{}{}s", rank_char(hi.rank), rank_char(lo.rank))
    } else {
        format!("{}{}o", rank_char(hi.rank), rank_char(lo.rank))
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Result from strategy lookup.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StrategyLookup {
    pub pot_odds:            Option<f64>,
    pub ev:                  Option<f64>,
    pub best_action:         Option<String>,
    pub best_action_freq:    Option<f64>,
    pub betsize_bb:          Option<f64>,
    pub betsize_pct_pot:     Option<f64>,
}

/// Look up strategy data for the given position and hero cards.
///
/// Returns `None` if the strategy file is not loaded or the scenario is
/// not found (e.g. on the flop/turn/river where this is preflop-only).
pub fn lookup(
    position: Position,
    hero_cards: &[Card; 2],
    street: Street,
    stack_bb: u32,
) -> Option<StrategyLookup> {
    // Strategy lookup only makes sense preflop
    if !matches!(street, Street::Preflop) {
        return None;
    }

    load_strategy(stack_bb)?;

    let guard = cache().lock().unwrap();
    let file = guard.get(&stack_bb)?;

    let sc_key = scenario_key_for_position(position);
    let scenario = file.scenarios.get(&sc_key)?;

    let hk = hand_key(&hero_cards[0], &hero_cards[1]);
    let hand_freqs = scenario.hands.get(&hk);

    // EV: look for an 'ev' action key (may not be present in all sources)
    let ev = hand_freqs.and_then(|f| f.get("ev")).copied();

    // Best action (highest non-fold frequency)
    let (best_action, best_action_freq, betsize_bb, betsize_pct_pot) = {
        let mut best: Option<(String, f64, Option<f64>, Option<f64>)> = None;
        for (action_name, action_data) in &scenario.actions {
            if action_name.to_lowercase() == "fold" { continue; }
            let freq = action_data.total_freq;
            if best.as_ref().map(|(_, f, _, _)| freq > *f).unwrap_or(true) {
                best = Some((action_name.clone(), freq, action_data.betsize_bb, action_data.betsize_pct_pot));
            }
        }
        if let Some((a, f, bb, pct)) = best {
            (Some(a), Some(f), bb, pct)
        } else {
            (None, None, None, None)
        }
    };

    Some(StrategyLookup {
        pot_odds: scenario.pot_odds,
        ev,
        best_action,
        best_action_freq,
        betsize_bb,
        betsize_pct_pot,
    })
}

/// Determine stack depth bucket for strategy file selection.
pub fn stack_bucket_bb(hero_stack: f64, big_blind: f64) -> u32 {
    let bb = if big_blind > 0.0 { hero_stack / big_blind } else { 100.0 };
    match bb as u32 {
        0..=7   => 5,
        8..=15  => 10,
        16..=25 => 20,
        26..=35 => 30,
        36..=50 => 40,
        51..=70 => 60,
        71..=90 => 80,
        _       => 100,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hand_key_pair() {
        let c1 = Card { rank: Rank::Ace, suit: Suit::Hearts };
        let c2 = Card { rank: Rank::Ace, suit: Suit::Spades };
        assert_eq!(hand_key(&c1, &c2), "AA");
    }

    #[test]
    fn hand_key_suited() {
        let c1 = Card { rank: Rank::King, suit: Suit::Hearts };
        let c2 = Card { rank: Rank::Queen, suit: Suit::Hearts };
        assert_eq!(hand_key(&c1, &c2), "KQs");
    }

    #[test]
    fn hand_key_offsuit_orders_high_first() {
        let c1 = Card { rank: Rank::Two, suit: Suit::Hearts };
        let c2 = Card { rank: Rank::Seven, suit: Suit::Clubs };
        assert_eq!(hand_key(&c1, &c2), "72o");
    }

    #[test]
    fn scenario_key_btn() {
        assert_eq!(scenario_key_for_position(Position::BTN), "BTN_rfi");
    }

    #[test]
    fn scenario_key_utg1() {
        assert_eq!(scenario_key_for_position(Position::UTG1), "UTG1_rfi");
    }

    #[test]
    fn stack_bucket_100bb() {
        assert_eq!(stack_bucket_bb(100.0, 1.0), 100);
        assert_eq!(stack_bucket_bb(20.0, 1.0), 20);
        assert_eq!(stack_bucket_bb(7.0, 1.0), 5);
    }

    #[test]
    fn lookup_returns_none_postflop() {
        let cards = [
            Card { rank: Rank::Ace, suit: Suit::Hearts },
            Card { rank: Rank::King, suit: Suit::Diamonds },
        ];
        // No strategy file in test env — should return None cleanly
        let r = lookup(Position::BTN, &cards, Street::Flop, 100);
        assert!(r.is_none(), "postflop lookup should return None");
    }
}
