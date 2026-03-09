<script setup lang="ts">
import { Layers3, Orbit, Route } from 'lucide-vue-next'
import { computed, ref } from 'vue'
import { useEventListener } from '@vueuse/core'
import type { ConnectionLayer, Layer } from '@/bindings'

const graphStore = useGraphStore()
const settings = useSettings()
const editorMode = useEditorMode()
const open = ref(false)

function parseJson(raw: string | null | undefined): Record<string, unknown> {
  if (!raw) return {}
  try {
    const parsed = JSON.parse(raw)
    return parsed && typeof parsed === 'object' ? (parsed as Record<string, unknown>) : {}
  } catch {
    return {}
  }
}

const activeTopicLayer = computed(() =>
  graphStore.layers.find(layer => layer.id === graphStore.activeLayerId) ?? graphStore.layers[0] ?? null
)

const activeConnectionLayers = computed(() =>
  graphStore.connectionLayers.filter(layer => graphStore.activeConnectionLayerIds.includes(layer.id))
)

function layerIndex(entity: { metadata: string }): number | null {
  const md = parseJson(entity.metadata)
  const raw = md.index
  return typeof raw === 'number' && Number.isFinite(raw) && raw >= 1 && raw <= 9 ? Math.round(raw) : null
}

type IndexedEntity<T> = { entity: T; index: number | null }

const indexedTopicLayers = computed<IndexedEntity<Layer>[]>(() =>
  graphStore.layers.map(layer => ({ entity: layer, index: layerIndex(layer) }))
)

const indexedConnectionLayers = computed<IndexedEntity<ConnectionLayer>[]>(() =>
  graphStore.connectionLayers.map(layer => ({ entity: layer, index: layerIndex(layer) }))
)

const compactTopicLabel = computed(() => {
  const name = activeTopicLayer.value?.name?.trim()
  if (!name) return 'No topic'
  return name.length > 18 ? `${name.slice(0, 18)}...` : name
})

const compactOverlayItems = computed(() =>
  activeConnectionLayers.value.map(layer => {
    const tokens = layer.name.split(/\s+/).filter(Boolean)
    const short = tokens.length > 1
      ? tokens.slice(0, 2).map(token => token[0]?.toUpperCase() ?? '').join('')
      : layer.name.slice(0, 3).toUpperCase()
    return { id: layer.id, name: layer.name, short }
  })
)

const overlayPrevDisplayKey = computed(() => (graphStore.focusViewActive ? settings.keys.overlayPrev : 'q'))
const overlayNextDisplayKey = computed(() => (graphStore.focusViewActive ? settings.keys.overlayNext : 'e'))

function connectionLayerPreviewStyle(layer: { id: string; metadata: string }): Record<string, string> {
  const md = parseJson(layer.metadata)
  const style = md.edge_style && typeof md.edge_style === 'object' ? (md.edge_style as Record<string, unknown>) : {}
  const id = layer.id.toLowerCase()
  const color = typeof style.color === 'string' ? style.color : '#8fa2d6'
  const width = typeof style.width === 'number' ? `${Math.max(2, Math.min(5, style.width + 0.6))}px` : '3px'
  const dashed = typeof style.dash_size === 'number' ? style.dash_size > 0 : id.includes('usage')
  const shape = typeof style.shape === 'string' ? style.shape : (id.includes('concept') ? 'arc' : id.includes('usage') ? 'wave' : 'straight')

  return {
    '--preview-color': color,
    '--preview-width': width,
    '--preview-style': dashed ? 'dashed' : 'solid',
    '--preview-shape': shape,
  } as Record<string, string>
}

function cycleTopic(direction: 1 | -1) {
  if (graphStore.layers.length <= 1) return
  const currentIndex = Math.max(0, graphStore.layers.findIndex(layer => layer.id === graphStore.activeLayerId))
  const nextIndex = (currentIndex + direction + graphStore.layers.length) % graphStore.layers.length
  const next = graphStore.layers[nextIndex]
  if (next) graphStore.loadNodes(next.id)
}

