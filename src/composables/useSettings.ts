import { reactive } from 'vue'

export interface Keybindings {
  flyMode:    string
  graphMode:  string
  jumpBack:   string
  search:     string
  settings:   string
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
  flyForward: 'w',
  flyBack:    's',
  flyLeft:    'a',
  flyRight:   'd',
  flyUp:      'k',
  flyDown:    'j',
}

const STORAGE_KEY = 'concept:keybindings'

function loadFromStorage(): Keybindings {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (raw) return { ...DEFAULT_KEYBINDINGS, ...(JSON.parse(raw) as Partial<Keybindings>) }
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
    const newKey = rawKey.toLowerCase()
    const oldKey = keys[action]
    // Swap if another action already uses this key
    const allActions = Object.keys(keys) as (keyof Keybindings)[]
    for (const k of allActions) {
      if (k !== action && keys[k] === newKey) {
        keys[k] = oldKey
        break
      }
    }
    keys[action] = newKey
    saveToStorage()
  }

  function resetToDefaults() {
    const defaultKeys = Object.keys(DEFAULT_KEYBINDINGS) as (keyof Keybindings)[]
    for (const k of defaultKeys) keys[k] = DEFAULT_KEYBINDINGS[k]
    saveToStorage()
  }

  return { keys, rebind, resetToDefaults }
}
