import React, { useState, useCallback } from 'react';
import { RangeEditGrid } from './RangeEditGrid';
import { rangeMapToString, stringToRangeMap, computeVPIP, totalCombos, loadGtoRange, type ActionType } from '../lib/rangeUtils';
import { useSolverStore } from '../store/solverStore';

const ACTION_TYPES: Array<{ key: ActionType; label: string }> = [
  { key: 'rfi',        label: 'RFI (opener)' },
  { key: 'called_rfi', label: 'Called RFI' },
  { key: '3bettor',    label: '3-bettor' },
  { key: '4bettor',    label: '4-bettor' },
];

const POSITIONS = ['utg','utg+1','lj','hj','co','btn','sb','bb'] as const;
const POS_LABELS: Record<string, string> = {
  utg:'UTG', 'utg+1':'UTG+1', lj:'LJ', hj:'HJ', co:'CO', btn:'BTN', sb:'SB', bb:'BB',
};
const STACKS = [5, 10, 20, 30, 40, 60, 80, 100];

type Mode = 'gto' | 'manual';

const tab = (active: boolean): React.CSSProperties => ({
  flex: 1, padding: '8px 0', fontSize: 12, fontWeight: active ? 700 : 400,
  cursor: 'pointer', border: 'none',
  background: active ? '#1a1a1a' : 'transparent',
  color: active ? '#e5e7eb' : '#6b7280',
  borderBottom: active ? '2px solid #22c55e' : '2px solid transparent',
  transition: 'all 0.1s',
});

const pill = (active: boolean, color = '#22c55e'): React.CSSProperties => ({
  padding: '3px 10px', borderRadius: 12, fontSize: 11, fontWeight: active ? 700 : 400,
  cursor: 'pointer', border: '1px solid',
  background: active ? `${color}20` : 'transparent',
  color: active ? color : '#6b7280',
  borderColor: active ? color : '#374151',
});

