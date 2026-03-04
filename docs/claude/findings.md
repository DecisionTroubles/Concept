# Findings — 3D Memory Graph Platform

## Project Structure (as of 2026-03-04)
- Tauri v2 + Vue 3 + TypeScript desktop app with implemented Rust backend + SQLite
- pnpm package manager, Vite v7, TailwindCSS v4, Pinia
- 3D stack installed and integrated: `three`, `@tresjs/core`, `@tresjs/cientos`, `@tresjs/post-processing`
- Force-layout in frontend via `d3-force-3d`
- Domain-pack seed pipeline active (`domains/japanese/pack.json`)
- UI overlays implemented: search, settings, node detail, layer panel, compass, buffers
- Frontend plugin kernel implemented (module overrides + theme registry)

## Key Source Files
| File | Purpose |
|------|---------|
| `src/App.vue` | Frontend root composed from kernel module slots |
| `src/stores/graph.ts` | Graph data/state orchestration |
| `src-tauri/src/lib.rs` | Rust command handlers + app bootstrap |
| `src-tauri/src/graph.rs` | Graph CRUD/query logic |
| `src-tauri/src/domain.rs` | Domain-pack import/seed logic |
| `src/core/kernel.ts` | Frontend plugin kernel |
| `src/composables/useTheme.ts` | Runtime theme registry/state application |
| `src/plugins/userPlugins.ts` | User override registration point |
| `docs/DESIGN.md` | Authoritative architecture spec |
| `docs/CLAUDE.md` | AI context file |
| `docs/OVERRIDES.md` | Override/plugin/theme guide |

## Architecture Notes
- Strict frontend/backend separation enforced by design: all graph logic in Rust
- Tauri IPC is the only communication channel (typed invoke calls)
- SQLite is the single source of truth; Pinia stores are derived/cached views
- Context edges are first-class citizens in the schema (not just metadata)

## Node Schema (decided)
```rust
struct Node {
    id: String,
    title: String,
    content: NodeContent,          // text | audio | image | video
    connections: Vec<EdgeRef>,
    layer: String,
    tags: Vec<String>,
    learned: bool,
    last_reviewed: Option<DateTime<Utc>>,
    next_review: Option<DateTime<Utc>>,
    weight: f32,
    metadata: HashMap<String, String>,
}

struct EdgeRef {
    target_id: String,
    edge_type: EdgeType,           // Context | Prerequisite | Semantic | UserDefined
    weight: f32,
}
```

## 3D Library Decision: TresJS
- **TresJS** chosen over raw Three.js and Babylon.js
- Vue 3 component model — Pinia reactive state flows directly into 3D scene
- Built on Three.js under the hood — full escape hatch via `useThree()`
- Packages installed:
  - `@tresjs/core 5.5.0` — Vue Three.js wrapper
  - `@tresjs/cientos 5.4.0` — OrbitControls, helpers
  - `@tresjs/post-processing 3.4.0` — bloom, vignette, DoF, FXAA
  - `three 0.183.1` + `@types/three`
  - `d3-force-3d 3.0.6` + `@types/d3-force` — force-directed layout

## shadcn-vue Setup
- Initialized with `zinc` base color, Tailwind v4 compatible (CSS vars in oklch)
- Dark mode: pinned permanently via `class="dark"` on `<html>`
- `src/lib/utils.ts` — `cn()` helper (clsx + tailwind-merge)
- Components installed: `button`, `badge`, `separator`, `sheet`, `tooltip`, `scroll-area`
- Lives under `src/components/ui/`
- `@custom-variant dark` in CSS — dark mode via `.dark` class (not media query)

## Three.js Graph Approach
- Force-directed layout: nodes repel, edges attract (spring simulation)
- Run layout in a Web Worker or use backend-computed positions
- Consider: `three-forcegraph` library as a starting point vs. custom implementation
- LOD (Level of Detail): for >500 nodes, use instanced meshes and reduce detail at distance

