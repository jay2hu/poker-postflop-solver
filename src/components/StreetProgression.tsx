import { ActionBadge } from './ActionBadge';
import type { StreetResult } from '../store/solverStore';

interface StreetProgressionProps {
  streetHistory: StreetResult[];
}

const SUITS: Record<string, string> = { h: '♥', d: '♦', c: '♣', s: '♠' };
const SUIT_COLORS: Record<string, string> = { h: '#ef4444', d: '#ef4444', s: '#e6edf3', c: '#4ade80' };

function InlineCard({ card }: { card: string }) {
  const rank = card.slice(0, -1);
  const suit = card.slice(-1);
  return (
    <span style={{
      display: 'inline-flex', alignItems: 'center',
      background: '#21262d', borderRadius: 3, padding: '1px 5px',
      fontSize: 11, fontFamily: 'monospace',
      color: SUIT_COLORS[suit] ?? '#e6edf3', marginRight: 2,
    }}>
      {rank}{SUITS[suit] ?? suit}
    </span>
  );
}

/** Shows a flop→turn→river recommendation progression side-by-side. */
export function StreetProgression({ streetHistory }: StreetProgressionProps) {
  if (streetHistory.length < 2) return null;

  return (
    <div style={{
      background: '#0d1117',
      border: '1px solid #21262d',
      borderRadius: 8,
      padding: '12px 14px',
      marginBottom: 16,
    }}>
      <div style={{ fontSize: 11, fontWeight: 700, color: '#6e7681', textTransform: 'uppercase', letterSpacing: '0.5px', marginBottom: 10 }}>
        Street progression
      </div>

      <div style={{ display: 'flex', gap: 12, flexWrap: 'wrap', alignItems: 'center' }}>
        {streetHistory.map((entry, i) => (
          <div key={i} style={{ display: 'flex', flexDirection: 'column', gap: 4, minWidth: 130 }}>
            {/* Street label + new cards */}
            <div style={{ display: 'flex', alignItems: 'center', gap: 4 }}>
              <span style={{
                fontSize: 10, fontWeight: 700, padding: '2px 6px',
                borderRadius: 4, background: '#21262d', color: '#8b949e',
                textTransform: 'uppercase',
              }}>
                {entry.street}
              </span>
              {/* Highlight the new card(s) for this street */}
              {entry.street === 'flop' && entry.board.slice(0, 3).map(c => <InlineCard key={c} card={c} />)}
              {entry.street === 'turn'  && <InlineCard card={entry.board[3]} />}
              {entry.street === 'river' && <InlineCard card={entry.board[4]} />}
            </div>

            {/* Action badge */}
            <ActionBadge
              action={entry.result.action}
              sizingBb={entry.result.sizing_bb ?? undefined}
              sizingPctPot={entry.result.sizing_pct_pot ?? undefined}
            />

            <div style={{ fontSize: 11, color: '#6e7681' }}>
              {entry.result.equity}% eq
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
