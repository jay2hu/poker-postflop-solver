/// Postflop equity computation: hand vs range and range vs range.
///
/// Uses Monte Carlo completion of the remaining board cards.

use std::collections::HashSet;

use crate::{Card, Rank, Suit};
use crate::range::Range;

// ---------------------------------------------------------------------------
// Board texture
// ---------------------------------------------------------------------------

/// Structural characteristics of the community cards.
#[derive(Debug, Clone, Default)]
pub struct BoardTexture {
    pub is_monotone: bool,
    pub is_paired: bool,
    pub is_connected: bool,           // 3+ cards within 4 ranks of each other
    pub flush_draw_possible: bool,    // 2+ same suit
    pub straight_draw_possible: bool, // 2+ connected within 4 ranks
    pub high_card: Option<Rank>,
}

impl BoardTexture {
    /// Analyse a board (3–5 cards) and return its texture.
    pub fn analyze(board: &[Card]) -> Self {
        if board.is_empty() {
            return Self::default();
        }

        // Paired board
        let mut rank_counts = std::collections::HashMap::new();
        for c in board { *rank_counts.entry(c.rank as u8).or_insert(0u8) += 1; }
        let is_paired = rank_counts.values().any(|&n| n >= 2);

        // Suit counts → flush draw / monotone
        let mut suit_counts = [0u8; 4];
        for c in board {
            suit_counts[suit_idx(c.suit)] += 1;
        }
        let max_suit = *suit_counts.iter().max().unwrap_or(&0);
        let flush_draw_possible = max_suit >= 2;
        let is_monotone = board.len() >= 3 && max_suit as usize == board.len();

        // Connectivity
        let mut ranks: Vec<u8> = board.iter().map(|c| rank_val(c.rank)).collect();
        if ranks.contains(&14) { ranks.push(1); } // ace-low
        ranks.sort_unstable();
        ranks.dedup();
        let straight_draw_possible = ranks.windows(2).any(|w| w[1] - w[0] <= 4);
        let is_connected = board.len() >= 3 && {
            ranks.windows(3).any(|w| w[2] - w[0] <= 4)
        };

        // High card
        let high_card = board.iter().map(|c| c.rank).max_by_key(|&r| rank_val(r));

        BoardTexture {
            is_monotone,
            is_paired,
            is_connected,
            flush_draw_possible,
            straight_draw_possible,
            high_card,
        }
    }
}

fn suit_idx(s: Suit) -> usize {
    match s { Suit::Hearts => 0, Suit::Diamonds => 1, Suit::Clubs => 2, Suit::Spades => 3 }
}

fn rank_val(r: Rank) -> u8 {
    match r {
        Rank::Two => 2,   Rank::Three => 3, Rank::Four => 4,  Rank::Five => 5,
        Rank::Six => 6,   Rank::Seven => 7, Rank::Eight => 8, Rank::Nine => 9,
        Rank::Ten => 10,  Rank::Jack => 11, Rank::Queen => 12,Rank::King => 13,
        Rank::Ace => 14,
    }
}

// ---------------------------------------------------------------------------
// Deck helpers
// ---------------------------------------------------------------------------

fn all_cards() -> Vec<Card> {
    let ranks = [
        Rank::Two, Rank::Three, Rank::Four, Rank::Five, Rank::Six, Rank::Seven,
        Rank::Eight, Rank::Nine, Rank::Ten, Rank::Jack, Rank::Queen, Rank::King, Rank::Ace,
    ];
    let suits = [Suit::Hearts, Suit::Diamonds, Suit::Clubs, Suit::Spades];
    let mut deck = Vec::with_capacity(52);
    for &r in &ranks {
        for &s in &suits {
            deck.push(Card { rank: r, suit: s });
        }
    }
    deck
}

/// Simple XOR-shift RNG (no external dep, deterministic seed).
struct Rng { state: u64 }
impl Rng {
    fn new(seed: u64) -> Self { Rng { state: seed ^ 0xdeadbeefcafebabe } }
    fn next(&mut self) -> u64 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        self.state
    }
    fn usize_below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

