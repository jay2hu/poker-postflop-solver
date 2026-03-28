import React from 'react';
import { useSolverStore } from './store/solverStore';
import { CardPicker } from './components/CardPicker';
import { RangeSelector } from './components/RangeSelector';
import { ResultPanel } from './components/ResultPanel';
import { TitleBar } from './components/TitleBar';

const inputStyle: React.CSSProperties = {
  width: '100%',
  padding: '6px 10px',
  background: '#161b22',
  border: '1px solid #30363d',
  borderRadius: '6px',
  color: '#e6edf3',
  fontSize: '13px',
  boxSizing: 'border-box',
};

const labelStyle: React.CSSProperties = {
  fontSize: '11px',
  fontWeight: 700,
  color: '#6e7681',
  textTransform: 'uppercase',
  letterSpacing: '0.5px',
  marginBottom: '6px',
  display: 'block',
};

function App() {
  const store = useSolverStore();
  const canSolve = store.heroHand.length === 2 && store.board.length >= 3;
  const handleTauri = async (action: 'minimize' | 'maximize' | 'close') => {
    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      const win = getCurrentWindow();
      if (action === 'minimize') await win.minimize();
      else if (action === 'maximize') await win.toggleMaximize();
      else await win.close();
    } catch {
      // browser mode — no-op
    }
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100vh', background: '#0d1117', color: '#e6edf3', fontFamily: 'system-ui, sans-serif', overflow: 'hidden' }}>
      <TitleBar
        onMin={() => handleTauri('minimize')}
        onMax={() => handleTauri('maximize')}
        onClose={() => handleTauri('close')}
      />

      <div style={{ display: 'flex', flex: 1, overflow: 'hidden' }}>
        {/* Left panel */}
        <div style={{ width: '380px', flexShrink: 0, borderRight: '1px solid #21262d', overflowY: 'auto', padding: '20px 16px', display: 'flex', flexDirection: 'column', gap: '18px' }}>

          <section>
            <label style={labelStyle}>Hero Hand (2 cards)</label>
            <CardPicker
              value={store.heroHand}
              maxCards={2}
              onChange={store.setHeroHand}
              disabledCards={store.board}
            />
          </section>

          <section>
            <label style={labelStyle}>Board (3–5 cards)</label>
            <CardPicker
              value={store.board}
              maxCards={5}
              onChange={store.setBoard}
              disabledCards={store.heroHand}
            />
          </section>

          <section>
            <label style={labelStyle}>Pot (bb)</label>
            <input
              type="number"
              min={0}
              value={store.potBb}
              onChange={(e) => store.setPotBb(Number(e.target.value))}
              style={inputStyle}
            />
          </section>

          <section style={{ display: 'flex', gap: '10px' }}>
            <div style={{ flex: 1 }}>
              <label style={labelStyle}>Hero Stack (bb)</label>
              <input type="number" min={0} value={store.heroStackBb} onChange={(e) => store.setHeroStackBb(Number(e.target.value))} style={inputStyle} />
            </div>
            <div style={{ flex: 1 }}>
              <label style={labelStyle}>Villain Stack (bb)</label>
              <input type="number" min={0} value={store.villainStackBb} onChange={(e) => store.setVillainStackBb(Number(e.target.value))} style={inputStyle} />
            </div>
          </section>

          <section>
            <label style={labelStyle}>To Call (bb)</label>
            <input type="number" min={0} value={store.toCallBb} onChange={(e) => store.setToCallBb(Number(e.target.value))} style={inputStyle} />
          </section>

          <section>
            <label style={labelStyle}>Position</label>
            <div style={{ display: 'flex', gap: '8px' }}>
              {['IP', 'OOP'].map((pos) => {
                const active = pos === 'IP' ? store.isIp : !store.isIp;
                return (
                  <button
                    key={pos}
                    onClick={() => store.setIsIp(pos === 'IP')}
                    style={{
                      padding: '6px 20px',
                      borderRadius: '6px',
                      fontSize: '13px',
                      fontWeight: 600,
                      cursor: 'pointer',
                      border: active ? '1px solid #58a6ff' : '1px solid #30363d',
                      background: active ? '#1f6feb33' : '#161b22',
                      color: active ? '#58a6ff' : '#8b949e',
                    }}
                  >
                    {pos}
                  </button>
                );
              })}
            </div>
          </section>

          <section>
            <label style={labelStyle}>Villain Range</label>
            <RangeSelector value={store.villainRange} onChange={store.setVillainRange} />
          </section>

          <div style={{ display: 'flex', gap: '8px' }}>
            <button
              onClick={store.solve}
              disabled={!canSolve || store.loading}
              style={{
                flex: 1,
                padding: '10px',
                borderRadius: '6px',
                fontSize: '14px',
                fontWeight: 700,
                cursor: canSolve && !store.loading ? 'pointer' : 'not-allowed',
                border: 'none',
                background: canSolve && !store.loading ? '#238636' : '#161b22',
                color: canSolve && !store.loading ? '#fff' : '#3d4451',
              }}
            >
              {store.loading ? 'Solving…' : 'SOLVE'}
            </button>
            <button
              onClick={store.reset}
              style={{
                padding: '10px 14px',
                borderRadius: '6px',
                fontSize: '13px',
                cursor: 'pointer',
                border: '1px solid #30363d',
                background: '#161b22',
                color: '#8b949e',
              }}
            >
              Reset
            </button>
          </div>

          {store.error && (
            <div style={{ fontSize: '12px', color: '#f87171', background: '#da363322', padding: '8px', borderRadius: '6px' }}>
              {store.error}
            </div>
          )}
        </div>

        {/* Right panel */}
        <div style={{ flex: 1, overflowY: 'auto', padding: '24px' }}>
          {store.result ? (
            <ResultPanel result={store.result} board={store.board} />
          ) : (
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: '100%', color: '#3d4451', fontSize: '14px' }}>
              {store.loading ? 'Calculating…' : 'Select hero hand + board and press SOLVE'}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

export default App;
