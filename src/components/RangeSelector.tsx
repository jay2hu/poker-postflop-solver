import React, { useState } from 'react';

const PRESETS = ['top5%', 'top10%', 'top15%', 'top20%', 'top30%', 'Custom'];

// Canonical hand list matching the Rust expand_range_shorthand
const RANGE_HANDS: Record<string, string[]> = {
  'top5%':  ['AA','KK','QQ','JJ','AKs','AKo','AQs'],
  'top10%': ['AA','KK','QQ','JJ','TT','AKs','AKo','AQs','AQo','AJs','KQs'],
  'top15%': ['AA','KK','QQ','JJ','TT','99','AKs','AKo','AQs','AQo','AJs','ATs','KQs','KJs','QJs'],
  'top20%': ['AA','KK','QQ','JJ','TT','99','88','AKs','AKo','AQs','AQo','AJs','AJo','ATs','KQs','KJs','KTs','QJs','JTs'],
  'top30%': ['AA','KK','QQ','JJ','TT','99','88','77','66','AKs','AKo','AQs','AQo','AJs','AJo','ATs','ATo','A9s','KQs','KJs','KTs','KQo','QJs','QTs','JTs','T9s','98s'],
};

function comboCount(hand: string): number {
  if (hand.length === 2 && hand[0] === hand[1]) return 6;   // pair
  if (hand.endsWith('s')) return 4;                          // suited
  return 12;                                                 // offsuit
}

function totalCombos(hands: string[]): number {
  return hands.reduce((s, h) => s + comboCount(h), 0);
}

interface RangeSelectorProps {
  value: string;
  onChange: (v: string) => void;
}

export const RangeSelector: React.FC<RangeSelectorProps> = ({ value, onChange }) => {
  const isValueCustom = !PRESETS.slice(0, -1).includes(value);
  const [activePreset, setActivePreset] = useState<string>(isValueCustom ? 'Custom' : value);
  const [customText, setCustomText] = useState(isValueCustom ? value : '');
  const [showPreview, setShowPreview] = useState(false);

  const handlePreset = (p: string) => {
    setActivePreset(p);
    setShowPreview(false);
    if (p === 'Custom') onChange(customText);
    else onChange(p);
  };

  const previewHands = RANGE_HANDS[activePreset] ?? [];
  const combos = totalCombos(previewHands);

  const handColor = (h: string) => {
    if (h.length === 2 && h[0] === h[1]) return '#f59e0b'; // pairs — amber
    if (h.endsWith('s')) return '#22c55e';                  // suited — green
    return '#60a5fa';                                       // offsuit — blue
  };

  return (
    <div>
      {/* Preset pills */}
      <div style={{ display: 'flex', gap: 5, flexWrap: 'wrap' }}>
        {PRESETS.map(p => (
          <button
            key={p}
            data-testid={`range-preset-${p}`}
            onClick={() => handlePreset(p)}
            style={{
              padding: '4px 10px', borderRadius: 12, fontSize: 11, fontWeight: 600,
              cursor: 'pointer',
              border: activePreset === p ? '1px solid #22c55e' : '1px solid #374151',
              background: activePreset === p ? 'rgba(34,197,94,0.12)' : 'transparent',
              color: activePreset === p ? '#4ade80' : '#6b7280',
            }}
          >{p}</button>
        ))}
      </div>

      {/* Custom input */}
      {activePreset === 'Custom' && (
        <input
          data-testid="range-custom-input"
          value={customText}
          onChange={e => { setCustomText(e.target.value); onChange(e.target.value); }}
          placeholder="e.g. AA,KK,AKs:0.8,AKo:0.5"
          style={{
            marginTop: 8, width: '100%', padding: '6px 10px',
            background: 'rgba(255,255,255,0.05)', border: '1px solid #374151',
            borderRadius: 6, color: '#e5e7eb', fontSize: 12, boxSizing: 'border-box' as const,
          }}
        />
      )}

      {/* Preview toggle — only for presets */}
      {activePreset !== 'Custom' && previewHands.length > 0 && (
        <div style={{ marginTop: 6 }}>
          <button
            onClick={() => setShowPreview(v => !v)}
            style={{
              background: 'none', border: 'none', cursor: 'pointer',
              color: '#6b7280', fontSize: 10, padding: 0,
              display: 'flex', alignItems: 'center', gap: 4,
            }}
          >
            <span style={{ fontSize: 8 }}>{showPreview ? '▼' : '▶'}</span>
            {previewHands.length} hands · {combos} combos
            {showPreview ? ' (hide)' : ' (show)'}
          </button>

          {showPreview && (
            <div style={{
              marginTop: 6, padding: '10px 12px',
              background: 'rgba(255,255,255,0.03)',
              border: '1px solid #1f2937', borderRadius: 6,
            }}>
              {/* Legend */}
              <div style={{ display: 'flex', gap: 12, marginBottom: 8, fontSize: 9, color: '#6b7280' }}>
                <span><span style={{ color: '#f59e0b' }}>■</span> Pairs</span>
                <span><span style={{ color: '#22c55e' }}>■</span> Suited</span>
                <span><span style={{ color: '#60a5fa' }}>■</span> Offsuit</span>
              </div>
              {/* Hand chips */}
              <div style={{ display: 'flex', flexWrap: 'wrap', gap: 4 }}>
                {previewHands.map(h => (
                  <span
                    key={h}
                    title={`${comboCount(h)} combos`}
                    style={{
                      fontSize: 11, fontFamily: 'monospace', fontWeight: 600,
                      color: handColor(h),
                      background: `${handColor(h)}18`,
                      border: `1px solid ${handColor(h)}40`,
                      borderRadius: 4, padding: '2px 6px',
                    }}
                  >{h}</span>
                ))}
              </div>
              {/* Summary */}
              <div style={{ marginTop: 8, fontSize: 10, color: '#4b5563', borderTop: '1px solid #1f2937', paddingTop: 6 }}>
                {previewHands.filter(h => h.length === 2).length} pairs ·{' '}
                {previewHands.filter(h => h.endsWith('s')).length} suited ·{' '}
                {previewHands.filter(h => h.endsWith('o')).length} offsuit ·{' '}
                {combos} total combos
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  );
};
