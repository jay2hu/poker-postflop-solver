/// GTO Wizard preflop strategy lookup from JSON files.
///
/// Files are stored in `strategies/gto_wizard/` relative to the equity-engine
/// crate root. Each JSON encodes per-hand action frequencies from GTO Wizard's
/// 8-player, 100bb stack preflop solver.
///
/// # File naming convention
/// - `opening_hero_{seat}_{pos}.json`          — hero opens (no bet facing)
/// - `vs_open_from_{villain_pos}_hero_{seat}_{pos}.json` — hero facing 1 open
/// - `vs_3bet_from_{villain_pos}_hero_{seat}_{pos}_open.json` — hero facing 3bet
///
/// # Position → seat mapping (8-max)
/// UTG=1, UTG+1=2, LJ=3, HJ=4, CO=5, BTN=6, SB=7, BB=8
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::OnceLock;

use serde::Deserialize;

use crate::{Card, Position, Rank, Suit};
use crate::preflop::{PreflopAction, PreflopDecision, classify, hand_description, HandCategory};

// ---------------------------------------------------------------------------
// JSON schema for strategy files
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct PlayerInfo {
    player: PlayerMeta,
    #[serde(default)]
    simple_hand_counters: HashMap<String, HandCounter>,
}

#[derive(Debug, Deserialize)]
struct PlayerMeta {
    is_hero: bool,
}

#[derive(Debug, Deserialize)]
struct HandCounter {
    #[serde(default)]
    actions_total_frequencies: HashMap<String, f64>,
}

#[derive(Debug, Deserialize)]
struct StrategyFile {
    players_info: Vec<PlayerInfo>,
}

// ---------------------------------------------------------------------------
// Strategy lookup
// ---------------------------------------------------------------------------

/// Per-hand action frequencies loaded from a strategy file.
#[derive(Debug, Clone)]
pub struct HandFrequencies {
    /// Fold frequency 0.0–1.0
    pub fold: f64,
    /// Call frequency (includes limp)
    pub call: f64,
    /// Raise/3bet frequency
    pub raise: f64,
}

impl HandFrequencies {
    /// Determine the recommended action (highest non-fold frequency wins).
    /// Ties broken by raise > call > fold.
    pub fn recommended_decision(&self) -> PreflopDecision {
        if self.raise > self.call && self.raise > self.fold {
            PreflopDecision::ThreeBet // covers open + 3bet
        } else if self.call > self.fold {
            PreflopDecision::Call
        } else {
            PreflopDecision::Fold
        }
    }
}

/// Loaded and cached strategy table: hand name → frequencies.
type StrategyTable = HashMap<String, HandFrequencies>;

/// Global strategy cache keyed by filename stem.
static STRATEGY_CACHE: OnceLock<std::sync::Mutex<HashMap<String, StrategyTable>>> =
    OnceLock::new();

fn cache() -> &'static std::sync::Mutex<HashMap<String, StrategyTable>> {
    STRATEGY_CACHE.get_or_init(|| std::sync::Mutex::new(HashMap::new()))
}

/// Load a strategy file from the strategies directory, using the in-memory cache.
fn load_strategy(file_stem: &str, strategies_dir: &PathBuf) -> Option<StrategyTable> {
    // Bail early if the directory doesn't exist — avoids returning a stale
    // cache hit when the caller deliberately passes a nonexistent path
    // (e.g. in tests that verify missing-file behaviour).
    if !strategies_dir.exists() {
        return None;
    }

    // Cache hit — key includes strategies_dir to isolate different stack depths
    let cache_key = format!("{}/{}", strategies_dir.display(), file_stem);
    {
        let guard = cache().lock().unwrap();
        if let Some(table) = guard.get(&cache_key) {
            return Some(table.clone());
        }
    }

    // Load from disk
    let path = strategies_dir.join(format!("{file_stem}.json"));
    let content = std::fs::read_to_string(&path).ok()?;
    let parsed: StrategyFile = serde_json::from_str(&content).ok()?;

    // Extract hero's hand counters
    let hero = parsed.players_info.into_iter().find(|p| p.player.is_hero)?;
    let table: StrategyTable = hero.simple_hand_counters
        .into_iter()
        .map(|(hand, counter)| {
            let freqs = &counter.actions_total_frequencies;
            let fold  = freqs.get("F").copied().unwrap_or(0.0);
            let call  = freqs.iter()
                .filter(|(k, _)| k.as_str() == "C")
                .map(|(_, v)| v)
                .sum::<f64>();
            let raise = freqs.iter()
                .filter(|(k, _)| k.starts_with('R') && k.as_str() != "RAI" || k.as_str() == "RAI")
                .map(|(_, v)| v)
                .sum::<f64>();
            (hand, HandFrequencies { fold, call, raise })
        })
        .collect();

    // Store in cache
    {
        let mut guard = cache().lock().unwrap();
        guard.insert(cache_key, table.clone());
    }

    Some(table)
}

