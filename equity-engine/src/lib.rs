use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Card primitives — mirror the TypeScript Card interface in SPEC.md §8
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Suit {
    #[serde(rename = "h")] Hearts,
    #[serde(rename = "d")] Diamonds,
    #[serde(rename = "c")] Clubs,
    #[serde(rename = "s")] Spades,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Rank {
    #[serde(rename = "2")] Two,
    #[serde(rename = "3")] Three,
    #[serde(rename = "4")] Four,
    #[serde(rename = "5")] Five,
    #[serde(rename = "6")] Six,
    #[serde(rename = "7")] Seven,
    #[serde(rename = "8")] Eight,
    #[serde(rename = "9")] Nine,
    #[serde(rename = "T")] Ten,
    #[serde(rename = "J")] Jack,
    #[serde(rename = "Q")] Queen,
    #[serde(rename = "K")] King,
    #[serde(rename = "A")] Ace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Card {
    pub rank: Rank,
    pub suit: Suit,
}

// ---------------------------------------------------------------------------
// Street
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Street {
    Preflop,
    Flop,
    Turn,
    River,
}

// ---------------------------------------------------------------------------
// Position
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Position {
    BTN, SB, BB, UTG, MP, CO,
    /// UTG+1 (8-max / 9-max specific)
    UTG1,
    /// UTG+2 (9-max specific)
    UTG2,
    /// Lojack / Middle position (8-max 5th seat from BTN)
    LJ,
    /// Hijack (one before CO)
    HJ,
}

// ---------------------------------------------------------------------------
// GameState — mirrors SPEC.md §8 GameState interface
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameState {
    pub street: Street,
    pub hero_cards: [Card; 2],
    pub board_cards: Vec<Card>,
    pub pot_size: f64,
    pub current_bet: f64,
    pub hero_stack: f64,
    pub villain_stack: f64,
    pub position: Position,
    pub to_act: bool,
    /// Number of active players (including hero). Valid range: 2–9. Default: 2 (heads-up).
    #[serde(default = "default_player_count")]
    pub player_count: u8,
    /// Hand context for villain range narrowing.
    #[serde(default)]
    pub context: HandContext,
    /// Per-seat opponent statistics gathered across the session.
    #[serde(default)]
    pub opponent_stats: Vec<OpponentStats>,
    /// Optional table identifier (window handle string) for multi-table support.
    #[serde(default)]
    pub table_id: Option<String>,
    /// Big blind size in chips. Used to compute effective stack depth in BBs.
    /// Default 1.0 (unknown); populated from hand history or OCR.
    #[serde(default = "default_big_blind")]
    pub big_blind_size: f64,
    /// Per-seat villain stacks keyed by seat number string ("1".."9").
    #[serde(default)]
    pub villain_stacks: std::collections::HashMap<String, f64>,
    /// 1-based dealer seat number (detected from dealer button position).
    #[serde(default)]
    pub dealer_seat: u8,
    /// Current outstanding bet/raise size that hero must call or raise against.
    #[serde(default)]
    pub active_bet_size: Option<f64>,
    /// Whether hero's action buttons are currently visible (fold/call/raise shown).
    #[serde(default)]
    pub hero_to_act: bool,
    /// EV for hero's hole cards from the v2 strategy lookup (bb units).
    #[serde(default)]
    pub strategy_ev: Option<f64>,
    /// Pot odds from the matched v2 strategy scenario (0.0–1.0).
    #[serde(default)]
    pub strategy_pot_odds: Option<f64>,
    /// Recommended bet size from strategy (bb units).
    #[serde(default)]
    pub strategy_betsize_bb: Option<f64>,
    /// Recommended bet size as % of pot.
    #[serde(default)]
    pub strategy_betsize_pct_pot: Option<f64>,
}

fn default_player_count() -> u8 { 2 }
fn default_big_blind() -> f64 { 1.0 }

// ---------------------------------------------------------------------------
// Player archetype classification
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PlayerArchetype {
    /// Very loose, passive — VPIP > 40%
    Fish,
    /// Loose, passive — VPIP > 35%, PFR < 10%
    CallingStation,
    /// Very tight — VPIP < 18%
    Nit,
    /// Tight-aggressive — VPIP 18–28%, PFR 14–22%
    Tag,
    /// Loose-aggressive — VPIP > 28%, PFR > 20%
    Lag,
    /// Insufficient data (< 20 hands observed)
    Unknown,
}

impl Default for PlayerArchetype {
    fn default() -> Self { PlayerArchetype::Unknown }
}

// ---------------------------------------------------------------------------
// Per-opponent statistics
// ---------------------------------------------------------------------------

