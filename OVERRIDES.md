# Overrides, Plugins, and Dataset Guide

This project supports customization at three levels:

1. `user-plugins/` for frontend extensions and overrides
2. note types + node content for structured node behavior
3. domain packs for loading your own 3D world/dataset

Core plugins load first. User plugins load after them.

---

## 1. Add your own plugin

Drop a file into `user-plugins/`:

```ts
import { definePlugin } from '@/core/plugin'

export default definePlugin({
  id: 'user.example',
  name: 'User Example',
})
```

Supported exports:

```ts
export default definePlugin({...})
export const plugin = definePlugin({...})
export const plugins = [definePlugin({...})]
```

User plugins are auto-loaded from:

- `user-plugins/**/*.ts`
- `user-plugins/**/*.js`
- `user-plugins/**/*.mjs`

---

## 2. Override a frontend module

Available module slots are defined in `src/core/modules.ts`.

Typical slots include:
- `GraphScene`
- `LayerPanel`
- `NodeDetailPanel`
- `ProgressOverlay`
- `SettingsPanel`
- `NodeSearch`
- `CompassHUD`
- `BufferOverlay`

Example:

```ts
import { definePlugin } from '@/core/plugin'
import MyBufferOverlay from '@/overrides/MyBufferOverlay.vue'

export default definePlugin({
  id: 'user.buffer-override',
  name: 'Buffer Override',
  modules: {
    BufferOverlay: MyBufferOverlay,
  },
})
```

Later plugins override earlier ones for the same slot.

---

## 3. Add a node extension page

Node extension pages are registered through `nodeWorkspaceExtensions`.

Example:

```ts
import { definePlugin } from '@/core/plugin'
import MyNodeTool from '@/components/MyNodeTool.vue'

export default definePlugin({
  id: 'user.node-tool',
  name: 'Node Tool',
  nodeWorkspaceExtensions: [
    {
      id: 'user.node-tool.panel',
      title: 'Node Tool',
      description: 'Custom page for nodes',
      slot: 'extensions.primary',
      order: 100,
      component: MyNodeTool,
    },
  ],
})
```

If a note type layout includes that extension id as an extension page, it becomes its own page in the centered node viewer.

---

## 4. Add a custom theme

```ts
import { definePlugin } from '@/core/plugin'

export default definePlugin({
  id: 'user.theme',
  name: 'Custom Theme',
  themes: [
    {
      id: 'ember-grid',
      name: 'Ember Grid',
      description: 'Warm dark theme with orange emphasis',
      vars: {
        '--app-canvas-bg': '#120d0c',
        '--app-overlay-bg': 'rgba(24, 16, 14, 0.96)',
        '--app-overlay-border': 'rgba(255, 164, 107, 0.22)',
        '--app-text-primary': '#f4e5dd',
        '--app-text-secondary': '#b59a90',
        '--app-accent': '#ffa46b',
      },
    },
  ],
})
```

Theme-aware UI should use app variables:
- `--app-canvas-bg`
- `--app-overlay-bg`
- `--app-overlay-border`
- `--app-text-primary`
- `--app-text-secondary`
- `--app-accent`

---

## 5. Update nodes in the app

Current in-app editing flow:

1. select a node in the 3D world
2. open `Settings`
3. go to `Authoring`
4. use `Selected Node Content`

You can edit:
- title
- tags
- note type
- fallback content
- structured note fields from the current note type schema

Then press `Save node`.

---

## 6. Create or update note types

In `Settings -> Authoring` you can:
- create a note type
- duplicate a note type
- edit field definitions
- edit page definitions
- assign built-in pages:
  - `connections`
  - `learning`
  - `history`
- assign extension pages by extension id

This is the current authoring foundation. It is functional, but still basic.

---

## 7. Load your own 3D world

The app seeds from a domain pack JSON file.

Current bundled example:
- `domains/japanese/pack.json`

To load your own world today:

1. create a new pack JSON using the `domain-pack-v2` format
2. include:
   - `world`
   - `note_types`
   - `relation_kinds`
   - `layers`
   - `connection_layers`
   - `nodes`
   - `edges`
3. place it in `domains/<your-world>/pack.json`
4. update the seed path in `src-tauri/src/graph.rs` if you want it bundled by default
5. reset/reseed the app data

Important:
- pack files must be UTF-8 **without BOM**
- node viewer pages come from note type `layout_json`
- node structured content comes from `note_fields`

See:
- `docs/domain-pack-v2.md`
- `domains/japanese/pack.json`

---

## 8. Recommended workflow

For safe customization:

1. keep core files as defaults
2. add your UI/theme/node extensions in `user-plugins/`
3. define world content through domain packs
4. use note types + note fields instead of raw `content_data` whenever possible
5. document major behavior changes in `docs/claude/`
