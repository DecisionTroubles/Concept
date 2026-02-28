<script setup lang="ts">
import { ref } from 'vue'
import { useEventListener } from '@vueuse/core'

const settings = useSettings()
type ActionKey = keyof typeof settings.keys

const EXCLUDED_KEYS = new Set([
  'Escape', 'Shift', 'Control', 'Alt', 'Meta',
  'CapsLock', 'NumLock', 'ScrollLock', 'Tab',
  'ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight',
  'Home', 'End', 'PageUp', 'PageDown', 'Insert', 'Delete',
  'F1','F2','F3','F4','F5','F6','F7','F8','F9','F10','F11','F12',
])

const GLOBAL_ACTIONS: Array<{ key: ActionKey; label: string }> = [
  { key: 'flyMode',   label: 'Enter fly mode' },
  { key: 'graphMode', label: 'Enter graph mode' },
  { key: 'jumpBack',  label: 'Jump back' },
  { key: 'search',    label: 'Search nodes' },
  { key: 'settings',  label: 'Settings panel' },
]

const FLY_ACTIONS: Array<{ key: ActionKey; label: string }> = [
  { key: 'flyForward', label: 'Forward' },
  { key: 'flyBack',    label: 'Back' },
  { key: 'flyLeft',    label: 'Strafe left' },
  { key: 'flyRight',   label: 'Strafe right' },
  { key: 'flyUp',      label: 'Ascend' },
  { key: 'flyDown',    label: 'Descend' },
]

const isOpen = ref(false)
const listeningAction = ref<ActionKey | null>(null)

function toggle() {
  if (listeningAction.value) return
  isOpen.value = !isOpen.value
}

function startListening(action: ActionKey) {
  listeningAction.value = action
}

function displayKey(key: string): string {
  return key.length === 1 ? key.toUpperCase() : key
}

useEventListener(document, 'keydown', (e: KeyboardEvent) => {
  const tag = (e.target as HTMLElement)?.tagName
  const isInput = tag === 'INPUT' || tag === 'TEXTAREA'
                || (e.target as HTMLElement)?.isContentEditable

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
}, { capture: true })
</script>

<template>
  <!-- Gear icon (always visible) -->
  <button class="gear-btn" :class="{ active: isOpen }" @click="toggle" title="Settings (T)">
    <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
      <circle cx="12" cy="12" r="3"/>
      <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"/>
    </svg>
  </button>

  <!-- Settings panel -->
  <Transition name="settings">
    <div v-if="isOpen" class="settings-panel">
      <!-- Header -->
      <div class="panel-header">
        <span class="panel-title">Keybindings</span>
        <button class="reset-btn" @click="settings.resetToDefaults()">Reset defaults</button>
      </div>

      <!-- Global section -->
      <div class="section">
        <div class="section-title">Global</div>
        <div v-for="item in GLOBAL_ACTIONS" :key="item.key" class="keybind-row">
          <span class="keybind-label">{{ item.label }}</span>
          <button
            class="key-badge"
            :class="{ listening: listeningAction === item.key }"
            @click="startListening(item.key)"
          >
            {{ listeningAction === item.key ? 'press key…' : displayKey(settings.keys[item.key]) }}
          </button>
        </div>
      </div>

      <div class="section-divider" />

      <!-- Fly mode section -->
      <div class="section">
        <div class="section-title">Fly mode</div>
        <div v-for="item in FLY_ACTIONS" :key="item.key" class="keybind-row">
          <span class="keybind-label">{{ item.label }}</span>
          <button
            class="key-badge"
            :class="{ listening: listeningAction === item.key }"
            @click="startListening(item.key)"
          >
            {{ listeningAction === item.key ? 'press key…' : displayKey(settings.keys[item.key]) }}
          </button>
        </div>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.gear-btn {
  position: fixed;
  top: 14px;
  right: 14px;
  z-index: 250;
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 8px;
  background: rgba(10, 13, 24, 0.85);
  border: 1px solid rgba(255, 255, 255, 0.08);
  color: #7a8099;
  cursor: pointer;
  transition: background 0.15s, color 0.15s, border-color 0.15s;
  backdrop-filter: blur(8px);
}

.gear-btn:hover,
.gear-btn.active {
  background: rgba(91, 143, 255, 0.15);
  border-color: rgba(91, 143, 255, 0.3);
  color: #5b8fff;
}

.settings-panel {
  position: fixed;
  top: 54px;
  right: 14px;
  width: 280px;
  z-index: 240;
  background: rgba(10, 13, 24, 0.96);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 12px;
  box-shadow: 0 12px 40px rgba(0, 0, 0, 0.6);
  overflow: hidden;
  backdrop-filter: blur(12px);
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 14px 10px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
}

.panel-title {
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: #7a8099;
  font-family: system-ui, sans-serif;
}

.reset-btn {
  font-size: 10px;
  color: #5b8fff;
  background: none;
  border: none;
  cursor: pointer;
  padding: 2px 6px;
  border-radius: 4px;
  font-family: system-ui, sans-serif;
  transition: background 0.1s;
}

.reset-btn:hover {
  background: rgba(91, 143, 255, 0.12);
}

.section {
  padding: 8px 6px;
}

.section-title {
  font-size: 9px;
  font-weight: 700;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  color: #4a5068;
  font-family: system-ui, sans-serif;
  padding: 4px 8px 6px;
}

.section-divider {
  height: 1px;
  background: rgba(255, 255, 255, 0.06);
  margin: 0 6px;
}

.keybind-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 5px 8px;
  border-radius: 6px;
  gap: 8px;
}

.keybind-row:hover {
  background: rgba(255, 255, 255, 0.03);
}

.keybind-label {
  font-size: 12px;
  color: #c8cad8;
  font-family: system-ui, sans-serif;
  flex: 1;
  min-width: 0;
}

.key-badge {
  font-size: 10px;
  font-family: ui-monospace, 'Cascadia Code', monospace;
  font-weight: 600;
  padding: 2px 8px;
  border-radius: 5px;
  border: 1px solid rgba(255, 255, 255, 0.15);
  background: rgba(255, 255, 255, 0.06);
  color: #e8eaf0;
  cursor: pointer;
  min-width: 28px;
  text-align: center;
  transition: background 0.1s, border-color 0.1s, color 0.1s;
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

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.65; }
}

/* Transition */
.settings-enter-active,
.settings-leave-active {
  transition: opacity 0.15s ease, transform 0.15s ease;
}
.settings-enter-from,
.settings-leave-to {
  opacity: 0;
  transform: translateY(-6px);
}
</style>