function cycleOverlay(direction: 1 | -1) {
  if (graphStore.connectionLayers.length === 0) return
  const currentId = graphStore.activeConnectionLayerIds[0] ?? graphStore.connectionLayers[0]?.id ?? null
  const currentIndex = Math.max(0, graphStore.connectionLayers.findIndex(layer => layer.id === currentId))
  const nextIndex = (currentIndex + direction + graphStore.connectionLayers.length) % graphStore.connectionLayers.length
  const next = graphStore.connectionLayers[nextIndex]
  if (next) graphStore.setConnectionLayerSelection([next.id])
}

function onExclusiveOverlaySelect(layerId: string) {
  graphStore.setConnectionLayerSelection([layerId])
}

function activateIndexedTopic(index: number) {
  const match = indexedTopicLayers.value.find(item => item.index === index)?.entity
  if (match) graphStore.loadNodes(match.id)
}

function activateIndexedOverlay(index: number) {
  const match = indexedConnectionLayers.value.find(item => item.index === index)?.entity
  if (match) graphStore.setConnectionLayerSelection([match.id])
}

useEventListener(document, 'keydown', (e: KeyboardEvent) => {
  const tag = (e.target as HTMLElement)?.tagName
  const isInput = tag === 'INPUT' || tag === 'TEXTAREA' || (e.target as HTMLElement)?.isContentEditable
  if (isInput || graphStore.activeBuffer !== 'none') return

  if (e.key === 'Tab' && editorMode.mode.value !== 'graph' && !graphStore.centeredNodePanel) {
    e.preventDefault()
    e.stopPropagation()
    open.value = !open.value
    return
  }

  const key = e.key.toLowerCase()
  const indexedDigit = Number.parseInt(e.key, 10)
  if (Number.isInteger(indexedDigit) && indexedDigit >= 1 && indexedDigit <= 9) {
    if (e.altKey && !e.shiftKey) {
      e.preventDefault()
      activateIndexedTopic(indexedDigit)
      return
    }
    if (e.altKey && e.shiftKey) {
      e.preventDefault()
      activateIndexedOverlay(indexedDigit)
      return
    }
  }
  const overlayPrevKey = graphStore.focusViewActive ? settings.keys.overlayPrev : 'q'
  const overlayNextKey = graphStore.focusViewActive ? settings.keys.overlayNext : 'e'
  if (key === settings.keys.topicLayerPrev) {
    e.preventDefault()
    e.stopPropagation()
    cycleTopic(-1)
    return
  }
  if (key === settings.keys.topicLayerNext) {
    e.preventDefault()
    e.stopPropagation()
    cycleTopic(1)
    return
  }
  if (key === overlayPrevKey) {
    e.preventDefault()
    e.stopPropagation()
    cycleOverlay(-1)
    return
  }
  if (key === overlayNextKey) {
    e.preventDefault()
    e.stopPropagation()
    cycleOverlay(1)
  }
})
</script>

