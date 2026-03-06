<script setup lang="ts">
import { ref } from 'vue'
import { useEventListener } from '@vueuse/core'
import { useTheme } from '@/composables/useTheme'
import AuthoringPanel from '@/components/authoring/AuthoringPanel.vue'

const settings = useSettings()
const themeState = useTheme()
const graphStore = useGraphStore()
type ActionKey = keyof typeof settings.keys
type GraphicsPreset = 'low' | 'medium' | 'high'

const EXCLUDED_KEYS = new Set([
  'Escape',
  'Shift',
  'Control',
  'Alt',
  'Meta',
  'CapsLock',
  'NumLock',
  'ScrollLock',
  'Tab',
  'ArrowUp',
  'ArrowDown',
  'ArrowLeft',
  'ArrowRight',
  'Home',
  'End',
  'PageUp',
  'PageDown',
  'Insert',
  'Delete',
  'F1',
  'F2',
  'F3',
  'F4',
  'F5',
  'F6',
  'F7',
  'F8',
  'F9',
  'F10',
  'F11',
  'F12',
])

const GLOBAL_ACTIONS: Array<{ key: ActionKey; label: string }> = [
  { key: 'flyMode', label: 'Enter fly mode' },
  { key: 'graphMode', label: 'Enter graph mode' },
  { key: 'jumpBack', label: 'Jump back' },
  { key: 'search', label: 'Search nodes' },
  { key: 'settings', label: 'Settings panel' },
  { key: 'openNode', label: 'Center node panel' },
  { key: 'pinNode', label: 'Toggle pin node' },
  { key: 'progressOverlay', label: 'Progress overview' },
  { key: 'worldPicker', label: 'World picker' },
  { key: 'pinnedBuffer', label: 'Pinned buffer' },
  { key: 'mapBuffer', label: 'Map buffer' },
]

const FLY_ACTIONS: Array<{ key: ActionKey; label: string }> = [
  { key: 'flyForward', label: 'Forward' },
  { key: 'flyBack', label: 'Back' },
  { key: 'flyLeft', label: 'Strafe left' },
  { key: 'flyRight', label: 'Strafe right' },
  { key: 'flyUp', label: 'Ascend' },
  { key: 'flyDown', label: 'Descend' },
]

const GRAPH_ACTIONS: Array<{ key: ActionKey; label: string }> = [
  { key: 'graphOrbitLeft', label: 'Orbit left' },
  { key: 'graphOrbitRight', label: 'Orbit right' },
  { key: 'graphTiltUp', label: 'Tilt up' },
  { key: 'graphTiltDown', label: 'Tilt down' },
  { key: 'graphZoomIn', label: 'Zoom in' },
  { key: 'graphZoomOut', label: 'Zoom out' },
]

const isOpen = ref(false)
const listeningAction = ref<ActionKey | null>(null)
const activeTab = ref<'hotkeys' | 'themes' | 'graphics' | 'learning' | 'worlds' | 'authoring'>('hotkeys')

function toggle() {
  if (listeningAction.value) return
  isOpen.value = !isOpen.value
}

function close() {
  if (listeningAction.value) return
  isOpen.value = false
}

async function resetGraphNow() {
  await graphStore.resetGraphData()
}

async function selectWorldNow(worldId: string) {
  await graphStore.selectWorld(worldId)
}

async function reloadActiveWorldNow() {
  await graphStore.reloadActiveWorld()
}

function startListening(action: ActionKey) {
  listeningAction.value = action
}

function displayKey(key: string): string {
  return key.length === 1 ? key.toUpperCase() : key
}

function actionAliases(action: ActionKey): string[] {
  if (action === 'search') {
    const flyKey = displayKey(settings.keys.flyMode)
    if (flyKey.length === 1) return [`${flyKey}${flyKey}`]
  }
  return []
}

function setGraphicsPreset(preset: GraphicsPreset) {
  settings.applyGraphicsPreset(preset)
}

function updateGraphicNumber(
  key: 'bloomIntensity' | 'bloomThreshold' | 'bloomSmoothing' | 'vignetteDarkness' | 'fogDensity' | 'nodeDetail',
  event: Event
) {
  settings.updateGraphics(key, Number((event.target as HTMLInputElement).value))
}

