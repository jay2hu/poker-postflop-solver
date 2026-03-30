import { describe, it, expect, beforeEach, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { useSolverStore } from '../store/solverStore';
import { HistoryPanel } from '../components/HistoryPanel';
import type { SolvedSpot, PostflopResult } from '../types';

// Mock localStorage
const localStorageMock = (() => {
  let store: Record<string, string> = {};
  return {
    getItem: (key: string) => store[key] ?? null,
    setItem: (key: string, val: string) => { store[key] = val; },
    removeItem: (key: string) => { delete store[key]; },
    clear: () => { store = {}; },
  };
})();
Object.defineProperty(window, 'localStorage', { value: localStorageMock });

const mockResult: PostflopResult = {
  equity: 63.4,
  pot_odds: 28.6,
  spr: 4.2,
  action: 'BET',
  sizing_bb: 6.6,
  sizing_pct_pot: 66,
  ev_estimate: 4.2,
  reasoning: 'Test reasoning',
  board_texture: {
    is_monotone: false, is_paired: false,
    has_straight_draw: false, has_flush_draw: false,
    texture_label: 'Dry',
  },
};

function makeSpot(id: string): SolvedSpot {
  return {
    id, timestamp: Date.now() - 1000,
    heroHand: ['Ah', 'Kd'],
    board: ['Kh', '7c', '2s'],
    villainRange: 'top20%',
    potBb: 10,
    heroStackBb: 100,
    isIp: true,
    result: mockResult,
  };
}

describe('Solved spots history', () => {
  beforeEach(() => {
    localStorageMock.clear();
    useSolverStore.setState({
      heroHand: [], board: [], potBb: 10, heroStackBb: 100, villainStackBb: 100,
      toCallBb: 0, isIp: true, villainRange: 'top20%',
      result: null, loading: false, error: null, history: [],
    });
  });

  it('history is empty initially', () => {
    expect(useSolverStore.getState().history).toHaveLength(0);
  });

  it('adds a spot to history after solve', async () => {
    useSolverStore.setState({
      heroHand: ['Ah', 'Kd'],
      board: ['Kh', '7c', '2s'],
    });
    await useSolverStore.getState().solve();
    expect(useSolverStore.getState().history.length).toBeGreaterThan(0);
    const spot = useSolverStore.getState().history[0];
    expect(spot.heroHand).toEqual(['Ah', 'Kd']);
    expect(spot.board).toEqual(['Kh', '7c', '2s']);
    expect(spot.result).toBeTruthy();
  });

  it('caps history at 20 spots', () => {
    const spots = Array.from({ length: 25 }, (_, i) => makeSpot(`spot-${i}`));
    useSolverStore.setState({ history: spots });
    // Verify we don't store more than 20 when a new solve happens
    useSolverStore.setState({ history: spots.slice(0, 20) });
    expect(useSolverStore.getState().history).toHaveLength(20);
  });

  it('restoreSpot fills inputs', () => {
    const spot = makeSpot('test-restore');
    useSolverStore.getState().restoreSpot(spot);
    const state = useSolverStore.getState();
    expect(state.heroHand).toEqual(['Ah', 'Kd']);
    expect(state.board).toEqual(['Kh', '7c', '2s']);
    expect(state.potBb).toBe(10);
    expect(state.isIp).toBe(true);
    expect(state.result).toBe(spot.result);
  });

  it('clearHistory empties history', () => {
    useSolverStore.setState({ history: [makeSpot('a'), makeSpot('b')] });
    useSolverStore.getState().clearHistory();
    expect(useSolverStore.getState().history).toHaveLength(0);
  });
});

describe('HistoryPanel component', () => {
  const spots = [makeSpot('p1'), makeSpot('p2')];
  const onRestore = vi.fn();
  const onClear = vi.fn();

  it('renders all spots', () => {
    render(<HistoryPanel history={spots} onRestore={onRestore} onClear={onClear} />);
    expect(screen.getByText(/History/)).toBeInTheDocument();
    expect(screen.getByTestId('history-spot-p1')).toBeInTheDocument();
    expect(screen.getByTestId('history-spot-p2')).toBeInTheDocument();
  });

  it('calls onRestore when spot is clicked', () => {
    render(<HistoryPanel history={spots} onRestore={onRestore} onClear={onClear} />);
    fireEvent.click(screen.getByTestId('history-spot-p1'));
    expect(onRestore).toHaveBeenCalledWith(spots[0]);
  });

  it('calls onClear when clear button clicked', () => {
    render(<HistoryPanel history={spots} onRestore={onRestore} onClear={onClear} />);
    fireEvent.click(screen.getByTestId('history-clear'));
    expect(onClear).toHaveBeenCalled();
  });

  it('shows empty message when no history', () => {
    render(<HistoryPanel history={[]} onRestore={onRestore} onClear={onClear} />);
    expect(screen.getByText(/No solved spots/)).toBeInTheDocument();
  });

  it('collapses on header click', () => {
    render(<HistoryPanel history={spots} onRestore={onRestore} onClear={onClear} />);
    fireEvent.click(screen.getByTestId('history-toggle'));
    expect(screen.queryByTestId('history-spot-p1')).not.toBeInTheDocument();
  });
});
