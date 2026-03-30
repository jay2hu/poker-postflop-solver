import { useState } from 'react';
import type { SolvedSpot } from '../types';
import { ActionBadge } from './ActionBadge';

interface HistoryPanelProps {
  history: SolvedSpot[];
  onRestore: (spot: SolvedSpot) => void;
  onClear: () => void;
}

const SUIT_COLORS: Record<string, string> = {
  h: '#ef4444', d: '#ef4444', s: '#e6edf3', c: '#4ade80',
};

function CardBadge({ card }: { card: string }) {
  const rank = card.slice(0, -1);
  const suit = card.slice(-1);
  const color = SUIT_COLORS[suit] ?? '#e6edf3';
  return (
    <span style={{
      display: 'inline-flex', alignItems: 'center',
      background: '#21262d', borderRadius: 3, padding: '1px 4px',
      fontSize: 11, fontFamily: 'monospace', color, marginRight: 2,
    }}>
      {rank}{suit === 'h' ? '♥' : suit === 'd' ? '♦' : suit === 'c' ? '♣' : '♠'}
    </span>
  );
}

function timeAgo(ts: number): string {
  const s = Math.floor((Date.now() - ts) / 1000);
  if (s < 60) return `${s}s ago`;
  if (s < 3600) return `${Math.floor(s / 60)}m ago`;
  if (s < 86400) return `${Math.floor(s / 3600)}h ago`;
  return `${Math.floor(s / 86400)}d ago`;
}

export function HistoryPanel({ history, onRestore, onClear }: HistoryPanelProps) {
  const [open, setOpen] = useState(true);

  return (
    <div style={{ borderTop: '1px solid #21262d', paddingTop: 12 }}>
      {/* Header row */}
      <div
        role="button"
        data-testid="history-toggle"
        onClick={() => setOpen((v) => !v)}
        style={{
          display: 'flex', alignItems: 'center', justifyContent: 'space-between',
          cursor: 'pointer', userSelect: 'none', marginBottom: open ? 8 : 0,
        }}
      >
        <span style={{ fontSize: 11, fontWeight: 700, color: '#6e7681', textTransform: 'uppercase', letterSpacing: '0.5px' }}>
          History {history.length > 0 && <span style={{ color: '#58a6ff' }}>({history.length})</span>}
        </span>
        <span style={{ color: '#6e7681', fontSize: 12 }}>{open ? '▲' : '▼'}</span>
      </div>

      {open && (
        <>
          {history.length === 0 ? (
            <div style={{ fontSize: 12, color: '#3d4451', padding: '8px 0' }}>
              No solved spots yet.
            </div>
          ) : (
            <>
              {history.map((spot) => (
                <div
                  key={spot.id}
                  data-testid={`history-spot-${spot.id}`}
                  onClick={() => onRestore(spot)}
                  role="button"
                  style={{
                    display: 'flex', alignItems: 'center', gap: 6,
                    padding: '6px 8px', borderRadius: 6, cursor: 'pointer',
                    marginBottom: 4, background: '#161b22',
                    border: '1px solid transparent',
                    transition: 'border-color 0.15s',
                  }}
                  onMouseEnter={(e) => ((e.currentTarget as HTMLDivElement).style.borderColor = '#30363d')}
                  onMouseLeave={(e) => ((e.currentTarget as HTMLDivElement).style.borderColor = 'transparent')}
                >
                  {/* Hero hand */}
                  <div style={{ display: 'flex', flexShrink: 0 }}>
                    {spot.heroHand.map((c) => <CardBadge key={c} card={c} />)}
                  </div>

                  {/* Board */}
                  <div style={{ display: 'flex', flexShrink: 0 }}>
                    {spot.board.slice(0, 3).map((c) => <CardBadge key={c} card={c} />)}
                    {spot.board.length > 3 && <CardBadge card={spot.board[3]} />}
                    {spot.board.length > 4 && <CardBadge card={spot.board[4]} />}
                  </div>

                  {/* Action */}
                  <div style={{ flexShrink: 0 }}>
                    <ActionBadge
                      action={spot.result.action}
                      sizingBb={spot.result.sizing_bb ?? undefined}
                    />
                  </div>

                  {/* Equity */}
                  <span style={{ fontSize: 11, color: '#8b949e', marginLeft: 'auto', flexShrink: 0 }}>
                    {spot.result.equity}%
                  </span>

                  {/* Time */}
                  <span style={{ fontSize: 10, color: '#3d4451', flexShrink: 0 }}>
                    {timeAgo(spot.timestamp)}
                  </span>
                </div>
              ))}

              <button
                data-testid="history-clear"
                onClick={(e) => { e.stopPropagation(); onClear(); }}
                style={{
                  marginTop: 4, padding: '4px 10px', fontSize: 11,
                  borderRadius: 6, cursor: 'pointer',
                  border: '1px solid #30363d', background: 'transparent', color: '#6e7681',
                }}
              >
                Clear history
              </button>
            </>
          )}
        </>
      )}
    </div>
  );
}
