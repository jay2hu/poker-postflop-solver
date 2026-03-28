import React, { memo } from 'react';
import type { BetOption } from '../types/solver';

interface Props {
  options: BetOption[];
  recommended_idx: number;
  pot_bb: number;
}

export const BetSizeComparison = memo(({ options, recommended_idx }: Props) => {
  if (!options.length) return null;

  const maxEv = Math.max(...options.map(o => Math.abs(o.ev)));

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
      <div style={{ fontSize: 10, color: '#6b7280', textTransform: 'uppercase', letterSpacing: 1 }}>
        Bet Size Comparison
      </div>
      {options.map((opt, i) => {
        const isRec = i === recommended_idx;
        const evBarW = maxEv > 0 ? Math.abs(opt.ev) / maxEv * 100 : 0;
        const evColor = opt.ev >= 0 ? '#22c55e' : '#ef4444';

        return (
          <div
            key={i}
            style={{
              background: isRec ? 'rgba(34,197,94,0.08)' : 'rgba(255,255,255,0.03)',
              border: `1px solid ${isRec ? '#22c55e40' : '#1f2937'}`,
              borderRadius: 6, padding: '8px 10px',
              display: 'flex', flexDirection: 'column', gap: 5,
            }}
          >
            {/* Header row */}
            <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
              {isRec && <span style={{ fontSize: 9, color: '#4ade80', fontWeight: 700 }}>★ REC</span>}
              <span style={{ fontWeight: 700, fontSize: 13, color: isRec ? '#4ade80' : '#e5e7eb' }}>
                {opt.label}
              </span>
              <span style={{ fontSize: 11, fontFamily: 'monospace', color: '#9ca3af', marginLeft: 'auto' }}>
                {opt.bb.toFixed(1)}bb
              </span>
            </div>

            {/* EV bar */}
            <div>
              <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: 2 }}>
                <span style={{ fontSize: 9, color: '#6b7280' }}>EV</span>
                <span style={{ fontSize: 10, fontFamily: 'monospace', color: evColor, fontWeight: 600 }}>
                  {opt.ev >= 0 ? '+' : ''}{opt.ev.toFixed(2)}bb
                </span>
              </div>
              <div style={{ height: 5, background: 'rgba(255,255,255,0.06)', borderRadius: 3, overflow: 'hidden' }}>
                <div style={{ height: '100%', width: `${evBarW}%`, background: evColor, borderRadius: 3, transition: 'width 0.3s ease' }} />
              </div>
            </div>

            {/* Fold equity */}
            <div style={{ fontSize: 10, color: '#6b7280' }}>
              Fold eq: <span style={{ color: '#fbbf24', fontFamily: 'monospace' }}>{(opt.fold_eq * 100).toFixed(0)}%</span>
            </div>
          </div>
        );
      })}
    </div>
  );
});
BetSizeComparison.displayName = 'BetSizeComparison';
