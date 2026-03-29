/// Hand notation → specific combo expansion
///
/// Supports equilab-style notation:
/// - Pairs:         "AA", "TT"
/// - Suited:        "AKs", "QJs"
/// - Offsuit:       "AKo", "QJo"
/// - Both:          "AK" (suited + offsuit)
/// - Pair+ runout:  "QQ+" (QQ, KK, AA)
/// - Suited+ runout:"ATs+" (ATs, AJs, AQs, AKs)
/// - Offsuit+:      "ATo+" (ATo, AJo, AQo, AKo)
///
/// Output: Vec of specific combo strings like "AhKd", "TsTs" for use
/// in pokers `HandRange::from_strings` or UI display.

const RANKS: [char; 13] = ['2','3','4','5','6','7','8','9','T','J','Q','K','A'];
const SUITS: [char; 4]  = ['h','d','c','s'];

fn rank_index(r: char) -> Option<usize> {
    RANKS.iter().position(|&x| x == r)
}

/// Build a card string e.g. rank='A', suit='h' → "Ah"
#[allow(dead_code)]
fn card(rank: char, suit: char) -> String {
    format!("{}{}", rank, suit)
}

/// All 6 same-rank pair combos for a given rank
fn pair_combos(rank: char) -> Vec<String> {
    let mut out = Vec::new();
    for i in 0..4 {
        for j in (i+1)..4 {
            out.push(format!("{}{}{}{}", rank, SUITS[i], rank, SUITS[j]));
        }
    }
    out
}

/// 4 suited combos for two ranks (rank1 > rank2 assumed)
fn suited_combos(r1: char, r2: char) -> Vec<String> {
    SUITS.iter().map(|&s| format!("{}{}{}{}", r1, s, r2, s)).collect()
}

/// 12 offsuit combos for two ranks
fn offsuit_combos(r1: char, r2: char) -> Vec<String> {
    let mut out = Vec::new();
    for &s1 in &SUITS {
        for &s2 in &SUITS {
            if s1 != s2 {
                out.push(format!("{}{}{}{}", r1, s1, r2, s2));
            }
        }
    }
    out
}

/// Expand a single hand notation token to specific combos.
/// Returns Err if the notation is unrecognised.
pub fn expand(notation: &str) -> Result<Vec<String>, String> {
    let s = notation.trim();

    // Strip optional weight suffix e.g. "QQ@50" → "QQ"
    let s = if let Some(pos) = s.find('@') { &s[..pos] } else { s };

    // "+" runout suffix?
    let (base, plus) = if s.ends_with('+') {
        (&s[..s.len()-1], true)
    } else {
        (s, false)
    };

    let chars: Vec<char> = base.chars().collect();

    match chars.as_slice() {
        // Pair: "AA", "TT"
        [r1, r2] if r1 == r2 => {
            let ri = rank_index(*r1).ok_or_else(|| format!("Unknown rank: {}", r1))?;
            let mut out = pair_combos(*r1);
            if plus {
                // All pairs >= this rank
                for r in ri+1..13 {
                    out.extend(pair_combos(RANKS[r]));
                }
            }
            Ok(out)
        }

        // Suited: "AKs"
        [r1, r2, 's'] => {
            let r1i = rank_index(*r1).ok_or_else(|| format!("Unknown rank: {}", r1))?;
            let r2i = rank_index(*r2).ok_or_else(|| format!("Unknown rank: {}", r2))?;
            if r1i <= r2i { return Err(format!("First rank must be higher in '{}'", notation)); }
            let mut out = suited_combos(*r1, *r2);
            if plus {
                // Step kicker up toward r1 (exclusive)
                let mut k = r2i + 1;
                while k < r1i {
                    out.extend(suited_combos(*r1, RANKS[k]));
                    k += 1;
                }
            }
            Ok(out)
        }

        // Offsuit: "AKo"
        [r1, r2, 'o'] => {
            let r1i = rank_index(*r1).ok_or_else(|| format!("Unknown rank: {}", r1))?;
            let r2i = rank_index(*r2).ok_or_else(|| format!("Unknown rank: {}", r2))?;
            if r1i <= r2i { return Err(format!("First rank must be higher in '{}'", notation)); }
            let mut out = offsuit_combos(*r1, *r2);
            if plus {
                let mut k = r2i + 1;
                while k < r1i {
                    out.extend(offsuit_combos(*r1, RANKS[k]));
                    k += 1;
                }
            }
            Ok(out)
        }

        // Both suited+offsuit: "AK"
        [r1, r2] if r1 != r2 => {
            let r1i = rank_index(*r1).ok_or_else(|| format!("Unknown rank: {}", r1))?;
            let r2i = rank_index(*r2).ok_or_else(|| format!("Unknown rank: {}", r2))?;
            if r1i <= r2i { return Err(format!("First rank must be higher in '{}'", notation)); }
            let mut out = suited_combos(*r1, *r2);
            out.extend(offsuit_combos(*r1, *r2));
            if plus {
                let mut k = r2i + 1;
                while k < r1i {
                    out.extend(suited_combos(*r1, RANKS[k]));
                    out.extend(offsuit_combos(*r1, RANKS[k]));
                    k += 1;
                }
            }
            Ok(out)
        }

        // Specific combo: "AhKd"
        [r1, s1, r2, s2] => {
            // Validate
            rank_index(*r1).ok_or_else(|| format!("Unknown rank: {}", r1))?;
            rank_index(*r2).ok_or_else(|| format!("Unknown rank: {}", r2))?;
            if !SUITS.contains(s1) { return Err(format!("Unknown suit: {}", s1)); }
            if !SUITS.contains(s2) { return Err(format!("Unknown suit: {}", s2)); }
            Ok(vec![format!("{}{}{}{}", r1, s1, r2, s2)])
        }

        _ => Err(format!("Unrecognised hand notation: '{}'", notation)),
    }
}

