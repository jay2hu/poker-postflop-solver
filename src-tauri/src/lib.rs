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
    pub debug_steps: Vec<String>,
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

    let equity_pct = (decision.equity * 1000.0).round() / 10.0;
    let pot_odds_rounded = (pot_odds_pct * 10.0).round() / 10.0;
    let spr_rounded = (spr * 10.0).round() / 10.0;

    // Define action_str before debug_steps (used in both)
    let action_str = match &decision.action {
        equity_engine::postflop_solver::PostflopAction::Fold    => "FOLD",
        equity_engine::postflop_solver::PostflopAction::Check   => "CHECK",
        equity_engine::postflop_solver::PostflopAction::Call    => "CALL",
        equity_engine::postflop_solver::PostflopAction::Bet(_)  => "BET",
        equity_engine::postflop_solver::PostflopAction::Raise(_)=> "RAISE",
    }.to_string();

    // Build step-by-step debug explanation
    let mut debug_steps: Vec<String> = vec![
        format!("1. Street: {} (board has {} cards)", match board_cards.len() { 3=>"Flop", 4=>"Turn", _=>"River" }, board_cards.len()),
        format!("2. Position: {}", if is_ip { "IP (in position — act last)" } else { "OOP (out of position — act first)" }),
        format!("3. Villain range: \"{}\" → {} combos after blocking",
            villain_range.trim(),
            villain_range_obj.combos.len()),
        format!("4. Board texture: {} (monotone={}, paired={}, flush_draw={}, straight_draw={})",
            texture_label, texture.is_monotone, texture.is_paired,
            texture.flush_draw_possible, texture.straight_draw_possible),
        format!("5. Hero equity vs villain range: {:.1}% (Monte Carlo, {} iterations)",
            equity_pct,
            match board_cards.len() { 3=>3000, 4=>5000, _=>7000 }),
        format!("6. SPR = eff_stack / pot = {:.1} / {:.1} = {:.1}",
            hero_stack_bb.min(villain_stack_bb), pot_bb, spr_rounded),
    ];

    if to_call_bb > 0.0 {
        debug_steps.push(format!("7. Facing bet: to_call={:.1}bb — pot_odds = {:.1} / ({:.1} + {:.1}) = {:.1}%",
            to_call_bb, to_call_bb, pot_bb, to_call_bb, pot_odds_rounded));
        let threshold = pot_odds_rounded / 100.0;
        debug_steps.push(format!("8. Equity ({:.1}%) vs pot_odds ({:.1}%): {}",
            equity_pct, pot_odds_rounded,
            if decision.equity < threshold * 0.85 { "equity < 85% of required → FOLD" }
            else if decision.equity > threshold * 1.3 { "equity > 130% of required → RAISE" }
            else { "equity in call range → CALL" }));
    } else {
        debug_steps.push(format!("7. First to act (to_call=0) — deciding bet/check"));
        debug_steps.push(format!("8. Equity {:.1}%: {}",
            equity_pct,
            if decision.equity > 0.65 { ">65% → BET 65% pot (value)" }
            else if decision.equity > 0.52 { ">52% → BET 33% pot (thin value/block)" }
            else { "≤52% → CHECK" }));
    }

    if spr_rounded < 3.0 {
        debug_steps.push(format!("9. SPR {:.1} < 3 → shallow stack adjustment applied", spr_rounded));
    }

    debug_steps.push(format!("10. Decision: {} (EV estimate: {:.2}bb)", action_str, (decision.ev_estimate * 10.0).round() / 10.0));

    Ok(PostflopResult {
        equity: equity_pct,
        pot_odds: pot_odds_rounded,
        spr: spr_rounded,
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
        debug_steps,
    })
}

// ---------------------------------------------------------------------------
// Task 1: analyze_range_on_board
// ---------------------------------------------------------------------------

