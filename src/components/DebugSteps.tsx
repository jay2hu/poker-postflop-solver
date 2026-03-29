import React, { useState } from 'react';

interface Props {
  steps: string[];
}

export const DebugSteps: React.FC<Props> = ({ steps }) => {
  const [open, setOpen] = useState(false);

  if (!steps?.length) return null;

  return (
    <div style={{ marginTop: 12 }}>
      <button
        onClick={() => setOpen(v => !v)}
        style={{
          display: 'flex', alignItems: 'center', gap: 6,
          background: 'none', border: 'none', cursor: 'pointer',
          color: '#6b7280', fontSize: 10, padding: 0,
          textTransform: 'uppercase', letterSpacing: 1,
        }}
      >
        <span style={{ fontSize: 8 }}>{open ? '▼' : '▶'}</span>
        How this was calculated
      </button>
      {open && (
        <div style={{
          marginTop: 8, padding: '10px 12px',
          background: 'rgba(255,255,255,0.03)',
          border: '1px solid #1f2937',
          borderRadius: 6,
          display: 'flex', flexDirection: 'column', gap: 5,
        }}>
          {steps.map((step, i) => (
            <div key={i} style={{ fontSize: 11, fontFamily: 'monospace', color: '#9ca3af', lineHeight: 1.5 }}>
              <span style={{ color: '#4b5563' }}>{step.split(':')[0]}:</span>
              <span style={{ color: '#e5e7eb' }}>{step.slice(step.indexOf(':') + 1)}</span>
            </div>
          ))}
        </div>
      )}
    </div>
  );
};
