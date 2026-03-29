/// Range representation: a set of hand combinations with weights.
///
/// Weight 1.0 = always in range, 0.0 = never, 0.0–1.0 = mixed strategy.

use crate::{Card, Position, Rank, Suit, Street};

// ---------------------------------------------------------------------------
// Range struct
// ---------------------------------------------------------------------------

/// A player's range: set of (card1, card2, weight) combos.
#[derive(Debug, Clone)]
pub struct Range {
    pub combos: Vec<(Card, Card, f64)>,
}

impl Default for Range {
    fn default() -> Self {
        Range { combos: Vec::new() }
    }
}

impl Range {
    /// Parse from a range string like `"AA:1.0,KK:1.0,AKs:0.85,AKo:0.6"`.
    ///
    /// Supported formats per token:
    /// - `AA` / `AKs` / `AKo`  → all combos at weight 1.0
    /// - `AA:0.5`               → all combos at weight 0.5
    /// - `AhKh`                 → exact combo at weight 1.0
    /// - `AhKh:0.7`             → exact combo at weight 0.7
    pub fn from_str(s: &str) -> Result<Self, String> {
        let mut combos: Vec<(Card, Card, f64)> = Vec::new();

        for token in s.split(',') {
            let token = token.trim();
            if token.is_empty() { continue; }

            // Split on ':' for optional weight
            let (hand_str, weight) = if let Some(idx) = token.rfind(':') {
                let w: f64 = token[idx+1..].parse()
                    .map_err(|_| format!("Invalid weight in '{token}'"))?;
                (&token[..idx], w.clamp(0.0, 1.0))
            } else {
                (token, 1.0)
            };

            let new_combos = expand_hand_str(hand_str)
                .map_err(|e| format!("Invalid hand '{hand_str}': {e}"))?;
            for (c1, c2) in new_combos {
                combos.push((c1, c2, weight));
            }
        }

        Ok(Range { combos })
    }

    /// Build a range from preflop position + action context.
    ///
    /// Uses the GTO strategy v2 files when available; falls back to a
    /// reasonable tight/loose default based on position.
    pub fn from_preflop_action(position: Position, facing: &str, table_size: u8) -> Self {
        // Try v2 strategy lookup first
        if let Some(dir) = crate::gto_strategy::default_strategies_dir() {
            let scenario_key = match facing {
                "rfi"   => format!("{}_rfi", pos_name(position)),
                _       => format!("{}_rfi", pos_name(position)),
            };
            // Stack bucket assumed 100bb for default
            let stack_bb = 100;
            // Load all combos with raise frequency > 0.1 as the range
            if let Some(lookup_range) = build_range_from_v2(&dir, &scenario_key, stack_bb) {
                if !lookup_range.combos.is_empty() {
                    return lookup_range;
                }
            }
        }

        // Fallback: position-based default range
        default_range_for_position(position)
    }

    /// Remove combos blocked by known cards (board or hero's hole cards).
    ///
    /// Any combo that shares a card with `known_cards` is set to weight 0
    /// and removed.
    pub fn remove_blockers(&mut self, known_cards: &[Card]) {
        self.combos.retain(|(c1, c2, _)| {
            !known_cards.contains(c1) && !known_cards.contains(c2)
        });
    }

    /// Total weighted combos.
    pub fn total_combos(&self) -> f64 {
        self.combos.iter().map(|(_, _, w)| w).sum()
    }

    /// Whether the range is empty.
    pub fn is_empty(&self) -> bool {
        self.combos.is_empty()
    }
}

// ---------------------------------------------------------------------------
// Hand string expansion
// ---------------------------------------------------------------------------

fn rank_from_char(c: char) -> Option<Rank> {
    match c {
        '2' => Some(Rank::Two),   '3' => Some(Rank::Three), '4' => Some(Rank::Four),
        '5' => Some(Rank::Five),  '6' => Some(Rank::Six),   '7' => Some(Rank::Seven),
        '8' => Some(Rank::Eight), '9' => Some(Rank::Nine),  'T' => Some(Rank::Ten),
        'J' => Some(Rank::Jack),  'Q' => Some(Rank::Queen), 'K' => Some(Rank::King),
        'A' => Some(Rank::Ace),   _ => None,
    }
}

