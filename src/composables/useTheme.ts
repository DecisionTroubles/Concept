import { computed, ref } from 'vue'
import { appKernel } from '@/core/kernel'
import type { ThemePreset } from '@/core/themes'

const STORAGE_KEY = 'concept:theme'

const availableThemes = ref<ThemePreset[]>([])
const activeThemeId = ref<string>('')
const appliedVarNames = new Set<string>()

function applyThemeVars(theme: ThemePreset) {
  const root = document.documentElement

  for (const varName of appliedVarNames) {
    if (!(varName in theme.vars)) root.style.removeProperty(varName)
  }

  for (const [name, value] of Object.entries(theme.vars)) {
    root.style.setProperty(name, value)
    appliedVarNames.add(name)
  }
}

function syncThemesFromKernel() {
  const themes = appKernel.listThemes()
  availableThemes.value = themes
  if (!activeThemeId.value && themes[0]) activeThemeId.value = themes[0].id
}

function initializeTheme() {
  if (!availableThemes.value.length) syncThemesFromKernel()
  if (!availableThemes.value.length) return

  const stored = localStorage.getItem(STORAGE_KEY)
  const fallback = availableThemes.value[0]!.id
  const selected = stored && appKernel.hasTheme(stored) ? stored : fallback
  setTheme(selected)
}

function setTheme(id: string) {
  const theme = availableThemes.value.find((t) => t.id === id)
  if (!theme) return
  activeThemeId.value = id
  applyThemeVars(theme)
  localStorage.setItem(STORAGE_KEY, id)
}

const activeTheme = computed(() =>
  availableThemes.value.find((t) => t.id === activeThemeId.value) ?? null,
)

const canvasColor = computed(() => activeTheme.value?.vars['--app-canvas-bg'] ?? '#080b14')

export function useTheme() {
  return {
    themes: availableThemes,
    activeThemeId,
    activeTheme,
    canvasColor,
    syncThemesFromKernel,
    initializeTheme,
    setTheme,
  }
}

