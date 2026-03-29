import React, { useState, useCallback } from 'react';
import { RangeEditGrid } from './RangeEditGrid';
import { rangeMapToString, stringToRangeMap, computeVPIP, totalCombos, loadGtoRange, type ActionType, type RangeMap } from '../lib/rangeUtils';
import { useSolverStore } from '../store/solverStore';

const ACTION_TYPES: Array<{ key: ActionType; label: string }> = [
  { key: 'rfi',        label: 'RFI' },
  { key: 'called_rfi', label: 'Called RFI' },
  { key: '3bettor',    label: '3-bettor' },
  { key: '4bettor',    label: '4-bettor' },
];

const POSITIONS = ['utg','utg+1','lj','hj','co','btn','sb','bb'] as const;
const POS_LABELS: Record<string, string> = {
  utg:'UTG', 'utg+1':'UTG+1', lj:'LJ', hj:'HJ', co:'CO', btn:'BTN', sb:'SB', bb:'BB',
};

const STACKS = [5, 10, 20, 30, 40, 60, 80, 100];

const pill = (active: boolean, color = '#22c55e'): React.CSSProperties => ({
  padding: '3px 10px', borderRadius: 12, fontSize: 11, fontWeight: active ? 700 : 400,
  cursor: 'pointer', border: '1px solid',
  background: active ? `${color}20` : 'transparent',
  color: active ? color : '#6b7280',
  borderColor: active ? color : '#374151',
});