function updateGraphicBoolean(key: 'bloomEnabled' | 'vignetteEnabled', event: Event) {
  settings.updateGraphics(key, (event.target as HTMLInputElement).checked)
}

function formatGraphicNumber(value: number, digits: number = 2): string {
  return value.toFixed(digits)
}

function updateDefaultScheduler(event: Event) {
  settings.setDefaultSchedulerKey((event.target as HTMLSelectElement).value)
}

useEventListener(
  document,
  'keydown',
  (e: KeyboardEvent) => {
    const tag = (e.target as HTMLElement)?.tagName
    const isInput = tag === 'INPUT' || tag === 'TEXTAREA' || (e.target as HTMLElement)?.isContentEditable

    // Open via keyboard when panel is closed
    if (!isOpen.value && !isInput && e.key.toLowerCase() === settings.keys.settings) {
      e.preventDefault()
      e.stopImmediatePropagation()
      isOpen.value = true
      return
    }

    if (!isOpen.value) return

    // Panel is open — block all other listeners from seeing this event
    e.stopImmediatePropagation()

    // Capturing a new key binding
    if (listeningAction.value) {
      e.preventDefault()
      if (e.key === 'Escape') {
        listeningAction.value = null
        return
      }
      if (!EXCLUDED_KEYS.has(e.key)) {
        settings.rebind(listeningAction.value, e.key)
        listeningAction.value = null
      }
      return
    }

    // Panel open, not listening — handle close triggers
    if (e.key === 'Escape' || e.key.toLowerCase() === settings.keys.settings) {
      e.preventDefault()
      isOpen.value = false
    }
  },
  { capture: true }
)
</script>

