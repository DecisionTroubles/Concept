export interface ThemePreset {
  id: string
  name: string
  description: string
  vars: Record<string, string>
}

export const DEFAULT_THEMES: ThemePreset[] = [
  {
    id: 'midnight-blue',
    name: 'Midnight Blue',
    description: 'Dark, cool, high-contrast default',
    vars: {
      '--app-canvas-bg': '#080b14',
      '--app-overlay-bg': 'rgba(10, 13, 24, 0.96)',
      '--app-overlay-border': 'rgba(255, 255, 255, 0.1)',
      '--app-text-primary': '#e8eaf0',
      '--app-text-secondary': '#7a8099',
      '--app-accent': '#5b8fff',
    },
  },
  {
    id: 'graphite-amber',
    name: 'Graphite Amber',
    description: 'Muted graphite with warm accents',
    vars: {
      '--app-canvas-bg': '#0e1014',
      '--app-overlay-bg': 'rgba(18, 19, 22, 0.96)',
      '--app-overlay-border': 'rgba(255, 210, 120, 0.18)',
      '--app-text-primary': '#f1ede3',
      '--app-text-secondary': '#a79b86',
      '--app-accent': '#f2a341',
    },
  },
  {
    id: 'deep-forest',
    name: 'Deep Forest',
    description: 'Low-glare green-tinted night theme',
    vars: {
      '--app-canvas-bg': '#09110d',
      '--app-overlay-bg': 'rgba(10, 20, 14, 0.95)',
      '--app-overlay-border': 'rgba(95, 215, 165, 0.2)',
      '--app-text-primary': '#dff6eb',
      '--app-text-secondary': '#87a99c',
      '--app-accent': '#52c18f',
    },
  },
]

