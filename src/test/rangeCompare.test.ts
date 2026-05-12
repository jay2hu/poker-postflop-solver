import { describe, it, expect } from 'vitest';
import type { RangeMap } from '../lib/rangeUtils';

// Mirror of compareCellBg logic for testing
function compareDelta(userFreq: number, gtoFreq: number): 'more' | 'less' | 'similar' {
  const delta = userFreq - gtoFreq;
  if (Math.abs(delta) < 0.05) return 'similar';
  return delta > 0 ? 'more' : 'less';
}

function diffCount(userRange: RangeMap, gtoRange: RangeMap): number {
  const all = new Set([...Object.keys(userRange), ...Object.keys(gtoRange)]);
  let count = 0;
  all.forEach(hand => {
    const delta = Math.abs((userRange[hand] ?? 0) - (gtoRange[hand] ?? 0));
    if (delta >= 0.05) count++;
  });
  return count;
}

describe('RangeCompare diff logic', () => {
  it('delta < 5% → similar', () => {
    expect(compareDelta(0.8, 0.78)).toBe('similar');
    expect(compareDelta(0.5, 0.5)).toBe('similar');
  });

  it('user has more (delta > 5%) → more (blue tint)', () => {
    expect(compareDelta(0.9, 0.6)).toBe('more');
    expect(compareDelta(1.0, 0.0)).toBe('more');
  });

  it('user has less (delta > 5%) → less (red tint)', () => {
    expect(compareDelta(0.1, 0.8)).toBe('less');
    expect(compareDelta(0.0, 1.0)).toBe('less');
  });

  it('5% threshold: exactly 0.05 is different (not strictly less than)', () => {
    expect(compareDelta(0.55, 0.5)).toBe('more');   // exactly 0.05 = different
    expect(compareDelta(0.54, 0.5)).toBe('similar'); // < 0.05 = similar
    expect(compareDelta(0.56, 0.5)).toBe('more');    // > 0.05 = different
  });
});

describe('diffCount', () => {
  it('identical ranges → 0 diffs', () => {
    const range: RangeMap = { AA: 1.0, KK: 1.0 };
    expect(diffCount(range, range)).toBe(0);
  });

  it('one hand differs → 1 diff', () => {
    const user: RangeMap = { AA: 1.0, KK: 0.5 };
    const gto:  RangeMap = { AA: 1.0, KK: 1.0 };
    expect(diffCount(user, gto)).toBe(1);
  });

  it('hand in user but not gto → counted as diff', () => {
    const user: RangeMap = { AA: 1.0, '72o': 0.3 };
    const gto:  RangeMap = { AA: 1.0 };
    expect(diffCount(user, gto)).toBe(1);
  });

  it('empty ranges → 0 diffs', () => {
    expect(diffCount({}, {})).toBe(0);
  });
});