<template>
  <!-- Gear icon (always visible) -->
  <button
    class="gear-btn"
    :class="{ active: isOpen }"
    @click="toggle"
    :title="`Settings (${displayKey(settings.keys.settings)})`"
  >
    <svg
      width="15"
      height="15"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="2"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <circle cx="12" cy="12" r="3" />
      <path
        d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"
      />
    </svg>
  </button>

  <!-- Settings panel -->
  <Teleport to="body">
    <Transition name="settings">
      <div v-if="isOpen" class="settings-backdrop" @click.self="close">
        <div class="settings-panel">
          <div class="panel-header">
            <div class="title-wrap">
              <span class="panel-title">Settings</span>
              <span class="panel-subtitle">Configure controls and interaction defaults</span>
            </div>
            <div class="header-actions">
              <button class="reset-btn" @click="settings.resetToDefaults()">Reset defaults</button>
              <button class="reset-btn" @click="resetGraphNow">Reset graph data</button>
              <button class="close-btn" @click="close" aria-label="Close settings">Esc</button>
            </div>
          </div>

          <div class="tabs">
            <button class="tab-btn" :class="{ active: activeTab === 'hotkeys' }" @click="activeTab = 'hotkeys'">
              Hotkeys
            </button>
            <button class="tab-btn" :class="{ active: activeTab === 'themes' }" @click="activeTab = 'themes'">
              Themes
            </button>
            <button class="tab-btn" :class="{ active: activeTab === 'graphics' }" @click="activeTab = 'graphics'">
              Graphics
            </button>
            <button class="tab-btn" :class="{ active: activeTab === 'learning' }" @click="activeTab = 'learning'">
              Learning
            </button>
            <button class="tab-btn" :class="{ active: activeTab === 'worlds' }" @click="activeTab = 'worlds'">
              Worlds
            </button>
            <button class="tab-btn" :class="{ active: activeTab === 'authoring' }" @click="activeTab = 'authoring'">
              Authoring
            </button>
          </div>

          <div class="tab-content" v-if="activeTab === 'hotkeys'">
            <div class="hotkey-grid">
              <section class="hotkey-card">
                <div class="section-title">Global</div>
                <div v-for="item in GLOBAL_ACTIONS" :key="item.key" class="keybind-row">
                  <span class="keybind-label">{{ item.label }}</span>
                  <div class="keybind-right">
                    <button
                      class="key-badge"
                      :class="{ listening: listeningAction === item.key }"
                      @click="startListening(item.key)"
                    >
                      {{ listeningAction === item.key ? 'press key…' : displayKey(settings.keys[item.key]) }}
                    </button>
                    <span v-for="alias in actionAliases(item.key)" :key="`${item.key}-${alias}`" class="key-alias">
                      {{ alias }}
                    </span>
                  </div>
                </div>
              </section>

              <section class="hotkey-card">
                <div class="section-title">Fly Mode</div>
                <div v-for="item in FLY_ACTIONS" :key="item.key" class="keybind-row">
                  <span class="keybind-label">{{ item.label }}</span>
                  <div class="keybind-right">
                    <button
                      class="key-badge"
                      :class="{ listening: listeningAction === item.key }"
                      @click="startListening(item.key)"
                    >
                      {{ listeningAction === item.key ? 'press key…' : displayKey(settings.keys[item.key]) }}
                    </button>
                    <span v-for="alias in actionAliases(item.key)" :key="`${item.key}-${alias}`" class="key-alias">
                      {{ alias }}
                    </span>
                  </div>
                </div>
              </section>

              <section class="hotkey-card">
                <div class="section-title">Graph Mode</div>
                <div v-for="item in GRAPH_ACTIONS" :key="item.key" class="keybind-row">
                  <span class="keybind-label">{{ item.label }}</span>
                  <div class="keybind-right">
                    <button
                      class="key-badge"
                      :class="{ listening: listeningAction === item.key }"
                      @click="startListening(item.key)"
                    >
                      {{ listeningAction === item.key ? 'press key…' : displayKey(settings.keys[item.key]) }}
                    </button>
                    <span v-for="alias in actionAliases(item.key)" :key="`${item.key}-${alias}`" class="key-alias">
                      {{ alias }}
                    </span>
                  </div>
                </div>
              </section>
            </div>
          </div>

          <div class="tab-content" v-else-if="activeTab === 'themes'">
            <div class="theme-list">
              <button
                v-for="preset in themeState.themes.value"
                :key="preset.id"
                class="theme-card"
                :class="{ active: themeState.activeThemeId.value === preset.id }"
                @click="themeState.setTheme(preset.id)"
              >
                <div class="theme-row">
                  <span class="theme-name">{{ preset.name }}</span>
                  <span class="theme-active" v-if="themeState.activeThemeId.value === preset.id">Active</span>
                </div>
                <p class="theme-desc">{{ preset.description }}</p>
                <div class="theme-swatches">
                  <span class="swatch" :style="{ background: preset.vars['--app-canvas-bg'] }" />
                  <span class="swatch" :style="{ background: preset.vars['--app-overlay-bg'] }" />
                  <span class="swatch" :style="{ background: preset.vars['--app-accent'] }" />
                  <span class="swatch" :style="{ background: preset.vars['--app-text-primary'] }" />
                </div>
              </button>
            </div>
          </div>

          <div class="tab-content" v-else-if="activeTab === 'graphics'">
            <div class="graphics-layout">
              <section class="graphics-card">
                <div class="section-title">Quality</div>
                <div class="preset-row">
                  <button
                    v-for="preset in (['low', 'medium', 'high'] as GraphicsPreset[])"
                    :key="preset"
                    class="preset-btn"
                    :class="{ active: settings.graphics.qualityPreset === preset }"
                    @click="setGraphicsPreset(preset)"
                  >
                    {{ preset }}
                  </button>
                </div>
                <div class="graphics-copy">
                  Current preset: <strong>{{ settings.graphics.qualityPreset }}</strong>
                </div>
              </section>

              <section class="graphics-card">
                <div class="section-title">Post Processing</div>
                <label class="toggle-row">
                  <span>Bloom</span>
                  <input
                    type="checkbox"
                    :checked="settings.graphics.bloomEnabled"
                    @change="updateGraphicBoolean('bloomEnabled', $event)"
                  />
                </label>
                <label class="slider-row">
                  <span>Bloom intensity</span>
                  <input
                    type="range"
                    min="0"
                    max="1.5"
                    step="0.01"
                    :value="settings.graphics.bloomIntensity"
                    @input="updateGraphicNumber('bloomIntensity', $event)"
                  />
                  <strong>{{ formatGraphicNumber(settings.graphics.bloomIntensity) }}</strong>
                </label>
                <label class="slider-row">
                  <span>Bloom threshold</span>
                  <input
                    type="range"
                    min="0"
                    max="1"
                    step="0.01"
                    :value="settings.graphics.bloomThreshold"
                    @input="updateGraphicNumber('bloomThreshold', $event)"
                  />
                  <strong>{{ formatGraphicNumber(settings.graphics.bloomThreshold) }}</strong>
                </label>
                <label class="slider-row">
                  <span>Bloom smoothing</span>
                  <input
                    type="range"
                    min="0"
                    max="1"
                    step="0.01"
                    :value="settings.graphics.bloomSmoothing"
                    @input="updateGraphicNumber('bloomSmoothing', $event)"
                  />
                  <strong>{{ formatGraphicNumber(settings.graphics.bloomSmoothing) }}</strong>
                </label>
                <label class="toggle-row">
                  <span>Vignette</span>
                  <input
                    type="checkbox"
                    :checked="settings.graphics.vignetteEnabled"
                    @change="updateGraphicBoolean('vignetteEnabled', $event)"
                  />
                </label>
                <label class="slider-row">
                  <span>Vignette darkness</span>
                  <input
                    type="range"
                    min="0"
                    max="1"
                    step="0.01"
                    :value="settings.graphics.vignetteDarkness"
                    @input="updateGraphicNumber('vignetteDarkness', $event)"
                  />
                  <strong>{{ formatGraphicNumber(settings.graphics.vignetteDarkness) }}</strong>
                </label>
              </section>

              <section class="graphics-card">
                <div class="section-title">Scene</div>
                <label class="slider-row">
                  <span>Fog density</span>
                  <input
                    type="range"
                    min="0"
                    max="0.03"
                    step="0.001"
                    :value="settings.graphics.fogDensity"
                    @input="updateGraphicNumber('fogDensity', $event)"
                  />
                  <strong>{{ formatGraphicNumber(settings.graphics.fogDensity, 3) }}</strong>
                </label>
                <label class="slider-row">
                  <span>Node detail</span>
                  <input
                    type="range"
                    min="0"
                    max="2"
                    step="1"
                    :value="settings.graphics.nodeDetail"
                    @input="updateGraphicNumber('nodeDetail', $event)"
                  />
                  <strong>{{ settings.graphics.nodeDetail }}</strong>
                </label>
                <div class="graphics-copy">
                  Lower detail reduces mesh complexity and helps on weaker GPUs.
                </div>
                <div class="graphics-actions">
                  <button class="reset-btn" @click="settings.resetGraphicsToDefaults()">Reset graphics</button>
                </div>
              </section>
            </div>
          </div>

          <div class="tab-content" v-else-if="activeTab === 'learning'">
            <div class="learning-layout">
              <section class="graphics-card">
                <div class="section-title">Scheduler</div>
                <label class="learning-field">
                  <span>Default review scheduler</span>
                  <select :value="settings.learning.defaultSchedulerKey" @change="updateDefaultScheduler">
                    <option v-for="scheduler in graphStore.schedulerAlgorithms" :key="scheduler.key" :value="scheduler.key">
                      {{ scheduler.name }}
                    </option>
                  </select>
                </label>
                <div class="graphics-copy">
                  Review buttons in the progress window and node panel use this scheduler by default.
                </div>
              </section>

              <section class="graphics-card">
                <div class="section-title">Available Algorithms</div>
                <article v-for="scheduler in graphStore.schedulerAlgorithms" :key="scheduler.key" class="scheduler-card">
                  <div class="scheduler-head">
                    <strong>{{ scheduler.name }}</strong>
                    <span class="scheduler-key">{{ scheduler.key }}</span>
                  </div>
                  <p>{{ scheduler.description }}</p>
                </article>
              </section>
            </div>
          </div>

          <div class="tab-content" v-else-if="activeTab === 'worlds'">
            <div class="learning-layout">
              <section class="graphics-card">
                <div class="section-title">Active World</div>
                <div class="world-summary">
                  <strong>{{ graphStore.worldConfig?.name ?? 'No world loaded' }}</strong>
                  <span>{{ graphStore.worldConfig?.id ?? 'No active world in database' }}</span>
                </div>
                <div class="graphics-copy">
                  Bundled packs are scanned from `domains/*/pack.json`. User packs can be added to the app data `worlds/` folder.
                </div>
                <div class="graphics-actions">
                  <button class="reset-btn" @click="reloadActiveWorldNow">Reload active world</button>
                </div>
              </section>

              <section class="graphics-card">
                <div class="section-title">Available Worlds</div>
                <div class="world-list">
                  <article v-for="world in graphStore.worldPacks" :key="world.pack_path" class="world-card" :class="{ active: world.is_active }">
                    <div class="world-head">
                      <div class="world-meta">
                        <strong>{{ world.world_name ?? 'Invalid pack' }}</strong>
                        <span>{{ world.world_id ?? world.pack_path }}</span>
                      </div>
                      <div class="world-badges">
                        <span class="scheduler-key">{{ world.source_kind }}</span>
                        <span v-if="world.is_loaded" class="theme-active">Loaded</span>
                        <span v-else-if="world.is_active" class="theme-active">Selected</span>
                      </div>
                    </div>
                    <p class="world-path">{{ world.pack_path }}</p>
                    <p v-if="world.error" class="world-error">{{ world.error }}</p>
                    <div class="world-actions">
                      <button class="reset-btn" :disabled="!world.valid || world.is_active || !world.world_id" @click="world.world_id && selectWorldNow(world.world_id)">
                        {{ world.is_active ? 'Current world' : 'Open world' }}
                      </button>
                    </div>
                  </article>
                </div>
              </section>
            </div>
          </div>

          <div class="tab-content" v-else>
            <AuthoringPanel />
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.gear-btn {
  position: fixed;
  top: 14px;
  right: 14px;
  z-index: var(--z-settings-gear);
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 8px;
  background: color-mix(in srgb, var(--app-overlay-bg) 88%, transparent);
  border: 1px solid var(--app-overlay-border);
  color: var(--app-text-secondary);
  cursor: pointer;
  transition:
    background 0.15s,
    color 0.15s,
    border-color 0.15s;
  backdrop-filter: blur(8px);
}