/// Compute per-hand equity data for the villain's range vs the hero's holding.
///
/// Returns bucket distribution, nut advantage, and per-hand equity breakdown.
#[tauri::command]
fn analyze_range_on_board(
    villain_range_str: String,
    hero_hand: Vec<String>,
    board: Vec<String>,
) -> Result<serde_json::Value, String> {
    // Parse hero hand
    if hero_hand.len() != 2 {
        return Err(format!("Hero hand must have 2 cards, got {}", hero_hand.len()));
    }
    let h1 = parse_card(&hero_hand[0])?;
    let h2 = parse_card(&hero_hand[1])?;

    // Parse board
    let board_cards: Vec<equity_engine::Card> = board.iter()
        .map(|s| parse_card(s))
        .collect::<Result<Vec<_>, _>>()?;

    // Build villain range and remove blockers
    let range_str = expand_range_shorthand(&villain_range_str);
    let mut villain_range = equity_engine::range::Range::from_str(&range_str)
        .map_err(|e| format!("Invalid range: {e}"))?;

    let mut known_cards = vec![h1, h2];
    known_cards.extend_from_slice(&board_cards);
    villain_range.remove_blockers(&known_cards);

    if villain_range.is_empty() {
        return Err("Villain range is empty after removing blockers".to_string());
    }

    // Build "hero as a range" (single hand at weight 1.0)
    let _hero_range = equity_engine::range::Range {
        combos: vec![(h1, h2, 1.0)],
    };

    // Compute per-hand equity for each unique hand in villain range
    // Group by canonical hand name, accumulate weighted equity
    use std::collections::HashMap;
    let mut hand_map: HashMap<String, (f64, f64, u32)> = HashMap::new(); // name → (total_weight, weighted_equity, combo_count)

    for (vc1, vc2, weight) in &villain_range.combos {
        let hand_name = canonical_hand_name(*vc1, *vc2);
        // villain hand equity vs hero's holding
        let villain_range_single = equity_engine::range::Range {
            combos: vec![(*vc1, *vc2, 1.0)],
        };
        // villain equity vs hero hand = 1 - hero_equity_vs_villain
        let hero_eq = equity_engine::postflop::hand_vs_range_equity(
            (h1, h2), &villain_range_single, &board_cards, 500,
        );
        let villain_eq = 1.0 - hero_eq;

        let entry = hand_map.entry(hand_name).or_insert((0.0, 0.0, 0));
        entry.0 += weight;
        entry.1 += villain_eq * weight;
        entry.2 += 1;
    }

    // Build hands array
    let mut hands = Vec::new();
    for (hand_name, (total_w, weighted_eq, combo_count)) in &hand_map {
        let avg_eq = if *total_w > 0.0 { weighted_eq / total_w } else { 0.5 };
        let bucket = equity_bucket(avg_eq);
        hands.push(serde_json::json!({
            "hand": hand_name,
            "weight": (total_w * 100.0).round() / 100.0,
            "equity": (avg_eq * 1000.0).round() / 1000.0,
            "combos": combo_count,
            "equity_bucket": bucket,
        }));
    }
    // Sort by equity descending
    hands.sort_by(|a, b| {
        let ea = a["equity"].as_f64().unwrap_or(0.0);
        let eb = b["equity"].as_f64().unwrap_or(0.0);
        eb.partial_cmp(&ea).unwrap_or(std::cmp::Ordering::Equal)
    });

    // Bucket distribution
    let total_weight: f64 = villain_range.combos.iter().map(|(_, _, w)| w).sum();
    let mut bucket_weights = [0.0f64; 5];
    for (vc1, vc2, weight) in &villain_range.combos {
        let villain_range_single = equity_engine::range::Range { combos: vec![(*vc1, *vc2, 1.0)] };
        let hero_eq = equity_engine::postflop::hand_vs_range_equity(
            (h1, h2), &villain_range_single, &board_cards, 200,
        );
        let villain_eq = 1.0 - hero_eq;
        bucket_weights[equity_bucket(villain_eq) as usize] += weight;
    }
    let equity_buckets: Vec<f64> = bucket_weights.iter()
        .map(|&w| (w / total_weight * 1000.0).round() / 1000.0)
        .collect();

    // Average equities
    let villain_avg_eq: f64 = villain_range.combos.iter().map(|(vc1, vc2, w)| {
        let vr = equity_engine::range::Range { combos: vec![(*vc1, *vc2, 1.0)] };
        let hero_eq = equity_engine::postflop::hand_vs_range_equity((h1, h2), &vr, &board_cards, 200);
        (1.0 - hero_eq) * w
    }).sum::<f64>() / total_weight;

    let hero_avg_eq = 1.0 - villain_avg_eq;

    // Nut advantage: % of range in top bucket (>80% equity)
    let nut_pct_villain = equity_buckets[4];
    // Hero nut % against villain range
    let hero_nut_eq = equity_engine::postflop::hand_vs_range_equity((h1, h2), &villain_range, &board_cards, 1000);
    let nut_pct_hero = if hero_nut_eq > 0.80 { 1.0 } else { 0.0 };

    let nut_advantage = if nut_pct_hero > nut_pct_villain + 0.05 {
        "hero"
    } else if nut_pct_villain > nut_pct_hero + 0.05 {
        "villain"
    } else {
        "neutral"
    };

    // Board texture label
    let texture = equity_engine::postflop::BoardTexture::analyze(&board_cards);
    let mut texture_parts: Vec<&str> = Vec::new();
    if texture.is_monotone { texture_parts.push("Monotone"); }
    else if texture.flush_draw_possible { texture_parts.push("Flush Draw"); }
    if texture.is_paired { texture_parts.push("Paired"); }
    if texture.straight_draw_possible { texture_parts.push("Straight Draw"); }
    let texture_label = if texture_parts.is_empty() { "Dry".to_string() } else { texture_parts.join(", ") };

    Ok(serde_json::json!({
        "hands": hands,
        "equity_buckets": equity_buckets,
        "villain_avg_equity": (villain_avg_eq * 1000.0).round() / 1000.0,
        "hero_avg_equity": (hero_avg_eq * 1000.0).round() / 1000.0,
        "nut_advantage": nut_advantage,
        "nut_pct_hero": (nut_pct_hero * 1000.0).round() / 1000.0,
        "nut_pct_villain": (nut_pct_villain * 1000.0).round() / 1000.0,
        "texture_label": texture_label,
    }))
}

