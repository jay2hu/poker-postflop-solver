import React, { memo } from 'react';

export const RANKS = ['A','K','Q','J','T','9','8','7','6','5','4','3','2'] as const;
export const SUITS = ['h','d','c','s'] as const;

export const SUIT_SYMBOLS: Record<string, string> = { h:'♥', d:'♦', c:'♣', s:'♠' };
export const SUIT_COLORS: Record<string, string>  = {
  h: '#ef4444',
  d: '#f87171',
  c: '#2dd4bf',  // teal
  s: '#94a3b8',  // slate
};

/** Display a single card: white bg, rank bold, suit colored */
export const CardBadge = memo(({ card, size = 'sm' }: { card: string; size?: 'sm' | 'md' }) => {
  const rank = card.slice(0, -1);
  const suit = card.slice(-1).toLowerCase();
  const color = SUIT_COLORS[suit] ?? '#e5e7eb';
  const sym   = SUIT_SYMBOLS[suit] ?? suit;
  const sm = size === 'sm';
  return (
    <span style={{
      display: 'inline-flex', alignItems: 'center', gap: 1,
      background: '#fff', borderRadius: sm ? 3 : 5,
      padding: sm ? '1px 5px' : '3px 8px',
      fontSize: sm ? 12 : 16,
      fontFamily: 'monospace', fontWeight: 700,
      lineHeight: 1, userSelect: 'none', color: '#111',
    }}>
      {rank}<span style={{ color, fontSize: sm ? 11 : 14 }}>{sym}</span>
    </span>
  );
});
CardBadge.displayName = 'CardBadge';

/** 52-card picker grid. onSelect called with card string. */
interface PickerProps {
  selected: string[];
  disabled: string[];
  onSelect: (card: string) => void;
  maxCards: number;
}

export const CardPicker = memo(({ selected, disabled, onSelect, maxCards }: PickerProps) => {
  const canSelect = selected.length < maxCards;
  return (
    <div>
      {/* Suit rows */}
      {SUITS.map(suit => (
        <div key={suit} style={{ display: 'flex', gap: 2, marginBottom: 2 }}>
          {RANKS.map(rank => {
            const card = `${rank}${suit}`;
            const isSel = selected.includes(card);
            const isDis = disabled.includes(card) && !isSel;
            const color = SUIT_COLORS[suit];
            return (
              <button
                key={card}
                disabled={isDis || (!isSel && !canSelect)}
                onClick={() => onSelect(card)}
                title={card}
                style={{
                  width: 24, height: 24, borderRadius: 3, border: '1px solid',
                  fontSize: 9, fontFamily: 'monospace', fontWeight: 700,
                  cursor: isDis || (!isSel && !canSelect) ? 'default' : 'pointer',
                  background: isSel ? color : isDis ? 'rgba(255,255,255,0.04)' : 'rgba(255,255,255,0.08)',
                  color: isSel ? '#fff' : isDis ? '#1f2937' : color,
                  borderColor: isSel ? color : isDis ? '#1f2937' : 'rgba(255,255,255,0.1)',
                  opacity: isDis ? 0.4 : 1,
                  transition: 'background 0.08s',
                }}
              >
                {rank}
              </button>
            );
          })}
          <span style={{ fontSize: 11, color: SUIT_COLORS[suit], marginLeft: 3, lineHeight: '24px' }}>
            {SUIT_SYMBOLS[suit]}
          </span>
        </div>
      ))}
    </div>
  );
});
CardPicker.displayName = 'CardPicker';
