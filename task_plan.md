# Task Plan — Concept (3D Knowledge Graph)

## Current Core Track — Config-first domain model

- [x] Add DB primitives for config-driven worlds (`worlds`, `relation_kinds`, `node_layers`, `edge_layers`)
- [x] Add backend query surfaces for world config and relation kinds
- [x] Implement domain pack v2 loader (id-based references)
- [x] Remove domain pack v1 support (v2-only fail-fast import)
- [x] Migrate bundled starter domain to v2 format
- [ ] Frontend: consume world config and relation kinds in store layer
- [~] Frontend: layer projection mode (node/edge visibility by membership + filters)
- [x] Split node layers and connection layers at data model level
- [x] Add connection layer API + store wiring
- [x] Add dual sections in Layer panel (node vs connection)
- [x] Add config-driven node/edge visual styles (world/relation/layer metadata)
- [ ] Refine edge highlighting styles for multi-layer overlap (dash, opacity, animated flow)
- [ ] Import UX for user-supplied v2 packs with clear validation error output

## Phase 1 — Rust Backend ✅

- SQLite schema, all CRUD, TauRPC, seed data

## Phase 2 — 3D Graph ✅

- Nodes (geometry by type), edges, raycasting, panels, force layout, labels, post-processing
- **Fix: black screen** — install `postprocessing` peer dep + replace raw Line2 with cientos Line2

## Phase 3 — Search & Navigation (next)

- Fuzzy search bar, keyboard navigation, breadcrumb trail

## Phase 4 — Edit & Create

- Add/edit/delete nodes and edges from the UI

## Phase 5 — Import / Export

- Export domain packs, import user data (v2 only)

## Phase 6 — Polish

- Animations, onboarding, settings panel
