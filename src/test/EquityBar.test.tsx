import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { EquityBar } from '../components/EquityBar';

describe('EquityBar', () => {
  it('renders fill at correct width for 65%', () => {
    render(<EquityBar equity={65} />);
    const fill = screen.getByTestId('equity-bar-fill');
    expect(fill).toHaveStyle({ width: '65%' });
  });

  it('renders fill at 0% for 0 equity', () => {
    render(<EquityBar equity={0} />);
    const fill = screen.getByTestId('equity-bar-fill');
    expect(fill).toHaveStyle({ width: '0%' });
  });

  it('renders fill at 100% for 100 equity', () => {
    render(<EquityBar equity={100} />);
    const fill = screen.getByTestId('equity-bar-fill');
    expect(fill).toHaveStyle({ width: '100%' });
  });

  it('displays equity percentage text', () => {
    render(<EquityBar equity={72.5} />);
    expect(screen.getByText('72.5%')).toBeInTheDocument();
  });
});
