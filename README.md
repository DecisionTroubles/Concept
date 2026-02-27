# 3D Memory Graph

A desktop app for learning through a navigable 3D graph of connected knowledge nodes. Designed initially for Japanese language learning but architecturally domain-agnostic.

Nodes represent learning items (words, grammar rules, kanji, concepts). Edges encode **context relationships** — the primary organizing principle of the system. The user moves through 3D space; proximity and clustering encode meaning.

## Tech Stack

- **Desktop**: Tauri v2 (Rust + WebView2)
- **Frontend**: Vue 3 + TypeScript, Pinia, TailwindCSS v4, TresJS (Three.js)
- **Backend**: Rust (Tauri command handlers, all graph/learning logic)
- **Storage**: SQLite via rusqlite
- **IPC**: TauRPC (typed, auto-generated TypeScript bindings)

## Development

```sh
pnpm i
pnpm tauri dev
```

**Run frontend tests:**
```sh
pnpm test
```

**Build:**
```sh
pnpm tauri build
```

## Structure

- `src/` — Vue 3 frontend (rendering and interaction only)
- `src-tauri/` — Rust backend (graph logic, learning algorithms, SQLite)
- `docs/` — Architecture and design documentation
