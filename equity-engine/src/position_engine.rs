/// Position derivation from seat numbers and dealer button detection.
///
/// # Terminology
/// Position is determined clockwise from the dealer:
/// - Seat 0 offset = BTN (dealer)
/// - Seat 1 offset = SB
/// - Seat 2 offset = BB
/// - Remaining seats = early/middle/late, varies by table size

use crate::Position;

// ---------------------------------------------------------------------------
// Position tables per table size
// ---------------------------------------------------------------------------

/// Returns the ordered list of positions for a given table size,
/// starting from the dealer (BTN) and going clockwise.
///
/// Index 0 = BTN, index 1 = SB, index 2 = BB, index 3+ = early positions.
pub fn positions_for_table_size(table_size: u8) -> Vec<Position> {
    match table_size {
        2 => vec![Position::BTN, Position::BB],
        3 => vec![Position::BTN, Position::SB, Position::BB],
        4 => vec![Position::BTN, Position::SB, Position::BB, Position::UTG],
        5 => vec![Position::BTN, Position::SB, Position::BB, Position::UTG, Position::CO],
        6 => vec![
            Position::BTN,
            Position::SB,
            Position::BB,
            Position::UTG,
            Position::HJ,
            Position::CO,
        ],
        7 => vec![
            Position::BTN,
            Position::SB,
            Position::BB,
            Position::UTG,
            Position::LJ,
            Position::HJ,
            Position::CO,
        ],
        8 => vec![
            Position::BTN,
            Position::SB,
            Position::BB,
            Position::UTG,
            Position::UTG1,
            Position::LJ,
            Position::HJ,
            Position::CO,
        ],
        9 => vec![
            Position::BTN,
            Position::SB,
            Position::BB,
            Position::UTG,
            Position::UTG1,
            Position::UTG2,
            Position::LJ,
            Position::HJ,
            Position::CO,
        ],
        _ => {
            // >9: use 9-max positions, ignore extra seats
            let base = positions_for_table_size(9);
            base
        }
    }
}

// ---------------------------------------------------------------------------
// Core derivation
// ---------------------------------------------------------------------------

/// Derive hero's position given table size, dealer seat, and hero seat.
///
/// Seats are 1-based. Wraps correctly around the table.
///
/// # Examples
/// ```
/// use equity_engine::{position_engine::derive_position, Position};
/// assert_eq!(derive_position(8, 3, 3), Position::BTN);  // hero IS dealer
/// assert_eq!(derive_position(8, 3, 6), Position::UTG);  // 3 past dealer: BTN=3,SB=4,BB=5,UTG=6
/// ```
pub fn derive_position(table_size: u8, dealer_seat: u8, hero_seat: u8) -> Position {
    let positions = positions_for_table_size(table_size);
    let n = table_size as usize;
    // Convert 1-based seats to 0-based offsets
    let dealer_0 = (dealer_seat as usize).wrapping_sub(1) % n;
    let hero_0   = (hero_seat  as usize).wrapping_sub(1) % n;
    // Clockwise offset from dealer
    let offset = (hero_0 + n - dealer_0) % n;
    let clamped = offset.min(positions.len() - 1);
    positions[clamped]
}

/// Return position for every active seat at the table.
///
/// Returns `(seat_number, Position)` tuples for seats 1..=table_size.
pub fn derive_all_positions(table_size: u8, dealer_seat: u8) -> Vec<(u8, Position)> {
    (1..=table_size)
        .map(|seat| (seat, derive_position(table_size, dealer_seat, seat)))
        .collect()
}

// ---------------------------------------------------------------------------
// Dealer button detection via bounding-box proximity
// ---------------------------------------------------------------------------

/// A seat region: seat number + bounding rect [x1, y1, x2, y2].
#[derive(Debug, Clone)]
pub struct SeatRegion {
    pub seat: u8,
    pub rect: [u32; 4],
}

impl SeatRegion {
    /// Center point of the seat region.
    pub fn center(&self) -> (u32, u32) {
        let [x1, y1, x2, y2] = self.rect;
        ((x1 + x2) / 2, (y1 + y2) / 2)
    }
}

