import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { ActionBadge } from '../components/ActionBadge';

describe('ActionBadge', () => {
  it('renders BET in green', () => {
    render(<ActionBadge action="BET" sizingBb={6.6} sizingPctPot={66} />);
    const badge = screen.getByTestId('action-badge-BET');
    expect(badge).toHaveStyle({ color: '#238636' });
  });

  it('renders CALL in blue', () => {
    render(<ActionBadge action="CALL" />);
    const badge = screen.getByTestId('action-badge-CALL');
    expect(badge).toHaveStyle({ color: '#1f6feb' });
  });

  it('renders CHECK in gray', () => {
    render(<ActionBadge action="CHECK" />);
    const badge = screen.getByTestId('action-badge-CHECK');
    expect(badge).toHaveStyle({ color: '#6e7681' });
  });

  it('renders FOLD in red', () => {
    render(<ActionBadge action="FOLD" />);
    const badge = screen.getByTestId('action-badge-FOLD');
    expect(badge).toHaveStyle({ color: '#da3633' });
  });

  it('renders RAISE in green', () => {
    render(<ActionBadge action="RAISE" />);
    const badge = screen.getByTestId('action-badge-RAISE');
    expect(badge).toHaveStyle({ color: '#238636' });
  });

  it('shows sizing when provided', () => {
    render(<ActionBadge action="BET" sizingBb={6.6} sizingPctPot={66} />);
    expect(screen.getByText('6.6 bb (66% pot)')).toBeInTheDocument();
  });
});
