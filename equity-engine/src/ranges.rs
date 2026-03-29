use std::path::Path;
use crate::{RangeProfile, Street};

/// Load a bundled default range profile from assets/ranges/<street>/<name>.json
pub fn load_default_profile(street: Street, name: &str) -> Result<RangeProfile, String> {
    let street_str = match street {
        Street::Preflop => "preflop",
        Street::Flop => "flop",
        Street::Turn => "turn",
        Street::River => "river",
    };

    // Assets path relative to crate root (works for dev; Tauri will bundle these)
    let assets_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/ranges");
    let path = assets_dir.join(street_str).join(format!("{}.json", name));

    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to load range profile '{}': {}", path.display(), e))?;

    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse range profile '{}': {}", name, e))
}

/// List available built-in profile names.
pub fn available_profiles() -> Vec<&'static str> {
    vec!["tight", "default", "loose"]
}