## SQLite Schema Sketch
```sql
CREATE TABLE nodes (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    layer TEXT NOT NULL,
    content_type TEXT,
    content_data TEXT,
    learned INTEGER DEFAULT 0,
    last_reviewed TEXT,
    next_review TEXT,
    weight REAL DEFAULT 1.0,
    metadata TEXT  -- JSON blob
);

CREATE TABLE edges (
    id TEXT PRIMARY KEY,
    source_id TEXT NOT NULL REFERENCES nodes(id),
    target_id TEXT NOT NULL REFERENCES nodes(id),
    edge_type TEXT NOT NULL,  -- Context | Prerequisite | Semantic | UserDefined
    weight REAL DEFAULT 1.0
);

CREATE TABLE layers (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    map_id TEXT NOT NULL,
    z_order INTEGER DEFAULT 0
);
```

## UI Framework Decision
- **shadcn-vue** chosen over PrimeVue. Rationale in docs/UI.md.
- Node shapes encode type (vocab=sphere, grammar=octahedron, kanji=box, particle=torus, concept=icosahedron)
- Node states: unseen (dim grey) / learned (emerald #3dd68c) / reachable-next (blue pulse on edges)
- Floating glassmorphism panels over full-viewport 3D canvas
- Three.js EffectComposer: bloom + vignette + DoF + FXAA

## Learning Model (simplified for now)
- No spaced repetition yet — user manually marks learned
- Marking learned: node turns green, connected edges pulse blue to show next reachable nodes
- SM-2 spaced repetition is planned for a future phase

## IPC Layer: TauRPC
- **Library**: [TauRPC](https://github.com/MatsDK/TauRPC) — trait-based, fully-typed IPC layer for Tauri
- Replaces manual `#[tauri::command]` + hand-written TypeScript types
- Uses [Specta](https://github.com/oscartbeaumont/specta) to auto-generate TypeScript bindings at app startup
- Trait defined once in Rust → TypeScript proxy generated automatically — no type drift

### Rust side
```toml
# Cargo.toml
taurpc = "0.7.1"
specta = { version = "=2.0.0-rc.22", features = ["derive"] }
tokio = { version = "1", features = ["full"] }
```
```rust
#[taurpc::procedures(export_to = "../src/bindings.ts")]
trait Api {
    async fn get_nodes() -> Vec<Node>;
}

#[taurpc::resolvers]
impl Api for ApiImpl {
    async fn get_nodes(self) -> Vec<Node> { ... }
}

// in builder:
.invoke_handler(taurpc::create_ipc_handler(ApiImpl.into_handler()))
```

### TypeScript side
```bash
pnpm install taurpc
```
```typescript
import { createTauRPCProxy } from 'taurpc'
import type { Procedures } from './bindings'

const taurpc = await createTauRPCProxy<Procedures>()
const nodes = await taurpc.get_nodes()
```

### Router pattern (for namespaced APIs)
```rust
let router = Router::new()
    .merge(GraphApiImpl.into_handler())
    .merge(LearningApiImpl.into_handler());
.invoke_handler(taurpc::create_ipc_handler(router))
```

### Struct sharing
```rust
#[taurpc::ipc_type]  // derives Serialize, Deserialize, Type, Clone
struct Node { ... }
```

## Frontend Extensibility Notes
- Module overrides are slot-based via kernel (`ModuleSlot`), not direct imports in app root
- Themes are plugin-provided presets (`ThemePreset`) and are user-selectable in Settings
- Default plugin + user plugin chain provides deterministic override order
- `docs/OVERRIDES.md` documents the expected customization workflow

## Open Questions
- [ ] Force-directed layout ownership: keep in frontend or move to Rust/worker for larger graphs?
- [ ] Quick-map UX: label strategy + viewport interactions for very dense worlds
- [ ] Graph map packaging strategy: one SQLite file per map vs multi-map DB
- [ ] Japanese starter data expansion source strategy (JMdict/Tatoeba/custom curation)