.gear-btn:hover,
.gear-btn.active {
  background: color-mix(in srgb, var(--app-accent) 15%, transparent);
  border-color: color-mix(in srgb, var(--app-accent) 35%, transparent);
  color: var(--app-accent);
}

.settings-backdrop {
  position: fixed;
  inset: 0;
  z-index: var(--z-settings-modal);
  backdrop-filter: blur(6px);
  background: rgba(0, 0, 0, 0.42);
  display: flex;
  justify-content: center;
  padding-top: 11vh;
}

.settings-panel {
  width: min(1040px, calc(100vw - 36px));
  max-height: min(760px, 78vh);
  display: flex;
  flex-direction: column;
  background: var(--app-overlay-bg);
  border: 1px solid var(--app-overlay-border);
  border-radius: 14px;
  box-shadow: 0 12px 40px rgba(0, 0, 0, 0.6);
  overflow: hidden;
  backdrop-filter: blur(12px);
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 14px;
  padding: 16px 18px 12px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
}

.title-wrap {
  display: flex;
  flex-direction: column;
  gap: 5px;
}

.panel-title {
  font-size: 13px;
  font-weight: 700;
  letter-spacing: 0.04em;
  color: var(--app-text-primary);
  font-family: system-ui, sans-serif;
}

.panel-subtitle {
  font-size: 12px;
  color: var(--app-text-secondary);
  font-family: system-ui, sans-serif;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.reset-btn {
  font-size: 11px;
  color: var(--app-accent);
  background: color-mix(in srgb, var(--app-accent) 13%, transparent);
  border: 1px solid color-mix(in srgb, var(--app-accent) 25%, transparent);
  cursor: pointer;
  padding: 6px 10px;
  border-radius: 7px;
  font-family: system-ui, sans-serif;
  transition:
    background 0.12s,
    border-color 0.12s;
}

.reset-btn:hover {
  background: color-mix(in srgb, var(--app-accent) 20%, transparent);
  border-color: color-mix(in srgb, var(--app-accent) 45%, transparent);
}

.close-btn {
  font-size: 11px;
  color: #7a8099;
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.12);
  cursor: pointer;
  padding: 6px 10px;
  border-radius: 7px;
  font-family: ui-monospace, 'Cascadia Code', monospace;
  transition:
    background 0.12s,
    color 0.12s;
}

