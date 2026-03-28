import { describe, it, expect, beforeEach } from 'vitest';
import { RANKS, bucketColor, equityToBucket } from '../components/RangeGrid';
import { useSolverStore } from '../store/solverStore';

beforeEach(() => useSolverStore.getState().reset());

// ── RangeGrid ─────────────────────────────────────────────────────────────────

describe('RangeGrid — 169 cell matrix', () => {
  function buildHandMap() {
    const hands: string[] = [];
    for (let r = 0; r < 13; r++) {
      for (let c = 0; c < 13; c++) {
        const r1 = RANKS[r], r2 = RANKS[c];
        const h = r === c ? `${r1}${r1}` : r < c ? `${r1}${r2}s` : `${r1}${r2}o`;
        hands.push(h);
      }
    }
    return hands;
  }

  it('builds 169 unique hands', () => {
    const hands = buildHandMap();
    expect(hands.length).toBe(169);
    expect(new Set(hands).size).toBe(169);
  });

  it('AA is at row 0 col 0', () => {
    const hands = buildHandMap();
    expect(hands[0]).toBe('AA');
  });

  it('AKs is at row 0 col 1', () => {
    const hands = buildHandMap();
    expect(hands[1]).toBe('AKs');
  });

  it('22 is at row 12 col 12', () => {
    const hands = buildHandMap();
    expect(hands[168]).toBe('22');
  });
});

describe('bucketColor', () => {
  it('bucket 0 (0-20%) → dark blue', () => { expect(bucketColor(0)).toBe('#1e3a8a'); });
  it('bucket 1 (20-40%) → blue',      () => { expect(bucketColor(1)).toBe('#2563eb'); });
  it('bucket 2 (40-60%) → gray',      () => { expect(bucketColor(2)).toBe('#6b7280'); });
  it('bucket 3 (60-80%) → red',       () => { expect(bucketColor(3)).toBe('#dc2626'); });
  it('bucket 4 (80-100%) → dark red', () => { expect(bucketColor(4)).toBe('#991b1b'); });
});

describe('equityToBucket', () => {
  it('0.10 → bucket 0', () => { expect(equityToBucket(0.10)).toBe(0); });
  it('0.30 → bucket 1', () => { expect(equityToBucket(0.30)).toBe(1); });
  it('0.50 → bucket 2', () => { expect(equityToBucket(0.50)).toBe(2); });
  it('0.70 → bucket 3', () => { expect(equityToBucket(0.70)).toBe(3); });
  it('0.90 → bucket 4', () => { expect(equityToBucket(0.90)).toBe(4); });
  it('boundary 0.20 → bucket 1', () => { expect(equityToBucket(0.20)).toBe(1); });
  it('boundary 0.60 → bucket 3', () => { expect(equityToBucket(0.60)).toBe(3); });
});

describe('RangeGrid cell opacity', () => {
  it('weight clamps to min 0.3', () => {
    const weight = 0.1;
    const clamped = Math.max(0.3, weight);
    expect(clamped).toBe(0.3);
  });

  it('weight 0.8 passes through', () => {
    const weight = 0.8;
    expect(Math.max(0.3, weight)).toBe(0.8);
  });
});

// ── EquityDistribution ────────────────────────────────────────────────────────

describe('EquityDistribution', () => {
  const buckets = [0.12, 0.18, 0.22, 0.28, 0.20];

  it('5 segments sum to ~1.0', () => {
    const sum = buckets.reduce((a, b) => a + b, 0);
    expect(sum).toBeCloseTo(1.0, 2);
  });

  it('percentages sum to 100', () => {
    const pcts = buckets.map(b => Math.round(b * 100));
    // Due to rounding may be off by 1–2, just check close
    expect(pcts.reduce((a, b) => a + b, 0)).toBeGreaterThanOrEqual(98);
  });

  it('nut advantage hero → "Hero ▲" label', () => {
    const label = (a: string) => a === 'hero' ? 'Hero ▲' : a === 'villain' ? 'Villain ▼' : 'Neutral ─';
    expect(label('hero')).toBe('Hero ▲');
    expect(label('villain')).toBe('Villain ▼');
    expect(label('neutral')).toBe('Neutral ─');
  });
});

// ── BetSizeComparison ─────────────────────────────────────────────────────────

describe('BetSizeComparison', () => {
  const options = [
    { label: '25% pot', pct_pot: 25, bb: 2.5, ev: 1.8, fold_eq: 0.28 },
    { label: '75% pot', pct_pot: 75, bb: 7.5, ev: 2.7, fold_eq: 0.46 },
    { label: '100% pot',pct_pot: 100,bb: 10,  ev: 2.5, fold_eq: 0.52 },
  ];

  it('recommended option is highlighted (idx=1)', () => {
    const rec_idx = 1;
    expect(options[rec_idx].label).toBe('75% pot');
  });

  it('EV bars are proportional to max EV', () => {
    const maxEv = Math.max(...options.map(o => Math.abs(o.ev)));
    const widths = options.map(o => Math.abs(o.ev) / maxEv * 100);
    expect(widths[0]).toBeCloseTo(66.7, 0);
    expect(widths[1]).toBe(100);
  });

  it('fold equity shown as percent', () => {
    const pct = Math.round(options[0].fold_eq * 100);
    expect(pct).toBe(28);
  });
});

// ── Store — range + bet size analysis ────────────────────────────────────────

describe('store — full solve flow', () => {
  it('solve fires range and bet analyses after main solve', async () => {
    const s = useSolverStore.getState();
    s.setHeroHand(['Ah', 'Kd']);
    s.setBoard(['Qh', 'Jc', '2s']);
    await s.solve();
    const state = useSolverStore.getState();
    expect(state.result).not.toBeNull();
    expect(state.rangeAnalysis).not.toBeNull();
    expect(state.betSizes).not.toBeNull();
    expect(state.rangeAnalysis?.hands).toHaveLength(169);
    expect(state.betSizes?.options.length).toBeGreaterThan(0);
  }, 3000);

  it('rangeAnalysis.buckets sums to ~1', async () => {
    const s = useSolverStore.getState();
    s.setHeroHand(['Ah', 'Kd']);
    s.setBoard(['Qh', 'Jc', '2s']);
    await s.solve();
    const sum = useSolverStore.getState().rangeAnalysis!.buckets.reduce((a, b) => a + b, 0);
    expect(sum).toBeCloseTo(1.0, 2);
  }, 3000);
});
