# Progress

## 2026-03-05 — Overlay stacking fix + config-core migration start

- Fixed root stacking issue by raising modal backdrop layers (not child panel z-index)
- Added centralized z-layer tokens in `src/assets/main.css` and wired node/settings/buffer overlays to those tokens
- Tuned buffer backdrop to match settings transparency so 3D world remains visible behind buffer
- Implemented domain pack v2 backend foundation (world config, relation kinds, node/edge layer memberships)
- Removed v1 domain pack support entirely; loader is now v2-only with fail-fast validation
- Migrated bundled `domains/japanese/pack.json` to v2 schema

## 2026-03-05 — Dual layer model wiring (continued)

- Added `connection_layers` and `edge_connection_layers` support to backend schema and migrations
- Added backend API for `get_connection_layers` and exposed connection-layer IDs on edge refs
- Updated domain pack loader to seed connection layers + edge connection memberships
- Updated layer panel to show independent "Node Layers" and "Connection Layers"
- Added client-side edge filtering by active connection layers

## 2026-03-05 — Configurable visual styles

- Added config-driven visual styling for nodes and edges (world defaults, node type styles, relation styles, layer styles)
- Wired GraphScene to resolve node color/emissive/edge width+color from pack config instead of hardcoded constants
- Added visual cues in layer panel (node swatch + connection line sample from metadata styles)

## 2026-02-27 — Fix black screen (Phase 2 completion)

- Installed `postprocessing` v6.38.3 (missing peer dep for `@tresjs/post-processing` Pmndrs effects)
- Replaced `three/examples/jsm` Line2/LineGeometry/LineMaterial with `<Line2>` from `@tresjs/cientos`
- Type-check passes clean (`pnpm vue-tsc --noEmit`)
- Phase 2 fully complete — nodes, edges, panels, bloom, vignette, fog, force layout all wired up

## Earlier

- Phase 1: Rust/SQLite/TauRPC backend ✅
- Phase 2: 3D scene scaffolding ✅
