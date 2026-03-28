import { create } from 'zustand';
import type { SolverState, PostflopResult } from '../types/solver';

const IS_TAURI = '__TAURI_INTERNALS__' in window;

/** Deterministic mock solve */
function mockSolve(potBb: number, heroStackBb: number, toCallBb: number, board: string[]): PostflopResult {
  const equity = 0.5 + (board.length * 0.032) % 0.3;
  const spr = heroStackBb / Math.max(potBb, 1);
  const potOdds = toCallBb > 0 ? toCallBb / (potBb + toCallBb) : 0;
  const action = equity > 0.6 ? 'bet' : equity > 0.45 ? 'call' : 'fold';
  const sizingPct = action === 'bet' ? 65 : null;
  const sizingBb = sizingPct ? parseFloat((potBb * sizingPct / 100).toFixed(1)) : null;
  return {
    equity: parseFloat(equity.toFixed(3)),
    action,
    sizing_bb: sizingBb,
    sizing_pct_pot: sizingPct,
    pot_odds: parseFloat(potOdds.toFixed(3)),
    spr: parseFloat(spr.toFixed(1)),
    reasoning: action === 'bet'
      ? 'Strong equity position. Bet for value and protection.'
      : action === 'call'
      ? 'Marginal equity — calling is correct given pot odds.'
      : 'Insufficient equity against villain range.',
    board_texture: board.length >= 3
      ? (new Set(board.map(c => c.slice(-1))).size === 1 ? 'Monotone' : 'Dry')
      : '—',
  };
}

interface Store extends SolverState {
  setHeroHand: (cards: string[]) => void;
  setBoard: (cards: string[]) => void;
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
  heroHand: [],
  board: [],
  potBb: 10,
  heroStackBb: 100,
  villainStackBb: 100,
  toCallBb: 0,
  isIp: true,
  villainRange: 'top15%',
  result: null,
  loading: false,
  error: null,
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
      set({ error: 'Need hero hand (2 cards) + board (3–5 cards)' });
      return;
    }
    set({ loading: true, error: null, result: null });
    try {
      let result: PostflopResult;
      if (IS_TAURI) {
        const { invoke } = await import('@tauri-apps/api/core');
        result = await invoke<PostflopResult>('solve_postflop', {
          heroHand, board, potBb, heroStackBb, villainStackBb, toCallBb, isIp,
          villainRangeStr: villainRange,
        });
      } else {
        await new Promise(r => setTimeout(r, 600));
        result = mockSolve(potBb, heroStackBb, toCallBb, board);
      }
      set({ result, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  reset: () => set({ ...INITIAL }),
}));
