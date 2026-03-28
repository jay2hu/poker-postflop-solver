import React from 'react';

const IS_TAURI = '__TAURI_INTERNALS__' in window;

export const Titlebar: React.FC = () => {
  const win = async () => {
    if (!IS_TAURI) return null;
    const { getCurrentWindow } = await import('@tauri-apps/api/window');
    return getCurrentWindow();
  };
  return (
    <div data-tauri-drag-region style={{
      height: 32, background: '#0a0a0a', borderBottom: '1px solid #1f1f1f',
      display: 'flex', alignItems: 'center', justifyContent: 'space-between',
      padding: '0 12px', flexShrink: 0, userSelect: 'none',
    }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
        <span style={{ fontSize: 14 }}>🃏</span>
        <span style={{ fontSize: 12, color: '#6b7280', fontWeight: 600 }}>Postflop Solver</span>
      </div>
      <div style={{ display: 'flex', gap: 4 }}>
        {[
          { icon: '─', color: '#fbbf24', fn: async () => { (await win())?.minimize(); } },
          { icon: '□', color: '#22c55e', fn: async () => { (await win())?.toggleMaximize(); } },
          { icon: '✕', color: '#ef4444', fn: async () => { (await win())?.close(); } },
        ].map(({ icon, color, fn }) => (
          <button key={icon} onClick={fn} style={{
            width: 18, height: 18, borderRadius: '50%', border: 'none',
            background: 'rgba(255,255,255,0.08)', color, fontSize: 9,
            cursor: 'pointer', display: 'flex', alignItems: 'center', justifyContent: 'center',
          }}
            onMouseEnter={e => (e.currentTarget.style.background = `${color}30`)}
            onMouseLeave={e => (e.currentTarget.style.background = 'rgba(255,255,255,0.08)')}
          >{icon}</button>
        ))}
      </div>
    </div>
  );
};
