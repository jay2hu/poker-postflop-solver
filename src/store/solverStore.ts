import { create } from 'zustand';
import type { PostflopResult } from '../types';

// Detect Tauri environment
const isTauri = () => typeof window !== 'undefined' && '__TAURI__' in window;

function mockSolve(params: {
  hero_hand: string[];
  board: string[];
  pot_bb: number;
  hero_stack_bb: number;
  villain_stack_bb: number;
  to_call_bb: number;
  is_ip: boolean;
  villain_range: string;
}): PostflopResult {
  const suits = params.board.map((c) => c[1]);
  const is_monotone = suits.length >= 3 && suits.every((s) => s === suits[0]);
  const ranks = params.board.map((c) => c[0]);
  const rankCounts = ranks.reduce<Record<string, number>>((acc, r) => {
    acc[r] = (acc[r] || 0) + 1;
    return acc;
  }, {});
  const is_paired = Object.values(rankCounts).some((v) => v >= 2);
  const rankOrder = 'AKQJT98765432';
  const rankNums = ranks.map((r) => rankOrder.indexOf(r));
  rankNums.sort((a, b) => a - b);
  let has_straight_draw = false;
  for (let i = 0; i < rankNums.length - 1; i++) {
    if (Math.abs(rankNums[i] - rankNums[i + 1]) <= 2) has_straight_draw = true;
  }
  const suitCounts = suits.reduce<Record<string, number>>((acc, s) => {
    acc[s] = (acc[s] || 0) + 1;
    return acc;
  }, {});
  const has_flush_draw = Object.values(suitCounts).some((v) => v >= 2);

  const parts = [
    is_monotone ? 'Monotone' : '',
    is_paired ? 'Paired' : '',
    has_straight_draw ? 'Straight Draw' : '',
    has_flush_draw && !is_monotone ? 'Flush Draw' : '',
  ].filter(Boolean);
  const texture_label = parts.length ? parts.join(', ') : 'Dry';

  const equity = 55 + Math.random() * 20;
  const spr = params.hero_stack_bb / params.pot_bb;
  const pot_odds = params.to_call_bb > 0 ? params.to_call_bb / (params.pot_bb + params.to_call_bb) : 0;

  let action = 'BET';
  if (params.to_call_bb > 0) {
    action = equity > 40 ? 'CALL' : 'FOLD';
  } else if (equity < 45) {
    action = 'CHECK';
  }

  const sizing_bb = action === 'BET' ? Math.round(params.pot_bb * 0.66 * 10) / 10 : null;
  const sizing_pct_pot = action === 'BET' ? 66 : null;

  return {
    equity: Math.round(equity * 10) / 10,
    pot_odds: Math.round(pot_odds * 1000) / 10,
    spr: Math.round(spr * 10) / 10,
    action,
    sizing_bb,
    sizing_pct_pot,
    ev_estimate: Math.round((equity / 100 - 0.5) * params.pot_bb * 10) / 10,
    reasoning: `With ${equity.toFixed(1)}% equity on a ${texture_label.toLowerCase()} board, ${action.toLowerCase()} is recommended. SPR of ${spr.toFixed(1)} suggests ${spr < 3 ? 'shallow stack play' : 'deep stack considerations'}.`,
    board_texture: {
      is_monotone,
      is_paired,
      has_straight_draw,
      has_flush_draw,
      texture_label,
    },
  };
}

interface SolverState {
  heroHand: string[];
  board: string[];
  potBb: number;
  heroStackBb: number;
  villainStackBb: number;
  toCallBb: number;
  isIp: boolean;
  villainRange: string;
  result: PostflopResult | null;
  loading: boolean;
  error: string | null;

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

export const useSolverStore = create<SolverState>((set, get) => ({
  heroHand: [],
  board: [],
  potBb: 10,
  heroStackBb: 100,
  villainStackBb: 100,
  toCallBb: 0,
  isIp: true,
  villainRange: 'top20%',
  result: null,
  loading: false,
  error: null,

  setHeroHand: (cards) => set({ heroHand: cards }),
  setBoard: (cards) => set({ board: cards }),
  setPotBb: (v) => set({ potBb: v }),
  setHeroStackBb: (v) => set({ heroStackBb: v }),
  setVillainStackBb: (v) => set({ villainStackBb: v }),
  setToCallBb: (v) => set({ toCallBb: v }),
  setIsIp: (v) => set({ isIp: v }),
  setVillainRange: (v) => set({ villainRange: v }),

  solve: async () => {
    const s = get();
    set({ loading: true, error: null, result: null });
    const params = {
      hero_hand: s.heroHand,
      board: s.board,
      pot_bb: s.potBb,
      hero_stack_bb: s.heroStackBb,
      villain_stack_bb: s.villainStackBb,
      to_call_bb: s.toCallBb,
      is_ip: s.isIp,
      villain_range: s.villainRange,
    };
    try {
      let result: PostflopResult;
      if (isTauri()) {
        const { invoke } = await import('@tauri-apps/api/core');
        result = await invoke<PostflopResult>('solve_postflop', params);
      } else {
        await new Promise((r) => setTimeout(r, 600));
        result = mockSolve(params);
      }
      set({ result, loading: false });
    } catch (err: unknown) {
      set({ error: String(err), loading: false });
    }
  },

  reset: () =>
    set({
      heroHand: [],
      board: [],
      potBb: 10,
      heroStackBb: 100,
      villainStackBb: 100,
      toCallBb: 0,
      isIp: true,
      villainRange: 'top20%',
      result: null,
      loading: false,
      error: null,
    }),
}));