export const VillainRangeBuilder: React.FC = () => {
  const { villainRangeMap, setVillainRangeMap } = useSolverStore();

  const [actionType, setActionType] = useState<ActionType>('rfi');
  const [position, setPosition] = useState('btn');
  const [stack, setStack] = useState(100);
  const [gtoLoading, setGtoLoading] = useState(false);
  const [gtoError, setGtoError] = useState('');
  const [rangeStr, setRangeStr] = useState(() => rangeMapToString(villainRangeMap));
  const [strEditing, setStrEditing] = useState(false);

  // Keep rangeStr in sync when map changes externally
  const handleCellChange = useCallback((hand: string, freq: number) => {
    const next = { ...villainRangeMap, [hand]: freq };
    if (freq <= 0) delete next[hand];
    setVillainRangeMap(next);
    if (!strEditing) setRangeStr(rangeMapToString(next));
  }, [villainRangeMap, setVillainRangeMap, strEditing]);

  const handleStrBlur = () => {
    const parsed = stringToRangeMap(rangeStr);
    setVillainRangeMap(parsed);
    setRangeStr(rangeMapToString(parsed));
    setStrEditing(false);
  };

  const handleLoadGto = async () => {
    setGtoLoading(true); setGtoError('');
    try {
      const map = await loadGtoRange(stack, actionType, position);
      setVillainRangeMap(map);
      setRangeStr(rangeMapToString(map));
    } catch (e) {
      setGtoError(String(e));
    } finally {
      setGtoLoading(false);
    }
  };

  const handleClearAll = () => {
    setVillainRangeMap({});
    setRangeStr('');
  };

  const vpip = computeVPIP(villainRangeMap);
  const combos = totalCombos(villainRangeMap);

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 14 }}>
      {/* Section 1: Preflop Context */}
      <div style={{ background: 'rgba(255,255,255,0.03)', border: '1px solid #1f2937', borderRadius: 8, padding: '12px 14px', display: 'flex', flexDirection: 'column', gap: 10 }}>
        <div style={{ fontSize: 10, color: '#6b7280', textTransform: 'uppercase', letterSpacing: 1 }}>Preflop Context</div>

        {/* Action type */}
        <div>
          <div style={{ fontSize: 10, color: '#4b5563', marginBottom: 5 }}>Villain's action</div>
          <div style={{ display: 'flex', gap: 4, flexWrap: 'wrap' }}>
            {ACTION_TYPES.map(a => (
              <button key={a.key} style={pill(actionType === a.key)} onClick={() => setActionType(a.key)}>{a.label}</button>
            ))}
          </div>
        </div>

        {/* Position */}
        <div>
          <div style={{ fontSize: 10, color: '#4b5563', marginBottom: 5 }}>Position</div>
          <div style={{ display: 'flex', gap: 3, flexWrap: 'wrap' }}>
            {POSITIONS.map(p => (
              <button key={p} style={pill(position === p)} onClick={() => setPosition(p)}>{POS_LABELS[p]}</button>
            ))}
          </div>
        </div>

        {/* Stack */}
        <div>
          <div style={{ fontSize: 10, color: '#4b5563', marginBottom: 5 }}>Stack depth</div>
          <div style={{ display: 'flex', gap: 3, flexWrap: 'wrap' }}>
            {STACKS.map(s => (
              <button key={s} style={pill(stack === s)} onClick={() => setStack(s)}>{s}bb</button>
            ))}
          </div>
        </div>

        {/* Load button */}
        <div style={{ display: 'flex', gap: 8, alignItems: 'center' }}>
          <button
            onClick={handleLoadGto}
            disabled={gtoLoading}
            style={{
              padding: '6px 14px', borderRadius: 6, border: 'none', fontSize: 12, fontWeight: 600,
              background: gtoLoading ? '#1a1a1a' : '#1d4ed8', color: gtoLoading ? '#374151' : '#fff',
              cursor: gtoLoading ? 'default' : 'pointer',
            }}
          >
            {gtoLoading ? '⏳ Loading…' : '📊 Load GTO Range'}
          </button>
          <button onClick={handleClearAll} style={{ padding: '6px 12px', borderRadius: 6, border: '1px solid #374151', background: 'transparent', color: '#6b7280', cursor: 'pointer', fontSize: 11 }}>
            Clear All
          </button>
        </div>
        {gtoError && <div style={{ fontSize: 11, color: '#f87171' }}>{gtoError}</div>}
      </div>

      {/* Section 2: Interactive grid */}
      <div>
        <div style={{ fontSize: 10, color: '#6b7280', textTransform: 'uppercase', letterSpacing: 1, marginBottom: 8 }}>
          Range Grid <span style={{ color: '#4b5563', fontStyle: 'italic', textTransform: 'none', letterSpacing: 0 }}>(click = toggle, right-click = frequency)</span>
        </div>
        <RangeEditGrid rangeMap={villainRangeMap} onChange={handleCellChange} />
        <div style={{ display: 'flex', gap: 12, marginTop: 8 }}>
          {[['#22c55e','100%'],['#16a34a','70-99%'],['#f59e0b','40-69%'],['#d97706','1-39%'],['#111827','0%']].map(([col, label]) => (
            <div key={label} style={{ display: 'flex', alignItems: 'center', gap: 3 }}>
              <div style={{ width: 10, height: 10, borderRadius: 2, background: col }} />
              <span style={{ fontSize: 9, color: '#6b7280' }}>{label}</span>
            </div>
          ))}
        </div>
      </div>

      {/* Section 3: Range string */}
      <div>
        <div style={{ fontSize: 10, color: '#6b7280', textTransform: 'uppercase', letterSpacing: 1, marginBottom: 6 }}>Range String</div>
        <textarea
          value={rangeStr}
          onChange={e => { setRangeStr(e.target.value); setStrEditing(true); }}
          onBlur={handleStrBlur}
          rows={3}
          style={{
            width: '100%', padding: '6px 10px', borderRadius: 6, fontSize: 11,
            background: 'rgba(255,255,255,0.06)', border: '1px solid #374151',
            color: '#e5e7eb', outline: 'none', fontFamily: 'monospace',
            resize: 'vertical', boxSizing: 'border-box',
          }}
          placeholder="AA,KK,QQ,AKs:1.0,AKo:0.85,..."
        />
        <div style={{ display: 'flex', gap: 12, marginTop: 4, fontSize: 12 }}>
          <span style={{ color: '#6b7280' }}>VPIP: <strong style={{ color: '#60a5fa' }}>{vpip.toFixed(1)}%</strong></span>
          <span style={{ color: '#6b7280' }}>Combos: <strong style={{ color: '#e5e7eb' }}>{combos}</strong></span>
        </div>
      </div>
    </div>
  );
};