/// Accumulated statistics for one opponent seat, tracked across the session.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct OpponentStats {
    /// Seat ID or anonymized player name.
    pub player_id: String,
    /// Total hands where this opponent was dealt in.
    pub hands_seen: u32,
    /// Hands where opponent voluntarily put money in preflop (VPIP numerator).
    pub vpip: u32,
    /// Hands where opponent raised preflop (PFR numerator).
    pub pfr: u32,
    /// 3-bets made preflop.
    pub three_bet: u32,
    /// Times opponent folded to a 3-bet.
    pub fold_to_3bet: u32,
    /// Postflop aggressive actions (bets/raises).
    pub aggression_count: u32,
    /// Postflop passive actions (checks/calls).
    pub check_call_count: u32,
}

impl OpponentStats {
    /// VPIP as fraction 0.0–1.0. Returns 0.0 if no hands seen.
    pub fn vpip_pct(&self) -> f64 {
        if self.hands_seen == 0 { return 0.0; }
        self.vpip as f64 / self.hands_seen as f64
    }

    /// PFR as fraction 0.0–1.0.
    pub fn pfr_pct(&self) -> f64 {
        if self.hands_seen == 0 { return 0.0; }
        self.pfr as f64 / self.hands_seen as f64
    }

    /// 3-bet percentage as fraction 0.0–1.0.
    pub fn three_bet_pct(&self) -> f64 {
        if self.hands_seen == 0 { return 0.0; }
        self.three_bet as f64 / self.hands_seen as f64
    }

    /// Aggression Factor = (bets + raises) / calls. Returns 0.0 when no passive actions.
    pub fn af(&self) -> f64 {
        if self.check_call_count == 0 { return self.aggression_count as f64; }
        self.aggression_count as f64 / self.check_call_count as f64
    }

    /// Classify this opponent into a play style archetype.
    /// Returns `Unknown` until 20+ hands are observed.
    pub fn archetype(&self) -> PlayerArchetype {
        if self.hands_seen < 20 {
            return PlayerArchetype::Unknown;
        }
        let vpip = self.vpip_pct();
        let pfr  = self.pfr_pct();

        if vpip > 0.35 && pfr < 0.10 {
            PlayerArchetype::CallingStation
        } else if vpip > 0.40 {
            PlayerArchetype::Fish
        } else if vpip < 0.18 {
            PlayerArchetype::Nit
        } else if vpip > 0.28 && pfr > 0.20 {
            PlayerArchetype::Lag
        } else if vpip >= 0.18 && vpip <= 0.28 && pfr >= 0.14 && pfr <= 0.22 {
            PlayerArchetype::Tag
        } else {
            PlayerArchetype::Unknown
        }
    }
}

/// Accumulated hand context for villain range selection.
/// Parsed from the hand history actions; defaults to no preflop aggression.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct HandContext {
    /// Villain raised preflop (at least one raise before the flop)
    pub preflop_raise_seen: bool,
    /// A 3-bet was made preflop
    pub preflop_3bet_seen: bool,
    /// Villain was the last aggressor on the current street
    pub villain_aggressor: bool,
    /// How many streets have been played (0=preflop, 1=flop, 2=turn, 3=river)
    pub streets_seen: u8,
    /// Number of times villain has bet or raised across all streets this hand.
    /// Used to narrow range on persistent multi-street aggression.
    #[serde(default)]
    pub villain_bet_count: u8,
    /// A 4-bet was made preflop (3 or more raises — open + 3bet + 4bet).
    /// 4bet range is very tight: AA/KK/AK and bluffs only.
    #[serde(default)]
    pub preflop_4bet_seen: bool,
    /// Squeeze situation: a 3bet after one or more callers preflop.
    /// Squeeze range is wider than a 3bet in a heads-up pot.
    #[serde(default)]
    pub squeeze_situation: bool,
}

// ---------------------------------------------------------------------------
// Board texture
// ---------------------------------------------------------------------------

/// Structural characteristics of the community cards.
/// Computed from board cards; used by suggestion logic to adjust thresholds.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BoardTexture {
    /// 2+ cards of the same suit on the board (flush draw possible)
    pub flush_draw_possible: bool,
    /// 2+ connected cards within 4 ranks (straight draw possible)
    pub straight_draw_possible: bool,
    /// 2 cards of the same rank on the board (paired board)
    pub paired_board: bool,
    /// All 3 flop cards are the same suit
    pub monotone: bool,
    /// Wet: flush draw OR straight draw possible
    pub wet: bool,
    /// Dry: not wet AND not paired
    pub dry: bool,
}

