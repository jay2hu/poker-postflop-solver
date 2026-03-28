import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { BoardDisplay } from '../components/BoardDisplay';

describe('BoardDisplay', () => {
  it('renders badges for each card', () => {
    render(<BoardDisplay cards={['As', 'Kh', 'Qd']} />);
    expect(screen.getByTestId('board-badge-As')).toBeInTheDocument();
    expect(screen.getByTestId('board-badge-Kh')).toBeInTheDocument();
    expect(screen.getByTestId('board-badge-Qd')).toBeInTheDocument();
  });

  it('applies red color for hearts', () => {
    render(<BoardDisplay cards={['Ah']} />);
    const badge = screen.getByTestId('board-badge-Ah');
    expect(badge).toHaveStyle({ color: '#f87171' });
  });

  it('applies red color for diamonds', () => {
    render(<BoardDisplay cards={['Kd']} />);
    const badge = screen.getByTestId('board-badge-Kd');
    expect(badge).toHaveStyle({ color: '#f87171' });
  });

  it('applies teal color for clubs', () => {
    render(<BoardDisplay cards={['2c']} />);
    const badge = screen.getByTestId('board-badge-2c');
    expect(badge).toHaveStyle({ color: '#2dd4bf' });
  });

  it('applies slate color for spades', () => {
    render(<BoardDisplay cards={['Ts']} />);
    const badge = screen.getByTestId('board-badge-Ts');
    expect(badge).toHaveStyle({ color: '#94a3b8' });
  });

  it('renders nothing for empty cards', () => {
    const { container } = render(<BoardDisplay cards={[]} />);
    expect(container.firstChild).toBeNull();
  });
});
