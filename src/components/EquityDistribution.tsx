import { memo } from 'react';
import { bucketColor } from './RangeGrid';

interface Props {
  equity_buckets: number[];   // 5 values summing to 1 (0-20,20-40,40-60,60-80,80-100%)
  heroAvgEquity: number;      // 0–1
  villainAvgEquity: number;   // 0–1
  nutAdvantage: 'hero' | 'villain' | 'neutral';
}

export const EquityDistribution = memo(({ equity_buckets, heroAvgEquity, villainAvgEquity, nutAdvantage }: Props) => {
  if (!equity_buckets?.length) return null;
  const pcts = equity_buckets.map(b => Math.round(b * 100));

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
      <div style={{ fontSize: 10, color: '#6b7280', textTransform: 'uppercase', letterSpacing: 1 }}>
        Villain Range Equity Distribution
      </div>

      {/* 5-segment stacked bar */}
      <div>
        <div style={{ display: 'flex', height: 14, borderRadius: 3, overflow: 'hidden' }}>
          {equity_buckets.map((b, i) => (
            <div
              key={i}
              style={{
                width: `${b * 100}%`,
                background: bucketColor(i),
                transition: 'width 0.3s ease',
              }}
              title={`Bucket ${i}: ${Math.round(b * 100)}%`}
            />
          ))}
        </div>
        {/* Percentage labels */}
        <div style={{ display: 'flex', marginTop: 3 }}>
          {pcts.map((p, i) => (
            <div
              key={i}
              style={{
                width: `${equity_buckets[i] * 100}%`,
                textAlign: 'center', fontSize: 9,
                color: '#6b7280', fontFamily: 'monospace',
                overflow: 'hidden', minWidth: 0,
              }}
            >
              {p > 5 ? `${p}%` : ''}
            </div>
          ))}
        </div>
      </div>

      {/* Avg equity */}
      <div style={{ display: 'flex', gap: 12, fontSize: 11 }}>
        <span style={{ color: '#6b7280' }}>Avg equity:</span>
        <span>
          <span style={{ color: '#4ade80', fontWeight: 700 }}>Hero {(heroAvgEquity * 100).toFixed(1)}%</span>
          <span style={{ color: '#4b5563', margin: '0 4px' }}>·</span>
          <span style={{ color: '#f87171', fontWeight: 700 }}>Villain {(villainAvgEquity * 100).toFixed(1)}%</span>
        </span>
      </div>

      {/* Nut advantage */}
      <div style={{ fontSize: 11, color: '#6b7280' }}>
        Nut advantage:{' '}
        <span style={{
          fontWeight: 700,
          color: nutAdvantage === 'hero' ? '#4ade80' : nutAdvantage === 'villain' ? '#f87171' : '#9ca3af',
        }}>
          {nutAdvantage === 'hero' ? 'Hero ▲' : nutAdvantage === 'villain' ? 'Villain ▼' : 'Neutral ─'}
        </span>
      </div>
    </div>
  );
});
EquityDistribution.displayName = 'EquityDistribution';
