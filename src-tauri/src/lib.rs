/// Poker Postflop Solver — Tauri backend
///
/// Exposes `solve_postflop` command that takes a manually entered scenario
/// and returns a GTO recommendation using the equity-engine postflop modules.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Result types (mirror types.ts on the frontend)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct BoardTextureInfo {
    pub is_monotone: bool,
    pub is_paired: bool,
    pub has_straight_draw: bool,
    pub has_flush_draw: bool,
    pub texture_label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PostflopResult {
    pub equity: f64,
    pub pot_odds: f64,
    pub spr: f64,
    pub action: String,
    pub sizing_bb: Option<f64>,
    pub sizing_pct_pot: Option<f64>,
    pub ev_estimate: f64,
    pub reasoning: String,
    pub board_texture: BoardTextureInfo,
}

// ---------------------------------------------------------------------------
// Card parsing helpers
// ---------------------------------------------------------------------------

fn parse_card(s: &str) -> Result<equity_engine::Card, String> {
    let s = s.trim();
    if s.len() < 2 { return Err(format!("Invalid card: '{s}'")); }
    let chars: Vec<char> = s.chars().collect();
    let rank = match chars[0] {
        '2' => equity_engine::Rank::Two,   '3' => equity_engine::Rank::Three,
        '4' => equity_engine::Rank::Four,  '5' => equity_engine::Rank::Five,
        '6' => equity_engine::Rank::Six,   '7' => equity_engine::Rank::Seven,
        '8' => equity_engine::Rank::Eight, '9' => equity_engine::Rank::Nine,
        'T' => equity_engine::Rank::Ten,   'J' => equity_engine::Rank::Jack,
        'Q' => equity_engine::Rank::Queen, 'K' => equity_engine::Rank::King,
        'A' => equity_engine::Rank::Ace,
        c => return Err(format!("Unknown rank '{c}'")),
    };
    let suit = match chars[1] {
        'h' => equity_engine::Suit::Hearts,  'd' => equity_engine::Suit::Diamonds,
        'c' => equity_engine::Suit::Clubs,   's' => equity_engine::Suit::Spades,
        c => return Err(format!("Unknown suit '{c}'")),
    };
    Ok(equity_engine::Card { rank, suit })
}

// ---------------------------------------------------------------------------
// Villain range string expansion
// ---------------------------------------------------------------------------

/// Expand shorthand ranges like "top10%" to a hand range string.
/// Falls back to the raw string if not a shorthand.
fn expand_range_shorthand(s: &str) -> String {
    let s = s.trim().to_lowercase();
    match s.as_str() {
        "top5%"  => "AA,KK,QQ,JJ,AKs,AKo,AQs".to_string(),
        "top10%" => "AA,KK,QQ,JJ,TT,AKs,AKo,AQs,AQo,AJs,KQs".to_string(),
        "top15%" => "AA,KK,QQ,JJ,TT,99,AKs,AKo,AQs,AQo,AJs,ATs,KQs,KJs,QJs".to_string(),
        "top20%" => "AA,KK,QQ,JJ,TT,99,88,AKs,AKo,AQs,AQo,AJs,AJo,ATs,KQs,KJs,KTs,QJs,JTs".to_string(),
        "top30%" => "AA,KK,QQ,JJ,TT,99,88,77,66,AKs,AKo,AQs,AQo,AJs,AJo,ATs,ATo,A9s,KQs,KJs,KTs,KQo,QJs,QTs,JTs,T9s,98s".to_string(),
        _ => s.clone(), // treat as literal range string
    }
}

// ---------------------------------------------------------------------------
// Core solve command
// ---------------------------------------------------------------------------

