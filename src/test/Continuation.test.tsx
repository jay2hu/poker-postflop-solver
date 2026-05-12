import { describe, it, expect, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import { useSolverStore } from '../store/solverStore';
import { StreetProgression } from '../components/StreetProgression';
import type { StreetResult } from '../store/solverStore';
import type { PostflopResult } from '../types';

// localStorage mock
const localStorageMock = (() => {
  let store: Record<string, string> = {};
  return {
    getItem: (k: string) => store[k] ?? null,
    setItem: (k: string, v: string) => { store[k] = v; },
    removeItem: (k: string) => { delete store[k]; },
    clear: () => { store = {}; },
  };
})();
Object.defineProperty(window, 'localStorage', { value: localStorageMock });

const mockResult: PostflopResult = {
  equity: 63.4, pot_odds: 28.6, spr: 4.2, action: 'BET',
  sizing_bb: 6.6, sizing_pct_pot: 66, ev_estimate: 4.2,
  reasoning: 'Test',
  board_texture: { is_monotone: false, is_paired: false, has_straight_draw: false, has_flush_draw: false, texture_label: 'Dry' },
};

describe('Turn/river continuation', () => {
  beforeEach(() => {
    localStorageMock.clear();
    useSolverStore.setState({
      heroHand: ['Ah', 'Kd'],
      board: ['Kh', '7c', '2s'],
      potBb: 10, heroStackBb: 100, villainStackBb: 100,
      toCallBb: 0, isIp: true, villainRange: 'top20%',
      result: mockResult, loading: false, error: null,
      history: [], streetHistory: [], showContinuationPicker: false, continuationBoard: [],
    });
  });

  it('opens continuation picker on openContinuationPicker', () => {
    useSolverStore.getState().openContinuationPicker();
    expect(useSolverStore.getState().showContinuationPicker).toBe(true);
    expect(useSolverStore.getState().continuationBoard).toEqual(['Kh', '7c', '2s']);
  });

  it('closes continuation picker on closeContinuationPicker', () => {
    useSolverStore.getState().openContinuationPicker();
    useSolverStore.getState().closeContinuationPicker();
    expect(useSolverStore.getState().showContinuationPicker).toBe(false);
  });

  it('continueToNextStreet adds card and appends streetHistory', async () => {
    // First solve creates the flop entry — mock it manually
    useSolverStore.setState({
      streetHistory: [{ street: 'flop', board: ['Kh', '7c', '2s'], result: mockResult }],
    });
    await useSolverStore.getState().continueToNextStreet('Td');
    const state = useSolverStore.getState();
    expect(state.board).toEqual(['Kh', '7c', '2s', 'Td']);
    expect(state.streetHistory).toHaveLength(2);
    expect(state.streetHistory[1].street).toBe('turn');
    expect(state.streetHistory[1].board).toEqual(['Kh', '7c', '2s', 'Td']);
  });

  it('streetHistory resets on fresh solve', async () => {
    useSolverStore.setState({
      streetHistory: [{ street: 'flop', board: ['Kh', '7c', '2s'], result: mockResult }],
    });
    await useSolverStore.getState().solve();
    // After solve, streetHistory should be reset (fresh solve clears continuation)
    // (the mock solve starts fresh — streetHistory cleared in solve())
    expect(useSolverStore.getState().streetHistory).toHaveLength(0);
  });

  it('reset clears streetHistory', () => {
    useSolverStore.setState({
      streetHistory: [{ street: 'flop', board: ['Kh', '7c', '2s'], result: mockResult }],
    });
    useSolverStore.getState().reset();
    expect(useSolverStore.getState().streetHistory).toHaveLength(0);
    expect(useSolverStore.getState().showContinuationPicker).toBe(false);
  });
});

describe('StreetProgression component', () => {
  const makeHistory = (n: number): StreetResult[] => [
    { street: 'flop',  board: ['Kh', '7c', '2s'], result: { ...mockResult, action: 'BET' } },
    { street: 'turn',  board: ['Kh', '7c', '2s', 'Td'], result: { ...mockResult, action: 'CHECK' } },
    { street: 'river', board: ['Kh', '7c', '2s', 'Td', '5h'], result: { ...mockResult, action: 'BET' } },
  ].slice(0, n) as StreetResult[];

  it('renders nothing with fewer than 2 streets', () => {
    const { container } = render(<StreetProgression streetHistory={makeHistory(1)} />);
    expect(container.firstChild).toBeNull();
  });

  it('renders both streets when 2 streets present', () => {
    render(<StreetProgression streetHistory={makeHistory(2)} />);
    expect(screen.getByText(/flop/i)).toBeInTheDocument();
    expect(screen.getByText(/turn/i)).toBeInTheDocument();
  });

  it('renders all three streets', () => {
    render(<StreetProgression streetHistory={makeHistory(3)} />);
    expect(screen.getByText(/flop/i)).toBeInTheDocument();
    expect(screen.getByText(/turn/i)).toBeInTheDocument();
    expect(screen.getByText(/river/i)).toBeInTheDocument();
  });

  it('shows street progression label', () => {
    render(<StreetProgression streetHistory={makeHistory(2)} />);
    expect(screen.getByText(/street progression/i)).toBeInTheDocument();
  });
});
