<script setup lang="ts">
import { ref, computed, nextTick, watch } from 'vue'
import { useEventListener } from '@vueuse/core'

const graphStore = useGraphStore()
const editorMode = useEditorMode()
const settings = useSettings()

const isOpen = ref(false)
const insertMode = ref(false)   // false = NORMAL, true = INSERT
const query = ref('')
const activeIndex = ref(0)
const inputRef = ref<HTMLInputElement | null>(null)

let lastFTime = 0

function open() {
  isOpen.value = true
  insertMode.value = false   // start in NORMAL mode, no autofocus
  query.value = ''
  activeIndex.value = 0
}

function close() {
  isOpen.value = false
}

function enterInsert() {
  insertMode.value = true
  nextTick(() => inputRef.value?.focus())
}

function selectResult(id: string) {
  graphStore.requestFocus(id)
  close()
}

const results = computed(() => {
  const q = query.value.toLowerCase().trim()
  if (!q) return graphStore.nodes.slice(0, 20)
  return graphStore.nodes
    .filter(n =>
      n.title.toLowerCase().includes(q) ||
      n.tags.some(t => t.toLowerCase().includes(q))
    )
    .slice(0, 20)
})

watch(query, () => { activeIndex.value = 0 })

useEventListener(document, 'keydown', (e: KeyboardEvent) => {
  const tag = (e.target as HTMLElement)?.tagName
  const isInput = tag === 'INPUT' || tag === 'TEXTAREA'
               || (e.target as HTMLElement)?.isContentEditable

  // flyMode double-tap to open search (capture phase, fires before GraphScene)
  if (!isOpen.value && e.key.toLowerCase() === settings.keys.flyMode && !isInput) {
    const now = Date.now()
    if (now - lastFTime < 350) {
      // Second f within 350ms → open search
      e.preventDefault()
      e.stopImmediatePropagation()
      lastFTime = 0
      // If first f triggered fly mode, exit it
      if (editorMode.mode.value === 'fly') editorMode.enterNormal()
      open()
      return
    }
    lastFTime = now
    // First f falls through → GraphScene enters fly mode (intended)
    return
  }

  // search key when not in input and modal not open
  if (!isOpen.value && !isInput && e.key === settings.keys.search) {
    e.preventDefault()
    open()
    return
  }

  if (!isOpen.value) return

  // Modal is open — handle all keys

  // Esc: INSERT → NORMAL, NORMAL → close
  if (e.key === 'Escape') {
    e.preventDefault()
    e.stopImmediatePropagation()
    if (insertMode.value) {
      insertMode.value = false
      inputRef.value?.blur()
    } else {
      close()
    }
    return
  }

  // Ctrl bindings — work in INSERT and NORMAL while modal is open
  if (e.ctrlKey) {
    if (e.key === 'm' || e.key === 'M') {          // Ctrl+M = Enter
      e.preventDefault()
      const r = results.value[activeIndex.value]
      if (r) selectResult(r.id)
      return
    }
    if (e.key === 'h' || e.key === 'H') {          // Ctrl+H = Backspace
      e.preventDefault()
      if (insertMode.value && query.value.length > 0)
        query.value = query.value.slice(0, -1)
      return
    }
    if (e.key === 'w' || e.key === 'W') {          // Ctrl+W = delete word back
      e.preventDefault()
      if (insertMode.value) query.value = query.value.replace(/\S+\s*$/, '')
      return
    }
    if (e.key === 'u' || e.key === 'U') {          // Ctrl+U = clear line
      e.preventDefault()
      if (insertMode.value) query.value = ''
      return
    }
  }

  // INSERT mode: let keystrokes reach the input naturally
  if (insertMode.value) return

  // NORMAL mode navigation
  if (e.key === 'j' || e.key === 'ArrowDown') {
    e.preventDefault()
    activeIndex.value = Math.min(activeIndex.value + 1, results.value.length - 1)
    return
  }
  if (e.key === 'k' || e.key === 'ArrowUp') {
    e.preventDefault()
    activeIndex.value = Math.max(activeIndex.value - 1, 0)
    return
  }
  if (e.key === 'i' || e.key === 'a') {
    e.preventDefault()
    enterInsert()
    return
  }
  if (e.key === settings.keys.search) {
    e.preventDefault()
    query.value = ''
    enterInsert()
    return
  }
  if (e.key === 'Enter') {
    e.preventDefault()
    const r = results.value[activeIndex.value]
    if (r) selectResult(r.id)
    return
  }
}, { capture: true })

function nodeTypeBadgeStyle(nodeType: string): { background: string; color: string } {
  switch (nodeType) {
    case 'grammar':  return { background: 'rgba(91,143,255,0.18)', color: '#5b8fff' }
    case 'kanji':    return { background: 'rgba(245,158,11,0.18)', color: '#f59e0b' }
    case 'vocab':    return { background: 'rgba(52,211,153,0.18)', color: '#34d399' }
    case 'concept':  return { background: 'rgba(167,139,250,0.18)', color: '#a78bfa' }
    case 'particle': return { background: 'rgba(251,113,133,0.18)', color: '#fb7185' }
    default:         return { background: 'rgba(120,130,170,0.18)', color: '#8090b0' }
  }
}
</script>

