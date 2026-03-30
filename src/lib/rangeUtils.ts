// Range utility functions

export type RangeMap = Record<string, number>; // hand → frequency 0–1

/** Convert RangeMap to string: "AA:1.0,KK:1.0,AKs:0.85,..." */
export function rangeMapToString(map: RangeMap): string {
  return Object.entries(map)
    .filter(([, freq]) => freq > 0)
    .sort((a, b) => b[1] - a[1] || a[0].localeCompare(b[0]))
    .map(([hand, freq]) => freq >= 0.995 ? hand : `${hand}:${freq.toFixed(2)}`)
    .join(',');
}

/** Parse range string back to RangeMap */
export function stringToRangeMap(s: string): RangeMap {
  const map: RangeMap = {};
  if (!s.trim()) return map;
  s.split(/[,\s]+/).forEach(token => {
    token = token.trim();
    if (!token) return;
    const colon = token.lastIndexOf(':');
    if (colon > 0) {
      const hand = token.slice(0, colon);
      const freq = parseFloat(token.slice(colon + 1));
      if (hand && !isNaN(freq)) map[hand] = Math.min(1, Math.max(0, freq));
    } else {
      map[token] = 1.0;
    }
  });
  return map;
}

/** Compute VPIP %: weighted average of all non-pure-fold hands */
export function computeVPIP(map: RangeMap): number {
  const TOTAL_HAND_COMBOS = 1326;
  const HAND_COMBOS: Record<string, number> = {};

  // Approximate combo counts
  const ranks = ['A','K','Q','J','T','9','8','7','6','5','4','3','2'];
  ranks.forEach((r1, i) => {
    ranks.forEach((r2, j) => {
      if (i === j) HAND_COMBOS[`${r1}${r1}`] = 6;
      else if (i < j) HAND_COMBOS[`${r1}${r2}s`] = 4;
      else if (i > j) HAND_COMBOS[`${r2}${r1}o`] = 12; // note: always higher rank first
    });
  });

  let played = 0;
  for (const [hand, freq] of Object.entries(map)) {
    const combos = HAND_COMBOS[hand] ?? (hand.length === 2 ? 6 : hand.endsWith('s') ? 4 : 12);
    played += combos * freq;
  }
  return (played / TOTAL_HAND_COMBOS) * 100;
}

/** Total weighted combos */
export function totalCombos(map: RangeMap): number {
  let total = 0;
  for (const [hand, freq] of Object.entries(map)) {
    const combos = hand.length === 2 && hand[0] === hand[1] ? 6
      : hand.endsWith('s') ? 4 : 12;
    total += combos * freq;
  }
  return Math.round(total);
}

// ── V2 strategy file → RangeMap ───────────────────────────────────────────────

/** Action type labels to match v2 scenario keys */
export type ActionType = 'rfi' | 'called_rfi' | '3bettor' | '4bettor';

const POS_TO_V2: Record<string, string> = {
  utg: 'UTG', 'utg+1': 'UTG1', lj: 'LJ', hj: 'HJ',
  co: 'CO', btn: 'BTN', sb: 'SB', bb: 'BB',
};

/** Map action type + position to a v2 scenario key */
export function v2ScenarioKey(action: ActionType, position: string, vsPosition?: string): string {
  const pos = POS_TO_V2[position] ?? position.toUpperCase();
  switch (action) {
    case 'rfi':        return `${pos}_rfi`;
    case 'called_rfi': return `${pos}_vs_open_${vsPosition ? (POS_TO_V2[vsPosition] ?? vsPosition.toUpperCase()) : 'BTN'}`;
    case '3bettor':    return `${pos}_vs3bet_${vsPosition ? (POS_TO_V2[vsPosition] ?? vsPosition.toUpperCase()) : 'UTG'}`;
    case '4bettor':    return `${pos}_vs4bet_${vsPosition ? (POS_TO_V2[vsPosition] ?? vsPosition.toUpperCase()) : 'UTG'}`;
  }
}

/** Extract range from a v2 scenario. Uses raise (rfi), call (called_rfi) etc. */
export function extractRangeFromScenario(
  scenario: { hands: Record<string, Record<string, number>> },
  targetAction: string, // 'raise' | 'call'
  minFreq = 0.05,
): RangeMap {
  const map: RangeMap = {};
  for (const [hand, actions] of Object.entries(scenario.hands)) {
    // Sum all raise-type actions (raise 2.3, raise 100, etc.) or call-type
    let freq = 0;
    for (const [action, f] of Object.entries(actions)) {
      if (action.toLowerCase().startsWith(targetAction)) freq += f;
    }
    if (freq >= minFreq) map[hand] = parseFloat(freq.toFixed(3));
  }
  return map;
}

/** Load v2 file and extract range */
export async function loadGtoRange(
  stack: number,
  action: ActionType,
  position: string,
  vsPosition?: string,
): Promise<RangeMap> {
  const res = await fetch(`/strategies/v2/mmt_8p_${stack}bb.json`);
  if (!res.ok) throw new Error(`Failed to load ${stack}bb strategy file`);
  const file = await res.json() as { scenarios: Record<string, { hands: Record<string, Record<string, number>> }> };

  const scenarioKey = v2ScenarioKey(action, position, vsPosition);
  const scenario = file.scenarios[scenarioKey]
    ?? file.scenarios[Object.keys(file.scenarios).find(k => k.toLowerCase() === scenarioKey.toLowerCase()) ?? ''];

  if (!scenario) throw new Error(`Scenario "${scenarioKey}" not found in ${stack}bb file`);

  const targetAction = action === 'called_rfi' ? 'call' : 'raise';
  return extractRangeFromScenario(scenario, targetAction);
}
