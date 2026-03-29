export interface PostflopResult {
  equity: number;
  action: 'bet' | 'raise' | 'call' | 'check' | 'fold';
  sizing_bb: number | null;
  sizing_pct_pot: number | null;
  pot_odds: number;
  spr: number;
  reasoning: string;
  board_texture: string;
}

export interface RangeHandData {
  hand: string;        // "AKs", "AA", "72o"
  weight: number;      // 0–1 how often this hand is in villain range
  equity: number;      // 0–1 villain's equity vs hero
  combos: number;
  equity_bucket: number; // 0=crushed(0-20%) 1=behind(20-40%) 2=even(40-60%) 3=ahead(60-80%) 4=crushing(80-100%)
}

export interface RangeAnalysis {
  hands: RangeHandData[];
  hero_avg_equity: number;
  villain_avg_equity: number;
  nut_advantage: 'hero' | 'villain' | 'neutral';
  equity_buckets: number[];   // 5 values summing to 1 (0-20,20-40,40-60,60-80,80-100%)
}

export interface BetOption {
  label: string;       // "33% pot"
  pct_pot: number;
  bb: number;
  ev: number;
  fold_eq: number;     // fold equity (0–1)
}

export interface BetSizeAnalysis {
  options: BetOption[];
  recommended_idx: number;
  pot_bb: number;
}

export interface SolverState {
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
  // New analysis state
  rangeAnalysis: RangeAnalysis | null;
  rangeLoading: boolean;
  betSizes: BetSizeAnalysis | null;
  betSizesLoading: boolean;
}