fn suit_from_char(c: char) -> Option<Suit> {
    match c {
        'h' => Some(Suit::Hearts), 'd' => Some(Suit::Diamonds),
        'c' => Some(Suit::Clubs),  's' => Some(Suit::Spades),
        _ => None,
    }
}

fn rank_val(r: Rank) -> u8 {
    match r {
        Rank::Two => 2, Rank::Three => 3, Rank::Four => 4, Rank::Five => 5,
        Rank::Six => 6, Rank::Seven => 7, Rank::Eight => 8, Rank::Nine => 9,
        Rank::Ten => 10, Rank::Jack => 11, Rank::Queen => 12, Rank::King => 13,
        Rank::Ace => 14,
    }
}

const ALL_SUITS: [Suit; 4] = [Suit::Hearts, Suit::Diamonds, Suit::Clubs, Suit::Spades];

/// Expand a hand string ("AA", "AKs", "AKo", "AhKh") to specific card combos.
fn expand_hand_str(s: &str) -> Result<Vec<(Card, Card)>, String> {
    let chars: Vec<char> = s.chars().collect();

    // Exact combo: e.g. "AhKh" (4 chars)
    if chars.len() == 4 {
        let r1 = rank_from_char(chars[0]).ok_or_else(|| format!("bad rank {}", chars[0]))?;
        let s1 = suit_from_char(chars[1]).ok_or_else(|| format!("bad suit {}", chars[1]))?;
        let r2 = rank_from_char(chars[2]).ok_or_else(|| format!("bad rank {}", chars[2]))?;
        let s2 = suit_from_char(chars[3]).ok_or_else(|| format!("bad suit {}", chars[3]))?;
        return Ok(vec![(Card { rank: r1, suit: s1 }, Card { rank: r2, suit: s2 })]);
    }

    // Pair: "AA", "KK", etc (2 chars, same rank)
    if chars.len() == 2 {
        let r1 = rank_from_char(chars[0]).ok_or_else(|| format!("bad rank {}", chars[0]))?;
        let r2 = rank_from_char(chars[1]).ok_or_else(|| format!("bad rank {}", chars[1]))?;
        if r1 != r2 {
            return Err(format!("offsuit/suited specifier missing"));
        }
        let mut combos = Vec::new();
        for i in 0..4 {
            for j in (i+1)..4 {
                combos.push((
                    Card { rank: r1, suit: ALL_SUITS[i] },
                    Card { rank: r1, suit: ALL_SUITS[j] },
                ));
            }
        }
        return Ok(combos);
    }

    // Suited/offsuit: "AKs", "AKo" (3 chars)
    if chars.len() == 3 {
        let r1 = rank_from_char(chars[0]).ok_or_else(|| format!("bad rank {}", chars[0]))?;
        let r2 = rank_from_char(chars[1]).ok_or_else(|| format!("bad rank {}", chars[1]))?;
        let spec = chars[2];
        let (hi, lo) = if rank_val(r1) >= rank_val(r2) { (r1, r2) } else { (r2, r1) };
        let mut combos = Vec::new();
        match spec {
            's' => {
                for &suit in &ALL_SUITS {
                    combos.push((Card { rank: hi, suit }, Card { rank: lo, suit }));
                }
            }
            'o' => {
                for i in 0..4 {
                    for j in 0..4 {
                        if i != j {
                            combos.push((
                                Card { rank: hi, suit: ALL_SUITS[i] },
                                Card { rank: lo, suit: ALL_SUITS[j] },
                            ));
                        }
                    }
                }
            }
            _ => return Err(format!("expected 's' or 'o', got '{spec}'")),
        }
        return Ok(combos);
    }

    Err(format!("unrecognised hand format '{s}'"))
}

