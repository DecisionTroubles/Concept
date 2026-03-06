# Findings - 3D Memory Graph Platform

## Project Structure
- Tauri v2 + Vue 3 + TypeScript desktop app with implemented Rust backend + SQLite
- pnpm package manager, Vite v7, TailwindCSS v4, Pinia
- 3D stack integrated: `three`, `@tresjs/core`, `@tresjs/cientos`, `@tresjs/post-processing`
- Force layout currently lives in frontend via `d3-force-3d`
- Domain-pack v2 seed pipeline active through `domains/japanese/pack.json`
- Frontend plugin kernel implemented with core plugins and drop-in `user-plugins/`
- Authoring is now split:
  - focused node editing via dedicated node editor overlay
  - global note type/library editing via Settings -> Authoring

## Key Source Files
| File | Purpose |
|------|---------|
| `src/App.vue` | Frontend root composed from kernel module slots |
| `src/stores/graph.ts` | Graph state orchestration |
| `src-tauri/src/lib.rs` | Rust command handlers + app bootstrap |
| `src-tauri/src/graph.rs` | Graph CRUD, note types, progress logic |
| `src-tauri/src/domain.rs` | Domain-pack import/seed logic |
| `src-tauri/src/scheduler.rs` | Scheduler abstraction + review events |
| `src-tauri/src/extensions.rs` | Node extension data persistence |
| `src/core/kernel.ts` | Frontend plugin kernel |
| `src/components/NodeDetailPanel.vue` | Side summary + centered node viewer |
| `src/components/authoring/AuthoringPanel.vue` | Global note type/library authoring surface |
| `src/components/NodeEditorOverlay.vue` | Focused node editing workspace |
| `README.md` | Usage and extension entrypoint |
| `docs/DESIGN.md` | Architecture / design reference |
| `OVERRIDES.md` | Plugin, override, and dataset workflow guide |

## Architecture Decisions
- Strict frontend/backend separation is still the right constraint:
  - frontend renders and edits
  - backend owns graph truth, learning logic, and persistence
- SQLite remains the single source of truth; Pinia stores are derived/cached views
- Domain behavior is pack-driven, not hardcoded per language/domain
- Node visibility layers and connection layers are separate systems
- Scheduler logic is isolated behind a backend module boundary

## Data Model Findings
- Nodes now use:
  - `note_type_id`
  - `note_fields`
  - `content_data` as fallback compatibility text
- Note types now carry:
  - `world_id`
  - `base_note_type_id`
  - `schema_json`
  - `layout_json`
  - `metadata`
- Domain packs now support:
  - `note_types`
  - per-node `note_type_id`
  - per-node `note_fields`
- Node extension data is stored generically by `node_id + extension_key`

## UI / UX Findings
- The side node panel must stay lightweight.
- The centered node window works better as a page/slide viewer than as a tabbed inspector.
- Notes/assets/AI-style tools should appear as separate pages, not one clumped "extensions" page.
- Buffers are a distinct concept from node viewing and should stay visually/behaviorally separate.

## Plugin / Override Findings
- Core plugins should remain internal defaults.
- User plugins should live outside `src/` in a dedicated drop-in folder.
- Frontend module overrides, theme registration, and node extension pages all fit this plugin kernel well.
- Current user plugin loading is Vite-bundled from `user-plugins/`; true runtime external plugin loading is still future work.

## Root Cause Notes

### Overlay stacking issue
- Problem source was stacking context at parent/backdrop level, not child z-index alone.
- Correct fix path is root modal/backdrop layering and consistent z-layer tokens.

### Black screen issue
1. Missing `postprocessing` peer dependency for `@tresjs/post-processing`
2. Raw `three/examples/jsm` Line2 path was unreliable under TresJS render integration

### Pack parse failure
- `domains/japanese/pack.json` failed when saved with UTF-8 BOM
- Rust JSON parser rejected BOM at byte 0
- Pack files must be UTF-8 without BOM

## Open Questions
- Force layout ownership for large worlds: frontend vs worker vs Rust
- Pack export/import UX and validation surface
- Real file-backed asset pipeline for node assets
- Dedicated review session flow and analytics surfaces