.close-btn:hover {
  background: rgba(255, 255, 255, 0.1);
  color: #c8cad8;
}

.tabs {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 10px 18px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
}

.tab-btn {
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.06em;
  text-transform: uppercase;
  color: #7a8099;
  background: transparent;
  border: 1px solid transparent;
  border-radius: 8px;
  padding: 6px 10px;
  cursor: pointer;
  font-family: system-ui, sans-serif;
  transition:
    color 0.12s,
    background 0.12s,
    border-color 0.12s;
}

.tab-btn:hover {
  color: #c8cad8;
  background: rgba(255, 255, 255, 0.04);
}

.tab-btn.active {
  color: var(--app-accent);
  background: color-mix(in srgb, var(--app-accent) 14%, transparent);
  border-color: color-mix(in srgb, var(--app-accent) 30%, transparent);
}

.tab-content {
  overflow: auto;
  padding: 16px 18px 18px;
}

.theme-list {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 10px;
}

.graphics-layout {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 12px;
}

.learning-layout {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12px;
}

.graphics-card {
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 12px;
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.preset-row {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.preset-btn {
  border: 1px solid rgba(255, 255, 255, 0.12);
  background: rgba(255, 255, 255, 0.04);
  color: #c8cad8;
  border-radius: 999px;
  padding: 6px 12px;
  font-size: 11px;
  font-weight: 700;
  text-transform: uppercase;
  cursor: pointer;
}

.preset-btn.active {
  color: var(--app-accent);
  border-color: color-mix(in srgb, var(--app-accent) 42%, transparent);
  background: color-mix(in srgb, var(--app-accent) 14%, transparent);
}

.slider-row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1.2fr) auto;
  align-items: center;
  gap: 12px;
}