<template>
  <div class="layer-hud">
    <div v-if="!open" class="layer-hint">
      <span>Topics {{ settings.keys.topicLayerPrev.toUpperCase() }}/{{ settings.keys.topicLayerNext.toUpperCase() }}</span>
      <span>Overlays {{ overlayPrevDisplayKey.toUpperCase() }}/{{ overlayNextDisplayKey.toUpperCase() }}</span>
      <span>Alt+1..9 direct</span>
    </div>
    <button class="layer-trigger" :class="{ active: open }" @click="open = !open">
      <span class="trigger-pill">
        <Layers3 :size="13" />
        <span class="pill-text">{{ compactTopicLabel }}</span>
        <span class="pill-key">{{ settings.keys.topicLayerPrev.toUpperCase() }}/{{ settings.keys.topicLayerNext.toUpperCase() }}</span>
      </span>

      <span class="trigger-pill trigger-pill-overlay">
        <Route :size="13" />
        <template v-if="compactOverlayItems.length > 0">
          <span
            v-for="overlay in compactOverlayItems.slice(0, 3)"
            :key="overlay.id"
            class="overlay-chip"
            :title="overlay.name"
          >
            {{ overlay.short }}
          </span>
          <span v-if="compactOverlayItems.length > 3" class="overlay-count">+{{ compactOverlayItems.length - 3 }}</span>
        </template>
        <span v-else class="pill-text">None</span>
        <span class="pill-key">{{ overlayPrevDisplayKey.toUpperCase() }}/{{ overlayNextDisplayKey.toUpperCase() }}</span>
      </span>

      <span v-if="graphStore.focusViewActive" class="trigger-pill trigger-pill-focus">
        <Orbit :size="13" />
        <span class="pill-text">Solar</span>
      </span>
    </button>

    <div v-if="open" class="layer-popover">
      <section class="popover-section">
        <div class="section-header">
          <Layers3 :size="13" />
          <span>Topics</span>
          <strong>{{ settings.keys.topicLayerPrev.toUpperCase() }}/{{ settings.keys.topicLayerNext.toUpperCase() }} · Alt+1..9</strong>
        </div>
        <div class="row-list">
          <button
            v-for="item in indexedTopicLayers"
            :key="item.entity.id"
            class="topic-row"
            :class="{ active: graphStore.activeLayerId === item.entity.id }"
            @click="graphStore.loadNodes(item.entity.id)"
          >
            <span class="row-main">
              <span v-if="item.index" class="row-index">{{ item.index }}</span>
              <span>{{ item.entity.name }}</span>
            </span>
            <strong v-if="graphStore.activeLayerId === item.entity.id">Live</strong>
          </button>
        </div>
      </section>

      <section class="popover-section">
        <div class="section-header">
          <Route :size="13" />
          <span>Overlays</span>
          <strong>{{ overlayPrevDisplayKey.toUpperCase() }}/{{ overlayNextDisplayKey.toUpperCase() }} · Alt+Shift+1..9</strong>
        </div>
        <div class="row-list">
          <button
            v-for="item in indexedConnectionLayers"
            :key="item.entity.id"
            class="overlay-row"
            :class="{ active: graphStore.activeConnectionLayerIds.includes(item.entity.id) }"
            @click="graphStore.toggleConnectionLayer(item.entity.id)"
            @dblclick.prevent="onExclusiveOverlaySelect(item.entity.id)"
          >
            <span class="overlay-preview" :style="connectionLayerPreviewStyle(item.entity)">
              <span class="preview-line" />
            </span>
            <span class="row-main">
              <span v-if="item.index" class="row-index">{{ item.index }}</span>
              <span class="overlay-name">{{ item.entity.name }}</span>
            </span>
          </button>
        </div>
        <div class="section-note">Click to combine overlays. Double-click to isolate one lens.</div>
      </section>

      <section class="popover-section" v-if="graphStore.selectedNodeId">
        <div class="section-header">
          <Orbit :size="13" />
          <span>Solar</span>
          <strong>{{ settings.keys.focusView.toUpperCase() }} / Space</strong>
        </div>
        <button
          class="topic-row"
          :class="{ active: graphStore.focusViewActive }"
          @click="graphStore.toggleFocusView()"
        >
          <span>{{ graphStore.focusViewActive ? 'Exit solar view' : 'Enter solar view' }}</span>
          <strong>{{ graphStore.selectedNode?.title ?? 'node' }}</strong>
        </button>
      </section>
    </div>
  </div>
</template>

<style scoped>
.layer-hud {
  position: fixed;
  top: 16px;
  left: 16px;
  z-index: 140;
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 8px;
}

.layer-hint {
  display: inline-flex;
  align-items: center;
  gap: 10px;
  padding: 0 6px;
  color: var(--app-text-secondary);
  font-size: 10px;
  letter-spacing: 0.06em;
  text-transform: uppercase;
}

.layer-trigger {
  width: fit-content;
  max-width: calc(100vw - 32px);
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 10px;
  border-radius: 999px;
  border: 1px solid color-mix(in srgb, var(--app-overlay-border) 88%, transparent);
  background: color-mix(in srgb, var(--app-overlay-bg) 92%, transparent);
  color: var(--app-text-primary);
  backdrop-filter: blur(14px);
  -webkit-backdrop-filter: blur(14px);
  box-shadow: 0 10px 24px rgba(0, 0, 0, 0.32);
  cursor: pointer;
  transition:
    transform 0.14s ease,
    border-color 0.14s ease,
    background 0.14s ease;
}

.layer-trigger:hover,
.layer-trigger.active {
  transform: translateY(-1px);
  border-color: color-mix(in srgb, var(--app-accent) 34%, transparent);
}

.trigger-pill {
  min-width: 0;
  display: inline-flex;
  align-items: center;
  gap: 7px;
  padding: 4px 8px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.04);
}

.trigger-pill-overlay {
  max-width: 240px;
}

.trigger-pill-focus {
  background: color-mix(in srgb, var(--app-accent) 14%, transparent);
}