// ---------------------------------------------------------------------------
// Position → seat mapping
// ---------------------------------------------------------------------------

fn position_seat(pos: Position) -> &'static str {
    match pos {
        Position::UTG  => "1_utg",
        Position::MP   => "2_utg+1",
        Position::UTG1 => "2_utg+1",
        Position::UTG2 => "3_lj",
        Position::LJ   => "3_lj",
        Position::HJ   => "4_hj",
        Position::CO   => "5_co",
        Position::BTN  => "6_btn",
        Position::SB   => "7_sb",
        Position::BB   => "8_bb",
    }
}

fn position_name_for_file(pos: Position) -> &'static str {
    match pos {
        Position::UTG  => "utg",
        Position::MP   => "utg+1",
        Position::UTG1 => "utg+1",
        Position::UTG2 => "lj",
        Position::LJ   => "lj",
        Position::HJ   => "hj",
        Position::CO   => "co",
        Position::BTN  => "btn",
        Position::SB   => "sb",
        Position::BB   => "bb",
    }
}

// ---------------------------------------------------------------------------
// Hand category → GTO Wizard hand name
// ---------------------------------------------------------------------------

/// Convert our `HandCategory` to GTO Wizard's hand name format.
/// Examples: Pair(Ace)→"AA", Suited(Ace,King)→"AKs", Offsuit(Seven,Two)→"72o"
fn hand_to_gto_name(hand: &HandCategory) -> String {
    fn rank_char(r: Rank) -> char {
        match r {
            Rank::Two => '2', Rank::Three => '3', Rank::Four => '4',
            Rank::Five => '5', Rank::Six => '6', Rank::Seven => '7',
            Rank::Eight => '8', Rank::Nine => '9', Rank::Ten => 'T',
            Rank::Jack => 'J', Rank::Queen => 'Q', Rank::King => 'K',
            Rank::Ace => 'A',
        }
    }
    match hand {
        HandCategory::Pair(r) => format!("{0}{0}", rank_char(*r)),
        HandCategory::Suited(hi, lo) => format!("{}{}s", rank_char(*hi), rank_char(*lo)),
        HandCategory::Offsuit(hi, lo) => format!("{}{}o", rank_char(*hi), rank_char(*lo)),
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Look up GTO Wizard preflop strategy for the given situation.
///
/// Map effective stack in big blinds to the strategy directory suffix.
///
/// GTO Wizard stack depths available: 5, 10, 20, 30, 40, 60, 80, 100bb.
pub fn stack_bucket(hero_stack_bb: f64) -> &'static str {
    match hero_stack_bb as u32 {
        0..=7    => "5stack",
        8..=15   => "10stack",
        16..=25  => "20stack",
        26..=35  => "30stack",
        36..=50  => "40stack",
        51..=70  => "60stack",
        71..=90  => "80stack",
        _        => "100stack",
    }
}

/// `strategies_dir` — path to the `strategies/gto_wizard/` directory.
/// `facing_open_from` — position of the player who opened (None if first to act).
/// `hero_stack_bb` — hero's effective stack in big blinds (used to pick depth dir).
///
/// Returns `None` if no matching strategy file is found.
pub fn gto_preflop_action(
    c1: &Card,
    c2: &Card,
    hero_pos: Position,
    facing_open_from: Option<Position>,
    strategies_dir: &PathBuf,
    hero_stack_bb: f64,
) -> Option<PreflopAction> {
    // Select the stack-depth sub-directory
    let bucket = stack_bucket(hero_stack_bb);
    // gto_wizard_100stack is in strategies/gto_wizard/ (the root),
    // others are in strategies/gto_wizard_{bucket}/
    let depth_dir = if bucket == "100stack" {
        strategies_dir.clone()
    } else {
        // parent of gto_wizard dir + gto_wizard_{bucket}
        let parent = strategies_dir.parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| strategies_dir.clone());
        parent.join(format!("gto_wizard_{bucket}"))
    };

    let hand = classify(c1, c2);
    let hand_name = hand_to_gto_name(&hand);
    let hand_desc = hand_description(&hand);
    let hero_seat = position_seat(hero_pos);

    // Select the right file
    let file_stem = match facing_open_from {
        None => {
            // Opening — use `opening_hero_{seat}.json`
            format!("opening_hero_{hero_seat}")
        }
        Some(villain_pos) => {
            // Facing an open — use `vs_open_from_{villain}_hero_{seat}.json`
            let villain = position_name_for_file(villain_pos);
            format!("vs_open_from_{villain}_hero_{hero_seat}")
        }
    };

    let table = load_strategy(&file_stem, &depth_dir)?;
    let freqs = table.get(&hand_name)?;
    let decision = freqs.recommended_decision();

    let pos_name = match hero_pos {
        Position::BTN  => "BTN", Position::CO  => "CO",
        Position::MP   => "MP",  Position::UTG => "UTG",
        Position::SB   => "SB",  Position::BB  => "BB",
        Position::HJ   => "HJ",  Position::LJ  => "LJ",
        Position::UTG1 => "UTG+1", Position::UTG2 => "UTG+2",
    };

    let action_str = match &decision {
        PreflopDecision::Open | PreflopDecision::ThreeBet => {
            if facing_open_from.is_some() { "3-bet" } else { "open" }
        }
        PreflopDecision::Call => "call",
        PreflopDecision::Fold => "fold",
    };

    let reasoning = format!(
        "{hand_desc} — GTO Wizard: {pos_name} {action_str} [{:.0}% raise / {:.0}% call / {:.0}% fold]",
        freqs.raise * 100.0,
        freqs.call  * 100.0,
        freqs.fold  * 100.0,
    );

    // Map ThreeBet → Open or ThreeBet depending on context
    let final_decision = if facing_open_from.is_none() {
        match decision {
            PreflopDecision::ThreeBet => PreflopDecision::Open,
            other => other,
        }
    } else {
        decision
    };

    Some(PreflopAction { decision: final_decision, reasoning })
}