<template>
  <Teleport to="body">
    <Transition name="modal">
      <div v-if="isOpen" class="modal-backdrop" @click.self="close">
        <div class="modal-panel">
          <!-- Search input -->
          <div class="search-bar">
            <svg class="search-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="11" cy="11" r="8"/><path d="m21 21-4.35-4.35"/>
            </svg>
            <input
              ref="inputRef"
              v-model="query"
              class="search-input"
              placeholder="Search nodes..."
              autocomplete="off"
              spellcheck="false"
            />
            <span class="vim-mode-badge" :class="insertMode ? 'insert' : 'normal'">
              {{ insertMode ? 'INSERT' : 'NORMAL' }}
            </span>
            <span class="search-kbd">Esc</span>
          </div>

          <div class="divider" />

          <!-- Results -->
          <div class="results-list">
            <div
              v-if="results.length === 0"
              class="no-results"
            >No nodes found</div>
            <div
              v-for="(node, i) in results"
              :key="node.id"
              :class="['result-row', { active: i === activeIndex }]"
              @click="selectResult(node.id)"
              @mouseenter="activeIndex = i"
            >
              <div class="result-left">
                <span class="result-title">{{ node.title }}</span>
                <span v-if="node.tags[0]" class="result-tag">{{ node.tags[0] }}</span>
              </div>
              <div class="result-right">
                <span class="result-type-badge" :style="nodeTypeBadgeStyle(node.node_type)">
                  {{ node.node_type }}
                </span>
                <span class="result-edges">{{ node.connections.length }} edges</span>
              </div>
            </div>
          </div>

          <div class="divider" />

          <!-- Footer -->
          <div class="modal-footer">
            <template v-if="!insertMode">
              <span>i=insert</span>
              <span>·</span>
              <span>j/k navigate</span>
              <span>·</span>
              <span>Enter select</span>
              <span>·</span>
              <span>Esc close</span>
            </template>
            <template v-else>
              <span>Esc=normal</span>
              <span>·</span>
              <span>Enter select</span>
            </template>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.modal-backdrop {
  position: fixed;
  inset: 0;
  z-index: 300;
  backdrop-filter: blur(6px);
  background: rgba(0, 0, 0, 0.4);
  display: flex;
  justify-content: center;
  padding-top: 15vh;
}

.modal-panel {
  background: var(--app-overlay-bg);
  border: 1px solid var(--app-overlay-border);
  border-radius: 14px;
  width: 100%;
  max-width: 600px;
  max-height: 70vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.7);
  align-self: flex-start;
}

.search-bar {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 14px 18px;
}

.search-icon {
  color: #7a8099;
  flex-shrink: 0;
}

.search-input {
  flex: 1;
  background: none;
  border: none;
  outline: none;
  color: var(--app-text-primary);
  font-size: 15px;
  font-family: system-ui, sans-serif;
  caret-color: #5b8fff;
}

.search-input::placeholder {
  color: #4a5068;
}

.vim-mode-badge {
  font-size: 10px;
  font-family: ui-monospace, 'Cascadia Code', monospace;
  font-weight: 700;
  letter-spacing: 0.05em;
  padding: 2px 7px;
  border-radius: 4px;
  flex-shrink: 0;
}

.vim-mode-badge.normal {
  background: rgba(120, 130, 170, 0.18);
  color: #7a8099;
  border: 1px solid rgba(120, 130, 170, 0.25);
}

.vim-mode-badge.insert {
  background: color-mix(in srgb, var(--app-accent) 20%, transparent);
  color: var(--app-accent);
  border: 1px solid color-mix(in srgb, var(--app-accent) 35%, transparent);
}

.search-kbd {
  font-size: 10px;
  color: #4a5068;
  border: 1px solid rgba(255,255,255,0.08);
  border-radius: 4px;
  padding: 1px 6px;
  font-family: system-ui, sans-serif;
}

.divider {
  height: 1px;
  background: rgba(255, 255, 255, 0.06);
  flex-shrink: 0;
}

.results-list {
  overflow-y: auto;
  flex: 1;
  padding: 6px;
}

.no-results {
  padding: 20px;
  text-align: center;
  color: #4a5068;
  font-size: 13px;
  font-family: system-ui, sans-serif;
}

.result-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  padding: 9px 12px;
  border-radius: 8px;
  cursor: pointer;
  transition: background 0.1s;
}

.result-row.active {
  background: color-mix(in srgb, var(--app-accent) 14%, transparent);
}

.result-left {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
  flex: 1;
}

.result-title {
  font-size: 13px;
  color: #e8eaf0;
  font-family: system-ui, sans-serif;
  font-weight: 500;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.result-tag {
  font-size: 10px;
  padding: 1px 7px;
  border-radius: 10px;
  background: color-mix(in srgb, var(--app-accent) 14%, transparent);
  color: var(--app-accent);
  border: 1px solid color-mix(in srgb, var(--app-accent) 28%, transparent);
  white-space: nowrap;
  flex-shrink: 0;
}

.result-right {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}

.result-type-badge {
  font-size: 10px;
  padding: 2px 8px;
  border-radius: 10px;
  font-family: system-ui, sans-serif;
  font-weight: 600;
}

.result-edges {
  font-size: 10px;
  color: #4a5068;
  font-family: system-ui, sans-serif;
  white-space: nowrap;
}

.modal-footer {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 18px;
  font-size: 11px;
  color: #4a5068;
  font-family: system-ui, sans-serif;
}

/* Transition */
.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.15s ease;
}
.modal-enter-active .modal-panel,
.modal-leave-active .modal-panel {
  transition: transform 0.15s ease, opacity 0.15s ease;
}
.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}
.modal-enter-from .modal-panel,
.modal-leave-to .modal-panel {
  transform: translateY(-8px);
  opacity: 0;
}
</style>
