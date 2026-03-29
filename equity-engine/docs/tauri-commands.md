# Tauri Command API — Equity Engine

**For Frontend Dev:** These are the Rust-side Tauri commands you invoke from the React/TypeScript layer.

---

## Feature Flags

The equity engine is a standalone Rust library. Tauri commands live in `src/commands.rs` and are gated behind the `tauri` feature flag.

When assembling the Tauri app shell (`src-tauri`), add the engine as a dependency with the feature enabled:

```toml
# src-tauri/Cargo.toml
[dependencies]
equity-engine = { path = "../equity-engine", features = ["tauri"] }
```

Register the commands in `main.rs`:

```rust
use equity_engine::commands::*;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            calculate_suggestion,
            calculate_suggestion_with_range,
            load_range_profile,
            list_range_profiles,
        ])
        .run(tauri::generate_context!())
        .unwrap();
}
```

---

## Commands

### `calculate_suggestion`

Primary command. Takes the current game state, uses the `"default"` villain range for the current street, returns a suggestion.

**TypeScript invoke:**
```typescript
import { invoke } from '@tauri-apps/api/tauri';

const suggestion = await invoke<Suggestion>('calculate_suggestion', {
  gameState: {
    street: 'flop',
    heroCards: [{ rank: 'A', suit: 'h' }, { rank: 'K', suit: 'd' }],
    boardCards: [
      { rank: 'Q', suit: 's' },
      { rank: 'J', suit: 'c' },
      { rank: '2', suit: 'h' },
    ],
    potSize: 120,
    currentBet: 60,
    heroStack: 800,
    villainStack: 750,
    position: 'BTN',
    toAct: true,
  }
});
```

**Input:** `GameState` (see types below)

**Output:** `Suggestion` (see types below) or throws string error

---

### `calculate_suggestion_with_range`

Same as above but lets you specify a villain range preset by name.

**TypeScript invoke:**
```typescript
const suggestion = await invoke<Suggestion>('calculate_suggestion_with_range', {
  gameState: { /* GameState */ },
  rangeName: 'loose'  // 'tight' | 'default' | 'loose'
});
```

**Input:** `GameState` + `rangeName: string`

**Output:** `Suggestion` or throws string error

---

### `load_range_profile`

Load a range profile for display purposes (e.g. showing the villain range in the HUD settings panel).

**TypeScript invoke:**
```typescript
const profile = await invoke<RangeProfile>('load_range_profile', {
  street: 'preflop',
  name: 'default'
});
```

**Input:** `street: 'preflop' | 'flop' | 'turn' | 'river'`, `name: string`

**Output:** `RangeProfile` or throws string error

---

### `list_range_profiles`

Returns the list of built-in profile names.

**TypeScript invoke:**
```typescript
const profiles = await invoke<string[]>('list_range_profiles');
// → ['tight', 'default', 'loose']
```

**Input:** none

**Output:** `string[]`

---

## TypeScript Types

Mirror of the Rust structs — match these exactly in your frontend types:

```typescript
type Suit = 'h' | 'd' | 'c' | 's';
type Rank = '2'|'3'|'4'|'5'|'6'|'7'|'8'|'9'|'T'|'J'|'Q'|'K'|'A';
interface Card { rank: Rank; suit: Suit; }

type Street = 'preflop' | 'flop' | 'turn' | 'river';
type Position = 'BTN' | 'SB' | 'BB' | 'UTG' | 'MP' | 'CO';

interface GameState {
  street: Street;
  heroCards: [Card, Card];
  boardCards: Card[];
  potSize: number;
  currentBet: number;
  heroStack: number;
  villainStack: number;
  position: Position;
  toAct: boolean;
  /** Number of active players including hero. Range: 2–9. Default: 2 (heads-up). */
  playerCount: number;
}

interface Suggestion {
  equity: number;           // 0.0–1.0
  potOdds: number;          // 0.0–1.0
  recommendation: 'FOLD' | 'CALL' | 'RAISE';
  confidence: 'HIGH' | 'MEDIUM' | 'LOW';
  reasoning: string;        // human-readable, includes position note for BTN/CO/BB
  calcTimeMs: number;       // for perf monitoring
  suggestedBetSize: number | null; // chips; populated on RAISE, null on CALL/FOLD
}

interface HandCombo {
  combo: string;            // e.g. "AKs", "QQ", "AhKd"
  weight: number;           // 0.0–1.0, default 1.0
}

interface RangeProfile {
  name: string;
  street: Street;
  combos: HandCombo[];
}
```

---

## Events (pushed by backend)

The screen parser (Techno Lead's workstream) will emit `game-state-update` when a new `GameState` is detected. The backend will process and emit `suggestion-update` with the `Suggestion`.

**Frontend listener:**
```typescript
import { listen } from '@tauri-apps/api/event';

await listen<Suggestion>('suggestion-update', (event) => {
  updateHUD(event.payload);
});
```

---

## Notes

- All command inputs use **camelCase** (TypeScript convention) — serde handles the mapping to Rust snake_case automatically.
- Equity calculation target: **<100ms**. The `calcTimeMs` field lets you monitor this in the HUD during dev.
- Range profiles are loaded from bundled assets at `equity-engine/assets/ranges/<street>/<name>.json`. Tauri will need these in the bundle — DevOps to configure asset bundling.
