import type { AppPlugin } from '@/core/plugin'
// import CustomGraphScene from '@/overrides/CustomGraphScene.vue'

export const exampleOverridePlugin: AppPlugin = {
  id: 'example.override',
  name: 'Example Override',
  modules: {
    // GraphScene: CustomGraphScene,
  },
  themes: [
    {
      id: 'sunset-terminal',
      name: 'Sunset Terminal',
      description: 'Warm terminal-like palette with high readability',
      vars: {
        '--app-canvas-bg': '#151012',
        '--app-overlay-bg': 'rgba(24, 16, 18, 0.95)',
        '--app-overlay-border': 'rgba(250, 140, 84, 0.28)',
        '--app-text-primary': '#f8e3d4',
        '--app-text-secondary': '#b89988',
        '--app-accent': '#fa8c54',
      },
    },
  ],
}

