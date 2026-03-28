import React from 'react';

interface TitleBarProps {
  onMin: () => void;
  onMax: () => void;
  onClose: () => void;
}

export const TitleBar: React.FC<TitleBarProps> = ({ onMin, onMax, onClose }) => {
  return (
    <div
      data-tauri-drag-region
      style={{
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
        height: '36px',
        padding: '0 12px',
        background: '#010409',
        borderBottom: '1px solid #21262d',
        WebkitAppRegion: 'drag',
        flexShrink: 0,
      } as React.CSSProperties}
    >
      <span style={{ fontSize: '13px', fontWeight: 600, color: '#e6edf3' }}>
        🃏 Poker Postflop Solver
      </span>
      <div
        style={{ display: 'flex', gap: '8px', WebkitAppRegion: 'no-drag' } as React.CSSProperties}
      >
        {[
          { label: '−', action: onMin, color: '#f59e0b' },
          { label: '⊞', action: onMax, color: '#10b981' },
          { label: '✕', action: onClose, color: '#ef4444' },
        ].map(({ label, action, color }) => (
          <button
            key={label}
            onClick={action}
            style={{
              width: '14px',
              height: '14px',
              borderRadius: '50%',
              background: color,
              border: 'none',
              cursor: 'pointer',
              fontSize: '9px',
              color: 'transparent',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              padding: 0,
            }}
            onMouseEnter={(e) => ((e.target as HTMLElement).style.color = '#000')}
            onMouseLeave={(e) => ((e.target as HTMLElement).style.color = 'transparent')}
          >
            {label}
          </button>
        ))}
      </div>
    </div>
  );
};
