import React, { memo } from 'react';
import { CardBadge } from './CardPicker';

export const BoardDisplay = memo(({ cards }: { cards: string[] }) => (
  <div style={{ display: 'flex', gap: 5, flexWrap: 'wrap', minHeight: 24 }}>
    {cards.map((c, i) => <CardBadge key={i} card={c} size="md" />)}
    {cards.length === 0 && <span style={{ fontSize: 11, color: '#4b5563', fontStyle: 'italic' }}>No board cards selected</span>}
  </div>
));
BoardDisplay.displayName = 'BoardDisplay';

export const EquityBar = memo(({ equity }: { equity: number }) => {
  const pct = Math.round(equity * 100);
  return (
    <div>
      <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: 4 }}>
        <span style={{ fontSize: 11, color: 'rgba(255,255,255,0.5)' }}>Hero equity</span>
        <span style={{ fontSize: 13, fontFamily: 'monospace', fontWeight: 700, color: pct >= 50 ? '#4ade80' : '#f87171' }}>{pct}%</span>
      </div>
      <div style={{ height: 8, borderRadius: 4, background: 'rgba(255,255,255,0.1)', overflow: 'hidden', display: 'flex' }}>
        <div style={{ width: `${pct}%`, background: pct >= 50 ? '#22c55e' : '#ef4444', borderRadius: '4px 0 0 4px', transition: 'width 0.3s ease' }} />
        <div style={{ flex: 1, background: '#374151' }} />
      </div>
      <div style={{ display: 'flex', justifyContent: 'space-between', marginTop: 2 }}>
        <span style={{ fontSize: 9, color: '#6b7280', fontFamily: 'monospace' }}>Hero {pct}%</span>
        <span style={{ fontSize: 9, color: '#6b7280', fontFamily: 'monospace' }}>Villain {100 - pct}%</span>
      </div>
    </div>
  );
});
EquityBar.displayName = 'EquityBar';

const ACTION_COLORS: Record<string, string> = {
  bet:   '#22c55e',
  raise: '#22c55e',
  call:  '#3b82f6',
  check: '#6b7280',
  fold:  '#ef4444',
};

export const ActionBadge = memo(({ action, sizingBb, sizingPct }: {
  action: string;
  sizingBb?: number | null;
  sizingPct?: number | null;
}) => {
  const color = ACTION_COLORS[action.toLowerCase()] ?? '#6b7280';
  const label = action.toUpperCase();
  return (
    <div style={{
      display: 'flex', alignItems: 'center', gap: 10,
      background: `${color}20`, border: `1px solid ${color}50`,
      borderRadius: 8, padding: '10px 14px',
    }}>
      <span style={{ fontSize: 20, fontWeight: 800, color, letterSpacing: 2 }}>{label}</span>
      {sizingBb != null && (
        <span style={{ fontSize: 14, fontFamily: 'monospace', color: 'rgba(255,255,255,0.8)' }}>
          {sizingPct != null ? `${sizingPct}% pot = ` : ''}{sizingBb.toFixed(1)}bb
        </span>
      )}
    </div>
  );
});
ActionBadge.displayName = 'ActionBadge';

const RANGE_PRESETS = ['top5%','top10%','top15%','top20%','top30%','Custom'];

export const RangeSelector = memo(({ value, onChange }: { value: string; onChange: (v: string) => void }) => {
  const isCustom = !RANGE_PRESETS.slice(0,-1).includes(value);
  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
      <div style={{ display: 'flex', flexWrap: 'wrap', gap: 4 }}>
        {RANGE_PRESETS.map(p => {
          const isActive = p === 'Custom' ? isCustom : value === p;
          return (
            <button key={p} onClick={() => onChange(p === 'Custom' ? '' : p)} style={{
              padding: '3px 10px', borderRadius: 12, fontSize: 11, fontWeight: isActive ? 700 : 400,
              cursor: 'pointer', border: '1px solid',
              background: isActive ? 'rgba(96,165,250,0.15)' : 'transparent',
              color: isActive ? '#60a5fa' : '#6b7280',
              borderColor: isActive ? '#3b82f6' : '#374151',
            }}>{p}</button>
          );
        })}
      </div>
      {isCustom && (
        <input
          value={value} onChange={e => onChange(e.target.value)}
          placeholder="e.g. AA,KK,AKs,AQo+"
          style={{
            width: '100%', padding: '6px 10px', borderRadius: 6, fontSize: 12,
            background: 'rgba(255,255,255,0.06)', border: '1px solid #374151',
            color: '#e5e7eb', outline: 'none', boxSizing: 'border-box', fontFamily: 'monospace',
          }}
        />
      )}
    </div>
  );
});
RangeSelector.displayName = 'RangeSelector';
