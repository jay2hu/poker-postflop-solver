# Poker Postflop Solver

A Tauri + React TypeScript app for postflop poker decision support.

## Features
- CardPicker with 4×13 grid (suits × ranks)
- BoardDisplay with suit-colored badges
- EquityBar, ActionBadge, RangeSelector
- Zustand store with solve() action
- Mock solver for browser mode, Tauri invoke for desktop

## Dev
```bash
npm install
npm run dev      # port 1423
npm test         # 31 tests
npm run build
```