.toggle-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.slider-row span,
.toggle-row span,
.graphics-copy {
  font-size: 12px;
  color: #c8cad8;
}

.slider-row input[type='range'] {
  width: 100%;
  min-width: 0;
  margin: 0;
}

.slider-row strong {
  font-size: 11px;
  color: var(--app-accent);
  min-width: 40px;
  text-align: right;
}

.graphics-actions {
  padding-top: 4px;
}

.learning-field {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.learning-field span {
  font-size: 12px;
  color: #c8cad8;
}

.scheduler-card {
  border-radius: 10px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  background: rgba(255, 255, 255, 0.03);
  padding: 12px;
}

.scheduler-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.scheduler-head strong {
  font-size: 13px;
  color: var(--app-text-primary);
}

.scheduler-key {
  font-size: 10px;
  color: var(--app-accent);
  border-radius: 999px;
  padding: 3px 8px;
  background: color-mix(in srgb, var(--app-accent) 12%, transparent);
  border: 1px solid color-mix(in srgb, var(--app-accent) 26%, transparent);
}

.scheduler-card p {
  margin: 8px 0 0;
  font-size: 12px;
  color: var(--app-text-secondary);
}

.world-summary {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.world-summary strong {
  font-size: 14px;
  color: var(--app-text-primary);
}

.world-summary span {
  font-size: 12px;
  color: var(--app-text-secondary);
}

.world-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.world-card {
  border-radius: 10px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  background: rgba(255, 255, 255, 0.03);
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.world-card.active {
  border-color: color-mix(in srgb, var(--app-accent) 40%, transparent);
  background: color-mix(in srgb, var(--app-accent) 8%, transparent);
}

.world-head {
  display: flex;
  justify-content: space-between;
  gap: 10px;
}

.world-meta {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.world-meta strong {
  font-size: 13px;
  color: var(--app-text-primary);
}

.world-meta span,
.world-path {
  font-size: 12px;
  color: var(--app-text-secondary);
  word-break: break-all;
}

.world-badges,
.world-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.world-error {
  margin: 0;
  font-size: 12px;
  color: #f39a8f;
}

.theme-card {
  text-align: left;
  border: 1px solid rgba(255, 255, 255, 0.1);
  background: rgba(255, 255, 255, 0.03);
  border-radius: 12px;
  padding: 12px;
  cursor: pointer;
  transition:
    border-color 0.12s,
    background 0.12s,
    transform 0.12s;
}

.theme-card:hover {
  border-color: color-mix(in srgb, var(--app-accent) 35%, transparent);
  background: color-mix(in srgb, var(--app-accent) 10%, transparent);
}

.theme-card.active {
  border-color: color-mix(in srgb, var(--app-accent) 50%, transparent);
  background: color-mix(in srgb, var(--app-accent) 14%, transparent);
}

.theme-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.theme-name {
  font-size: 13px;
  font-weight: 700;
  color: var(--app-text-primary);
}

.theme-active {
  font-size: 10px;
  color: var(--app-accent);
  border: 1px solid color-mix(in srgb, var(--app-accent) 40%, transparent);
  background: color-mix(in srgb, var(--app-accent) 14%, transparent);
  border-radius: 999px;
  padding: 2px 7px;
}

.theme-desc {
  margin: 6px 0 0;
  font-size: 12px;
  color: #8f96ae;
  line-height: 1.35;
}

.theme-swatches {
  display: flex;
  gap: 6px;
  margin-top: 10px;
}

.swatch {
  width: 22px;
  height: 12px;
  border-radius: 999px;
  border: 1px solid rgba(255, 255, 255, 0.14);
}

.hotkey-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 12px;
}

.hotkey-card {
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 12px;
  padding: 10px;
}

.section-title {
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  color: #7a8099;
  font-family: system-ui, sans-serif;
  padding: 3px 8px 8px;
}

.keybind-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 7px 8px;
  border-radius: 6px;
  gap: 8px;
}

.keybind-row:hover {
  background: rgba(255, 255, 255, 0.03);
}

.keybind-label {
  font-size: 13px;
  color: #c8cad8;
  font-family: system-ui, sans-serif;
  flex: 1;
  min-width: 0;
}

.keybind-right {
  display: inline-flex;
  align-items: center;
  gap: 6px;
}

.key-badge {
  font-size: 11px;
  font-family: ui-monospace, 'Cascadia Code', monospace;
  font-weight: 600;
  padding: 3px 9px;
  border-radius: 5px;
  border: 1px solid rgba(255, 255, 255, 0.15);
  background: rgba(255, 255, 255, 0.06);
  color: #e8eaf0;
  cursor: pointer;
  min-width: 28px;
  text-align: center;
  transition:
    background 0.1s,
    border-color 0.1s,
    color 0.1s;
  white-space: nowrap;
  flex-shrink: 0;
}

.key-badge:hover {
  background: rgba(91, 143, 255, 0.15);
  border-color: rgba(91, 143, 255, 0.35);
  color: #5b8fff;
}

.key-badge.listening {
  background: rgba(245, 158, 11, 0.15);
  border-color: rgba(245, 158, 11, 0.4);
  color: #f59e0b;
  animation: pulse 1s ease-in-out infinite;
}

.key-alias {
  font-size: 10px;
  font-family: ui-monospace, 'Cascadia Code', monospace;
  font-weight: 600;
  color: #7a8099;
  border: 1px solid rgba(255, 255, 255, 0.12);
  background: rgba(255, 255, 255, 0.04);
  padding: 2px 7px;
  border-radius: 5px;
}

@keyframes pulse {
  0%,
  100% {
    opacity: 1;
  }
  50% {
    opacity: 0.65;
  }
}

@media (max-width: 1060px) {
  .settings-backdrop {
    padding-top: 8vh;
  }

  .hotkey-grid {
    grid-template-columns: 1fr 1fr;
  }

  .graphics-layout {
    grid-template-columns: 1fr 1fr;
  }

  .learning-layout {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 760px) {
  .settings-backdrop {
    padding-top: 0;
    align-items: stretch;
  }

  .settings-panel {
    width: 100vw;
    max-height: 100vh;
    border-radius: 0;
    border-left: none;
    border-right: none;
  }

  .panel-header {
    flex-direction: column;
    align-items: flex-start;
  }

  .header-actions {
    width: 100%;
    justify-content: flex-end;
  }

  .hotkey-grid {
    grid-template-columns: 1fr;
  }

  .theme-list {
    grid-template-columns: 1fr;
  }

  .graphics-layout {
    grid-template-columns: 1fr;
  }

  .learning-layout {
    grid-template-columns: 1fr;
  }

  .slider-row {
    grid-template-columns: 1fr;
    align-items: stretch;
  }

  .slider-row strong {
    text-align: left;
  }
}

/* Transition */
.settings-enter-active,
.settings-leave-active {
  transition: opacity 0.16s ease;
}
.settings-enter-from,
.settings-leave-to {
  opacity: 0;
}
.settings-enter-active .settings-panel,
.settings-leave-active .settings-panel {
  transition:
    transform 0.16s ease,
    opacity 0.16s ease;
}
.settings-enter-from .settings-panel,
.settings-leave-to .settings-panel {
  transform: translateY(-10px);
  opacity: 0;
}
</style>
