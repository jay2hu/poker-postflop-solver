use crate::{Card, Rank, Street, Suit};

/// Best hand or draw description shown in the HUD.
/// Returns the made hand name on the flop/turn/river,
/// or a draw name when no made hand is strong enough.
/// Returns None on preflop (no board to evaluate).
pub fn hand_category(hero: &[Card; 2], board: &[Card], street: Street) -> Option<String> {
    if board.is_empty() || street == Street::Preflop {
        return None;
    }

    let mut all: Vec<Card> = hero.to_vec();
    all.extend_from_slice(board);

    // -----------------------------------------------------------------------
    // Made hand evaluation (best 5-card hand from all combinations)
    // -----------------------------------------------------------------------
    let made = best_made_hand(&all);

    // On the river there are no draws — just show the made hand
    if street == Street::River {
        return Some(made.to_string());
    }

    // On flop/turn: check draws and show whichever is more informative
    let draw = best_draw(hero, board);

    // Made hand priority threshold: if we have pair or better, show made hand
    // If we have a draw only (or high card), show the draw
    match made {
        MadeHand::HighCard => draw.map(|d| d.to_string()).or_else(|| Some(made.to_string())),
        MadeHand::OnePair  => {
            // Pair + strong draw is more informative than just pair
            match draw {
                Some(Draw::FlushDraw) | Some(Draw::OESD) => {
                    Some(format!("{} + {}", made, draw.unwrap()))
                }
                _ => Some(made.to_string()),
            }
        }
        _ => Some(made.to_string()),
    }
}

// ---------------------------------------------------------------------------
// Made hand ranking
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum MadeHand {
    HighCard,
    OnePair,
    TwoPair,
    ThreeOfAKind,
    Straight,
    Flush,
    FullHouse,
    FourOfAKind,
    StraightFlush,
    RoyalFlush,
}

impl std::fmt::Display for MadeHand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            MadeHand::HighCard      => "High Card",
            MadeHand::OnePair       => "One Pair",
            MadeHand::TwoPair       => "Two Pair",
            MadeHand::ThreeOfAKind  => "Three of a Kind",
            MadeHand::Straight      => "Straight",
            MadeHand::Flush         => "Flush",
            MadeHand::FullHouse     => "Full House",
            MadeHand::FourOfAKind   => "Four of a Kind",
            MadeHand::StraightFlush => "Straight Flush",
            MadeHand::RoyalFlush    => "Royal Flush",
        };
        write!(f, "{s}")
    }
}

fn rank_val(r: Rank) -> u8 {
    match r {
        Rank::Two => 2,   Rank::Three => 3,  Rank::Four => 4,  Rank::Five => 5,
        Rank::Six => 6,   Rank::Seven => 7,  Rank::Eight => 8, Rank::Nine => 9,
        Rank::Ten => 10,  Rank::Jack => 11,  Rank::Queen => 12, Rank::King => 13,
        Rank::Ace => 14,
    }
}

fn suit_idx(s: Suit) -> usize {
    match s {
        Suit::Hearts => 0, Suit::Diamonds => 1,
        Suit::Clubs => 2,  Suit::Spades => 3,
    }
}

/// Find the best 5-card made hand from a set of cards (any size).
fn best_made_hand(cards: &[Card]) -> MadeHand {
    let mut rank_counts = [0u8; 15]; // index by rank value 2–14
    let mut suit_counts = [0u8; 4];
    let mut ranks: Vec<u8> = Vec::with_capacity(cards.len());

    for c in cards {
        let rv = rank_val(c.rank);
        rank_counts[rv as usize] += 1;
        suit_counts[suit_idx(c.suit)] += 1;
        ranks.push(rv);
    }

    // Check flush (5+ of same suit)
    let has_flush = suit_counts.iter().any(|&n| n >= 5);

    // Straight / straight flush
    let mut rv_set: Vec<u8> = (2u8..=14).filter(|&v| rank_counts[v as usize] > 0).collect();
    if rv_set.contains(&14) { rv_set.insert(0, 1); } // wheel ace
    rv_set.dedup();

    let has_straight = rv_set.windows(5).any(|w| w[4] - w[0] == 4 && w.len() == 5);

    // Count pairs/trips/quads
    let quads  = rank_counts.iter().filter(|&&n| n >= 4).count();
    let trips  = rank_counts.iter().filter(|&&n| n >= 3).count();
    let pairs  = rank_counts.iter().filter(|&&n| n >= 2).count();

    // Classify best hand
    if has_flush && has_straight {
        // Check royal (T J Q K A all same suit) — approximation: any straight flush
        let top = ranks.iter().max().copied().unwrap_or(0);
        if top == 14 && rv_set.windows(5).any(|w| w == [10, 11, 12, 13, 14]) {
            return MadeHand::RoyalFlush;
        }
        return MadeHand::StraightFlush;
    }
    if quads > 0         { return MadeHand::FourOfAKind; }
    if trips > 0 && pairs > 1 { return MadeHand::FullHouse; }
    if has_flush         { return MadeHand::Flush; }
    if has_straight      { return MadeHand::Straight; }
    if trips > 0         { return MadeHand::ThreeOfAKind; }
    if pairs >= 2        { return MadeHand::TwoPair; }
    if pairs == 1        { return MadeHand::OnePair; }
    MadeHand::HighCard
}

