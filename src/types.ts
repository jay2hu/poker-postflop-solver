export interface PostflopResult {
  equity: number;
  pot_odds: number;
  spr: number;
  action: string;
  sizing_bb: number | null;
  sizing_pct_pot: number | null;
  ev_estimate: number;
  reasoning: string;
  board_texture: {
    is_monotone: boolean;
    is_paired: boolean;
    has_straight_draw: boolean;
    has_flush_draw: boolean;
    texture_label: string;
  };
}