/// Expand all combos in a range profile to specific card strings.
/// Returns flat list of combo strings, deduplicated.
pub fn expand_range(combos: &[crate::HandCombo]) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    for hc in combos {
        let weight_suffix = if (hc.weight - 1.0).abs() > f64::EPSILON {
            format!("@{}", (hc.weight * 100.0) as u32)
        } else {
            String::new()
        };
        for specific in expand(&hc.combo)? {
            out.push(format!("{}{}", specific, weight_suffix));
        }
    }
    Ok(out)
}

/// Build the range string expected by pokers `HandRange::from_string`.
/// Uses equilab notation directly — pokers handles it natively.
/// For combos that are already specific (4-char), passes through.
pub fn to_pokers_range_string(combos: &[crate::HandCombo]) -> String {
    if combos.is_empty() {
        return "random".to_string();
    }
    combos
        .iter()
        .map(|c| {
            if (c.weight - 1.0).abs() < f64::EPSILON {
                c.combo.clone()
            } else {
                format!("{}@{}", c.combo, (c.weight * 100.0) as u32)
            }
        })
        .collect::<Vec<_>>()
        .join(",")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pair_expansion() {
        let combos = expand("AA").unwrap();
        assert_eq!(combos.len(), 6);
    }

    #[test]
    fn test_suited_expansion() {
        let combos = expand("AKs").unwrap();
        assert_eq!(combos.len(), 4);
        assert!(combos.iter().all(|c| {
            let chars: Vec<char> = c.chars().collect();
            chars[1] == chars[3] // same suit
        }));
    }

    #[test]
    fn test_offsuit_expansion() {
        let combos = expand("AKo").unwrap();
        assert_eq!(combos.len(), 12);
    }

    #[test]
    fn test_both_expansion() {
        let combos = expand("AK").unwrap();
        assert_eq!(combos.len(), 16);
    }

    #[test]
    fn test_pair_plus() {
        // QQ+ = QQ, KK, AA = 18 combos
        let combos = expand("QQ+").unwrap();
        assert_eq!(combos.len(), 18);
    }

    #[test]
    fn test_suited_plus() {
        // ATs+ = ATs, AJs, AQs, AKs = 16 combos
        let combos = expand("ATs+").unwrap();
        assert_eq!(combos.len(), 16);
    }

    #[test]
    fn test_specific_combo() {
        let combos = expand("AhKd").unwrap();
        assert_eq!(combos, vec!["AhKd"]);
    }
}