// ---------------------------------------------------------------------------
// V2 strategy range building
// ---------------------------------------------------------------------------

fn pos_name(p: Position) -> &'static str {
    match p {
        Position::BTN => "BTN", Position::CO => "CO", Position::MP => "HJ",
        Position::UTG => "UTG", Position::SB => "SB", Position::BB => "BB",
        Position::UTG1 => "UTG1", Position::UTG2 => "UTG2",
        Position::LJ => "LJ",   Position::HJ => "HJ",
    }
}

fn build_range_from_v2(
    dir: &std::path::PathBuf,
    scenario_key: &str,
    stack_bb: u32,
) -> Option<Range> {
    let lookup = crate::gto_strategy::lookup_by_scenario(scenario_key, dir, stack_bb)?;
    // Build range: include all hands where raise/open frequency > 0.1
    let mut combos: Vec<(Card, Card, f64)> = Vec::new();
    for (hand_str, freqs) in &lookup {
        let raise_freq: f64 = freqs.iter()
            .filter(|(k, _)| !k.to_lowercase().contains("fold") && k.as_str() != "F")
            .map(|(_, v)| v)
            .sum();
        if raise_freq > 0.05 {
            if let Ok(hand_combos) = expand_hand_str(hand_str) {
                for (c1, c2) in hand_combos {
                    combos.push((c1, c2, raise_freq.min(1.0)));
                }
            }
        }
    }
    Some(Range { combos })
}

// ---------------------------------------------------------------------------
// Default ranges by position (fallback)
// ---------------------------------------------------------------------------

