import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { RangeSelector } from '../components/RangeSelector';

describe('RangeSelector', () => {
  it('renders all preset buttons', () => {
    render(<RangeSelector value="top20%" onChange={() => {}} />);
    ['top5%', 'top10%', 'top15%', 'top20%', 'top30%', 'Custom'].forEach((p) => {
      expect(screen.getByTestId(`range-preset-${p}`)).toBeInTheDocument();
    });
  });

  it('calls onChange when preset clicked', () => {
    const onChange = vi.fn();
    render(<RangeSelector value="top20%" onChange={onChange} />);
    fireEvent.click(screen.getByTestId('range-preset-top5%'));
    expect(onChange).toHaveBeenCalledWith('top5%');
  });

  it('shows custom input when Custom selected', () => {
    render(<RangeSelector value="top20%" onChange={() => {}} />);
    expect(screen.queryByTestId('range-custom-input')).not.toBeInTheDocument();
    fireEvent.click(screen.getByTestId('range-preset-Custom'));
    expect(screen.getByTestId('range-custom-input')).toBeInTheDocument();
  });

  it('calls onChange when typing in custom input', () => {
    const onChange = vi.fn();
    render(<RangeSelector value="AA,KK" onChange={onChange} />);
    // non-preset value → shows as Custom mode
    const input = screen.getByTestId('range-custom-input');
    fireEvent.change(input, { target: { value: 'AA,KK,QQ' } });
    expect(onChange).toHaveBeenCalledWith('AA,KK,QQ');
  });
});
