import { memo, useMemo } from 'react';
import type { RangeMap } from '../lib/rangeUtils';

const RANKS = ['A','K','Q','J','T','9','8','7','6','5','4','3','2'] as const;

function handFromCell(row: number, col: number): string {
  const r = RANKS[row], c = RANKS[col];
  if (row === col) return `${r}${r}`;
  if (row < col)  return `${r}${c}s`;
  return `${r}${c}o`;
}

/** Compute cell background for compare view */
function compareCellBg(userFreq: number, gtoFreq: number): string {
  const delta = userFreq - gtoFreq;
  if (Math.abs(delta) < 0.05) {
    // Similar — neutral
    return userFreq > 0.05 ? '#22c55e30' : '#1a1a1a';
  }
  if (delta > 0) {
    // User has MORE of this hand than GTO — blue tint
    const intensity = Math.min(delta * 2, 1);
    return `rgba(59,130,246,${0.15 + intensity * 0.4})`;
  }
  // User has FEWER — red tint
  const intensity = Math.min(-delta * 2, 1);
  return `rgba(239,68,68,${0.15 + intensity * 0.4})`;
}

interface SingleRangeGridProps {
  rangeMap: RangeMap;
  label: string;
  compareMap?: RangeMap;  // if set, cells are colored by diff vs this map
  size?: number;
}

const SingleRangeGrid = memo(({ rangeMap, label, compareMap, size = 26 }: SingleRangeGridProps) => (
  <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center' }}>
    <div style={{ fontSize: 10, fontWeight: 700, color: '#9ca3af', marginBottom: 4, textTransform: 'uppercase', letterSpacing: 1 }}>
      {label}
    </div>
    <div style={{ display: 'flex', marginLeft: 18, marginBottom: 2 }}>
      {RANKS.map(r => (
        <div key={r} style={{ width: size, textAlign: 'center', fontSize: 8, color: '#374151', fontFamily: 'monospace' }}>{r}</div>
      ))}
    </div>
    {RANKS.map((rank, row) => (
      <div key={rank} style={{ display: 'flex', alignItems: 'center', marginBottom: 1 }}>
        <div style={{ width: 16, textAlign: 'right', paddingRight: 2, fontSize: 8, color: '#374151', fontFamily: 'monospace' }}>{rank}</div>
        {RANKS.map((_, col) => {
          const name = handFromCell(row, col);
          const userFreq = rangeMap[name] ?? 0;
          const bg = compareMap
            ? compareCellBg(userFreq, compareMap[name] ?? 0)
            : userFreq > 0.05
              ? `rgba(34,197,94,${Math.max(0.2, userFreq)})`
              : '#1a1a1a';
          return (
            <div
              key={col}
              title={`${name}: ${Math.round(userFreq * 100)}%`}
              style={{
                width: size, height: size - 4,
                background: bg,
                border: '1px solid rgba(0,0,0,0.3)',
                borderRadius: 1,
                marginRight: 1,
                display: 'flex', alignItems: 'center', justifyContent: 'center',
                fontSize: 7, fontFamily: 'monospace', color: 'rgba(255,255,255,0.6)',
                cursor: 'default',
              }}
            >
              {name.slice(0,2)}
            </div>
          );
        })}
      </div>
    ))}
  </div>
));
SingleRangeGrid.displayName = 'SingleRangeGrid';

interface Props {
  userRange: RangeMap;
  gtoRange: RangeMap;
}

export const RangeCompare = memo(({ userRange, gtoRange }: Props) => {
  const diffCount = useMemo(() => {
    const all = new Set([...Object.keys(userRange), ...Object.keys(gtoRange)]);
    let count = 0;
    all.forEach(hand => {
      const delta = Math.abs((userRange[hand] ?? 0) - (gtoRange[hand] ?? 0));
      if (delta >= 0.05) count++;
    });
    return count;
  }, [userRange, gtoRange]);

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
      {/* Side-by-side grids */}
      <div style={{ display: 'flex', gap: 16, alignItems: 'flex-start', flexWrap: 'wrap' }}>
        <SingleRangeGrid
          rangeMap={userRange}
          label="Your Estimate"
          compareMap={gtoRange}
        />
        <SingleRangeGrid
          rangeMap={gtoRange}
          label="GTO Baseline"
        />
      </div>

      {/* Legend + stats */}
      <div style={{ display: 'flex', gap: 16, alignItems: 'center', flexWrap: 'wrap' }}>
        <span style={{ fontSize: 11, color: '#9ca3af' }}>
          <strong style={{ color: '#fbbf24' }}>{diffCount}</strong> hand{diffCount !== 1 ? 's' : ''} differ ≥5%
        </span>
        <div style={{ display: 'flex', gap: 8 }}>
          {[
            { color: 'rgba(59,130,246,0.6)', label: 'More than GTO' },
            { color: 'rgba(239,68,68,0.5)',  label: 'Less than GTO' },
            { color: 'rgba(34,197,94,0.3)',  label: 'Similar' },
          ].map(({ color, label }) => (
            <div key={label} style={{ display: 'flex', alignItems: 'center', gap: 3 }}>
              <div style={{ width: 10, height: 10, borderRadius: 2, background: color }} />
              <span style={{ fontSize: 9, color: '#6b7280' }}>{label}</span>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
});
RangeCompare.displayName = 'RangeCompare';
