import { create } from 'zustand';
import type { SolverState, PostflopResult, RangeAnalysis, BetSizeAnalysis, RangeHandData } from '../types/solver';
import { RANKS } from '../components/RangeGrid';

const IS_TAURI = '__TAURI_INTERNALS__' in window;

// ── Mock generators ───────────────────────────────────────────────────────────

function mockSolve(potBb: number, heroStackBb: number, toCallBb: number, board: string[]): PostflopResult {
  const equity = 0.5 + (board.length * 0.032) % 0.3;
  const spr    = heroStackBb / Math.max(potBb, 1);
  const potOdds = toCallBb > 0 ? toCallBb / (potBb + toCallBb) : 0;
  const action = equity > 0.6 ? 'bet' : equity > 0.45 ? 'call' : 'fold';
  const sizingPct = action === 'bet' ? 65 : null;
  const sizingBb  = sizingPct ? parseFloat((potBb * sizingPct / 100).toFixed(1)) : null;
  const suits: Set<string> = new Set(board.map(c => c.slice(-1)));
  const texture = suits.size === 1 ? 'Monotone' : suits.size === 2 ? 'Two-tone' : 'Rainbow';
  return {
    equity: parseFloat(equity.toFixed(3)),
    action, sizing_bb: sizingBb, sizing_pct_pot: sizingPct,
    pot_odds: parseFloat(potOdds.toFixed(3)),
    spr: parseFloat(spr.toFixed(1)),
    reasoning: action === 'bet' ? 'Strong equity — bet for value and protection.'
      : action === 'call' ? 'Marginal spot — calling is correct given pot odds.'
      : 'Insufficient equity against villain range.',
    board_texture: texture,
  };
}

function mockRangeAnalysis(board: string[], heroHand: string[]): RangeAnalysis {
  // Build 169-hand mock
  const hands: RangeHandData[] = [];
  for (let r = 0; r < 13; r++) {
    for (let c = 0; c < 13; c++) {
      const rank1 = RANKS[r], rank2 = RANKS[c];
      const hand = r === c ? `${rank1}${rank1}` : r < c ? `${rank1}${rank2}s` : `${rank1}${rank2}o`;
      const inRange = heroHand.every(h => !h.startsWith(rank1) && !h.startsWith(rank2));
      const weight  = inRange ? (0.3 + (r + c) % 7 * 0.1) : 0;
      const equity  = 0.2 + ((r * 13 + c + board.length) % 60) / 100;
      const bucket  = equity < 0.2 ? 0 : equity < 0.4 ? 1 : equity < 0.6 ? 2 : equity < 0.8 ? 3 : 4;
      hands.push({ hand, weight: parseFloat(weight.toFixed(2)), equity: parseFloat(equity.toFixed(3)), combos: Math.floor(weight * 6), equity_bucket: bucket });
    }
  }
  return {
    hands,
    hero_avg_equity: 0.564,
    villain_avg_equity: 0.436,
    nut_advantage: 'hero',
    buckets: [0.12, 0.18, 0.22, 0.28, 0.20],
  };
}

function mockBetSizes(potBb: number): BetSizeAnalysis {
  const opts = [
    { label: '25% pot', pct_pot: 25, bb: potBb * 0.25, ev: 1.8,  fold_eq: 0.28 },
    { label: '50% pot', pct_pot: 50, bb: potBb * 0.5,  ev: 2.4,  fold_eq: 0.38 },
    { label: '75% pot', pct_pot: 75, bb: potBb * 0.75, ev: 2.7,  fold_eq: 0.46 },
    { label: '100% pot',pct_pot: 100,bb: potBb,         ev: 2.5,  fold_eq: 0.52 },
    { label: 'All-in',  pct_pot: 200,bb: potBb * 2,    ev: 1.9,  fold_eq: 0.62 },
  ].map(o => ({ ...o, bb: parseFloat(o.bb.toFixed(1)) }));
  return { options: opts, recommended_idx: 2, pot_bb: potBb };
}

// ── Store ─────────────────────────────────────────────────────────────────────

interface Store extends SolverState {
  setHeroHand: (c: string[]) => void;
  setBoard: (c: string[]) => void;
  setPotBb: (v: number) => void;
  setHeroStackBb: (v: number) => void;
  setVillainStackBb: (v: number) => void;
  setToCallBb: (v: number) => void;
  setIsIp: (v: boolean) => void;
  setVillainRange: (v: string) => void;
  solve: () => Promise<void>;
  reset: () => void;
}

const INITIAL: SolverState = {
  heroHand: [], board: [], potBb: 10,
  heroStackBb: 100, villainStackBb: 100, toCallBb: 0,
  isIp: true, villainRange: 'top15%',
  result: null, loading: false, error: null,
  rangeAnalysis: null, rangeLoading: false,
  betSizes: null, betSizesLoading: false,
};

export const useSolverStore = create<Store>((set, get) => ({
  ...INITIAL,
  setHeroHand: (heroHand) => set({ heroHand }),
  setBoard: (board) => set({ board }),
  setPotBb: (potBb) => set({ potBb }),
  setHeroStackBb: (heroStackBb) => set({ heroStackBb }),
  setVillainStackBb: (villainStackBb) => set({ villainStackBb }),
  setToCallBb: (toCallBb) => set({ toCallBb }),
  setIsIp: (isIp) => set({ isIp }),
  setVillainRange: (villainRange) => set({ villainRange }),

  solve: async () => {
    const { heroHand, board, potBb, heroStackBb, villainStackBb, toCallBb, isIp, villainRange } = get();
    if (heroHand.length < 2 || board.length < 3) {
      set({ error: 'Need hero hand (2 cards) + board (3–5 cards)' }); return;
    }
    set({ loading: true, error: null, result: null, rangeAnalysis: null, betSizes: null });
    try {
      let result: PostflopResult;
      if (IS_TAURI) {
        const { invoke } = await import('@tauri-apps/api/core');
        result = await invoke<PostflopResult>('solve_postflop', {
          heroHand, board, potBb, heroStackBb, villainStackBb, toCallBb, isIp, villainRangeStr: villainRange,
        });
      } else {
        await new Promise(r => setTimeout(r, 600));
        result = mockSolve(potBb, heroStackBb, toCallBb, board);
      }
      set({ result, loading: false });

      // Fire secondary analyses in parallel
      set({ rangeLoading: true, betSizesLoading: true });
      const [rangeAnalysis, betSizes] = await Promise.all([
        IS_TAURI
          ? import('@tauri-apps/api/core').then(({ invoke }) =>
              invoke<RangeAnalysis>('analyze_range_on_board', { heroHand, board, villainRangeStr: villainRange }))
          : (async () => { await new Promise(r => setTimeout(r, 400)); return mockRangeAnalysis(board, heroHand); })(),
        IS_TAURI
          ? import('@tauri-apps/api/core').then(({ invoke }) =>
              invoke<BetSizeAnalysis>('compare_bet_sizes', { heroHand, board, potBb, heroStackBb, villainStackBb, villainRangeStr: villainRange }))
          : (async () => { await new Promise(r => setTimeout(r, 300)); return mockBetSizes(potBb); })(),
      ]);
      set({ rangeAnalysis, betSizes, rangeLoading: false, betSizesLoading: false });
    } catch (e) {
      set({ error: String(e), loading: false, rangeLoading: false, betSizesLoading: false });
    }
  },

  reset: () => set({ ...INITIAL }),
}));
