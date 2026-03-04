# Task Plan — 3D Memory Graph Platform

## Project Summary
A Tauri v2 + Vue 3 + Three.js + Rust + SQLite desktop app for context-driven knowledge graph learning. Nodes represent learning items; edges encode context relationships. The 3D graph is the primary UI metaphor.

## Status: IMPLEMENTATION PHASE

---

## Phase 0 — Architecture & Documentation ✅ COMPLETE
- [x] Write DESIGN.md with concrete tech decisions
- [x] Create docs/claude/ planning directory
- [x] Write docs/CLAUDE.md (AI context file)
- [x] Finalize node/edge schema in findings.md
- [x] Add override/customization guide (`docs/OVERRIDES.md`)

## Phase 1 — Rust Backend Foundation
- [x] Set up SQLite with rusqlite in Cargo.toml
- [x] Define Node, Edge, Layer structs with serde
- [x] Implement graph CRUD Tauri commands
- [ ] Basic spaced repetition (SM-2) logic
- [ ] Unit tests for graph + SR logic

## Phase 2 — Three.js Graph Renderer
- [x] Add three.js to frontend deps
- [x] Basic 3D scene: camera, lights, orbit controls
- [x] Render typed nodes + edges
- [x] Force-directed layout in 3D
- [x] Node click → info panel

## Phase 3 — Layer System
- [x] Layer switching UI
- [ ] Cross-layer edge rendering
- [ ] Depth navigation (drill into sub-layer)

## Phase 4 — Adaptive Learning UI
- [x] Node state visualization (selected / pinned / learned / neighbor)
- [x] Recommendation-style highlights (neighbor emphasis + compass)
- [ ] Review session flow

## Phase 5 — Data & Import
- [ ] Domain pack format (SQLite DB file)
- [ ] Japanese starter pack (grammar → kanji layers)
- [ ] Anki import/export

## Phase 6 — Polish
- [x] Themes / CSS override support (preset registry + runtime switch)
- [ ] Performance: LOD for large graphs
- [x] Settings panel (center modal, hotkeys + themes tabs)
- [x] Buffer overlays (pinned cards/list + quick map)

## Cross-cutting
- [x] Plugin kernel for frontend module overrides
- [x] Example plugin entrypoint + user plugin registry

---

## Key Decisions
| Decision | Choice | Rationale |
|----------|--------|-----------|
| 3D library | Three.js | Lighter, larger graph-rendering ecosystem |
| Storage | SQLite via rusqlite | Pragmatic, offline, Anki-compat |
| SR algorithm | Manual mark-as-learned (SM-2 is future) | Simpler, ships first |
| UI framework | shadcn-vue (Radix Vue) | Dark-first, Tailwind-native, no canvas bleed |
| Node geometry | Shape encodes type (sphere/octahedron/box/torus) | Visual language without labels |
| Frontend framework | Vue 3 + Pinia | Already in template |
| Styling | TailwindCSS v4 | Already in template |