.pill-text {
  min-width: 0;
  font-size: 12px;
  font-weight: 700;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.pill-key {
  font-size: 10px;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--app-text-secondary);
}

.overlay-chip,
.overlay-count {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 24px;
  height: 22px;
  padding: 0 6px;
  border-radius: 999px;
  background: color-mix(in srgb, var(--app-accent) 12%, transparent);
  color: var(--app-text-primary);
  font-size: 11px;
  font-weight: 700;
}

.layer-popover {
  width: min(380px, calc(100vw - 32px));
  padding: 10px;
  border-radius: 16px;
  border: 1px solid color-mix(in srgb, var(--app-overlay-border) 92%, transparent);
  background:
    linear-gradient(180deg, color-mix(in srgb, var(--app-overlay-bg) 96%, transparent), color-mix(in srgb, var(--app-canvas-bg) 82%, transparent));
  backdrop-filter: blur(14px);
  -webkit-backdrop-filter: blur(14px);
  box-shadow: 0 16px 40px rgba(0, 0, 0, 0.42);
  display: grid;
  gap: 10px;
}

.popover-section {
  display: grid;
  gap: 6px;
}

.section-header {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--app-text-secondary);
  padding: 2px 4px;
}

.section-header strong {
  margin-left: auto;
  font-size: 10px;
  color: var(--app-text-secondary);
}

.row-list {
  display: grid;
  gap: 6px;
}

.row-main {
  min-width: 0;
  display: inline-flex;
  align-items: center;
  gap: 8px;
}

.row-index {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 18px;
  height: 18px;
  border-radius: 999px;
  background: color-mix(in srgb, var(--app-accent) 12%, transparent);
  color: var(--app-text-primary);
  font-size: 10px;
  font-weight: 700;
  flex-shrink: 0;
}

.topic-row,
.overlay-row {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
  padding: 9px 10px;
  border-radius: 10px;
  border: 1px solid transparent;
  background: rgba(255, 255, 255, 0.03);
  color: var(--app-text-primary);
  cursor: pointer;
  text-align: left;
  transition:
    background 0.12s ease,
    border-color 0.12s ease,
    transform 0.12s ease;
}

.topic-row:hover,
.overlay-row:hover {
  transform: translateX(1px);
  background: rgba(255, 255, 255, 0.05);
}

.topic-row.active,
.overlay-row.active {
  border-color: color-mix(in srgb, var(--app-accent) 34%, transparent);
  background: color-mix(in srgb, var(--app-accent) 12%, transparent);
}

.topic-row strong,
.overlay-name {
  margin-left: auto;
  font-size: 12px;
}

.overlay-name {
  margin-left: 0;
}

.overlay-preview {
  width: 44px;
  height: 18px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.preview-line {
  width: 100%;
  border-top: var(--preview-width) var(--preview-style) var(--preview-color);
  border-radius: 999px;
  position: relative;
  opacity: 0.98;
}

.overlay-preview[style*='arc'] .preview-line,
.overlay-preview[style*='wave'] .preview-line {
  transform: translateY(1px);
}

.overlay-preview[style*='arc'] .preview-line::after {
  content: '';
  position: absolute;
  inset: -7px 2px auto 2px;
  height: 8px;
  border-top: var(--preview-width) var(--preview-style) var(--preview-color);
  border-radius: 999px 999px 0 0;
}

.overlay-preview[style*='wave'] .preview-line::before,
.overlay-preview[style*='wave'] .preview-line::after {
  content: '';
  position: absolute;
  width: 16px;
  height: 7px;
  border-top: var(--preview-width) var(--preview-style) var(--preview-color);
  border-radius: 999px 999px 0 0;
  top: -6px;
}

.overlay-preview[style*='wave'] .preview-line::before {
  left: 2px;
}

.overlay-preview[style*='wave'] .preview-line::after {
  right: 2px;
}

.section-note {
  padding: 2px 4px 0;
  font-size: 11px;
  color: var(--app-text-secondary);
}

@media (max-width: 760px) {
  .layer-hint {
    flex-wrap: wrap;
    gap: 6px 10px;
  }

  .layer-trigger {
    flex-wrap: wrap;
    border-radius: 18px;
  }

  .trigger-pill-overlay {
    max-width: 100%;
  }
}
</style>
