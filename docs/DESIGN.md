# Project: 3D Memory Graph Platform

## Overview

A cross-domain, offline-first desktop application for learning via a navigable 3D graph of connected knowledge nodes. Each node represents a learning item (word, grammar rule, concept, kanji, etc.). Nodes are connected by **context relationships** — the primary organizing principle of the entire system.

Designed initially for Japanese language learning but architecturally domain-agnostic. Any structured knowledge domain can be loaded as a graph map.

---

## Core Concept

The central idea that distinguishes this project:

- **Context-first edges**: nodes are connected not by arbitrary tags but by how they naturally co-occur, depend on each other, or belong to the same usage context.
- **Layered graphs**: a map has multiple layers. For Japanese: grammar layer (top) → vocabulary layer → kanji layer. Each layer is its own graph; nodes in one layer can link down to nodes in another.
- **3D navigation**: the user moves through 3D space; proximity and clustering encode meaning. Related nodes are spatially close.
- **Adaptive learning**: the graph guides which nodes to visit next based on the user's progress and spaced repetition state.

---

## Architecture

### Layer Overview

```
┌─────────────────────────────────────────┐
│  Frontend (Vue 3 + TypeScript)          │
│  - 3D graph rendering (Three.js)        │
│  - UI panels, overlays, menus           │
│  - Node interaction: click, hover, drag │
│  - Pinia stores for local UI state      │
└──────────────┬──────────────────────────┘
               │ Tauri IPC (invoke / events)
┌──────────────▼──────────────────────────┐
│  Backend (Rust / Tauri commands)        │
│  - Graph data management                │
│  - Adaptive learning logic              │
│  - Spaced repetition scheduling         │
│  - Node placement & clustering          │
│  - SQLite persistence (via rusqlite)    │
└──────────────┬──────────────────────────┘
               │
┌──────────────▼──────────────────────────┐
│  Storage (SQLite)                       │
│  - Nodes, edges, metadata               │
│  - User progress, review schedule       │
│  - Card content (text, audio, images)   │
└─────────────────────────────────────────┘
```

### Principle: strict separation of concerns
- Frontend owns **rendering and interaction only** — no graph logic, no learning algorithms.
- Backend (Rust) owns **all data, all logic** — graph structure, adaptive scheduling, persistence.
- All frontend↔backend communication goes through **Tauri IPC** (typed commands and events).

---

## Tech Stack

| Layer         | Technology                         | Notes                                          |
|---------------|------------------------------------|------------------------------------------------|
| Desktop shell | Tauri v2                           | Windows / macOS / Linux                        |
| Frontend      | Vue 3 + TypeScript                 | Composition API, `<script setup>`              |
| State         | Pinia                              | UI-layer state only                            |
| Styling       | TailwindCSS v4                     | SCSS override supported for themes             |
| 3D rendering  | TresJS + Three.js                  | Vue-native Three.js; Pinia state → reactive 3D |
| Backend       | Rust                               | Tauri command handlers                         |
| Storage       | SQLite via rusqlite                | Offline-first; Anki-compatible schema possible |
| IPC           | Tauri invoke / events              | Typed via serde_json / TypeScript interfaces   |
| Package mgr   | pnpm                               |                                                |
| Build tool    | Vite v7                            |                                                |
| Testing       | Vitest (frontend), cargo test (Rust)|                                               |

---

## Node Data Model

```rust
struct Node {
    id: String,
    title: String,
    content: NodeContent,          // text | audio | image | video
    connections: Vec<EdgeRef>,     // typed edges to other nodes
    layer: String,                 // e.g. "grammar", "kanji", "vocab"
    tags: Vec<String>,
    learned: bool,                 // user-toggled; drives node color + edge highlights
    weight: f32,                   // importance score — affects node size in 3D
    metadata: HashMap<String, String>,
}

struct EdgeRef {
    target_id: String,
    edge_type: EdgeType,           // Context | Prerequisite | Semantic | UserDefined
    weight: f32,
}
```

---

## Graph & Layers

- Each **map** has one or more named layers.
- Within a layer, nodes form their own 3D graph.
- Cross-layer edges connect nodes between layers (e.g., grammar node → kanji nodes it uses).
- Layout: force-directed in 3D; nodes with stronger connections cluster spatially.
- Frontend renders only the active layer by default; cross-layer edges shown as visual cues.

---

## Learning Model (current scope)

- User manually marks a node as **learned** via the detail panel.
- Marking learned updates the node's color/state immediately (optimistic UI update + Rust backend persist).
- When a node is marked learned, connected edges highlight to indicate reachable next nodes.
- No automated scheduling in the current phase. Spaced repetition (SM-2) is a planned future feature.

---

## Extensibility

- Domain packs: a graph map is a self-contained SQLite DB that can be distributed/imported.
- Anki import/export: nodes map to Anki cards; edges are metadata.
- Plugin API (future): custom layout algorithms, domain-specific edge logic.
- Optional cloud sync (future): export/import graph snapshots as JSON.

---

## UI & Visual Design

See `docs/UI.md` for full UI specification including:
- Framework choice (shadcn-vue over PrimeVue — rationale documented)
- Node geometry language (shape encodes type: sphere=vocab, octahedron=grammar, etc.)
- Node state colors (unseen / learned / reachable-next)
- Floating panel layout and glassmorphism style
- Three.js post-processing (bloom, vignette, DoF)
- Camera navigation and interaction model

---

## Key Design Constraints

1. **Offline-first**: all data local; no network required.
2. **Frontend is a dumb renderer**: business logic never leaks into Vue components.
3. **Context edges are first-class**: not an afterthought — the edge type drives layout and learning.
4. **Scalable**: must handle thousands of nodes without degrading 3D rendering.
5. **Single source of truth**: SQLite is the source; Pinia stores are derived views.
