import React from 'react';

interface EquityBarProps {
  equity: number; // 0–100
}

export const EquityBar: React.FC<EquityBarProps> = ({ equity }) => {
  return (
    <div style={{ width: '100%' }}>
      <div
        style={{
          display: 'flex',
          justifyContent: 'space-between',
          marginBottom: '4px',
          fontSize: '12px',
          color: '#8b949e',
        }}
      >
        <span>Hero</span>
        <span>{equity.toFixed(1)}%</span>
      </div>
      <div
        data-testid="equity-bar-bg"
        style={{
          width: '100%',
          height: '10px',
          background: '#21262d',
          borderRadius: '5px',
          overflow: 'hidden',
        }}
      >
        <div
          data-testid="equity-bar-fill"
          style={{
            width: `${equity}%`,
            height: '100%',
            background: '#238636',
            borderRadius: '5px',
            transition: 'width 0.4s ease',
          }}
        />
      </div>
    </div>
  );
};