// ---------------------------------------------------------------------------
// Draw detection (flop/turn only)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
enum Draw {
    FlushDraw,   // 4 to a flush
    OESD,        // open-ended straight draw
    Gutshot,     // inside straight draw
}

impl std::fmt::Display for Draw {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Draw::FlushDraw => "Flush Draw",
            Draw::OESD      => "OESD",
            Draw::Gutshot   => "Gutshot",
        };
        write!(f, "{s}")
    }
}

fn best_draw(hero: &[Card; 2], board: &[Card]) -> Option<Draw> {
    let mut all: Vec<Card> = hero.to_vec();
    all.extend_from_slice(board);

    // Flush draw: 4+ cards of same suit (not 5 — that's a made flush)
    let mut suit_counts = [0u8; 4];
    for c in &all { suit_counts[suit_idx(c.suit)] += 1; }
    if suit_counts.iter().any(|&n| n == 4) {
        return Some(Draw::FlushDraw);
    }

    // Straight draws using all cards
    let mut rv_set: Vec<u8> = all.iter().map(|c| rank_val(c.rank)).collect();
    if rv_set.contains(&14) { rv_set.push(1); }
    rv_set.sort_unstable();
    rv_set.dedup();

    // OESD: 4 consecutive ranks (not on the ends, i.e. not already a made straight)
    for window in rv_set.windows(4) {
        if window[3] - window[0] == 3 {
            // Check not a made straight (5th card completing it not present)
            let low_open  = window[0] > 1  && !rv_set.contains(&(window[0] - 1));
            let high_open = window[3] < 14 && !rv_set.contains(&(window[3] + 1));
            if low_open || high_open {
                return Some(Draw::OESD);
            }
        }
    }

    // Gutshot: 4 cards with exactly one gap
    for window in rv_set.windows(5) {
        let consecutive_with_gap = window[4] - window[0] == 4 && window.len() == 5;
        let missing_one = window.windows(2).filter(|w| w[1] - w[0] == 2).count() == 1;
        if consecutive_with_gap && missing_one {
            return Some(Draw::Gutshot);
        }
    }

    None
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Rank::*, Suit::*};

    fn c(rank: Rank, suit: Suit) -> Card { Card { rank, suit } }

    #[test]
    fn royal_flush() {
        let cards = vec![c(Ten, Hearts), c(Jack, Hearts), c(Queen, Hearts), c(King, Hearts), c(Ace, Hearts)];
        assert_eq!(best_made_hand(&cards), MadeHand::RoyalFlush);
    }

    #[test]
    fn flush() {
        let cards = vec![c(Two, Hearts), c(Five, Hearts), c(Nine, Hearts), c(Jack, Hearts), c(Ace, Hearts)];
        assert_eq!(best_made_hand(&cards), MadeHand::Flush);
    }

    #[test]
    fn two_pair() {
        let cards = vec![c(Ace, Hearts), c(Ace, Diamonds), c(King, Hearts), c(King, Clubs), c(Two, Spades)];
        assert_eq!(best_made_hand(&cards), MadeHand::TwoPair);
    }

    #[test]
    fn flush_draw_detected() {
        let hero = [c(Ace, Hearts), c(King, Hearts)];
        let board = vec![c(Two, Hearts), c(Seven, Hearts), c(Jack, Clubs)];
        assert_eq!(best_draw(&hero, &board), Some(Draw::FlushDraw));
    }

    #[test]
    fn oesd_detected() {
        let hero = [c(Seven, Clubs), c(Eight, Diamonds)];
        let board = vec![c(Five, Hearts), c(Six, Spades), c(Jack, Clubs)];
        assert_eq!(best_draw(&hero, &board), Some(Draw::OESD));
    }

    #[test]
    fn hand_category_preflop_none() {
        let hero = [c(Ace, Hearts), c(King, Diamonds)];
        assert!(hand_category(&hero, &[], Street::Preflop).is_none());
    }

    #[test]
    fn hand_category_flop_one_pair() {
        let hero = [c(Ace, Hearts), c(King, Diamonds)];
        let board = vec![c(Ace, Clubs), c(Seven, Spades), c(Two, Hearts)];
        let cat = hand_category(&hero, &board, Street::Flop).unwrap();
        assert!(cat.contains("One Pair") || cat.contains("Pair"), "{cat}");
    }
}