export const VillainRangeBuilder: React.FC = () => {
  const { villainRangeMap, setVillainRangeMap } = useSolverStore();

  const [mode, setMode] = useState<Mode>('gto');
  const [actionType, setActionType] = useState<ActionType>('rfi');
  const [position, setPosition] = useState('btn');
  const [stack, setStack] = useState(100);
  const [gtoLoading, setGtoLoading] = useState(false);
  const [gtoLoaded, setGtoLoaded] = useState(false);
  const [gtoError, setGtoError] = useState('');
  const [rangeStr, setRangeStr] = useState(() => rangeMapToString(villainRangeMap));
  const [strEditing, setStrEditing] = useState(false);

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
    setGtoLoading(true); setGtoError(''); setGtoLoaded(false);
    try {
      const map = await loadGtoRange(stack, actionType, position);
      setVillainRangeMap(map);
      setRangeStr(rangeMapToString(map));
      setGtoLoaded(true);
    } catch (e) {
      setGtoError(String(e));
    } finally {
      setGtoLoading(false);
    }
  };

  const handleClearAll = () => {
    setVillainRangeMap({});
    setRangeStr('');
    setGtoLoaded(false);
  };

  const vpip = computeVPIP(villainRangeMap);
  const combos = totalCombos(villainRangeMap);
  const hasRange = Object.keys(villainRangeMap).length > 0;

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 0 }}>

      {/* Mode tabs */}
      <div style={{ display: 'flex', borderBottom: '1px solid #1f2937', marginBottom: 14 }}>
        <button style={tab(mode === 'gto')} onClick={() => setMode('gto')}>📊 From GTO</button>
        <button style={tab(mode === 'manual')} onClick={() => setMode('manual')}>✏️ Manual</button>
      </div>

      {/* GTO mode */}
      {mode === 'gto' && (
        <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
          <div style={{ fontSize: 11, color: '#6b7280', lineHeight: 1.5 }}>
            Auto-fill villain's range from GTO preflop strategy.
          </div>

          {/* Action type */}
          <div>
            <div style={{ fontSize: 10, color: '#4b5563', marginBottom: 5 }}>Villain's action</div>
            <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
              {ACTION_TYPES.map(a => (
                <button key={a.key} style={{ ...pill(actionType === a.key), textAlign: 'left' as const }}
                  onClick={() => setActionType(a.key)}>{a.label}</button>
              ))}
            </div>
          </div>

          {/* Position */}
          <div>
            <div style={{ fontSize: 10, color: '#4b5563', marginBottom: 5 }}>Villain's position</div>
            <div style={{ display: 'flex', gap: 3, flexWrap: 'wrap' as const }}>
              {POSITIONS.map(p => (
                <button key={p} style={pill(position === p)} onClick={() => setPosition(p)}>{POS_LABELS[p]}</button>
              ))}
            </div>
          </div>

          {/* Stack */}
          <div>
            <div style={{ fontSize: 10, color: '#4b5563', marginBottom: 5 }}>Stack depth</div>
            <div style={{ display: 'flex', gap: 3, flexWrap: 'wrap' as const }}>
              {STACKS.map(s => (
                <button key={s} style={pill(stack === s)} onClick={() => setStack(s)}>{s}bb</button>
              ))}
            </div>
          </div>

          {/* Load button */}
          <button
            onClick={handleLoadGto}
            disabled={gtoLoading}
            style={{
              padding: '8px 0', borderRadius: 6, border: 'none', fontSize: 13, fontWeight: 700,
              background: gtoLoading ? '#1a1a1a' : '#1d4ed8',
              color: gtoLoading ? '#374151' : '#fff',
              cursor: gtoLoading ? 'default' : 'pointer',
              width: '100%',
            }}
          >
            {gtoLoading ? '⏳ Loading…' : gtoLoaded ? '✓ Loaded — Reload' : '→ Load GTO Range'}
          </button>

          {gtoError && <div style={{ fontSize: 11, color: '#f87171' }}>{gtoError}</div>}

          {gtoLoaded && (
            <div style={{ fontSize: 11, color: '#4ade80', background: 'rgba(34,197,94,0.08)', border: '1px solid rgba(34,197,94,0.2)', borderRadius: 6, padding: '8px 12px' }}>
              ✓ {POS_LABELS[position]} {ACTION_TYPES.find(a => a.key === actionType)?.label} range loaded at {stack}bb
              <br /><span style={{ color: '#6b7280' }}>VPIP {vpip.toFixed(1)}% · {combos} combos</span>
            </div>
          )}

          {gtoLoaded && (
            <button onClick={() => setMode('manual')} style={{
              padding: '6px 0', borderRadius: 6, border: '1px solid #374151',
              background: 'transparent', color: '#9ca3af', cursor: 'pointer', fontSize: 11, width: '100%',
            }}>
              ✏️ Edit loaded range manually →
            </button>
          )}
        </div>
      )}

      {/* Manual mode */}
      {mode === 'manual' && (
        <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
            <div style={{ fontSize: 11, color: '#6b7280' }}>
              Click cells to toggle · Right-click for frequency
            </div>
            {hasRange && (
              <button onClick={handleClearAll} style={{
                padding: '3px 10px', borderRadius: 5, border: '1px solid #374151',
                background: 'transparent', color: '#6b7280', cursor: 'pointer', fontSize: 10,
              }}>Clear all</button>
            )}
          </div>

          {/* Grid */}
          <RangeEditGrid rangeMap={villainRangeMap} onChange={handleCellChange} />

          {/* Legend */}
          <div style={{ display: 'flex', gap: 10, flexWrap: 'wrap' as const }}>
            {[['#22c55e','100%'],['#16a34a','70-99%'],['#f59e0b','40-69%'],['#d97706','1-39%'],['#1f2937','0%']].map(([col, label]) => (
              <div key={label} style={{ display: 'flex', alignItems: 'center', gap: 3 }}>
                <div style={{ width: 10, height: 10, borderRadius: 2, background: col, border: '1px solid #374151' }} />
                <span style={{ fontSize: 9, color: '#6b7280' }}>{label}</span>
              </div>
            ))}
          </div>

          {/* Range string */}
          <div>
            <div style={{ fontSize: 10, color: '#4b5563', marginBottom: 5 }}>Range string (editable)</div>
            <textarea
              value={rangeStr}
              onChange={e => { setRangeStr(e.target.value); setStrEditing(true); }}
              onBlur={handleStrBlur}
              rows={2}
              style={{
                width: '100%', padding: '6px 10px', borderRadius: 6, fontSize: 10,
                background: 'rgba(255,255,255,0.05)', border: '1px solid #374151',
                color: '#e5e7eb', outline: 'none', fontFamily: 'monospace',
                resize: 'vertical' as const, boxSizing: 'border-box' as const,
              }}
              placeholder="AA,KK,QQ,AKs:1.0,AKo:0.85,..."
            />
          </div>

          {/* Stats */}
          <div style={{ display: 'flex', gap: 16, fontSize: 12 }}>
            <span style={{ color: '#6b7280' }}>VPIP: <strong style={{ color: '#60a5fa' }}>{vpip.toFixed(1)}%</strong></span>
            <span style={{ color: '#6b7280' }}>Combos: <strong style={{ color: '#e5e7eb' }}>{combos}</strong></span>
          </div>
        </div>
      )}
    </div>
  );
};
