# Override Guide — Modules, Themes, Plugins

This project now includes a lightweight frontend plugin kernel so customizations
can be done without editing core files directly.

Core files:
- `src/core/kernel.ts` — plugin/module/theme registry
- `src/core/modules.ts` — overridable module slots
- `src/core/themes.ts` — built-in theme presets
- `src/plugins/defaultPlugin.ts` — default module wiring
- `src/plugins/userPlugins.ts` — where custom plugins are registered
- `src/plugins/example.override.ts` — working example plugin

---

## 1) Override a UI module

Available slots (`ModuleSlot`) include:
- `GraphScene`
- `LayerPanel`
- `NodeDetailPanel`
- `ModeIndicator`
- `SettingsPanel`
- `NodeSearch`
- `CompassHUD`
- `BufferOverlay`

Create a plugin:

```ts
// src/plugins/my.plugin.ts
import type { AppPlugin } from '@/core/plugin'
import MyBufferOverlay from '@/overrides/MyBufferOverlay.vue'

export const myPlugin: AppPlugin = {
  id: 'my.plugin',
  name: 'My Overrides',
  modules: {
    BufferOverlay: MyBufferOverlay,
  },
}
```

Register it:

```ts
// src/plugins/userPlugins.ts
import { myPlugin } from '@/plugins/my.plugin'

export const userPlugins = [
  myPlugin,
]
```

Order matters: later plugins override earlier plugins for the same slot.

---

## 2) Add a custom theme preset

```ts
// src/plugins/my.theme.plugin.ts
import type { AppPlugin } from '@/core/plugin'

export const myThemePlugin: AppPlugin = {
  id: 'my.theme.plugin',
  name: 'Custom Themes',
  themes: [
    {
      id: 'neon-slate',
      name: 'Neon Slate',
      description: 'Cool slate with cyan accents',
      vars: {
        '--app-canvas-bg': '#0a0f18',
        '--app-overlay-bg': 'rgba(12, 18, 30, 0.95)',
        '--app-overlay-border': 'rgba(90, 230, 255, 0.24)',
        '--app-text-primary': '#e7f6ff',
        '--app-text-secondary': '#85a8b8',
        '--app-accent': '#5ae6ff',
      },
    },
  ],
}
```

Once registered in `userPlugins`, the theme appears in Settings → Themes.

---

## 3) Make custom components theme-aware

Use CSS vars instead of hardcoded values:
- `var(--app-canvas-bg)`
- `var(--app-overlay-bg)`
- `var(--app-overlay-border)`
- `var(--app-text-primary)`
- `var(--app-text-secondary)`
- `var(--app-accent)`

Example:

```css
.panel {
  background: var(--app-overlay-bg);
  border: 1px solid var(--app-overlay-border);
  color: var(--app-text-primary);
}
```

---

## 4) Recommended workflow for your own changes

1. Build new component/theme in `src/overrides/` or `src/plugins/`.
2. Register a plugin in `src/plugins/userPlugins.ts`.
3. Keep core files untouched when possible.
4. Document behavior differences in `docs/claude/progress.md`.

This keeps upgrades/refactors low-risk while preserving custom behavior.

