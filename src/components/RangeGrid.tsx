import React, { memo, useState } from 'react';
import type { RangeHandData } from '../types/solver';

export const RANKS = ['A','K','Q','J','T','9','8','7','6','5','4','3','2'] as const;

function handFromCell(row: number, col: number): string {
  const r = RANKS[row], c = RANKS[col];
  if (row === col) return `${r}${r}`;
  if (row < col)  return `${r}${c}s`;
  return `${r}${c}o`;
}

/** Equity bucket → background color (villain's equity vs hero) */
export function bucketColor(bucket: number): string {
  switch (bucket) {
    case 0: return '#1e3a8a'; // 0-20% — villain crushed
    case 1: return '#2563eb'; // 20-40% — villain behind
    case 2: return '#6b7280'; // 40-60% — even
    case 3: return '#dc2626'; // 60-80% — villain ahead
    case 4: return '#991b1b'; // 80-100% — villain crushing
    default: return '#374151';
  }
}

/** Equity (0–1) → bucket 0..4 */
export function equityToBucket(eq: number): number {
  if (eq < 0.2) return 0;
  if (eq < 0.4) return 1;
  if (eq < 0.6) return 2;
  if (eq < 0.8) return 3;
  return 4;
}

interface Props {
  hands: RangeHandData[];
  loading?: boolean;
}

export const RangeGrid = memo(({ hands, loading }: Props) => {
  const [tooltip, setTooltip] = useState<{ x: number; y: number; text: string } | null>(null);

  // Build lookup: hand name → data
  const handMap = new Map<string, RangeHandData>();
  hands.forEach(h => handMap.set(h.hand, h));

  if (loading) {
    return (
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: 200, color: '#6b7280', fontSize: 12 }}>
        ⏳ Analysing range…
      </div>
    );
  }

  return (
    <div style={{ position: 'relative' }}>
      {/* Column labels */}
      <div style={{ display: 'flex', marginLeft: 22, marginBottom: 2 }}>
        {RANKS.map(r => (
          <div key={r} style={{ width: 32, textAlign: 'center', fontSize: 9, color: '#4b5563', fontFamily: 'monospace', fontWeight: 600 }}>{r}</div>
        ))}
      </div>

      {/* Grid rows */}
      {RANKS.map((rank, row) => (
        <div key={rank} style={{ display: 'flex', alignItems: 'center', marginBottom: 1 }}>
          {/* Row label */}
          <div style={{ width: 18, textAlign: 'right', paddingRight: 4, fontSize: 9, color: '#4b5563', fontFamily: 'monospace', fontWeight: 600 }}>{rank}</div>
          {RANKS.map((_, col) => {
            const name = handFromCell(row, col);
            const data = handMap.get(name);
            const bucket = data ? data.equity_bucket : 2;
            const weight = data ? Math.max(0.3, data.weight) : 0.15;
            const color = bucketColor(bucket);

            return (
              <div
                key={col}
                style={{
                  width: 32, height: 28,
                  background: color,
                  opacity: weight,
                  border: '1px solid rgba(0,0,0,0.3)',
                  cursor: data ? 'pointer' : 'default',
                  fontSize: 8, fontFamily: 'monospace', fontWeight: 700,
                  color: 'rgba(255,255,255,0.7)',
                  display: 'flex', alignItems: 'center', justifyContent: 'center',
                  userSelect: 'none',
                  transition: 'opacity 0.1s',
                  marginRight: 1,
                }}
                onMouseEnter={e => {
                  if (data) {
                    setTooltip({
                      x: e.clientX + 8, y: e.clientY + 8,
                      text: `${name}: ${(data.equity * 100).toFixed(1)}% eq · ${data.combos} combos`,
                    });
                  }
                }}
                onMouseLeave={() => setTooltip(null)}
              >
                {name.length <= 2 ? name : name.slice(0,2)}
              </div>
            );
          })}
        </div>
      ))}

      {/* Tooltip */}
      {tooltip && (
        <div style={{
          position: 'fixed', left: tooltip.x, top: tooltip.y,
          background: 'rgba(0,0,0,0.9)', border: '1px solid #374151',
          borderRadius: 5, padding: '4px 8px', fontSize: 11, color: '#e5e7eb',
          fontFamily: 'monospace', zIndex: 100, pointerEvents: 'none',
          whiteSpace: 'nowrap',
        }}>
          {tooltip.text}
        </div>
      )}

      {/* Legend */}
      <div style={{ display: 'flex', gap: 8, marginTop: 6, flexWrap: 'wrap' }}>
        {[
          { label: '0–20%', color: '#1e3a8a' },
          { label: '20–40%', color: '#2563eb' },
          { label: '40–60%', color: '#6b7280' },
          { label: '60–80%', color: '#dc2626' },
          { label: '80–100%', color: '#991b1b' },
        ].map(({ label, color }) => (
          <div key={label} style={{ display: 'flex', alignItems: 'center', gap: 4 }}>
            <div style={{ width: 10, height: 10, borderRadius: 2, background: color }} />
            <span style={{ fontSize: 9, color: '#6b7280' }}>{label}</span>
          </div>
        ))}
      </div>
    </div>
  );
});
RangeGrid.displayName = 'RangeGrid';