fn shuffle<T>(v: &mut Vec<T>, rng: &mut Rng) {
    for i in (1..v.len()).rev() {
        let j = rng.usize_below(i + 1);
        v.swap(i, j);
    }
}

// ---------------------------------------------------------------------------
// Hand strength evaluation (simple rank-based)
// ---------------------------------------------------------------------------

/// Hand rank categories (higher = better).
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
enum HandRank {
    HighCard     = 0,
    OnePair      = 1,
    TwoPair      = 2,
    ThreeOfAKind = 3,
    Straight     = 4,
    Flush        = 5,
    FullHouse    = 6,
    FourOfAKind  = 7,
    StraightFlush= 8,
}

/// Evaluate best 5-card hand from up to 7 cards.
/// Returns (HandRank, tiebreaker score as u64 — higher is better).
fn evaluate_hand(cards: &[Card]) -> (HandRank, u64) {
    if cards.len() < 5 {
        let rv = cards.iter().map(|c| rank_val(c.rank) as u64).max().unwrap_or(0);
        return (HandRank::HighCard, rv);
    }

    // Try all C(n,5) combinations
    let n = cards.len();
    let mut best = (HandRank::HighCard, 0u64);

    let combos: Vec<Vec<usize>> = if n == 5 {
        vec![vec![0,1,2,3,4]]
    } else {
        five_combos(n)
    };

    for combo in combos {
        let hand: Vec<Card> = combo.iter().map(|&i| cards[i]).collect();
        let score = eval5(&hand);
        if score > best { best = score; }
    }
    best
}

fn five_combos(n: usize) -> Vec<Vec<usize>> {
    let mut result = Vec::new();
    for a in 0..n {
        for b in (a+1)..n {
            for c in (b+1)..n {
                for d in (c+1)..n {
                    for e in (d+1)..n {
                        result.push(vec![a,b,c,d,e]);
                    }
                }
            }
        }
    }
    result
}

fn eval5(hand: &[Card]) -> (HandRank, u64) {
    let mut ranks: Vec<u8> = hand.iter().map(|c| rank_val(c.rank)).collect();
    let suits: Vec<Suit> = hand.iter().map(|c| c.suit).collect();
    ranks.sort_unstable_by(|a, b| b.cmp(a)); // descending

    let is_flush = suits.windows(2).all(|w| w[0] == w[1]);

    // Straight check (including A-low)
    let is_straight = {
        let deduped: Vec<u8> = {
            let mut d = ranks.clone();
            d.dedup();
            d
        };
        if deduped.len() == 5 && deduped[0] - deduped[4] == 4 {
            true
        } else if deduped == vec![14, 5, 4, 3, 2] {
            true // wheel
        } else {
            false
        }
    };

    // Count rank frequencies
    let mut freq: std::collections::HashMap<u8, u8> = std::collections::HashMap::new();
    for &r in &ranks { *freq.entry(r).or_insert(0) += 1; }
    let mut counts: Vec<(u8, u8)> = freq.into_iter().collect();
    counts.sort_by(|a, b| b.1.cmp(&a.1).then(b.0.cmp(&a.0)));
    let freq_pattern: Vec<u8> = counts.iter().map(|c| c.1).collect();

    let tiebreak = ranks.iter().enumerate()
        .map(|(i, &r)| (r as u64) << ((4 - i) * 4))
        .sum::<u64>();

    if is_straight && is_flush {
        return (HandRank::StraightFlush, tiebreak);
    }
    if freq_pattern[0] == 4 { return (HandRank::FourOfAKind, tiebreak); }
    if freq_pattern[0] == 3 && freq_pattern[1] == 2 { return (HandRank::FullHouse, tiebreak); }
    if is_flush { return (HandRank::Flush, tiebreak); }
    if is_straight { return (HandRank::Straight, tiebreak); }
    if freq_pattern[0] == 3 { return (HandRank::ThreeOfAKind, tiebreak); }
    if freq_pattern[0] == 2 && freq_pattern[1] == 2 { return (HandRank::TwoPair, tiebreak); }
    if freq_pattern[0] == 2 { return (HandRank::OnePair, tiebreak); }
    (HandRank::HighCard, tiebreak)
}

