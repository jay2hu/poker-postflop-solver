import React from 'react';

const PRESETS = ['top5%', 'top10%', 'top15%', 'top20%', 'top30%', 'Custom'];

interface RangeSelectorProps {
  value: string;
  onChange: (v: string) => void;
}

export const RangeSelector: React.FC<RangeSelectorProps> = ({ value, onChange }) => {
  const isValueCustom = !PRESETS.slice(0, -1).includes(value);
  const [activePreset, setActivePreset] = React.useState<string>(
    isValueCustom ? 'Custom' : value
  );
  const [customText, setCustomText] = React.useState(isValueCustom ? value : '');

  const handlePreset = (p: string) => {
    setActivePreset(p);
    if (p === 'Custom') {
      onChange(customText);
    } else {
      onChange(p);
    }
  };

  return (
    <div>
      <div style={{ display: 'flex', gap: '6px', flexWrap: 'wrap' }}>
        {PRESETS.map((p) => (
          <button
            key={p}
            data-testid={`range-preset-${p}`}
            onClick={() => handlePreset(p)}
            style={{
              padding: '4px 10px',
              borderRadius: '12px',
              fontSize: '12px',
              fontWeight: 600,
              cursor: 'pointer',
              border: activePreset === p ? '1px solid #58a6ff' : '1px solid #30363d',
              background: activePreset === p ? '#1f6feb33' : '#161b22',
              color: activePreset === p ? '#58a6ff' : '#8b949e',
            }}
          >
            {p}
          </button>
        ))}
      </div>
      {activePreset === 'Custom' && (
        <input
          data-testid="range-custom-input"
          value={customText}
          onChange={(e) => {
            setCustomText(e.target.value);
            onChange(e.target.value);
          }}
          placeholder="e.g. AA,KK,AKs"
          style={{
            marginTop: '8px',
            width: '100%',
            padding: '6px 10px',
            background: '#161b22',
            border: '1px solid #30363d',
            borderRadius: '6px',
            color: '#e6edf3',
            fontSize: '13px',
            boxSizing: 'border-box',
          }}
        />
      )}
    </div>
  );
};