impl BoardTexture {
    /// Analyse a board (typically 3–5 cards) and return its texture.
    pub fn from_cards(board: &[Card]) -> Self {
        if board.is_empty() {
            return Self::default();
        }

        // Paired board: any two cards share a rank
        let paired_board = {
            let mut seen = std::collections::HashMap::new();
            board.iter().any(|c| {
                let count = seen.entry(c.rank as u8).or_insert(0u8);
                *count += 1;
                *count >= 2
            })
        };

        // Flush draw: 2+ cards of same suit
        let flush_draw_possible = {
            let suit_idx = |s: Suit| match s {
                Suit::Hearts => 0usize, Suit::Diamonds => 1,
                Suit::Clubs => 2, Suit::Spades => 3,
            };
            let mut suit_counts = [0u8; 4];
            for c in board {
                suit_counts[suit_idx(c.suit)] += 1;
            }
            suit_counts.iter().any(|&n| n >= 2)
        };

        // Monotone: all 3+ cards same suit (only meaningful for 3+ card boards)
        let monotone = board.len() >= 3 && {
            let first_suit = board[0].suit;
            board.iter().all(|c| c.suit == first_suit)
        };

        // Straight draw: any two cards within 4 ranks of each other
        let straight_draw_possible = {
            let mut ranks: Vec<u8> = board.iter().map(|c| rank_value(c.rank)).collect();
            // Ace can act as 1 for wheel draws
            if ranks.contains(&14) {
                ranks.push(1);
            }
            ranks.sort_unstable();
            ranks.dedup();
            ranks.windows(2).any(|w| w[1] - w[0] <= 4)
        };

        let wet = flush_draw_possible || straight_draw_possible;
        let dry = !wet && !paired_board;

        BoardTexture {
            flush_draw_possible,
            straight_draw_possible,
            paired_board,
            monotone,
            wet,
            dry,
        }
    }
}

fn rank_value(r: Rank) -> u8 {
    match r {
        Rank::Two => 2,   Rank::Three => 3,  Rank::Four => 4,
        Rank::Five => 5,  Rank::Six => 6,    Rank::Seven => 7,
        Rank::Eight => 8, Rank::Nine => 9,   Rank::Ten => 10,
        Rank::Jack => 11, Rank::Queen => 12, Rank::King => 13,
        Rank::Ace => 14,
    }
}

// ---------------------------------------------------------------------------
// Range profile — JSON-based, one per street
// ---------------------------------------------------------------------------

/// A range is a list of hand combos the villain can hold, each with a weight
/// (0.0–1.0, representing how often they play that combo in this range).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandCombo {
    /// Two-card notation e.g. "AhKd", "TsTc"
    pub combo: String,
    /// Frequency weight (default 1.0 = always in range)
    #[serde(default = "default_weight")]
    pub weight: f64,
}

fn default_weight() -> f64 { 1.0 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RangeProfile {
    pub name: String,           // "tight" | "default" | "loose"
    pub street: Street,
    pub combos: Vec<HandCombo>,
}

// ---------------------------------------------------------------------------
// Suggestion output — mirrors SPEC.md §8 Suggestion interface
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Recommendation {
    Fold,
    Call,
    Raise,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Confidence {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Suggestion {
    /// Hero equity vs villain range (0.0–1.0)
    pub equity: f64,
    /// Pot odds required to call (0.0–1.0)
    pub pot_odds: f64,
    pub recommendation: Recommendation,
    pub confidence: Confidence,
    /// Human-readable explanation for the HUD
    pub reasoning: String,
    /// Actual calculation time for perf monitoring
    pub calc_time_ms: u64,
    /// Recommended bet size in chips when recommendation is RAISE; None for CALL/FOLD.
    pub suggested_bet_size: Option<f64>,
    /// GTO approximation hints (MDF, action frequencies, bluff ratio).
    pub gto_hints: crate::gto::GtoHints,
    /// Made hand or best draw category (e.g. "Two Pair", "Flush Draw", "OESD").
    /// None on preflop.
    pub hand_category: Option<String>,
}

// ---------------------------------------------------------------------------
// Equity engine request — Tauri command input
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EquityRequest {
    pub game_state: GameState,
    pub range_profile: RangeProfile,
    /// Monte Carlo simulation count — default 10_000
    #[serde(default = "default_simulations")]
    pub simulations: u32,
}

fn default_simulations() -> u32 { 10_000 }

// ---------------------------------------------------------------------------
// Core engine (stub — Monte Carlo implementation next)
// ---------------------------------------------------------------------------

pub mod engine;
pub mod suggestion;
pub mod ranges;
pub mod commands;
pub mod notation;
pub mod preflop;
pub mod gto;
pub mod gto_strategy;
pub mod strategy_v2;
pub mod position_engine;
pub mod range;
pub mod postflop;
pub mod postflop_solver;
pub mod hand_eval;
#[cfg(feature = "gto-solver")]
pub mod gto_solver;
