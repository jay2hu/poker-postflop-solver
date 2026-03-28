use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SolveRequest {
    hero_hand: Vec<String>,
    board: Vec<String>,
    pot_bb: f64,
    hero_stack_bb: f64,
    villain_stack_bb: f64,
    to_call_bb: f64,
    is_ip: bool,
    villain_range_str: String,
}

#[derive(Debug, Serialize)]
struct PostflopResult {
    equity: f64,
    action: String,
    sizing_bb: Option<f64>,
    sizing_pct_pot: Option<f64>,
    pot_odds: f64,
    spr: f64,
    reasoning: String,
    board_texture: String,
}

#[tauri::command]
fn solve_postflop(
    hero_hand: Vec<String>,
    board: Vec<String>,
    pot_bb: f64,
    hero_stack_bb: f64,
    _villain_stack_bb: f64,
    to_call_bb: f64,
    _is_ip: bool,
    _villain_range_str: String,
) -> Result<PostflopResult, String> {
    if hero_hand.len() < 2 { return Err("Need 2 hero cards".into()); }
    if board.len() < 3     { return Err("Need 3-5 board cards".into()); }

    // Stub — real impl will call equity engine
    let equity = 0.55_f64 + (board.len() as f64 * 0.02).min(0.15);
    let spr = hero_stack_bb / pot_bb.max(0.5);
    let pot_odds = if to_call_bb > 0.0 { to_call_bb / (pot_bb + to_call_bb) } else { 0.0 };
    let suits: std::collections::HashSet<char> = board.iter().filter_map(|c| c.chars().last()).collect();
    let texture = if suits.len() == 1 { "Monotone" } else if suits.len() == 2 { "Two-tone" } else { "Rainbow" };

    let (action, sizing_bb, sizing_pct_pot) = if equity > 0.60 {
        ("bet", Some((pot_bb * 0.65).round() / 10.0 * 10.0), Some(65.0_f64))
    } else if equity > 0.45 {
        ("call", None, None)
    } else {
        ("fold", None, None)
    };

    Ok(PostflopResult {
        equity: (equity * 1000.0).round() / 1000.0,
        action: action.to_string(),
        sizing_bb,
        sizing_pct_pot,
        pot_odds: (pot_odds * 1000.0).round() / 1000.0,
        spr: (spr * 10.0).round() / 10.0,
        reasoning: format!("Equity {:.0}% on {} board.", equity * 100.0, texture),
        board_texture: texture.to_string(),
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![solve_postflop])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
