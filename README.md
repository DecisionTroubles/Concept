# 3D Memory Graph

A desktop app for learning through a navigable 3D graph of connected knowledge nodes. Designed initially for Japanese language learning but architecturally domain-agnostic.

Nodes represent learning items such as words, grammar rules, kanji, and concepts. Edges encode context relationships. The user moves through 3D space; proximity and clustering encode meaning.

## Tech Stack

- Desktop: Tauri v2 (Rust + WebView2)
- Frontend: Vue 3 + TypeScript, Pinia, TailwindCSS v4, TresJS (Three.js)
- Backend: Rust (Tauri command handlers, graph and learning logic)
- Storage: SQLite via rusqlite
- IPC: TauRPC (typed, auto-generated TypeScript bindings)

## Development

```sh
pnpm i
pnpm tauri dev
```

Frontend tests:

```sh
pnpm test
```

Build:

```sh
pnpm tauri build
```

## Dev Notes

`src/bindings.ts` is generated and must be committed. If the API changes, run `pnpm tauri dev` and commit the updated file.

`vue-devtools` may exit with code `0` shortly after startup. That is expected. The dev script uses `--kill-others-on-fail`, so Vite should only be killed when a process exits non-zero.

## Structure

- `src/` — Vue frontend
- `src-tauri/` — Rust backend
- `docs/` — architecture and design notes
- `domains/` — bundled dataset/domain packs
- `user-plugins/` — drop-in user plugins loaded after core plugins

## User Plugins

User plugins are auto-loaded from `user-plugins/**/*.ts`, `user-plugins/**/*.js`, and `user-plugins/**/*.mjs`.

Supported exports:

```ts
export default definePlugin({...})
```

```ts
export const plugin = definePlugin({...})
```

```ts
export const plugins = [definePlugin({...}), definePlugin({...})]
```

Minimal example:

```ts
import { definePlugin } from '@/core/plugin'

export default definePlugin({
  id: 'user.example',
  name: 'User Example',
  nodeWorkspaceExtensions: [
    {
      id: 'user.example.notes',
      title: 'Extra Notes',
      description: 'Adds a custom node workspace block.',
      slot: 'extensions.primary',
      order: 100,
    },
  ],
})
```

Core plugins load first. User plugins load after them, so a user plugin can override module slots or add node workspace extensions without editing `src/plugins/defaultPlugin.ts`.