/// Detect which seat the dealer button is closest to.
///
/// `button_rect` is the bounding box [x1, y1, x2, y2] of the detected
/// dealer button puck (from the calibrator config).
/// `seats` is the list of known seat regions.
///
/// Returns the 1-based seat number of the nearest seat, or `None` if
/// `seats` is empty.
pub fn detect_dealer_seat(button_rect: [u32; 4], seats: &[SeatRegion]) -> Option<u8> {
    if seats.is_empty() {
        return None;
    }

    let bx = (button_rect[0] + button_rect[2]) / 2;
    let by = (button_rect[1] + button_rect[3]) / 2;

    let nearest = seats.iter().min_by_key(|s| {
        let (cx, cy) = s.center();
        let dx = cx as i64 - bx as i64;
        let dy = cy as i64 - by as i64;
        dx * dx + dy * dy
    })?;

    Some(nearest.seat)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Position;

    // -----------------------------------------------------------------------
    // 8-handed tests
    // -----------------------------------------------------------------------

    #[test]
    fn eight_handed_hero_is_dealer() {
        // dealer=seat3, hero=seat3 → BTN
        assert_eq!(derive_position(8, 3, 3), Position::BTN);
    }

    #[test]
    fn eight_handed_hero_5_is_bb() {
        // dealer=seat3, 8-handed: BTN=3,SB=4,BB=5,UTG=6,UTG1=7,LJ=8,HJ=1,CO=2
        assert_eq!(derive_position(8, 3, 5), Position::BB);
    }

    #[test]
    fn eight_handed_hero_is_utg_correct() {
        // dealer=seat3, hero=seat6 → UTG (offset 3)
        assert_eq!(derive_position(8, 3, 6), Position::UTG);
    }

    // -----------------------------------------------------------------------
    // 6-max tests
    // -----------------------------------------------------------------------

    #[test]
    fn six_max_hero_is_co() {
        // dealer=seat1, table=6
        // BTN=1,SB=2,BB=3,UTG=4,HJ=5,CO=6
        // hero=seat4 → UTG
        // hero=seat6 → CO
        assert_eq!(derive_position(6, 1, 6), Position::CO);
    }

    #[test]
    fn six_max_dealer_1_hero_4_utg() {
        assert_eq!(derive_position(6, 1, 4), Position::UTG);
    }

    // -----------------------------------------------------------------------
    // 9-max tests
    // -----------------------------------------------------------------------

    #[test]
    fn nine_max_all_positions() {
        let all = derive_all_positions(9, 1);
        // seat1=BTN, seat2=SB, seat3=BB, seat4=UTG, seat5=UTG1, seat6=UTG2, seat7=LJ, seat8=HJ, seat9=CO
        assert_eq!(all[0], (1, Position::BTN));
        assert_eq!(all[1], (2, Position::SB));
        assert_eq!(all[2], (3, Position::BB));
        assert_eq!(all[3], (4, Position::UTG));
        assert_eq!(all[4], (5, Position::UTG1));
        assert_eq!(all[5], (6, Position::UTG2));
        assert_eq!(all[6], (7, Position::LJ));
        assert_eq!(all[7], (8, Position::HJ));
        assert_eq!(all[8], (9, Position::CO));
    }

    // -----------------------------------------------------------------------
    // Wrap-around test
    // -----------------------------------------------------------------------

    #[test]
    fn wraparound_dealer_8_hero_2_is_bb_eight_handed() {
        // dealer=8, 8-handed: BTN=8,SB=1,BB=2,...
        assert_eq!(derive_position(8, 8, 2), Position::BB);
    }

    #[test]
    fn wraparound_dealer_8_hero_1_is_sb_eight_handed() {
        assert_eq!(derive_position(8, 8, 1), Position::SB);
    }

    // -----------------------------------------------------------------------
    // derive_all_positions
    // -----------------------------------------------------------------------

    #[test]
    fn six_max_all_positions_dealer_1() {
        let all = derive_all_positions(6, 1);
        assert_eq!(all[0], (1, Position::BTN));
        assert_eq!(all[1], (2, Position::SB));
        assert_eq!(all[2], (3, Position::BB));
        assert_eq!(all[3], (4, Position::UTG));
        assert_eq!(all[4], (5, Position::HJ));
        assert_eq!(all[5], (6, Position::CO));
    }

    // -----------------------------------------------------------------------
    // detect_dealer_seat
    // -----------------------------------------------------------------------

    #[test]
    fn detect_dealer_seat_nearest() {
        let seats = vec![
            SeatRegion { seat: 1, rect: [0, 0, 100, 60] },   // center (50, 30)
            SeatRegion { seat: 2, rect: [300, 0, 400, 60] },  // center (350, 30)
            SeatRegion { seat: 3, rect: [600, 0, 700, 60] },  // center (650, 30)
        ];
        // Button near seat 2
        assert_eq!(detect_dealer_seat([340, 70, 360, 90], &seats), Some(2));
        // Button near seat 1
        assert_eq!(detect_dealer_seat([40, 70, 60, 90], &seats), Some(1));
        // Button near seat 3
        assert_eq!(detect_dealer_seat([640, 70, 660, 90], &seats), Some(3));
    }

    #[test]
    fn detect_dealer_seat_empty_returns_none() {
        assert_eq!(detect_dealer_seat([0, 0, 10, 10], &[]), None);
    }
}
