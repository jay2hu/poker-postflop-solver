/// <reference types="vite/client" />
import React, { useState } from 'react';
import { TitleBar as Titlebar } from './components/TitleBar';
import { CardPicker, SUIT_SYMBOLS, SUIT_COLORS } from './components/CardPicker';
import { EquityBar } from './components/EquityBar';
import { ActionBadge } from './components/ActionBadge';
import { RangeSelector } from './components/RangeSelector';
import { BoardDisplay } from './components/BoardDisplay';
import { useSolverStore } from './store/solverStore';
import './index.css';
import { RangeGrid } from './components/RangeGrid';
import { EquityDistribution } from './components/EquityDistribution';
import { BetSizeComparison } from './components/BetSizeComparison';

const SectionLabel = ({ children }: { children: React.ReactNode }) => (
  <div style={{ fontSize: 10, color: '#6b7280', textTransform: 'uppercase', letterSpacing: 1, marginBottom: 6 }}>{children}</div>
);

const NumInput = ({ label, value, onChange, min = 0, max = 999 }: {
  label: string; value: number; onChange: (v: number) => void; min?: number; max?: number;
}) => (
  <div style={{ display: 'flex', flexDirection: 'column', gap: 3 }}>
    <label style={{ fontSize: 10, color: '#6b7280' }}>{label}</label>
    <input
      type="number" min={min} max={max} value={value}
      onChange={e => onChange(parseFloat(e.target.value) || 0)}
      style={{
        width: '100%', padding: '5px 8px', borderRadius: 5, fontSize: 13,
        background: 'rgba(255,255,255,0.07)', border: '1px solid #374151',
        color: '#e5e7eb', outline: 'none', fontFamily: 'monospace', boxSizing: 'border-box',
      }}
    />
  </div>
);

