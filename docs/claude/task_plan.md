# Task Plan - 3D Memory Graph Platform

## Status
Implementation phase. Core graph, learning foundation, plugin system, buffers, node viewer, and authoring foundation are implemented.

---

## Completed

### Core App
- [x] Tauri + Vue + Rust + SQLite foundation
- [x] typed IPC bindings
- [x] persistent graph/world schema
- [x] reset/reseed flow

### Graph Runtime
- [x] 3D graph rendering
- [x] force layout
- [x] node selection/focus
- [x] layer panel
- [x] connection layer filtering
- [x] search
- [x] pinned buffer
- [x] map buffer

### Node System
- [x] side node summary
- [x] centered page-based node viewer
- [x] note types with schema + layout
- [x] structured node fields
- [x] authored built-in pages
- [x] authored extension pages

### Learning
- [x] node progress model
- [x] review events
- [x] scheduler abstraction
- [x] progress overlay
- [x] node learning controls

### Extensibility
- [x] frontend plugin kernel
- [x] core plugin vs user plugin split
- [x] node extension registry
- [x] persisted node extension data
- [x] real `Node Notes`
- [x] real `Node Assets`

### Authoring Foundation
- [x] note type authoring tab
- [x] note type duplication
- [x] selected-node structured content editing
- [x] pack-defined note type import support

---

## Next Major Work

### Pack / Authoring
- [ ] pack export flow
- [ ] pack validation diagnostics
- [ ] richer note type layout authoring
- [ ] live preview during authoring
- [ ] richer field widgets

### Learning Product
- [ ] dedicated review session flow
- [ ] due queue / answer / reveal / grade loop
- [ ] analytics by cluster / note type / neighborhood

### Media / Assets
- [ ] file-backed asset storage
- [ ] previews / playback
- [ ] media placement within authored pages

### Runtime / Performance
- [ ] large-graph performance pass
- [ ] chunk/code splitting
- [ ] further camera/navigation refinement
- [ ] edge readability improvements

---

## Current Decisions
- Domain packs are JSON-based v2 packs.
- Note types are reusable templates with schema + layout, not only field lists.
- Centered node viewing is page-based, not a settings-like tab inspector.
- Scheduler logic is separated and replaceable.
- User plugins load after core plugins from a dedicated drop-in folder.
