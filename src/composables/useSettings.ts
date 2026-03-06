import { reactive } from 'vue'

export interface Keybindings {
  flyMode:    string
  graphMode:  string
  jumpBack:   string
  search:     string
  settings:   string
  openNode:   string
  editNode:   string
  pinNode:    string
  progressOverlay: string
  worldPicker: string
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

export interface GraphicsSettings {
  qualityPreset: 'low' | 'medium' | 'high' | 'custom'
  bloomEnabled: boolean
  bloomIntensity: number
  bloomThreshold: number
  bloomSmoothing: number
  vignetteEnabled: boolean
  vignetteDarkness: number
  fogDensity: number
  nodeDetail: number
}

export interface LearningSettings {
  defaultSchedulerKey: string
}

const DEFAULT_KEYBINDINGS: Keybindings = {
  flyMode:    'f',
  graphMode:  'g',
  jumpBack:   'q',
  search:     '/',
  settings:   't',
  openNode:   'e',
  editNode:   'x',
  pinNode:    'p',
  progressOverlay: 'n',
  worldPicker: 'o',
  pinnedBuffer: 'b',
  mapBuffer:    'm',
  graphOrbitLeft: 'h',
  graphOrbitRight: 'l',
  graphTiltUp: 'k',
  graphTiltDown: 'j',
  graphZoomIn: 'i',
  graphZoomOut: 'u',
  flyForward: 'w',
  flyBack:    's',
  flyLeft:    'a',
  flyRight:   'd',
  flyUp:      'r',
  flyDown:    'v',
}

const STORAGE_KEY = 'concept:keybindings'
const GRAPHICS_STORAGE_KEY = 'concept:graphics-settings'
const LEARNING_STORAGE_KEY = 'concept:learning-settings'

const GRAPHICS_PRESETS: Record<'low' | 'medium' | 'high', Omit<GraphicsSettings, 'qualityPreset'>> = {
  low: {
    bloomEnabled: false,
    bloomIntensity: 0.35,
    bloomThreshold: 0.42,
    bloomSmoothing: 0.24,
    vignetteEnabled: false,
    vignetteDarkness: 0.2,
    fogDensity: 0.01,
    nodeDetail: 0,
  },
  medium: {
    bloomEnabled: true,
    bloomIntensity: 0.75,
    bloomThreshold: 0.28,
    bloomSmoothing: 0.52,
    vignetteEnabled: true,
    vignetteDarkness: 0.42,
    fogDensity: 0.014,
    nodeDetail: 1,
  },
  high: {
    bloomEnabled: true,
    bloomIntensity: 1.05,
    bloomThreshold: 0.18,
    bloomSmoothing: 0.68,
    vignetteEnabled: true,
    vignetteDarkness: 0.6,
    fogDensity: 0.018,
    nodeDetail: 2,
  },
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, value))
}

function sanitizeGraphics(input: Partial<GraphicsSettings>): GraphicsSettings {
  const preset = input.qualityPreset === 'low' || input.qualityPreset === 'high' || input.qualityPreset === 'custom'
    ? input.qualityPreset
    : 'medium'
  const base = preset !== 'custom' ? GRAPHICS_PRESETS[preset] : GRAPHICS_PRESETS.medium
  return {
    qualityPreset: preset,
    bloomEnabled: input.bloomEnabled ?? base.bloomEnabled,
    bloomIntensity: clamp(input.bloomIntensity ?? base.bloomIntensity, 0, 1.5),
    bloomThreshold: clamp(input.bloomThreshold ?? base.bloomThreshold, 0, 1),
    bloomSmoothing: clamp(input.bloomSmoothing ?? base.bloomSmoothing, 0, 1),
    vignetteEnabled: input.vignetteEnabled ?? base.vignetteEnabled,
    vignetteDarkness: clamp(input.vignetteDarkness ?? base.vignetteDarkness, 0, 1),
    fogDensity: clamp(input.fogDensity ?? base.fogDensity, 0, 0.03),
    nodeDetail: Math.round(clamp(input.nodeDetail ?? base.nodeDetail, 0, 2)),
  }
}

