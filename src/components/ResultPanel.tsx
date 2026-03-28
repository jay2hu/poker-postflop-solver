import React from 'react';
import type { PostflopResult } from '../types';
import { EquityBar } from './EquityBar';
import { ActionBadge } from './ActionBadge';
import { BoardDisplay } from './BoardDisplay';

interface ResultPanelProps {
  result: PostflopResult;
  board: string[];
}

export const ResultPanel: React.FC<ResultPanelProps> = ({ result, board }) => {
  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '20px' }}>
      <div>
        <Label>Board</Label>
        <BoardDisplay cards={board} />
        <div style={{ marginTop: '6px', fontSize: '13px', color: '#58a6ff' }}>
          {result.board_texture.texture_label}
        </div>
      </div>

      <div>
        <Label>Equity</Label>
        <EquityBar equity={result.equity} />
      </div>

      <div style={{ display: 'flex', gap: '24px' }}>
        <Stat label="SPR" value={result.spr.toFixed(1)} />
        <Stat label="Pot Odds" value={`${result.pot_odds.toFixed(1)}%`} />
        <Stat label="EV Est." value={`${result.ev_estimate > 0 ? '+' : ''}${result.ev_estimate} bb`} />
      </div>

      <div>
        <Label>Recommended Action</Label>
        <ActionBadge
          action={result.action}
          sizingBb={result.sizing_bb}
          sizingPctPot={result.sizing_pct_pot}
        />
      </div>

      <div>
        <Label>Reasoning</Label>
        <p style={{ fontSize: '13px', color: '#8b949e', lineHeight: 1.6, margin: 0 }}>
          {result.reasoning}
        </p>
      </div>
    </div>
  );
};

const Label: React.FC<{ children: React.ReactNode }> = ({ children }) => (
  <div style={{ fontSize: '11px', fontWeight: 700, color: '#6e7681', letterSpacing: '0.5px', textTransform: 'uppercase', marginBottom: '6px' }}>
    {children}
  </div>
);

const Stat: React.FC<{ label: string; value: string }> = ({ label, value }) => (
  <div>
    <div style={{ fontSize: '11px', color: '#6e7681', textTransform: 'uppercase', letterSpacing: '0.5px' }}>{label}</div>
    <div style={{ fontSize: '20px', fontWeight: 700, color: '#e6edf3' }}>{value}</div>
  </div>
);
