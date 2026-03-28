import React from 'react';

const ACTION_COLORS: Record<string, string> = {
  BET: '#238636',
  RAISE: '#238636',
  CALL: '#1f6feb',
  CHECK: '#6e7681',
  FOLD: '#da3633',
};

interface ActionBadgeProps {
  action: string;
  sizingBb?: number | null;
  sizingPctPot?: number | null;
}

export const ActionBadge: React.FC<ActionBadgeProps> = ({ action, sizingBb, sizingPctPot }) => {
  const color = ACTION_COLORS[action.toUpperCase()] || '#6e7681';
  return (
    <div
      data-testid={`action-badge-${action}`}
      style={{
        display: 'inline-flex',
        flexDirection: 'column',
        alignItems: 'center',
        padding: '12px 24px',
        borderRadius: '8px',
        background: `${color}22`,
        border: `2px solid ${color}`,
        color,
      }}
    >
      <span style={{ fontSize: '28px', fontWeight: 800, letterSpacing: '1px' }}>
        {action.toUpperCase()}
      </span>
      {sizingBb != null && sizingPctPot != null && (
        <span style={{ fontSize: '13px', fontWeight: 500, marginTop: '2px', color: '#8b949e' }}>
          {sizingBb} bb ({sizingPctPot}% pot)
        </span>
      )}
    </div>
  );
};