/// Locate the strategies directory relative to the equity-engine crate root.
/// Returns the `strategies/gto_wizard` path if it exists, else None.
pub fn default_strategies_dir() -> Option<PathBuf> {
    // Try relative paths from likely working directories
    let candidates = [
        PathBuf::from("strategies/gto_wizard"),
        PathBuf::from("equity-engine/strategies/gto_wizard"),
        PathBuf::from("../equity-engine/strategies/gto_wizard"),
    ];
    candidates.into_iter().find(|p| p.exists())
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

    fn strategies_dir() -> Option<PathBuf> {
        default_strategies_dir()
    }

    #[test]
    fn hand_to_gto_name_conversions() {
        use crate::preflop::classify;
        let aa = classify(&card(Rank::Ace, Suit::Hearts), &card(Rank::Ace, Suit::Spades));
        assert_eq!(hand_to_gto_name(&aa), "AA");

        let aks = classify(&card(Rank::Ace, Suit::Hearts), &card(Rank::King, Suit::Hearts));
        assert_eq!(hand_to_gto_name(&aks), "AKs");

        let sevtwo = classify(&card(Rank::Seven, Suit::Hearts), &card(Rank::Two, Suit::Clubs));
        assert_eq!(hand_to_gto_name(&sevtwo), "72o");
    }

    #[test]
    fn btn_opens_ak() {
        let dir = match strategies_dir() { Some(d) => d, None => return };
        let result = gto_preflop_action(
            &card(Rank::Ace, Suit::Hearts),
            &card(Rank::King, Suit::Clubs),
            Position::BTN, None, &dir,
            100.0,
        );
        let action = result.expect("BTN AKo should have GTO entry");
        assert!(
            matches!(action.decision, PreflopDecision::Open),
            "BTN AKo should open, got {:?}: {}", action.decision, action.reasoning
        );
    }

    #[test]
    fn utg_folds_72o() {
        let dir = match strategies_dir() { Some(d) => d, None => return };
        let result = gto_preflop_action(
            &card(Rank::Seven, Suit::Hearts),
            &card(Rank::Two, Suit::Clubs),
            Position::UTG, None, &dir,
            100.0,
        );
        let action = result.expect("UTG 72o should have GTO entry");
        assert!(
            matches!(action.decision, PreflopDecision::Fold),
            "UTG 72o should fold, got {:?}: {}", action.decision, action.reasoning
        );
    }

    #[test]
    fn btn_calls_facing_co_open_with_pocket_pair() {
        let dir = match strategies_dir() { Some(d) => d, None => return };
        let result = gto_preflop_action(
            &card(Rank::Seven, Suit::Hearts),
            &card(Rank::Seven, Suit::Clubs),
            Position::BTN, Some(Position::CO), &dir,
            100.0,
        );
        let action = result.expect("BTN 77 vs CO open should have entry");
        // 77 vs CO open — GTO should call or raise, not fold
        assert!(
            !matches!(action.decision, PreflopDecision::Fold),
            "BTN 77 vs CO open should not fold: {:?}", action.decision
        );
    }

    #[test]
    fn missing_file_returns_none() {
        // Use UTG+1 — no strategy file exists for that position key in any
        // test run, AND we pass a nonexistent directory, so the cache can
        // never have a hit for this lookup regardless of test order.
        // Position::UTG maps to hero seat 1 (file stem opening_hero_1_utg);
        // the nonexistent dir guarantees the file can't be read.
        let dir = PathBuf::from("/tmp/nonexistent_strategies_dir_xyz_abc_unique");
        let result = gto_preflop_action(
            &card(Rank::Two, Suit::Clubs),
            &card(Rank::Seven, Suit::Diamonds),
            Position::UTG, None, &dir,
            100.0,
        );
        assert!(result.is_none(), "missing dir should return None");
    }

    #[test]
    fn hand_frequencies_recommended_decision() {
        let raise = HandFrequencies { raise: 0.8, call: 0.1, fold: 0.1 };
        assert!(matches!(raise.recommended_decision(), PreflopDecision::ThreeBet));

        let call = HandFrequencies { raise: 0.1, call: 0.8, fold: 0.1 };
        assert!(matches!(call.recommended_decision(), PreflopDecision::Call));

        let fold = HandFrequencies { raise: 0.1, call: 0.1, fold: 0.8 };
        assert!(matches!(fold.recommended_decision(), PreflopDecision::Fold));
    }
}

/// Look up a scenario by key and return {hand_str -> {action -> freq}} map.
/// Used by `range.rs` to build ranges from GTO data.
pub fn lookup_by_scenario(
    scenario_key: &str,
    strategies_dir: &PathBuf,
    stack_bb: u32,
) -> Option<std::collections::HashMap<String, std::collections::HashMap<String, f64>>> {
    let bucket = stack_bucket(stack_bb as f64);
    let depth_dir = if bucket == "100stack" {
        strategies_dir.clone()
    } else {
        let parent = strategies_dir.parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| strategies_dir.clone());
        parent.join(format!("gto_wizard_{bucket}"))
    };
    let table = load_strategy(scenario_key, &depth_dir)?;
    Some(table.into_iter().map(|(hand, freqs)| {
        let action_map: std::collections::HashMap<String, f64> = std::collections::HashMap::from([
            ("fold".to_string(),  freqs.fold),
            ("call".to_string(),  freqs.call),
            ("raise".to_string(), freqs.raise),
        ]);
        (hand, action_map)
    }).collect())
}