// ---------------------------------------------------------------------------
// Equity functions
// ---------------------------------------------------------------------------

/// Compute equity of hero's specific hand vs villain's range on a given board.
///
/// Returns hero's equity (0.0–1.0, where 0.5 = breakeven).
pub fn hand_vs_range_equity(
    hero_hand: (Card, Card),
    villain_range: &Range,
    board: &[Card],
    iterations: u32,
) -> f64 {
    if villain_range.combos.is_empty() { return 0.5; }

    let deck = all_cards();
    let mut rng = Rng::new(42 + board.len() as u64 * 13);

    let mut wins = 0.0f64;
    let mut total_weight = 0.0f64;
    let iterations = iterations.max(100);

    // Sample villain combos weighted by frequency
    let villain_combos: Vec<&(Card, Card, f64)> = villain_range.combos.iter()
        .filter(|(c1, c2, _)| {
            *c1 != hero_hand.0 && *c1 != hero_hand.1
                && *c2 != hero_hand.0 && *c2 != hero_hand.1
                && !board.contains(c1) && !board.contains(c2)
        })
        .collect();

    if villain_combos.is_empty() { return 0.5; }

    for _ in 0..iterations {
        // Pick villain combo weighted by w
        let villain_combo = sample_combo(&villain_combos, &mut rng);
        let (vc1, vc2, vw) = villain_combo;

        // Build available deck
        let used: HashSet<u8> = [hero_hand.0, hero_hand.1, *vc1, *vc2]
            .iter().chain(board.iter())
            .map(|c| card_id(c))
            .collect();

        let mut available: Vec<Card> = deck.iter()
            .filter(|c| !used.contains(&card_id(c)))
            .cloned()
            .collect();
        shuffle(&mut available, &mut rng);

        // Complete the board
        let need = 5 - board.len().min(5);
        if available.len() < need { continue; }
        let run_out: Vec<Card> = available[..need].to_vec();

        let mut full_board = board.to_vec();
        full_board.extend_from_slice(&run_out);

        // Evaluate
        let hero_cards: Vec<Card> = vec![hero_hand.0, hero_hand.1].into_iter()
            .chain(full_board.iter().cloned()).collect();
        let villain_cards: Vec<Card> = vec![*vc1, *vc2].into_iter()
            .chain(full_board.iter().cloned()).collect();

        let hero_score = evaluate_hand(&hero_cards);
        let villain_score = evaluate_hand(&villain_cards);

        total_weight += vw;
        if hero_score > villain_score {
            wins += vw;
        } else if hero_score == villain_score {
            wins += vw * 0.5;
        }
    }

    if total_weight == 0.0 { 0.5 } else { wins / total_weight }
}

fn card_id(c: &Card) -> u8 {
    let r = match c.rank {
        Rank::Two => 0, Rank::Three => 1, Rank::Four => 2, Rank::Five => 3,
        Rank::Six => 4, Rank::Seven => 5, Rank::Eight => 6, Rank::Nine => 7,
        Rank::Ten => 8, Rank::Jack => 9, Rank::Queen => 10, Rank::King => 11,
        Rank::Ace => 12,
    };
    let s = match c.suit { Suit::Hearts => 0, Suit::Diamonds => 1, Suit::Clubs => 2, Suit::Spades => 3 };
    r * 4 + s
}

fn sample_combo<'a>(combos: &[&'a (Card, Card, f64)], rng: &mut Rng) -> &'a (Card, Card, f64) {
    let total: f64 = combos.iter().map(|c| c.2).sum();
    if total == 0.0 { return combos[rng.usize_below(combos.len())]; }
    let mut target = (rng.next() as f64 / u64::MAX as f64) * total;
    for combo in combos {
        target -= combo.2;
        if target <= 0.0 { return combo; }
    }
    combos[combos.len() - 1]
}

