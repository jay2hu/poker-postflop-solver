import { describe, it, expect } from 'vitest';
import { evaluateHandStrength } from '../lib/handEvaluator';

describe('evaluateHandStrength', () => {
  it('returns null with incomplete hand', () => {
    expect(evaluateHandStrength(['Ah'], ['Kh', '7c', '2s'])).toBeNull();
    expect(evaluateHandStrength(['Ah', 'Kd'], ['Kh', '7c'])).toBeNull();
  });

  it('detects straight flush', () => {
    const r = evaluateHandStrength(['9h', '8h'], ['7h', '6h', '5h']);
    expect(r?.category).toBe('straight_flush');
    expect(r?.strength).toBe('strong');
  });

  it('detects four of a kind', () => {
    const r = evaluateHandStrength(['Ah', 'Ad'], ['Ac', 'As', 'Kh']);
    expect(r?.category).toBe('four_of_a_kind');
  });

  it('detects full house', () => {
    const r = evaluateHandStrength(['Kh', 'Kd'], ['Kc', 'Ah', 'Ac']);
    expect(r?.category).toBe('full_house');
  });

  it('detects flush', () => {
    const r = evaluateHandStrength(['Ah', 'Kh'], ['Qh', '7h', '2h']);
    expect(r?.category).toBe('flush');
  });

  it('detects straight', () => {
    const r = evaluateHandStrength(['Ah', 'Kd'], ['Qh', 'Jc', 'Ts']);
    expect(r?.category).toBe('straight');
  });

  it('detects three of a kind', () => {
    const r = evaluateHandStrength(['Ah', 'Ad'], ['Ac', 'Kh', '7c']);
    expect(r?.category).toBe('three_of_a_kind');
  });

  it('detects two pair', () => {
    const r = evaluateHandStrength(['Ah', 'Kd'], ['Ac', 'Kh', '7c']);
    expect(r?.category).toBe('two_pair');
  });

  it('detects TPTK', () => {
    const r = evaluateHandStrength(['Ah', 'Kd'], ['Ac', '7h', '2s']);
    expect(r?.category).toBe('tptk');
    expect(r?.strength).toBe('medium');
  });

  it('detects top pair weak kicker', () => {
    const r = evaluateHandStrength(['Ah', '3d'], ['Ac', '7h', '2s']);
    expect(r?.category).toBe('top_pair');
  });

  it('detects middle pair', () => {
    const r = evaluateHandStrength(['7h', '3d'], ['Ac', '7c', '2s']);
    expect(r?.category).toBe('middle_pair');
  });

  it('detects bottom pair', () => {
    const r = evaluateHandStrength(['2h', '3d'], ['Ac', '7c', '2s']);
    expect(r?.category).toBe('bottom_pair');
  });

  it('detects flush draw', () => {
    const r = evaluateHandStrength(['Ah', 'Kh'], ['Qh', 'Jh', '2s']);  // 4 hearts
    expect(r?.category).toBe('flush_draw');
    expect(r?.strength).toBe('draw');
  });

  it('detects nut flush draw label', () => {
    const r = evaluateHandStrength(['Ah', 'Kh'], ['Qh', 'Jh', '2s']);  // 4 hearts
    expect(r?.label).toContain('Nut');
  });

  it('detects OESD', () => {
    const r = evaluateHandStrength(['7h', '6d'], ['5s', '4c', 'Ah']);
    expect(r?.category).toBe('oesd');
  });

  it('detects overcards', () => {
    const r = evaluateHandStrength(['Ah', 'Kd'], ['Th', '5c', '2s']);  // no draw — [14,13,10,5,2]
    expect(r?.category).toBe('overcards');
    expect(r?.strength).toBe('weak');
  });

  it('detects high card (nothing)', () => {
    const r = evaluateHandStrength(['3h', '5d'], ['Ah', 'Kc', 'Qs']);
    // 3/5 are below all board cards and no pair
    expect(r).not.toBeNull();
    expect(['weak','draw']).toContain(r?.strength);  // weak hand
  });

  it('strength values are valid', () => {
    const validStrengths = ['strong', 'medium', 'draw', 'weak'];
    const r = evaluateHandStrength(['Ah', 'Kd'], ['Ac', '7h', '2s']);
    expect(validStrengths).toContain(r?.strength);
  });
});
