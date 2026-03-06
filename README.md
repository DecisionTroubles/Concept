# 3D Memory Graph

Desktop app for learning through a navigable 3D graph of connected knowledge nodes.

The system is domain-agnostic. A world is loaded from a domain pack and rendered as a 3D knowledge map with structured nodes, relation-driven edges, learning progress, and extension pages.

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
- `domains/` - bundled domain packs
- `docs/` - design/spec docs
- `docs/claude/` - findings, progress, and planning notes
- `user-plugins/` - drop-in user plugins
- `OVERRIDES.md` - customization/plugin/dataset guide

## Load Your Own 3D World

The app currently seeds from a domain pack JSON file.

Bundled example:
- `domains/japanese/pack.json`

To load your own world:

1. create `domains/<your-world>/pack.json`
2. follow `WORLD_PACK.md`
3. define:
   - `world`
   - `note_types`
   - `relation_kinds`
   - `layers`
   - `connection_layers`
   - `nodes`
   - `edges`
4. restart the app
5. open `Settings -> Worlds`
6. choose your world and open it

Important:
- pack files must be UTF-8 without BOM
- nodes should use `note_type_id` + `note_fields`
- page order in the centered node viewer comes from note type `layout_json`
- bundled worlds are scanned from `domains/*/pack.json`

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

`Settings -> Authoring` is now for global note type management, not focused node editing.

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
