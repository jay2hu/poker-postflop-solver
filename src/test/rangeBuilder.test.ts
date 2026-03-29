import { describe, it, expect, beforeEach } from 'vitest';
import {
  rangeMapToString, stringToRangeMap, computeVPIP, totalCombos,
  extractRangeFromScenario, v2ScenarioKey,
} from '../lib/rangeUtils';
import type { RangeMap } from '../lib/rangeUtils';
import { useSolverStore } from '../store/solverStore';

beforeEach(() => useSolverStore.getState().reset());

// ── rangeMapToString ──────────────────────────────────────────────────────────

describe('rangeMapToString', () => {
  it('full frequency → no colon', () => {
    const str = rangeMapToString({ AA: 1.0, KK: 1.0 });
    expect(str).toContain('AA');
    expect(str).toContain('KK');
    expect(str).not.toContain('AA:');
  });

  it('partial frequency → with colon', () => {
    const str = rangeMapToString({ AKs: 0.5 });
    expect(str).toBe('AKs:0.50');
  });

  it('excludes zero-freq hands', () => {
    const str = rangeMapToString({ AA: 1.0, KK: 0.0 });
    expect(str).not.toContain('KK');
  });

  it('empty map → empty string', () => {
    expect(rangeMapToString({})).toBe('');
  });

  it('round-trips with stringToRangeMap', () => {
    const original: RangeMap = { AA: 1.0, AKs: 0.75, AKo: 0.5 };
    const str = rangeMapToString(original);
    const parsed = stringToRangeMap(str);
    expect(Object.keys(parsed)).toHaveLength(3);
    expect(parsed['AKs']).toBeCloseTo(0.75, 2);
  });
});

// ── stringToRangeMap ──────────────────────────────────────────────────────────

describe('stringToRangeMap', () => {
  it('parses full freq hand', () => {
    const map = stringToRangeMap('AA,KK');
    expect(map['AA']).toBe(1.0);
    expect(map['KK']).toBe(1.0);
  });

  it('parses partial freq', () => {
    const map = stringToRangeMap('AKs:0.85');
    expect(map['AKs']).toBeCloseTo(0.85, 2);
  });

  it('empty string → empty map', () => {
    expect(Object.keys(stringToRangeMap(''))).toHaveLength(0);
  });

  it('clamps freq to [0,1]', () => {
    const map = stringToRangeMap('AA:1.5');
    expect(map['AA']).toBe(1.0);
  });

  it('round-trip with rangeMapToString', () => {
    const str = 'AA,KK,QQ,AKs:0.90,AKo:0.60';
    const map = stringToRangeMap(str);
    const back = rangeMapToString(map);
    const map2 = stringToRangeMap(back);
    expect(map2['AKs']).toBeCloseTo(0.90, 2);
    expect(map2['AKo']).toBeCloseTo(0.60, 2);
  });
});

// ── computeVPIP ───────────────────────────────────────────────────────────────

describe('computeVPIP', () => {
  it('empty range → 0% VPIP', () => {
    expect(computeVPIP({})).toBe(0);
  });

  it('AA only → ~0.45% VPIP (6/1326)', () => {
    const vpip = computeVPIP({ AA: 1.0 });
    expect(vpip).toBeCloseTo(6 / 1326 * 100, 1);
  });

  it('partial freq reduces VPIP', () => {
    const full  = computeVPIP({ AKs: 1.0 });
    const half  = computeVPIP({ AKs: 0.5 });
    expect(half).toBeCloseTo(full * 0.5, 1);
  });
});

// ── totalCombos ───────────────────────────────────────────────────────────────

describe('totalCombos', () => {
  it('AA → 6 combos', () => { expect(totalCombos({ AA: 1.0 })).toBe(6); });
  it('AKs → 4 combos', () => { expect(totalCombos({ AKs: 1.0 })).toBe(4); });
  it('AKo → 12 combos', () => { expect(totalCombos({ AKo: 1.0 })).toBe(12); });
  it('partial freq scales combos', () => {
    expect(totalCombos({ AKs: 0.5 })).toBe(2); // round(4*0.5)
  });
});

// ── v2ScenarioKey ─────────────────────────────────────────────────────────────

describe('v2ScenarioKey', () => {
  it('rfi + btn → BTN_rfi', () => { expect(v2ScenarioKey('rfi', 'btn')).toBe('BTN_rfi'); });
  it('rfi + utg → UTG_rfi', () => { expect(v2ScenarioKey('rfi', 'utg')).toBe('UTG_rfi'); });
  it('called_rfi + bb → BB_vs_open_BTN (default)', () => {
    expect(v2ScenarioKey('called_rfi', 'bb')).toContain('BB_vs_open_');
  });
  it('3bettor + sb vs utg → SB_vs3bet_UTG', () => {
    expect(v2ScenarioKey('3bettor', 'sb', 'utg')).toBe('SB_vs3bet_UTG');
  });
});

// ── extractRangeFromScenario ──────────────────────────────────────────────────

describe('extractRangeFromScenario', () => {
  const mockScenario = {
    hands: {
      AA:  { fold: 0.0, 'raise 2.3': 1.0 },
      AKs: { fold: 0.1, 'raise 2.3': 0.9 },
      '72o': { fold: 1.0, 'raise 2.3': 0.0 },
    }
  };

  it('extracts hands with raise > 0.05', () => {
    const map = extractRangeFromScenario(mockScenario, 'raise');
    expect(map['AA']).toBe(1.0);
    expect(map['AKs']).toBeCloseTo(0.9);
  });

  it('excludes pure fold hands', () => {
    const map = extractRangeFromScenario(mockScenario, 'raise');
    expect(map['72o']).toBeUndefined();
  });

  it('sums multiple raise actions', () => {
    const s = { hands: { AA: { 'raise 2.3': 0.6, 'raise 5': 0.4 } } };
    const map = extractRangeFromScenario(s, 'raise');
    expect(map['AA']).toBeCloseTo(1.0);
  });
});

// ── Store: villainRangeMap ────────────────────────────────────────────────────

describe('store villainRangeMap', () => {
  it('defaults to empty map', () => {
    expect(Object.keys(useSolverStore.getState().villainRangeMap)).toHaveLength(0);
  });

  it('setVillainRangeMap updates map and derives villainRange string', () => {
    useSolverStore.getState().setVillainRangeMap({ AA: 1.0, KK: 1.0 });
    const state = useSolverStore.getState();
    expect(state.villainRangeMap['AA']).toBe(1.0);
    expect(state.villainRange).toContain('AA');
  });

  it('reset clears rangeMap', () => {
    useSolverStore.getState().setVillainRangeMap({ AA: 1.0 });
    useSolverStore.getState().reset();
    expect(Object.keys(useSolverStore.getState().villainRangeMap)).toHaveLength(0);
  });
});