fn default_range_for_position(position: Position) -> Range {
    // Return a reasonable opening range string based on position
    let range_str = match position {
        Position::BTN => "AA,KK,QQ,JJ,TT,99,88,77,66,55,44,33,22,AKs,AQs,AJs,ATs,A9s,A8s,A7s,A6s,A5s,A4s,A3s,A2s,AKo,AQo,AJo,ATo,KQs,KJs,KTs,K9s,K8s,KQo,KJo,QJs,QTs,Q9s,JTs,J9s,T9s,98s,87s,76s,65s,54s",
        Position::CO  => "AA,KK,QQ,JJ,TT,99,88,77,66,55,AKs,AQs,AJs,ATs,A9s,A8s,A5s,A4s,AKo,AQo,AJo,ATo,KQs,KJs,KTs,K9s,KQo,KJo,QJs,QTs,J9s,JTs,T9s,98s,87s,76s",
        Position::MP | Position::HJ => "AA,KK,QQ,JJ,TT,99,88,77,AKs,AQs,AJs,ATs,A9s,A5s,AKo,AQo,AJo,KQs,KJs,KTs,KQo,QJs,QTs,JTs,T9s,98s",
        Position::UTG | Position::UTG1 | Position::UTG2 | Position::LJ => "AA,KK,QQ,JJ,TT,99,88,AKs,AQs,AJs,ATs,AKo,AQo,KQs,KJs,QJs,JTs",
        Position::SB  => "AA,KK,QQ,JJ,TT,99,88,77,66,AKs,AQs,AJs,ATs,A9s,A8s,A5s,AKo,AQo,AJo,KQs,KJs,KTs,KQo,QJs,JTs,T9s,98s,87s",
        Position::BB  => "AA,KK,QQ,JJ,TT,99,88,77,66,55,44,AKs,AQs,AJs,ATs,A9s,A8s,A7s,A6s,A5s,A4s,AKo,AQo,AJo,ATo,A9o,KQs,KJs,KTs,K9s,KQo,KJo,QJs,QTs,Q9s,JTs,J9s,T9s,98s,87s,76s,65s,54s,43s",
    };
    Range::from_str(range_str).unwrap_or_default()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Card, Rank, Suit};

    fn card(rank: Rank, suit: Suit) -> Card { Card { rank, suit } }

    #[test]
    fn range_from_str_pairs() {
        let r = Range::from_str("AA,KK").expect("parse");
        // AA = 6 combos, KK = 6 combos
        assert_eq!(r.combos.len(), 12);
        assert!(r.combos.iter().all(|(_, _, w)| (*w - 1.0).abs() < 0.001));
    }

    #[test]
    fn range_from_str_with_weight() {
        let r = Range::from_str("AA:1.0,KK:1.0,AKs:0.5").expect("parse");
        // AA=6, KK=6, AKs=4 combos
        assert_eq!(r.combos.len(), 16);
        // AKs combos have weight 0.5
        let aks: Vec<_> = r.combos.iter()
            .filter(|(c1, c2, _)| c1.rank == Rank::Ace && c2.rank == Rank::King && c1.suit == c2.suit)
            .collect();
        assert_eq!(aks.len(), 4);
        assert!(aks.iter().all(|(_, _, w)| (*w - 0.5).abs() < 0.001));
    }

    #[test]
    fn range_from_str_suited() {
        let r = Range::from_str("AKs").expect("parse");
        assert_eq!(r.combos.len(), 4); // AhKh, AdKd, AcKc, AsKs
        // All same suit
        assert!(r.combos.iter().all(|(c1, c2, _)| c1.suit == c2.suit));
        assert!(r.combos.iter().all(|(c1, c2, _)| c1.rank == Rank::Ace && c2.rank == Rank::King));
    }

    #[test]
    fn range_from_str_offsuit() {
        let r = Range::from_str("AKo").expect("parse");
        assert_eq!(r.combos.len(), 12); // 4*3 = 12 offsuit combos
        assert!(r.combos.iter().all(|(c1, c2, _)| c1.suit != c2.suit));
    }

    #[test]
    fn range_from_str_exact_combo() {
        let r = Range::from_str("AhKh").expect("parse");
        assert_eq!(r.combos.len(), 1);
        assert_eq!(r.combos[0].0, card(Rank::Ace, Suit::Hearts));
        assert_eq!(r.combos[0].1, card(Rank::King, Suit::Hearts));
    }

    #[test]
    fn range_from_str_invalid_weight() {
        assert!(Range::from_str("AA:xyz").is_err());
    }

    #[test]
    fn remove_blockers_removes_blocked_combos() {
        // AA range: 6 combos (AhAs, AhAd, AhAc, AsAd, AsAc, AdAc)
        let mut r = Range::from_str("AA").expect("parse");
        assert_eq!(r.combos.len(), 6);
        // Block Ah — removes AhAx combos (AhAs, AhAd, AhAc = 3 combos)
        r.remove_blockers(&[card(Rank::Ace, Suit::Hearts)]);
        assert_eq!(r.combos.len(), 3, "should remove 3 combos containing Ah");
        // None remaining should contain Ah
        assert!(!r.combos.iter().any(|(c1, c2, _)|
            *c1 == card(Rank::Ace, Suit::Hearts) || *c2 == card(Rank::Ace, Suit::Hearts)
        ));
    }

    #[test]
    fn remove_blockers_two_cards() {
        let mut r = Range::from_str("AA,AKs").expect("parse");
        // Block both Ah and Kh → removes AhAx, AhKh
        r.remove_blockers(&[
            card(Rank::Ace, Suit::Hearts),
            card(Rank::King, Suit::Hearts),
        ]);
        // No combo should contain Ah or Kh
        assert!(!r.combos.iter().any(|(c1, c2, _)|
            *c1 == card(Rank::Ace, Suit::Hearts) || *c2 == card(Rank::Ace, Suit::Hearts) ||
            *c1 == card(Rank::King, Suit::Hearts) || *c2 == card(Rank::King, Suit::Hearts)
        ));
    }

    #[test]
    fn total_combos_weighted() {
        let r = Range::from_str("AA:1.0,AKs:0.5").expect("parse");
        // 6 * 1.0 + 4 * 0.5 = 8.0
        assert!((r.total_combos() - 8.0).abs() < 0.01);
    }
}
