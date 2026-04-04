import { memo } from 'react';
import type { HandStrengthResult } from '../lib/handEvaluator';

const SUIT_SYMBOLS: Record<string, string> = { h: '♥', d: '♦', c: '♣', s: '♠' };
const SUIT_COLORS:  Record<string, string>  = { h: '#ef4444', d: '#f97316', c: '#2dd4bf', s: '#94a3b8' };

function CardInline({ card }: { card: string }) {
  const rank = card.slice(0, -1);
  const suit = card.slice(-1).toLowerCase();
  return (
    <span style={{ fontFamily: 'monospace', fontWeight: 700, color: SUIT_COLORS[suit] ?? '#e5e7eb' }}>
      {rank}<span>{SUIT_SYMBOLS[suit] ?? suit}</span>
    </span>
  );
}

const STRENGTH_COLORS: Record<string, string> = {
  strong: '#4ade80',
  medium: '#fbbf24',
  draw:   '#60a5fa',
  weak:   '#f87171',
};

interface Props {
  result: HandStrengthResult;
  heroHand: string[];
  board: string[];
}

export const HandStrength = memo(({ result, heroHand, board }: Props) => {
  const color = STRENGTH_COLORS[result.strength];

  return (
    <div style={{
      background: `${color}10`, border: `1px solid ${color}30`,
      borderRadius: 8, padding: '10px 12px',
      display: 'flex', flexDirection: 'column', gap: 6,
    }}>
      {/* Category label */}
      <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
        <span style={{
          fontSize: 13, fontWeight: 800, color,
          textTransform: 'uppercase', letterSpacing: 0.5,
        }}>
          {result.label}
        </span>
      </div>

      {/* Description */}
      <div style={{ fontSize: 11, color: 'rgba(255,255,255,0.6)' }}>
        {result.description}
      </div>

      {/* Cards summary */}
      <div style={{ display: 'flex', alignItems: 'center', gap: 6, fontSize: 11 }}>
        <span style={{ color: 'rgba(255,255,255,0.4)' }}>Hero:</span>
        {heroHand.map((c, i) => <CardInline key={i} card={c} />)}
        <span style={{ color: 'rgba(255,255,255,0.2)', margin: '0 2px' }}>on</span>
        {board.map((c, i) => <CardInline key={i} card={c} />)}
      </div>
    </div>
  );
});
HandStrength.displayName = 'HandStrength';
