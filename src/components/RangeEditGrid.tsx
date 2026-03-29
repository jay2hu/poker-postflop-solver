import React, { memo, useRef, useState } from 'react';
import type { RangeMap } from '../lib/rangeUtils';

export const RANKS = ['A','K','Q','J','T','9','8','7','6','5','4','3','2'] as const;

function handFromCell(row: number, col: number): string {
  const r = RANKS[row], c = RANKS[col];
  if (row === col) return `${r}${r}`;
  if (row < col)  return `${r}${c}s`;
  return `${r}${c}o`;
}

/** Frequency → cell background color */
function freqColor(freq: number): string {
  if (freq <= 0)    return '#111827';       // not in range
  if (freq >= 0.95) return '#22c55e';       // bright green — 100%
  if (freq >= 0.7)  return '#16a34a';       // medium green — 70-95%
  if (freq >= 0.4)  return '#f59e0b';       // yellow — 40-70%
  return '#d97706';                          // amber — <40%
}

interface FreqSliderProps {
  hand: string;
  freq: number;
  x: number;
  y: number;
  onClose: () => void;
  onChange: (hand: string, freq: number) => void;
}

const FreqSlider = memo(({ hand, freq, x, y, onClose, onChange }: FreqSliderProps) => {
  const ref = useRef<HTMLDivElement>(null);
  React.useEffect(() => {
    const h = (e: MouseEvent) => { if (ref.current && !ref.current.contains(e.target as Node)) onClose(); };
    document.addEventListener('mousedown', h);
    return () => document.removeEventListener('mousedown', h);
  }, [onClose]);

  const pct = Math.round(freq * 100);
  const left = Math.min(x, window.innerWidth - 180);
  const top  = Math.min(y, window.innerHeight - 100);

  return (
    <div ref={ref} style={{
      position: 'fixed', left, top, zIndex: 200,
      background: '#1a1a1a', border: '1px solid #374151', borderRadius: 7,
      padding: '10px 12px', width: 170,
      boxShadow: '0 4px 16px rgba(0,0,0,0.6)',
      display: 'flex', flexDirection: 'column', gap: 6,
    }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <span style={{ fontSize: 14, fontWeight: 800, color: '#e5e7eb', fontFamily: 'monospace' }}>{hand}</span>
        <span style={{ fontSize: 12, color: '#4ade80', fontFamily: 'monospace' }}>{pct}%</span>
      </div>
      <input
        type="range" min={0} max={100} step={5} value={pct}
        onChange={e => onChange(hand, Number(e.target.value) / 100)}
        style={{ width: '100%', accentColor: '#22c55e', cursor: 'pointer' }}
      />
      <div style={{ display: 'flex', gap: 4 }}>
        {[0, 25, 50, 75, 100].map(v => (
          <button key={v} onClick={() => onChange(hand, v / 100)} style={{
            flex: 1, padding: '2px 0', borderRadius: 3, fontSize: 9,
            border: '1px solid #374151', background: pct === v ? '#22c55e20' : 'transparent',
            color: pct === v ? '#4ade80' : '#6b7280', cursor: 'pointer',
          }}>{v}%</button>
        ))}
      </div>
    </div>
  );
});
FreqSlider.displayName = 'FreqSlider';

interface Props {
  rangeMap: RangeMap;
  onChange: (hand: string, freq: number) => void;
}

export const RangeEditGrid = memo(({ rangeMap, onChange }: Props) => {
  const [slider, setSlider] = useState<{ hand: string; x: number; y: number } | null>(null);

  const handleClick = (hand: string) => {
    const current = rangeMap[hand] ?? 0;
    onChange(hand, current > 0 ? 0 : 1.0);
  };

  const handleContextMenu = (e: React.MouseEvent, hand: string) => {
    e.preventDefault();
    setSlider({ hand, x: e.clientX + 8, y: e.clientY + 8 });
  };

  return (
    <div style={{ position: 'relative' }}>
      {/* Column labels */}
      <div style={{ display: 'flex', marginLeft: 20, marginBottom: 2 }}>
        {RANKS.map(r => (
          <div key={r} style={{ width: 28, textAlign: 'center', fontSize: 9, color: '#4b5563', fontFamily: 'monospace', fontWeight: 600 }}>{r}</div>
        ))}
      </div>

      {RANKS.map((rank, row) => (
        <div key={rank} style={{ display: 'flex', alignItems: 'center', marginBottom: 1 }}>
          <div style={{ width: 18, textAlign: 'right', paddingRight: 3, fontSize: 9, color: '#4b5563', fontFamily: 'monospace', fontWeight: 600 }}>{rank}</div>
          {RANKS.map((_, col) => {
            const hand = handFromCell(row, col);
            const freq = rangeMap[hand] ?? 0;
            const bg = freqColor(freq);
            return (
              <div
                key={col}
                onClick={() => handleClick(hand)}
                onContextMenu={e => handleContextMenu(e, hand)}
                title={`${hand} — ${Math.round(freq * 100)}%`}
                style={{
                  width: 28, height: 24, background: bg,
                  border: `1px solid ${freq > 0 ? 'rgba(34,197,94,0.3)' : '#1f2937'}`,
                  cursor: 'pointer', marginRight: 1,
                  display: 'flex', alignItems: 'center', justifyContent: 'center',
                  fontSize: 8, fontFamily: 'monospace', fontWeight: 700,
                  color: 'rgba(255,255,255,0.7)',
                  userSelect: 'none',
                  transition: 'background 0.1s',
                  borderRadius: 1,
                }}
              >
                {hand.length <= 2 ? hand : hand.slice(0,2)}
              </div>
            );
          })}
        </div>
      ))}

      {/* Frequency slider popover */}
      {slider && (
        <FreqSlider
          hand={slider.hand}
          freq={rangeMap[slider.hand] ?? 0}
          x={slider.x} y={slider.y}
          onClose={() => setSlider(null)}
          onChange={(hand, freq) => { onChange(hand, freq); }}
        />
      )}
    </div>
  );
});
RangeEditGrid.displayName = 'RangeEditGrid';