fn equity_bucket(eq: f64) -> u8 {
    match (eq * 100.0) as u32 {
        0..=19  => 0,
        20..=39 => 1,
        40..=59 => 2,
        60..=79 => 3,
        _       => 4,
    }
}

/// Canonical hand name: "AA", "AKs", "AKo" from two cards
fn canonical_hand_name(c1: equity_engine::Card, c2: equity_engine::Card) -> String {
    fn rank_char(r: equity_engine::Rank) -> char {
        match r {
            equity_engine::Rank::Two => '2', equity_engine::Rank::Three => '3',
            equity_engine::Rank::Four => '4', equity_engine::Rank::Five => '5',
            equity_engine::Rank::Six => '6', equity_engine::Rank::Seven => '7',
            equity_engine::Rank::Eight => '8', equity_engine::Rank::Nine => '9',
            equity_engine::Rank::Ten => 'T', equity_engine::Rank::Jack => 'J',
            equity_engine::Rank::Queen => 'Q', equity_engine::Rank::King => 'K',
            equity_engine::Rank::Ace => 'A',
        }
    }
    fn rank_val(r: equity_engine::Rank) -> u8 {
        match r {
            equity_engine::Rank::Two => 2, equity_engine::Rank::Three => 3,
            equity_engine::Rank::Four => 4, equity_engine::Rank::Five => 5,
            equity_engine::Rank::Six => 6, equity_engine::Rank::Seven => 7,
            equity_engine::Rank::Eight => 8, equity_engine::Rank::Nine => 9,
            equity_engine::Rank::Ten => 10, equity_engine::Rank::Jack => 11,
            equity_engine::Rank::Queen => 12, equity_engine::Rank::King => 13,
            equity_engine::Rank::Ace => 14,
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
// Task 2: compare_bet_sizes
// ---------------------------------------------------------------------------

/// Compare EV of different bet sizes for a first-to-act spot.
#[tauri::command]
fn compare_bet_sizes(
    hero_hand: Vec<String>,
    board: Vec<String>,
    pot_bb: f64,
    hero_stack_bb: f64,
    villain_stack_bb: f64,
    villain_range_str: String,
) -> Result<serde_json::Value, String> {
    let h1 = parse_card(&hero_hand[0])?;
    let h2 = parse_card(&hero_hand[1])?;

    let board_cards: Vec<equity_engine::Card> = board.iter()
        .map(|s| parse_card(s))
        .collect::<Result<Vec<_>, _>>()?;

    let range_str = expand_range_shorthand(&villain_range_str);
    let mut villain_range = equity_engine::range::Range::from_str(&range_str)
        .map_err(|e| format!("Invalid range: {e}"))?;

    let mut known_cards = vec![h1, h2];
    known_cards.extend_from_slice(&board_cards);
    villain_range.remove_blockers(&known_cards);

    if villain_range.is_empty() {
        return Err("Villain range is empty after removing blockers".to_string());
    }

    // Hero equity vs villain range
    let hero_equity = equity_engine::postflop::hand_vs_range_equity(
        (h1, h2), &villain_range, &board_cards, 3000,
    );

    let eff_stack = hero_stack_bb.min(villain_stack_bb);
    let villain_folding_freq = (1.0 - hero_equity * 1.4).clamp(0.0, 0.85);

    // Bet size options
    let sizes: &[(&str, f64)] = &[
        ("Check",    0.0),
        ("33% pot",  0.33),
        ("50% pot",  0.50),
        ("75% pot",  0.75),
        ("Pot",      1.00),
    ];

    let mut options = Vec::new();
    let mut best_ev = f64::NEG_INFINITY;
    let mut best_idx = 0usize;

    for (i, (label, pct_pot)) in sizes.iter().enumerate() {
        let bet_bb = (pot_bb * pct_pot).min(eff_stack);

        let (ev, fold_eq) = if *pct_pot == 0.0 {
            // Check: EV = pot_bb * hero_equity (simplified: we "win" equity fraction of pot)
            let ev = pot_bb * hero_equity;
            (ev, 0.0f64)
        } else {
            let fold_equity = (bet_bb / (pot_bb + bet_bb)) * villain_folding_freq;
            let ev_if_called = hero_equity * (pot_bb + 2.0 * bet_bb);
            let ev_if_folded = pot_bb + bet_bb;
            let ev = fold_equity * ev_if_folded + (1.0 - fold_equity) * ev_if_called;
            (ev, fold_equity)
        };

        if ev > best_ev {
            best_ev = ev;
            best_idx = i;
        }

        options.push(serde_json::json!({
            "label":    label,
            "pct_pot":  pct_pot,
            "bb":       (bet_bb * 10.0).round() / 10.0,
            "ev":       (ev * 10.0).round() / 10.0,
            "fold_eq":  (fold_eq * 1000.0).round() / 1000.0,
        }));
    }

    Ok(serde_json::json!({
        "options":         options,
        "recommended_idx": best_idx,
        "hero_equity":     (hero_equity * 1000.0).round() / 1000.0,
    }))
}

// ---------------------------------------------------------------------------
// App entry point
// ---------------------------------------------------------------------------

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            solve_postflop,
            analyze_range_on_board,
            compare_bet_sizes,
        ])
        .run(tauri::generate_context!())
        .expect("error while running poker-postflop-solver");
}
