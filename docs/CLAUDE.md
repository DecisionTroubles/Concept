# Project Context for AI Tools

This file provides context for Claude, Copilot, Cursor, and any other AI assistant working in this codebase. Read this before making any suggestions or changes.

---

## What This Project Is

A desktop application for learning through a navigable 3D graph of knowledge nodes. The user moves through 3D space; each node is a learning item (word, grammar rule, concept, kanji, etc.); nodes are connected by edges that encode **context relationships** — the core organizing idea of the system.

The primary use case is Japanese language learning, but the architecture is deliberately domain-agnostic. Any structured knowledge domain can be represented as a graph map.

See `docs/DESIGN.md` for the full architecture specification, data models, tech stack decisions, and design constraints. See `docs/UI.md` for the visual design system, UI framework choice, and interaction model.

---

## Tech Stack at a Glance

- **Desktop**: Tauri v2 (Rust + WebView2)
- **Frontend**: Vue 3 + TypeScript, Pinia, TailwindCSS v4, Three.js
- **Backend**: Rust (Tauri command handlers)
- **Storage**: SQLite via rusqlite
- **IPC**: Tauri typed invoke/events
- **Package manager**: pnpm
- **Frontend extensibility**: plugin kernel (module slot overrides + theme registry)

---

## Non-Negotiable Architectural Rules

1. **Frontend is a renderer, not a thinker.** Vue components handle display and interaction only. Graph logic, learning algorithms, and data access belong in Rust.
2. **All data lives in SQLite.** Pinia stores are derived/cached views — they never become the source of truth.
3. **All frontend↔backend communication goes through Tauri IPC.** No direct file access from the frontend.
4. **Context edges are first-class.** The `EdgeType` (Context, Prerequisite, Semantic, UserDefined) drives both layout and learning — it is never an afterthought.
5. **Offline-first.** No network calls required for core functionality.

---

## Planning Files

Detailed planning, research findings, and session logs are in `docs/claude/`:

- `docs/claude/task_plan.md` — phases, progress tracking, key decisions
- `docs/claude/findings.md` — research, schema sketches, open questions
- `docs/claude/progress.md` — session log, what was done and what is next

Customization/override guide:
- `docs/OVERRIDES.md` — how to override UI modules, add themes, and register plugins

When working on this project, consult these files for context on decisions already made and questions still open.
