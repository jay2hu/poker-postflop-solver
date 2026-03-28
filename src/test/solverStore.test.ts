import { describe, it, expect, beforeEach } from 'vitest';
import { useSolverStore } from '../store/solverStore';

describe('solverStore', () => {
  beforeEach(() => {
    useSolverStore.getState().reset();
  });

  it('setHeroHand updates heroHand', () => {
    useSolverStore.getState().setHeroHand(['Ah', 'Kd']);
    expect(useSolverStore.getState().heroHand).toEqual(['Ah', 'Kd']);
  });

  it('setBoard updates board', () => {
    useSolverStore.getState().setBoard(['Qh', 'Jd', 'Ts']);
    expect(useSolverStore.getState().board).toEqual(['Qh', 'Jd', 'Ts']);
  });

  it('reset clears all state', () => {
    useSolverStore.getState().setHeroHand(['Ah', 'Kd']);
    useSolverStore.getState().setBoard(['Qh', 'Jd', 'Ts']);
    useSolverStore.getState().reset();
    const state = useSolverStore.getState();
    expect(state.heroHand).toEqual([]);
    expect(state.board).toEqual([]);
    expect(state.result).toBeNull();
  });

  it('solve() sets result and loading=false', async () => {
    useSolverStore.getState().setHeroHand(['Ah', 'Kd']);
    useSolverStore.getState().setBoard(['Qh', 'Jd', 'Ts']);
    await useSolverStore.getState().solve();
    const state = useSolverStore.getState();
    expect(state.loading).toBe(false);
    expect(state.result).not.toBeNull();
    expect(state.result?.action).toBeTruthy();
    expect(state.result?.equity).toBeGreaterThan(0);
  });

  it('solve() result has valid board_texture', async () => {
    useSolverStore.getState().setHeroHand(['Ah', 'Kd']);
    useSolverStore.getState().setBoard(['Qh', 'Jd', 'Ts']);
    await useSolverStore.getState().solve();
    const result = useSolverStore.getState().result;
    expect(result?.board_texture).toBeTruthy();
    expect(typeof result?.board_texture.texture_label).toBe('string');
  });
});