const App = () => {
  const {
    heroHand, board, potBb, heroStackBb, villainStackBb, toCallBb, isIp, villainRange,
    result, loading, error,
    rangeAnalysis, rangeLoading, betSizes, betSizesLoading,
    setHeroHand, setBoard, setPotBb, setHeroStackBb, setVillainStackBb, setToCallBb,
    setIsIp, setVillainRange, solve, reset,
  } = useSolverStore();

  const [showHeroPicker, setShowHeroPicker] = useState(false);
  const [rangeExpanded, setRangeExpanded] = useState(true);
  const [betSizesExpanded, setBetSizesExpanded] = useState(true);
  const [showBoardPicker, setShowBoardPicker] = useState(false);

  const allUsedCards = [...heroHand, ...board];

  const toggleCard = (card: string, current: string[], setter: (c: string[]) => void, max: number) => {
    if (current.includes(card)) setter(current.filter(c => c !== card));
    else if (current.length < max) setter([...current, card]);
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100vh', background: '#0f0f0f', color: '#e5e7eb' }}>
      <Titlebar />
      <div style={{ display: 'flex', flex: 1, overflow: 'hidden' }}>

        {/* ── Left input panel ── */}
        <div style={{ width: 340, background: '#111', borderRight: '1px solid #1f1f1f', overflowY: 'auto', padding: 16, display: 'flex', flexDirection: 'column', gap: 16 }}>

          {/* Hero hand */}
          <div>
            <SectionLabel>Hero Hand</SectionLabel>
            <div style={{ display: 'flex', gap: 6, marginBottom: 6, flexWrap: 'wrap' }}>
              {heroHand.length > 0
                ? heroHand.map((c, i) => (
                    <button key={i} onClick={() => setHeroHand(heroHand.filter(x => x !== c))}
                      style={{ background: 'none', border: 'none', cursor: 'pointer', padding: 0 }}
                      title="Remove">
                      <span style={{ display: 'inline-flex', alignItems: 'center', gap: 2, background: '#fff', borderRadius: 4, padding: '2px 7px', fontSize: 14, fontWeight: 700, fontFamily: 'monospace', color: '#111' }}>
                        {c.slice(0,-1)}<span style={{ color: SUIT_COLORS[c.slice(-1)] || '#111', fontSize: 12 }}>{SUIT_SYMBOLS[c.slice(-1)] || ''}</span>
                      </span>
                    </button>
                  ))
                : <span style={{ fontSize: 12, color: '#4b5563', fontStyle: 'italic' }}>No cards selected</span>
              }
              <button onClick={() => setShowHeroPicker(v => !v)} style={{
                padding: '2px 8px', borderRadius: 4, fontSize: 11, cursor: 'pointer',
                border: '1px solid #374151', background: showHeroPicker ? '#1e3a1e' : 'transparent', color: '#9ca3af',
              }}>{showHeroPicker ? 'Done' : heroHand.length > 0 ? 'Edit' : '+ Pick'}</button>
              {heroHand.length > 0 && <button onClick={() => setHeroHand([])} style={{ padding: '2px 6px', borderRadius: 4, fontSize: 10, cursor: 'pointer', border: '1px solid #374151', background: 'transparent', color: '#6b7280' }}>Clear</button>}
            </div>
            {showHeroPicker && (
              <CardPicker
                selected={heroHand}
                disabled={allUsedCards.filter(c => !heroHand.includes(c))}
                onSelect={c => toggleCard(c, heroHand, setHeroHand, 2)}
                maxCards={2}
              />
            )}
          </div>

          {/* Board */}
          <div>
            <SectionLabel>Board</SectionLabel>
            <BoardDisplay cards={board} />
            <div style={{ display: 'flex', gap: 6, marginTop: 6 }}>
              <button onClick={() => setShowBoardPicker(v => !v)} style={{
                padding: '2px 8px', borderRadius: 4, fontSize: 11, cursor: 'pointer',
                border: '1px solid #374151', background: showBoardPicker ? '#1e3a1e' : 'transparent', color: '#9ca3af',
              }}>{showBoardPicker ? 'Done' : board.length > 0 ? 'Edit' : '+ Pick'}</button>
              {board.length > 0 && <button onClick={() => setBoard([])} style={{ padding: '2px 6px', borderRadius: 4, fontSize: 10, cursor: 'pointer', border: '1px solid #374151', background: 'transparent', color: '#6b7280' }}>Clear</button>}
            </div>
            {showBoardPicker && (
              <div style={{ marginTop: 8 }}>
                <CardPicker
                  selected={board}
                  disabled={allUsedCards.filter(c => !board.includes(c))}
                  onSelect={c => toggleCard(c, board, setBoard, 5)}
                  maxCards={5}
                />
              </div>
            )}
          </div>

          {/* Pot & stacks */}
          <div>
            <SectionLabel>Stack & Pot</SectionLabel>
            <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 8 }}>
              <NumInput label="Pot (bb)" value={potBb} onChange={setPotBb} min={0.5} />
              <NumInput label="To call (bb)" value={toCallBb} onChange={setToCallBb} min={0} />
              <NumInput label="Hero stack (bb)" value={heroStackBb} onChange={setHeroStackBb} min={1} />
              <NumInput label="Villain stack (bb)" value={villainStackBb} onChange={setVillainStackBb} min={1} />
            </div>
          </div>

          {/* Position */}
          <div>
            <SectionLabel>Position</SectionLabel>
            <div style={{ display: 'flex', gap: 6 }}>
              {[true, false].map(ip => (
                <button key={String(ip)} onClick={() => setIsIp(ip)} style={{
                  flex: 1, padding: '6px', borderRadius: 5, fontSize: 12, fontWeight: isIp === ip ? 700 : 400,
                  cursor: 'pointer', border: '1px solid',
                  background: isIp === ip ? 'rgba(34,197,94,0.15)' : 'transparent',
                  color: isIp === ip ? '#4ade80' : '#6b7280',
                  borderColor: isIp === ip ? '#22c55e' : '#374151',
                }}>
                  {ip ? 'IP (In Position)' : 'OOP (Out of Position)'}
                </button>
              ))}
            </div>
          </div>

          {/* Villain range */}
          <div>
            <SectionLabel>Villain Range</SectionLabel>
            <RangeSelector value={villainRange} onChange={setVillainRange} />
          </div>

          {/* Solve / reset buttons */}
          <div style={{ display: 'flex', gap: 8, marginTop: 4 }}>
            <button
              onClick={solve}
              disabled={loading || heroHand.length < 2 || board.length < 3}
              style={{
                flex: 3, padding: '10px', borderRadius: 7, border: 'none', fontSize: 14, fontWeight: 700,
                cursor: loading || heroHand.length < 2 || board.length < 3 ? 'default' : 'pointer',
                background: loading ? '#1a1a1a' : heroHand.length < 2 || board.length < 3 ? '#1a1a1a' : '#16a34a',
                color: loading || heroHand.length < 2 || board.length < 3 ? '#374151' : '#fff',
              }}
            >
              {loading ? '⏳ Solving…' : '→ SOLVE'}
            </button>
            <button onClick={reset} style={{ flex: 1, padding: '10px', borderRadius: 7, border: '1px solid #374151', background: 'transparent', color: '#6b7280', cursor: 'pointer', fontSize: 12 }}>Reset</button>
          </div>

          {error && <div style={{ fontSize: 12, color: '#f87171', background: 'rgba(239,68,68,0.08)', borderRadius: 6, padding: '8px 10px' }}>{error}</div>}
        </div>

        {/* ── Right results panel ── */}
        <div style={{ flex: 1, padding: 24, overflowY: 'auto' }}>
          {!result && !loading && (
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: '100%', color: '#4b5563', textAlign: 'center' }}>
              <div>
                <div style={{ fontSize: 40, marginBottom: 12 }}>🃏</div>
                <div style={{ fontSize: 15, fontWeight: 600, color: '#6b7280' }}>No solution yet</div>
                <div style={{ fontSize: 12, marginTop: 6 }}>Select hero hand, board, and click Solve</div>
              </div>
            </div>
          )}

          {loading && (
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: '100%', color: '#6b7280' }}>
              <div style={{ textAlign: 'center' }}>
                <div style={{ fontSize: 14, marginBottom: 8 }}>⏳ Computing…</div>
                <div style={{ fontSize: 12 }}>Running equity calculations</div>
              </div>
            </div>
          )}

          {result && !loading && (
            <div style={{ display: 'flex', flexDirection: 'column', gap: 20, maxWidth: 480 }}>
              {/* Board + texture */}
              <div>
                <div style={{ display: 'flex', alignItems: 'center', gap: 12, marginBottom: 8 }}>
                  <BoardDisplay cards={board} />
                  {result.board_texture !== '—' && (
                    <span style={{ fontSize: 11, color: '#6b7280', padding: '2px 8px', border: '1px solid #374151', borderRadius: 12 }}>
                      {result.board_texture}
                    </span>
                  )}
                </div>
              </div>

              {/* Equity */}
              <EquityBar equity={result.equity} />

              {/* SPR + pot odds */}
              <div style={{ display: 'flex', gap: 20 }}>
                {[
                  { label: 'SPR', value: result.spr.toFixed(1) },
                  { label: 'Pot odds', value: `${(result.pot_odds * 100).toFixed(1)}%` },
                ].map(({ label, value }) => (
                  <div key={label} style={{ display: 'flex', flexDirection: 'column' }}>
                    <span style={{ fontSize: 10, color: '#6b7280', marginBottom: 2 }}>{label}</span>
                    <span style={{ fontSize: 16, fontWeight: 700, fontFamily: 'monospace', color: '#e5e7eb' }}>{value}</span>
                  </div>
                ))}
              </div>

              {/* Action */}
              <div>
                <SectionLabel>Recommended Action</SectionLabel>
                <ActionBadge action={result.action} sizingBb={result.sizing_bb} sizingPct={result.sizing_pct_pot} />
              </div>

              {/* Reasoning */}
              <div>
                <SectionLabel>Reasoning</SectionLabel>
                <div style={{ fontSize: 13, color: '#9ca3af', lineHeight: 1.7, background: 'rgba(255,255,255,0.03)', border: '1px solid #1f1f1f', borderRadius: 6, padding: '10px 12px' }}>
                  {result.reasoning}
                </div>
              </div>

              {/* Range Analysis */}
              {(rangeAnalysis || rangeLoading) && (
                <div>
                  <button onClick={() => setRangeExpanded(v => !v)} style={{ background:'none',border:'none',cursor:'pointer',padding:0,display:'flex',alignItems:'center',gap:4,marginBottom:8 }}>
                    <span style={{ fontSize:10,color:'#6b7280',textTransform:'uppercase',letterSpacing:1 }}>{rangeExpanded?'▾':'▸'} Range Analysis</span>
                  </button>
                  {rangeExpanded && (
                    <div style={{ display:'flex',flexDirection:'column',gap:16 }}>
                      <RangeGrid hands={rangeAnalysis?.hands??[]} loading={rangeLoading} />
                      {rangeAnalysis && <EquityDistribution buckets={rangeAnalysis.buckets} heroAvgEquity={rangeAnalysis.hero_avg_equity} villainAvgEquity={rangeAnalysis.villain_avg_equity} nutAdvantage={rangeAnalysis.nut_advantage} />}
                    </div>
                  )}
                </div>
              )}

              {/* Bet Size Comparison */}
              {(betSizes || betSizesLoading) && (
                <div>
                  <button onClick={() => setBetSizesExpanded(v => !v)} style={{ background:'none',border:'none',cursor:'pointer',padding:0,display:'flex',alignItems:'center',gap:4,marginBottom:8 }}>
                    <span style={{ fontSize:10,color:'#6b7280',textTransform:'uppercase',letterSpacing:1 }}>{betSizesExpanded?'▾':'▸'} Bet Size Comparison</span>
                  </button>
                  {betSizesExpanded && betSizesLoading && <div style={{fontSize:12,color:'#6b7280'}}>⏳ Calculating bet sizes…</div>}
                  {betSizesExpanded && betSizes && <BetSizeComparison options={betSizes.options} recommended_idx={betSizes.recommended_idx} pot_bb={betSizes.pot_bb} />}
                </div>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export default App;
