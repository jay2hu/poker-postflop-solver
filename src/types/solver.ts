export interface PostflopResult {
  equity: number;              // 0.0–1.0 hero equity
  action: 'bet' | 'raise' | 'call' | 'check' | 'fold';
  sizing_bb: number | null;
  sizing_pct_pot: number | null;
  pot_odds: number;            // 0.0–1.0
  spr: number;                 // stack-to-pot ratio
  reasoning: string;
  board_texture: string;       // e.g. "Dry", "Wet", "Monotone"
}

export interface SolverState {
  heroHand: string[];          // ["Ah", "Kd"]
  board: string[];             // ["Kh", "7c", "2s"] (3–5 cards)
  potBb: number;
  heroStackBb: number;
  villainStackBb: number;
  toCallBb: number;
  isIp: boolean;
  villainRange: string;        // "top5%" | "top10%" | ... | custom text
  result: PostflopResult | null;
  loading: boolean;
  error: string | null;
}