/// Compute a GTO postflop recommendation for a manually entered scenario.
#[tauri::command]
fn solve_postflop(
    hero_hand: Vec<String>,
    board: Vec<String>,
    pot_bb: f64,
    hero_stack_bb: f64,
    villain_stack_bb: f64,
    to_call_bb: f64,
    is_ip: bool,
    villain_range: String,
) -> Result<PostflopResult, String> {
    // Validate
    if hero_hand.len() != 2 {
        return Err(format!("Hero hand must have exactly 2 cards, got {}", hero_hand.len()));
    }
    if board.len() < 3 || board.len() > 5 {
        return Err(format!("Board must have 3–5 cards, got {}", board.len()));
    }

    // Parse cards
    let h1 = parse_card(&hero_hand[0])?;
    let h2 = parse_card(&hero_hand[1])?;
    let board_cards: Vec<equity_engine::Card> = board.iter()
        .map(|s| parse_card(s))
        .collect::<Result<Vec<_>, _>>()?;

    // Build villain range
    let range_str = expand_range_shorthand(&villain_range);
    let mut villain_range_obj = equity_engine::range::Range::from_str(&range_str)
        .map_err(|e| format!("Invalid villain range: {e}"))?;

    // Remove blockers (hero cards + board)
    let mut known_cards = vec![h1, h2];
    known_cards.extend_from_slice(&board_cards);
    villain_range_obj.remove_blockers(&known_cards);

    if villain_range_obj.is_empty() {
        return Err("Villain range is empty after removing blockers".to_string());
    }

    // Determine position (IP = BTN, OOP = BB as rough approximation)
    let position = if is_ip { equity_engine::Position::BTN } else { equity_engine::Position::BB };

    // Determine street from board length
    let street = match board_cards.len() {
        3 => equity_engine::Street::Flop,
        4 => equity_engine::Street::Turn,
        5 => equity_engine::Street::River,
        _ => equity_engine::Street::Flop,
    };

    // Run recommendation
    let decision = equity_engine::postflop_solver::recommend(
        (h1, h2),
        position,
        &villain_range_obj,
        &board_cards,
        pot_bb,
        hero_stack_bb,
        villain_stack_bb,
        to_call_bb,
        street,
    );

    // Build board texture info
    let texture = equity_engine::postflop::BoardTexture::analyze(&board_cards);
    let texture_label = {
        let mut parts: Vec<&str> = Vec::new();
        if texture.is_monotone { parts.push("Monotone"); }
        else if texture.flush_draw_possible { parts.push("Flush Draw"); }
        if texture.is_paired { parts.push("Paired"); }
        if texture.straight_draw_possible { parts.push("Straight Draw"); }
        if parts.is_empty() { "Dry".to_string() } else { parts.join(", ") }
    };

    let spr = if pot_bb > 0.0 {
        hero_stack_bb.min(villain_stack_bb) / pot_bb
    } else { 99.0 };

    let pot_odds_pct = if to_call_bb > 0.0 {
        to_call_bb / (pot_bb + to_call_bb) * 100.0
    } else { 0.0 };

    let action_str = match &decision.action {
        equity_engine::postflop_solver::PostflopAction::Fold    => "FOLD",
        equity_engine::postflop_solver::PostflopAction::Check   => "CHECK",
        equity_engine::postflop_solver::PostflopAction::Call    => "CALL",
        equity_engine::postflop_solver::PostflopAction::Bet(_)  => "BET",
        equity_engine::postflop_solver::PostflopAction::Raise(_)=> "RAISE",
    }.to_string();

    Ok(PostflopResult {
        equity: (decision.equity * 1000.0).round() / 10.0,  // % with 1 decimal
        pot_odds: (pot_odds_pct * 10.0).round() / 10.0,
        spr: (spr * 10.0).round() / 10.0,
        action: action_str,
        sizing_bb: decision.sizing_bb.map(|v| (v * 10.0).round() / 10.0),
        sizing_pct_pot: decision.sizing_pct_pot.map(|v| (v * 100.0).round()),
        ev_estimate: (decision.ev_estimate * 10.0).round() / 10.0,
        reasoning: decision.reasoning,
        board_texture: BoardTextureInfo {
            is_monotone: texture.is_monotone,
            is_paired: texture.is_paired,
            has_straight_draw: texture.straight_draw_possible,
            has_flush_draw: texture.flush_draw_possible,
            texture_label,
        },
    })
}

// ---------------------------------------------------------------------------
// App entry point
// ---------------------------------------------------------------------------

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![solve_postflop])
        .run(tauri::generate_context!())
        .expect("error while running poker-postflop-solver");
}
