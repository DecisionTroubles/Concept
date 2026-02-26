# Task Plan — 3D Memory Graph Platform

## Project Summary
A Tauri v2 + Vue 3 + Three.js + Rust + SQLite desktop app for context-driven knowledge graph learning. Nodes represent learning items; edges encode context relationships. The 3D graph is the primary UI metaphor.

## Status: ARCHITECTURE PHASE

---

## Phase 0 — Architecture & Documentation ✅ IN PROGRESS
- [x] Write DESIGN.md with concrete tech decisions
- [x] Create docs/claude/ planning directory
- [ ] Write docs/CLAUDE.md (AI context file)
- [ ] Finalize node/edge schema in findings.md

## Phase 1 — Rust Backend Foundation
- [ ] Set up SQLite with rusqlite in Cargo.toml
- [ ] Define Node, Edge, Layer structs with serde
- [ ] Implement graph CRUD Tauri commands
- [ ] Basic spaced repetition (SM-2) logic
- [ ] Unit tests for graph + SR logic

## Phase 2 — Three.js Graph Renderer
- [ ] Add three.js to frontend deps
- [ ] Basic 3D scene: camera, lights, orbit controls
- [ ] Render nodes as spheres, edges as lines
- [ ] Force-directed layout in 3D (basic spring simulation)
- [ ] Node click → info panel

## Phase 3 — Layer System
- [ ] Layer switching UI
- [ ] Cross-layer edge rendering
- [ ] Depth navigation (drill into sub-layer)

## Phase 4 — Adaptive Learning UI
- [ ] Node state visualization (unseen / in-progress / learned / due)
- [ ] Recommendation highlights (glow/pulse for next nodes)
- [ ] Review session flow

## Phase 5 — Data & Import
- [ ] Domain pack format (SQLite DB file)
- [ ] Japanese starter pack (grammar → kanji layers)
- [ ] Anki import/export

## Phase 6 — Polish
- [ ] Themes / CSS override support
- [ ] Performance: LOD for large graphs
- [ ] Settings panel

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
