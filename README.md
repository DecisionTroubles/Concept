# 3D Memory Graph

Desktop app for learning through a navigable 3D graph of connected knowledge nodes.

The app loads a world from a pack, renders it as a 3D map, and lets you read, edit, and navigate structured nodes with note-type-driven detail pages.

## Stack

- Desktop: Tauri v2
- Frontend: Vue 3 + TypeScript + Pinia + TresJS/Three.js
- Backend: Rust
- Storage: SQLite via `rusqlite`
- IPC: TauRPC

## Development

```sh
pnpm i
pnpm tauri dev
```

Checks:

```sh
cargo check --manifest-path src-tauri/Cargo.toml
pnpm vue-tsc --noEmit
pnpm vite build
```

## Project Structure

- `src/` - Vue frontend
- `src-tauri/` - Rust backend
- `docs/` - design/spec docs
- `docs/claude/` - findings, progress, and planning notes
- `user-plugins/` - drop-in user plugins
- `OVERRIDES.md` - customization/plugin/dataset guide
- `WORLD_PACK.md` - source-pack authoring guide

## World Packs

Runtime still loads canonical `pack.json` v2 packs, but you should author packs in the new source-pack format:

- `pack.toml`
- `theme.toml`
- `note-types/*.toml`
- `relation-kinds/*.toml`
- `layers/*.toml`
- `connection-layers/*.toml`
- `groups/*.toml`
- `nodes/*.md`

That source pack is then compiled into runtime `pack.json`.

Use [WORLD_PACK.md](/C:/Projects/concept/WORLD_PACK.md) for the actual authoring workflow.

## Current Pack Flow

- app startup creates a starter source pack in app data if needed
- the starter source pack is compiled to `pack.json`
- pack library sources can be either:
  - source packs
  - runtime packs
- source packs are compiled before install/sync
- runtime packs still work for compatibility

## Create A New World

You can create a local world from the app.

That now generates:

- a source-pack folder on disk
- a compiled runtime `pack.json`

So a “blank” or “starter” world is no longer hand-authored as one giant JSON file.

## Update Nodes In-App

Current editing flow:

1. select a node in the 3D world
2. press `X`
3. or click the pencil button in the side panel / centered node viewer

You can edit:

- title
- tags
- note type
- fallback content
- structured note fields from the current note type schema

Important:

- these are runtime/editorial edits
- they do not currently round-trip back into source-pack files

## Plugins and Overrides

User plugins are auto-loaded from:

- `user-plugins/**/*.ts`
- `user-plugins/**/*.js`
- `user-plugins/**/*.mjs`

Supported exports:

```ts
export default definePlugin({...})
export const plugin = definePlugin({...})
export const plugins = [definePlugin({...})]
```

Use plugins to:

- override frontend module slots
- add themes
- add node extension pages

See:

- `OVERRIDES.md`
- `user-plugins/example.plugin.ts`

## Notes

- `src/bindings.ts` is generated and should be committed after IPC/type changes
- stale dev processes can interfere with startup; fully restart when changing seed/data behavior
- some older docs and notes may still mention `domains/*/pack.json`; current code should be treated as the source of truth