function loadFromStorage(): Keybindings {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (raw) {
      const parsed = { ...DEFAULT_KEYBINDINGS, ...(JSON.parse(raw) as Partial<Keybindings>) }
      // Migration 2026-03:
      // If the user still has untouched old defaults, move them to the new layout:
      // - fly movement: WASD + RF (left hand)
      // - graph camera: HJKL + IU (vim-style right hand cluster)
      const hasLegacyFlyCluster =
        parsed.flyForward === 'i' &&
        parsed.flyBack === 'k' &&
        parsed.flyLeft === 'j' &&
        parsed.flyRight === 'l' &&
        parsed.flyUp === 'u' &&
        parsed.flyDown === 'o'

      const hasLegacyGraphCluster =
        parsed.graphOrbitLeft === 'h' &&
        parsed.graphOrbitRight === 'l' &&
        parsed.graphTiltUp === 'u' &&
        parsed.graphTiltDown === 'o' &&
        parsed.graphZoomIn === 'i' &&
        parsed.graphZoomOut === 'k'

      if (hasLegacyFlyCluster) {
        parsed.flyForward = 'w'
        parsed.flyBack = 's'
        parsed.flyLeft = 'a'
        parsed.flyRight = 'd'
        parsed.flyUp = 'r'
        parsed.flyDown = 'v'
      }

      if (hasLegacyGraphCluster) {
        parsed.graphOrbitLeft = 'h'
        parsed.graphOrbitRight = 'l'
        parsed.graphTiltUp = 'k'
        parsed.graphTiltDown = 'j'
        parsed.graphZoomIn = 'i'
        parsed.graphZoomOut = 'u'
      }

      return parsed
    }
  } catch { /* ignore */ }
  return { ...DEFAULT_KEYBINDINGS }
}

function loadGraphicsFromStorage(): GraphicsSettings {
  try {
    const raw = localStorage.getItem(GRAPHICS_STORAGE_KEY)
    if (raw) return sanitizeGraphics(JSON.parse(raw) as Partial<GraphicsSettings>)
  } catch {
    /* ignore */
  }
  return sanitizeGraphics({ qualityPreset: 'medium', ...GRAPHICS_PRESETS.medium })
}

function loadLearningFromStorage(): LearningSettings {
  try {
    const raw = localStorage.getItem(LEARNING_STORAGE_KEY)
    if (raw) {
      const parsed = JSON.parse(raw) as Partial<LearningSettings>
      return {
        defaultSchedulerKey:
          typeof parsed.defaultSchedulerKey === 'string' && parsed.defaultSchedulerKey.trim().length > 0
            ? parsed.defaultSchedulerKey
            : 'basic-v1',
      }
    }
  } catch {
    /* ignore */
  }
  return { defaultSchedulerKey: 'basic-v1' }
}

// Module-level singleton — shared across all useSettings() calls
const keys = reactive<Keybindings>(loadFromStorage())
const graphics = reactive<GraphicsSettings>(loadGraphicsFromStorage())
const learning = reactive<LearningSettings>(loadLearningFromStorage())

function saveToStorage() {
  localStorage.setItem(STORAGE_KEY, JSON.stringify({ ...keys }))
}

function saveGraphicsToStorage() {
  localStorage.setItem(GRAPHICS_STORAGE_KEY, JSON.stringify({ ...graphics }))
}

function saveLearningToStorage() {
  localStorage.setItem(LEARNING_STORAGE_KEY, JSON.stringify({ ...learning }))
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

  function applyGraphicsPreset(preset: 'low' | 'medium' | 'high') {
    const next = sanitizeGraphics({ qualityPreset: preset, ...GRAPHICS_PRESETS[preset] })
    Object.assign(graphics, next)
    saveGraphicsToStorage()
  }

  function updateGraphics<K extends keyof GraphicsSettings>(key: K, value: GraphicsSettings[K]) {
    const next = sanitizeGraphics({ ...graphics, [key]: value, qualityPreset: 'custom' })
    Object.assign(graphics, next)
    graphics.qualityPreset = 'custom'
    saveGraphicsToStorage()
  }

  function resetGraphicsToDefaults() {
    applyGraphicsPreset('medium')
  }

  function setDefaultSchedulerKey(key: string) {
    learning.defaultSchedulerKey = key
    saveLearningToStorage()
  }

  return {
    keys,
    graphics,
    learning,
    rebind,
    resetToDefaults,
    applyGraphicsPreset,
    updateGraphics,
    resetGraphicsToDefaults,
    setDefaultSchedulerKey,
  }
}