/// Compute equity for all hands in hero's range vs villain's range.
/// Returns sorted vec of (hand, equity) — highest equity first.
pub fn range_vs_range_equity(
    hero_range: &Range,
    villain_range: &Range,
    board: &[Card],
    iterations: u32,
) -> Vec<((Card, Card), f64)> {
    let per_hand = (iterations / hero_range.combos.len().max(1) as u32).max(50);
    let mut results: Vec<((Card, Card), f64)> = hero_range.combos.iter()
        .map(|(c1, c2, _)| {
            let eq = hand_vs_range_equity((*c1, *c2), villain_range, board, per_hand);
            ((*c1, *c2), eq)
        })
        .collect();
    results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    results
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Card, Rank, Suit};
    use crate::range::Range;

    fn card(rank: Rank, suit: Suit) -> Card { Card { rank, suit } }

    #[test]
    fn board_texture_monotone() {
        // Ah Kh Qh — monotone
        let board = vec![
            card(Rank::Ace, Suit::Hearts),
            card(Rank::King, Suit::Hearts),
            card(Rank::Queen, Suit::Hearts),
        ];
        let t = BoardTexture::analyze(&board);
        assert!(t.is_monotone, "Ah Kh Qh should be monotone");
        assert!(t.flush_draw_possible);
        assert!(!t.is_paired);
    }

    #[test]
    fn board_texture_dry() {
        // Kd 7c 2h — rainbow, no connects
        let board = vec![
            card(Rank::King, Suit::Diamonds),
            card(Rank::Seven, Suit::Clubs),
            card(Rank::Two, Suit::Hearts),
        ];
        let t = BoardTexture::analyze(&board);
        assert!(!t.is_monotone);
        assert!(!t.is_paired);
        assert!(!t.flush_draw_possible);
    }

    #[test]
    fn board_texture_paired() {
        let board = vec![
            card(Rank::King, Suit::Hearts),
            card(Rank::King, Suit::Spades),
            card(Rank::Two, Suit::Clubs),
        ];
        let t = BoardTexture::analyze(&board);
        assert!(t.is_paired);
    }

    #[test]
    fn board_texture_high_card() {
        let board = vec![
            card(Rank::Ace, Suit::Hearts),
            card(Rank::Seven, Suit::Clubs),
            card(Rank::Two, Suit::Diamonds),
        ];
        let t = BoardTexture::analyze(&board);
        assert_eq!(t.high_card, Some(Rank::Ace));
    }

    #[test]
    fn hand_vs_range_equity_strong_hand() {
        // AhKh vs broadway range on Kd 7c 2h → top pair top kicker, should be > 0.6
        let hero = (card(Rank::Ace, Suit::Hearts), card(Rank::King, Suit::Hearts));
        let villain = Range::from_str("AKs,AQs,AJs,KQs,QJs,JTs").expect("range");
        let board = vec![
            card(Rank::King, Suit::Diamonds),
            card(Rank::Seven, Suit::Clubs),
            card(Rank::Two, Suit::Hearts),
        ];
        let eq = hand_vs_range_equity(hero, &villain, &board, 2000);
        assert!(eq > 0.55, "AhKh on K72 rainbow vs broadway range should be >0.55, got {eq:.3}");
    }

    #[test]
    fn hand_vs_range_equity_weak_hand_vs_strong_range() {
        // 72o vs a tight strong range should have very low equity
        let hero = (card(Rank::Seven, Suit::Hearts), card(Rank::Two, Suit::Clubs));
        let villain = Range::from_str("AA,KK,QQ,AKs,AKo").expect("range");
        let board = vec![
            card(Rank::Ace, Suit::Diamonds),
            card(Rank::King, Suit::Clubs),
            card(Rank::Queen, Suit::Hearts),
        ];
        let eq = hand_vs_range_equity(hero, &villain, &board, 2000);
        assert!(eq < 0.15, "72o on AKQ vs AA/KK/QQ should have <0.15 equity, got {eq:.3}");
    }
}
