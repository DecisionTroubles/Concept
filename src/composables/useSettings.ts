import { reactive } from 'vue'

export interface Keybindings {
  flyMode:    string
  graphMode:  string
  jumpBack:   string
  search:     string
  settings:   string
  openNode:   string
  pinNode:    string
  pinnedBuffer: string
  mapBuffer:    string
  graphOrbitLeft: string
  graphOrbitRight: string
  graphTiltUp: string
  graphTiltDown: string
  graphZoomIn: string
  graphZoomOut: string
  flyForward: string
  flyBack:    string
  flyLeft:    string
  flyRight:   string
  flyUp:      string
  flyDown:    string
}

const DEFAULT_KEYBINDINGS: Keybindings = {
  flyMode:    'f',
  graphMode:  'g',
  jumpBack:   'q',
  search:     '/',
  settings:   't',
  openNode:   'e',
  pinNode:    'p',
  pinnedBuffer: 'b',
  mapBuffer:    'm',
  graphOrbitLeft: 'h',
  graphOrbitRight: 'l',
  graphTiltUp: 'u',
  graphTiltDown: 'o',
  graphZoomIn: 'i',
  graphZoomOut: 'k',
  flyForward: 'i',
  flyBack:    'k',
  flyLeft:    'j',
  flyRight:   'l',
  flyUp:      'u',
  flyDown:    'o',
}

const STORAGE_KEY = 'concept:keybindings'

function loadFromStorage(): Keybindings {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (raw) {
      const parsed = { ...DEFAULT_KEYBINDINGS, ...(JSON.parse(raw) as Partial<Keybindings>) }
      // Migrate old default fly cluster (WASD + R/V) to the new UIOP/HJKL-style layout
      // only when all six old defaults are still untouched.
      const isOldFlyDefault =
        parsed.flyForward === 'w' &&
        parsed.flyBack === 's' &&
        parsed.flyLeft === 'a' &&
        parsed.flyRight === 'd' &&
        parsed.flyUp === 'r' &&
        parsed.flyDown === 'v'

      if (isOldFlyDefault) {
        parsed.flyForward = 'i'
        parsed.flyBack = 'k'
        parsed.flyLeft = 'j'
        parsed.flyRight = 'l'
        parsed.flyUp = 'u'
        parsed.flyDown = 'o'
      }

      return parsed
    }
  } catch { /* ignore */ }
  return { ...DEFAULT_KEYBINDINGS }
}

// Module-level singleton — shared across all useSettings() calls
const keys = reactive<Keybindings>(loadFromStorage())

function saveToStorage() {
  localStorage.setItem(STORAGE_KEY, JSON.stringify({ ...keys }))
}

export function useSettings() {
  function rebind(action: keyof Keybindings, rawKey: string) {
    keys[action] = rawKey.toLowerCase()
    saveToStorage()
  }

  function resetToDefaults() {
    const defaultKeys = Object.keys(DEFAULT_KEYBINDINGS) as (keyof Keybindings)[]
    for (const k of defaultKeys) keys[k] = DEFAULT_KEYBINDINGS[k]
    saveToStorage()
  }

  return { keys, rebind, resetToDefaults }
}
