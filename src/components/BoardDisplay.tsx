import React from 'react';

const SUIT_COLORS: Record<string, string> = {
  s: '#94a3b8',
  h: '#f87171',
  d: '#f87171',
  c: '#2dd4bf',
};
const SUIT_SYMBOLS: Record<string, string> = { s: '♠', h: '♥', d: '♦', c: '♣' };

interface BoardDisplayProps {
  cards: string[];
}

export const BoardDisplay: React.FC<BoardDisplayProps> = ({ cards }) => {
  if (!cards.length) return null;
  return (
    <div style={{ display: 'flex', gap: '4px', flexWrap: 'wrap' }}>
      {cards.map((card) => {
        const rank = card[0];
        const suit = card[1];
        const color = SUIT_COLORS[suit] || '#8b949e';
        return (
          <span
            key={card}
            data-testid={`board-badge-${card}`}
            style={{
              display: 'inline-flex',
              alignItems: 'center',
              gap: '1px',
              padding: '2px 6px',
              borderRadius: '4px',
              border: `1px solid ${color}`,
              background: `${color}18`,
              color,
              fontWeight: 700,
              fontSize: '13px',
            }}
          >
            {rank}
            <span style={{ fontSize: '11px' }}>{SUIT_SYMBOLS[suit]}</span>
          </span>
        );
      })}
    </div>
  );
};
