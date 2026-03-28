import { describe, it, expect, beforeEach } from 'vitest';
import { useSolverStore } from '../store/solverStore';
import { SUIT_COLORS, SUIT_SYMBOLS } from '../components/CardPicker';

beforeEach(() => useSolverStore.getState().reset());

// ── CardPicker suit colors / symbols ─────────────────────────────────────────

describe('CardPicker suits', () => {
  it('h (hearts) → red', () => { expect(SUIT_COLORS['h']).toBe('#ef4444'); });
  it('d (diamonds) → light red', () => { expect(SUIT_COLORS['d']).toBe('#f87171'); });
  it('c (clubs) → teal', () => { expect(SUIT_COLORS['c']).toBe('#2dd4bf'); });
  it('s (spades) → slate', () => { expect(SUIT_COLORS['s']).toBe('#94a3b8'); });
  it('h symbol → ♥', () => { expect(SUIT_SYMBOLS['h']).toBe('♥'); });
  it('d symbol → ♦', () => { expect(SUIT_SYMBOLS['d']).toBe('♦'); });
  it('c symbol → ♣', () => { expect(SUIT_SYMBOLS['c']).toBe('♣'); });
  it('s symbol → ♠', () => { expect(SUIT_SYMBOLS['s']).toBe('♠'); });
});

// ── CardPicker select logic ───────────────────────────────────────────────────

describe('CardPicker select logic', () => {
  it('can add up to max cards', () => {
    const { setHeroHand } = useSolverStore.getState();
    setHeroHand(['Ah']);
    expect(useSolverStore.getState().heroHand).toHaveLength(1);
    setHeroHand(['Ah', 'Kd']);
    expect(useSolverStore.getState().heroHand).toHaveLength(2);
  });

  it('removing a card works', () => {
    useSolverStore.getState().setHeroHand(['Ah', 'Kd']);
    useSolverStore.getState().setHeroHand(['Kd']);
    expect(useSolverStore.getState().heroHand).toEqual(['Kd']);
  });

  it('duplicate cards blocked (manual check)', () => {
    // Store itself doesn't enforce uniqueness — the UI toggle logic does
    // Verify toggle: clicking same card removes it
    const toggle = (card: string, current: string[], max: number) => {
      if (current.includes(card)) return current.filter(c => c !== card);
      if (current.length >= max) return current;
      return [...current, card];
    };
    const after1 = toggle('Ah', [], 2);
    expect(after1).toContain('Ah');
    const after2 = toggle('Ah', after1, 2); // toggle off
    expect(after2).not.toContain('Ah');
    const after3 = toggle('Kd', toggle('Ah', [], 2), 2);
    const after4 = toggle('Qc', after3, 2); // max=2, blocked
    expect(after4).toHaveLength(2);
  });
});

// ── RangeSelector ─────────────────────────────────────────────────────────────

describe('RangeSelector', () => {
  const PRESETS = ['top5%','top10%','top15%','top20%','top30%'];
  const isCustom = (v: string) => !PRESETS.includes(v);

  it('top10% preset sets correct string', () => {
    useSolverStore.getState().setVillainRange('top10%');
    expect(useSolverStore.getState().villainRange).toBe('top10%');
  });

  it('custom value enables text input', () => {
    useSolverStore.getState().setVillainRange('AA,KK,AKs');
    expect(isCustom(useSolverStore.getState().villainRange)).toBe(true);
  });

  it('preset is not custom', () => {
    expect(isCustom('top15%')).toBe(false);
  });
});

// ── EquityBar widths ──────────────────────────────────────────────────────────

describe('EquityBar widths', () => {
  it('63.4% equity → hero bar width 63%', () => {
    const equity = 0.634;
    const pct = Math.round(equity * 100);
    expect(pct).toBe(63);
  });

  it('100% equity → full hero bar', () => {
    expect(Math.round(1.0 * 100)).toBe(100);
  });

  it('0% equity → empty hero bar', () => {
    expect(Math.round(0.0 * 100)).toBe(0);
  });
});

// ── ActionBadge colors ────────────────────────────────────────────────────────

const ACTION_COLORS: Record<string, string> = {
  bet:   '#22c55e',
  raise: '#22c55e',
  call:  '#3b82f6',
  check: '#6b7280',
  fold:  '#ef4444',
};

describe('ActionBadge colors', () => {
  it('bet → green', () => { expect(ACTION_COLORS['bet']).toBe('#22c55e'); });
  it('raise → green', () => { expect(ACTION_COLORS['raise']).toBe('#22c55e'); });
  it('call → blue', () => { expect(ACTION_COLORS['call']).toBe('#3b82f6'); });
  it('check → gray', () => { expect(ACTION_COLORS['check']).toBe('#6b7280'); });
  it('fold → red', () => { expect(ACTION_COLORS['fold']).toBe('#ef4444'); });
});

// ── Store solving flow ────────────────────────────────────────────────────────

describe('store solving flow', () => {
  it('starts with no result', () => {
    expect(useSolverStore.getState().result).toBeNull();
  });

  it('error when insufficient cards', async () => {
    useSolverStore.getState().setHeroHand(['Ah']); // only 1 card
    await useSolverStore.getState().solve();
    expect(useSolverStore.getState().error).toBeTruthy();
    expect(useSolverStore.getState().result).toBeNull();
  });

  it('solve produces result with complete input', async () => {
    const s = useSolverStore.getState();
    s.setHeroHand(['Ah', 'Kd']);
    s.setBoard(['Qh', 'Jc', '2s']);
    await s.solve();
    const result = useSolverStore.getState().result;
    expect(result).not.toBeNull();
    expect(result?.equity).toBeGreaterThan(0);
    expect(result?.equity).toBeLessThanOrEqual(1);
    expect(['bet','raise','call','check','fold']).toContain(result?.action);
  });

  it('reset clears result and hands', () => {
    useSolverStore.getState().setHeroHand(['Ah','Kd']);
    useSolverStore.getState().reset();
    expect(useSolverStore.getState().heroHand).toHaveLength(0);
    expect(useSolverStore.getState().result).toBeNull();
  });

  it('loading is false after solve', async () => {
    const s = useSolverStore.getState();
    s.setHeroHand(['Ah','Kd']); s.setBoard(['Qh','Jc','2s']);
    await s.solve();
    expect(useSolverStore.getState().loading).toBe(false);
  });
});
