import React from 'react';

const SUITS = ['s', 'h', 'd', 'c'] as const;
export const SUIT_SYMBOLS: Record<string, string> = { s: '♠', h: '♥', d: '♦', c: '♣' };
export const SUIT_COLORS: Record<string, string> = {
  s: '#94a3b8',
  h: '#f87171',
  d: '#f87171',
  c: '#2dd4bf',
};
const RANKS = ['A', 'K', 'Q', 'J', 'T', '9', '8', '7', '6', '5', '4', '3', '2'] as const;

interface CardPickerProps {
  value: string[];
  maxCards: number;
  onChange: (cards: string[]) => void;
  disabledCards?: string[];
}

export const CardPicker: React.FC<CardPickerProps> = ({
  value,
  maxCards,
  onChange,
  disabledCards = [],
}) => {
  const toggle = (card: string) => {
    if (disabledCards.includes(card)) return;
    if (value.includes(card)) {
      onChange(value.filter((c) => c !== card));
    } else {
      if (value.length >= maxCards) return;
      onChange([...value, card]);
    }
  };

  return (
    <div style={{ userSelect: 'none' }}>
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(13, 1fr)', gap: '2px' }}>
        {SUITS.map((suit) =>
          RANKS.map((rank) => {
            const card = `${rank}${suit}`;
            const selected = value.includes(card);
            const disabled = disabledCards.includes(card);
            const color = SUIT_COLORS[suit];
            return (
              <button
                key={card}
                onClick={() => toggle(card)}
                disabled={disabled}
                data-testid={`card-${card}`}
                style={{
                  width: '28px',
                  height: '32px',
                  fontSize: '10px',
                  fontWeight: 600,
                  cursor: disabled ? 'not-allowed' : 'pointer',
                  border: selected ? `2px solid ${color}` : '1px solid #30363d',
                  borderRadius: '3px',
                  background: selected ? `${color}22` : disabled ? '#161b22' : '#0d1117',
                  color: disabled ? '#3d4451' : color,
                  display: 'flex',
                  flexDirection: 'column',
                  alignItems: 'center',
                  justifyContent: 'center',
                  lineHeight: 1,
                  padding: 0,
                  transition: 'all 0.1s',
                }}
              >
                <span>{rank}</span>
                <span style={{ fontSize: '8px' }}>{SUIT_SYMBOLS[suit]}</span>
              </button>
            );
          })
        )}
      </div>
    </div>
  );
};
