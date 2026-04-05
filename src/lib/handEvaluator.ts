// Hand strength evaluator — pure TypeScript, no Rust needed

export type HandCategory =
  | 'straight_flush'
  | 'four_of_a_kind'
  | 'full_house'
  | 'flush'
  | 'straight'
  | 'three_of_a_kind'
  | 'two_pair'
  | 'tptk'           // top pair top kicker
  | 'top_pair'       // top pair weak kicker
  | 'middle_pair'
  | 'bottom_pair'
  | 'flush_draw'
  | 'oesd'           // open-ended straight draw
  | 'gutshot'
  | 'overcards'
  | 'nothing';

export interface HandStrengthResult {
  category: HandCategory;
  label: string;           // human-readable
  description: string;     // brief description
  strength: 'strong' | 'medium' | 'draw' | 'weak';
}

// ── Card parsing ──────────────────────────────────────────────────────────────

const RANK_VALUES: Record<string, number> = {
  '2':2, '3':3, '4':4, '5':5, '6':6, '7':7, '8':8,
  '9':9, 'T':10, 'J':11, 'Q':12, 'K':13, 'A':14,
};

function parseCard(card: string): { rank: number; suit: string } {
  return { rank: RANK_VALUES[card.slice(0, -1).toUpperCase()] ?? 0, suit: card.slice(-1).toLowerCase() };
}

function rankStr(rv: number): string {
  const m: Record<number, string> = { 14:'A', 13:'K', 12:'Q', 11:'J', 10:'T' };
  return m[rv] ?? String(rv);
}

// ── Evaluation helpers ────────────────────────────────────────────────────────

function hasStraight(ranks: number[]): boolean {
  const unique = [...new Set(ranks)].sort((a, b) => a - b);
  // Check regular straight
  for (let i = 0; i <= unique.length - 5; i++) {
    if (unique[i + 4] - unique[i] === 4 && new Set(unique.slice(i, i + 5)).size === 5) return true;
  }
  // Wheel: A-2-3-4-5
  if (unique.includes(14) && [2,3,4,5].every(r => unique.includes(r))) return true;
  return false;
}

function hasStraightDraw(_heroRanks: number[], allRanks: number[]): 'oesd' | 'gutshot' | null {
  const unique = [...new Set(allRanks)].sort((a, b) => a - b);
  // Check for 4-card sequences (OESD)
  for (let i = 0; i <= unique.length - 4; i++) {
    const window = unique.slice(i, i + 4);
    if (window[3] - window[0] === 3 && new Set(window).size === 4) {
      // Is open-ended (not A-high or 2-low where one end is blocked)
      const lo = window[0], hi = window[3];
      if (lo > 2 && hi < 14) return 'oesd';
    }
  }
  // Gutshot: 4 cards with a single gap
  for (let i = 0; i <= unique.length - 4; i++) {
    const slice5 = unique.slice(i, i + 5);
    if (slice5.length >= 5) {
      const gaps = slice5.filter((_, j) => j > 0 && slice5[j] - slice5[j-1] === 2);
      if (gaps.length === 1) return 'gutshot';
    }
  }
  // Gutshot via 5-card window with one gap
  for (let start = 2; start <= 10; start++) {
    const seq = [start, start+1, start+2, start+3, start+4];
    const have = seq.filter(r => unique.includes(r));
    if (have.length === 4) return 'gutshot';
  }
  return null;
}

// ── Main evaluator ────────────────────────────────────────────────────────────

