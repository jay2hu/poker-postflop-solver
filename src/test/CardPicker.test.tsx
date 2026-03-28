import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { CardPicker } from '../components/CardPicker';

describe('CardPicker', () => {
  it('renders 52 cards', () => {
    render(
      <CardPicker value={[]} maxCards={2} onChange={() => {}} />
    );
    // 4 suits × 13 ranks = 52
    const cards = document.querySelectorAll('[data-testid^="card-"]');
    expect(cards).toHaveLength(52);
  });

  it('selects a card on click', () => {
    const onChange = vi.fn();
    render(<CardPicker value={[]} maxCards={2} onChange={onChange} />);
    fireEvent.click(screen.getByTestId('card-Ah'));
    expect(onChange).toHaveBeenCalledWith(['Ah']);
  });

  it('deselects a card when already selected', () => {
    const onChange = vi.fn();
    render(<CardPicker value={['Ah']} maxCards={2} onChange={onChange} />);
    fireEvent.click(screen.getByTestId('card-Ah'));
    expect(onChange).toHaveBeenCalledWith([]);
  });

  it('does not select disabled cards', () => {
    const onChange = vi.fn();
    render(
      <CardPicker value={[]} maxCards={2} onChange={onChange} disabledCards={['Ah']} />
    );
    fireEvent.click(screen.getByTestId('card-Ah'));
    expect(onChange).not.toHaveBeenCalled();
  });

  it('enforces maxCards limit', () => {
    const onChange = vi.fn();
    render(<CardPicker value={['Ah', 'Kh']} maxCards={2} onChange={onChange} />);
    fireEvent.click(screen.getByTestId('card-Qh'));
    expect(onChange).not.toHaveBeenCalled();
  });

  it('shows Clear button when cards selected and clears on click', () => {
    const onChange = vi.fn();
    render(<CardPicker value={['Ah']} maxCards={2} onChange={onChange} />);
    const clearBtn = screen.getByText('Clear');
    expect(clearBtn).toBeInTheDocument();
    fireEvent.click(clearBtn);
    expect(onChange).toHaveBeenCalledWith([]);
  });
});