export function evaluateHandStrength(
  heroHand: string[],  // ["Ah", "Kd"]
  board: string[],     // ["Kh", "7c", "2s"]
): HandStrengthResult | null {
  if (heroHand.length < 2 || board.length < 3) return null;

  const heroParsed = heroHand.map(parseCard);
  const boardParsed = board.map(parseCard);
  const allParsed   = [...heroParsed, ...boardParsed];

  const heroRanks  = heroParsed.map(c => c.rank);
  const boardRanks = boardParsed.map(c => c.rank);
  const allRanks   = allParsed.map(c => c.rank);
  const allSuits   = allParsed.map(c => c.suit);
  const heroSuits  = heroParsed.map(c => c.suit);

  // Rank frequency maps
  const rankFreq = new Map<number, number>();
  allRanks.forEach(r => rankFreq.set(r, (rankFreq.get(r) ?? 0) + 1));
  const boardRankFreq = new Map<number, number>();
  boardRanks.forEach(r => boardRankFreq.set(r, (boardRankFreq.get(r) ?? 0) + 1));

  const pairs  = [...rankFreq.entries()].filter(([, c]) => c >= 2).map(([r]) => r).sort((a, b) => b - a);
  const trips  = [...rankFreq.entries()].filter(([, c]) => c >= 3).map(([r]) => r);
  const quads  = [...rankFreq.entries()].filter(([, c]) => c >= 4).map(([r]) => r);

  // Suit counts for flush
  const suitFreq = new Map<string, number>();
  allSuits.forEach(s => suitFreq.set(s, (suitFreq.get(s) ?? 0) + 1));
  const hasFlush = [...suitFreq.values()].some(c => c >= 5);
  const flushDrawSuit = [...suitFreq.entries()].find(([, c]) => c === 4)?.[0];
  const hasFlushDraw  = !!flushDrawSuit && heroSuits.includes(flushDrawSuit);

  const hasStraightMade = hasStraight(allRanks);
  const straightDraw    = !hasStraightMade ? hasStraightDraw(heroRanks, allRanks) : null;

  const sortedBoard = [...boardRanks].sort((a, b) => b - a);
  const topBoardRank = sortedBoard[0];

  // ── Classify ──────────────────────────────────────────────────────────────

  // Straight/Royal flush (simplified — check flush + straight)
  if (hasFlush && hasStraightMade) {
    return { category: 'straight_flush', label: 'Straight Flush', description: 'Extremely rare premium hand', strength: 'strong' };
  }
  if (quads.length > 0) {
    return { category: 'four_of_a_kind', label: 'Four of a Kind', description: `Quads — ${rankStr(quads[0])}s`, strength: 'strong' };
  }
  if (trips.length > 0 && pairs.length >= 2) {
    return { category: 'full_house', label: 'Full House', description: `${rankStr(trips[0])}s full of ${rankStr(pairs.filter(r => r !== trips[0])[0] ?? 0)}s`, strength: 'strong' };
  }
  if (hasFlush) {
    return { category: 'flush', label: 'Flush', description: 'Flush — strong made hand', strength: 'strong' };
  }
  if (hasStraightMade) {
    return { category: 'straight', label: 'Straight', description: 'Straight — strong made hand', strength: 'strong' };
  }
  if (trips.length > 0) {
    return { category: 'three_of_a_kind', label: 'Three of a Kind', description: `Trip ${rankStr(trips[0])}s`, strength: 'strong' };
  }
  if (pairs.length >= 2) {
    return { category: 'two_pair', label: 'Two Pair', description: `Two pair: ${rankStr(pairs[0])}s and ${rankStr(pairs[1])}s`, strength: 'medium' };
  }

  // Single pair — determine where it is
  if (pairs.length === 1) {
    const pairRank = pairs[0];
    const heroHasPair = heroRanks.includes(pairRank) && boardRanks.includes(pairRank);
    const isPocketPair = heroRanks[0] === heroRanks[1] && pairRank === heroRanks[0];

    if (isPocketPair) {
      if (pairRank > topBoardRank) {
        return { category: 'tptk', label: 'Overpair', description: `Pocket ${rankStr(pairRank)}s above board`, strength: 'medium' };
      }
      return { category: 'middle_pair', label: 'Pocket Pair (underpair)', description: `${rankStr(pairRank)}s on ${rankStr(topBoardRank)}-high board`, strength: 'weak' };
    }

    if (heroHasPair) {
      if (pairRank === topBoardRank) {
        const kicker = heroRanks.filter(r => r !== pairRank)[0];
        const isTopKicker = kicker >= (sortedBoard[1] ?? 0);
        if (isTopKicker && kicker >= 10) {
          return { category: 'tptk', label: 'Top Pair (top kicker)', description: `Pair of ${rankStr(pairRank)}s, ${rankStr(kicker)} kicker`, strength: 'medium' };
        }
        return { category: 'top_pair', label: 'Top Pair (weak kicker)', description: `Pair of ${rankStr(pairRank)}s, weak kicker`, strength: 'medium' };
      }
      const boardPos = sortedBoard.indexOf(pairRank);
      if (boardPos === 1) return { category: 'middle_pair', label: 'Middle Pair', description: `Pair of ${rankStr(pairRank)}s (2nd board card)`, strength: 'weak' };
      return { category: 'bottom_pair', label: 'Bottom Pair', description: `Pair of ${rankStr(pairRank)}s (bottom of board)`, strength: 'weak' };
    }
  }

  // Draws
  if (hasFlushDraw) {
    const isNutDraw = heroRanks.includes(14) || heroRanks.includes(13);
    return {
      category: 'flush_draw',
      label: isNutDraw ? 'Nut Flush Draw' : 'Flush Draw',
      description: isNutDraw ? 'Nut flush draw — 9 outs' : 'Flush draw — 9 outs',
      strength: 'draw',
    };
  }
  if (straightDraw === 'oesd') {
    return { category: 'oesd', label: 'Open-Ended Straight Draw', description: 'OESD — 8 outs', strength: 'draw' };
  }
  if (straightDraw === 'gutshot') {
    return { category: 'gutshot', label: 'Gutshot', description: 'Inside straight draw — 4 outs', strength: 'draw' };
  }

  // Overcards / nothing
  const overcards = heroRanks.filter(r => r > topBoardRank);
  if (overcards.length >= 1) {
    return { category: 'overcards', label: `Overcards (${overcards.map(rankStr).join('')})`, description: 'No pair — floating with overcards', strength: 'weak' };
  }
  return { category: 'nothing', label: 'High Card', description: 'No pair, no draw', strength: 'weak' };
}
