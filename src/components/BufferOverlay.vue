<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useEventListener } from '@vueuse/core'
import { Layers3, Pin, X } from 'lucide-vue-next'
import OverlayShell from '@/components/ui/OverlayShell.vue'

const graphStore = useGraphStore()
const settings = useSettings()

const isOpen = computed(() => graphStore.activeBuffer !== 'none')
const isPinnedBuffer = computed(() => graphStore.activeBuffer === 'pinned')
const isMapBuffer = computed(() => graphStore.activeBuffer === 'map')
const pinnedViewMode = ref<'list' | 'cards'>('cards')
const hoveredMapNodeId = ref<string | null>(null)
const mapZoom = ref(1.22)
const mapCenterX = ref(0)
const mapCenterZ = ref(0)
const mapDragging = ref(false)
const dragStartX = ref(0)
const dragStartY = ref(0)
const dragStartCenterX = ref(0)
const dragStartCenterZ = ref(0)
const BUFFER_ORDER: Array<Exclude<typeof graphStore.activeBuffer.value, 'none'>> = ['pinned', 'map']

type MapEdge = {
  id: string
  source: string
  target: string
}

type MapDrawNode = {
  id: string
  title: string
  nodeType: string
  degree: number
  pinned: boolean
  selected: boolean
  learned: boolean
  sx: number
  sy: number
  r: number
  inView: boolean
  label: boolean
}

type MapDrawEdge = {
  id: string
  source: string
  target: string
  x1: number
  y1: number
  x2: number
  y2: number
  opacity: number
  width: number
  highlight: boolean
}

const quickMap = computed(() => {
  const items = graphStore.nodes
  if (items.length === 0) return { nodes: [] as MapDrawNode[], edges: [] as MapDrawEdge[] }

  const fallbackR = Math.max(10, items.length * 0.9)
  const worldNodes = items.map((n, i) => ({
    id: n.id,
    title: n.title,
    nodeType: n.node_type,
    pinned: graphStore.isNodePinned(n.id),
    selected: graphStore.selectedNodeId === n.id,
    learned: n.learned,
    wx: n.pos_x ?? Math.cos((i / Math.max(1, items.length)) * Math.PI * 2) * fallbackR,
    wz: n.pos_z ?? Math.sin((i / Math.max(1, items.length)) * Math.PI * 2) * fallbackR,
  }))

  const nodeIds = new Set(items.map(n => n.id))
  const edgeKeySet = new Set<string>()
  const edges: MapEdge[] = []
  const degreeMap = new Map<string, number>()

  for (const n of items) degreeMap.set(n.id, 0)

  for (const n of items) {
    for (const conn of n.connections) {
      if (!nodeIds.has(conn.target_id)) continue
      const a = n.id < conn.target_id ? n.id : conn.target_id
      const b = n.id < conn.target_id ? conn.target_id : n.id
      const key = `${a}::${b}`
      if (edgeKeySet.has(key)) continue
      edgeKeySet.add(key)
      edges.push({ id: conn.id, source: a, target: b })
      degreeMap.set(a, (degreeMap.get(a) ?? 0) + 1)
      degreeMap.set(b, (degreeMap.get(b) ?? 0) + 1)
    }
  }

  const halfSpan = 36 / mapZoom.value
  const project = (wx: number, wz: number) => {
    const sx = 50 + ((wx - mapCenterX.value) / halfSpan) * 50
    const sy = 50 + ((wz - mapCenterZ.value) / halfSpan) * 50
    return { sx, sy }
  }

  const drawNodes: MapDrawNode[] = worldNodes.map(n => {
    const isHover = hoveredMapNodeId.value === n.id
    const degree = degreeMap.get(n.id) ?? 0
    const p = project(n.wx, n.wz)
    const inView = p.sx >= -5 && p.sx <= 105 && p.sy >= -5 && p.sy <= 105
    const labelInView = p.sx >= 3 && p.sx <= 97 && p.sy >= 4 && p.sy <= 96
    return {
      id: n.id,
      title: n.title,
      nodeType: n.nodeType,
      degree,
      pinned: n.pinned,
      selected: n.selected,
      learned: n.learned,
      sx: p.sx,
      sy: p.sy,
      r: (n.selected ? 2.35 : n.pinned ? 2.0 : 1.45) + Math.min(0.62, degree * 0.07) + (isHover ? 0.45 : 0),
      inView,
      label: labelInView,
    }
  })

  const drawNodeById = new Map(drawNodes.map(n => [n.id, n]))
  const drawEdges: MapDrawEdge[] = edges
    .map(e => {
      const a = drawNodeById.get(e.source)
      const b = drawNodeById.get(e.target)
      if (!a || !b) return null
      const highlight = a.selected || b.selected
      const inView = a.inView || b.inView
      if (!inView) return null
      return {
        id: e.id,
        source: e.source,
        target: e.target,
        x1: a.sx,
        y1: a.sy,
        x2: b.sx,
        y2: b.sy,
        opacity: highlight ? 0.84 : 0.27,
        width: highlight ? 0.44 : 0.26,
        highlight,
      }
    })
    .filter((e): e is MapDrawEdge => !!e)

  return { nodes: drawNodes, edges: drawEdges }
})

const hoveredMapNode = computed(() => quickMap.value.nodes.find(n => n.id === hoveredMapNodeId.value) ?? null)
const mapStats = computed(() => ({
  nodes: quickMap.value.nodes.length,
  edges: quickMap.value.edges.length,
  pinned: quickMap.value.nodes.filter(n => n.pinned).length,
}))

function focusNode(id: string) {
  graphStore.requestFocus(id)
}

function cycleBuffer(direction: 1 | -1) {
  const current = graphStore.activeBuffer
  const index = BUFFER_ORDER.indexOf(current === 'none' ? 'pinned' : current)
  const safeIndex = index >= 0 ? index : 0
  const nextIndex = (safeIndex + direction + BUFFER_ORDER.length) % BUFFER_ORDER.length
  graphStore.openBuffer(BUFFER_ORDER[nextIndex])
}

function clampMapView() {
  mapZoom.value = Math.max(0.45, Math.min(3.2, mapZoom.value))
}

function zoomMap(step: number) {
  mapZoom.value += step
  clampMapView()
}

function panMap(dx: number, dy: number) {
  mapCenterX.value += dx
  mapCenterZ.value += dy
  clampMapView()
}

function resetMapView() {
  mapZoom.value = 1.22
  mapCenterX.value = 0
  mapCenterZ.value = 0
}

function onMapWheel(e: WheelEvent) {
  const delta = e.deltaY > 0 ? -0.09 : 0.09
  zoomMap(delta)
}

function recenterOnSelection() {
  const sel = graphStore.nodes.find(n => n.id === graphStore.selectedNodeId)
  if (sel && sel.pos_x !== null && sel.pos_z !== null) {
    mapCenterX.value = sel.pos_x
    mapCenterZ.value = sel.pos_z
  }
}

function onMapPointerDown(e: PointerEvent) {
  mapDragging.value = true
  dragStartX.value = e.clientX
  dragStartY.value = e.clientY
  dragStartCenterX.value = mapCenterX.value
  dragStartCenterZ.value = mapCenterZ.value
}

function onMapPointerMove(e: PointerEvent) {
  if (!mapDragging.value) return
  const dx = e.clientX - dragStartX.value
  const dy = e.clientY - dragStartY.value
  const halfSpan = 36 / mapZoom.value
  const worldPerPx = (halfSpan * 2) / 740
  mapCenterX.value = dragStartCenterX.value - dx * worldPerPx
  mapCenterZ.value = dragStartCenterZ.value - dy * worldPerPx
}

function onMapPointerUp() {
  mapDragging.value = false
}

watch(isMapBuffer, open => {
  if (open) recenterOnSelection()
})

useEventListener(
  document,
  'keydown',
  (e: KeyboardEvent) => {
    if (!isOpen.value) return
    const tag = (e.target as HTMLElement)?.tagName
    const isInput = tag === 'INPUT' || tag === 'TEXTAREA' || (e.target as HTMLElement)?.isContentEditable
    if (isInput) return

    if (e.key === 'Escape') {
      e.preventDefault()
      e.stopImmediatePropagation()
      graphStore.closeBuffer()
      return
    }

    if (e.key === 'Tab') {
      e.preventDefault()
      e.stopImmediatePropagation()
      cycleBuffer(e.ctrlKey || e.shiftKey ? -1 : 1)
      return
    }

    const k = e.key.toLowerCase()
    if (k === settings.keys.pinnedBuffer) {
      e.preventDefault()
      graphStore.openBuffer('pinned')
      return
    }
    if (k === settings.keys.mapBuffer) {
      e.preventDefault()
      graphStore.openBuffer('map')
      return
    }

    if (isMapBuffer.value) {
      const step = Math.max(0.8, 2.4 / mapZoom.value)
      if (k === 'w') {
        e.preventDefault()
        panMap(0, -step)
        return
      }
      if (k === 's') {
        e.preventDefault()
        panMap(0, step)
        return
      }
      if (k === 'a') {
        e.preventDefault()
        panMap(-step, 0)
        return
      }
      if (k === 'd') {
        e.preventDefault()
        panMap(step, 0)
        return
      }
      if (k === settings.keys.graphZoomIn || k === '=' || k === '+') {
        e.preventDefault()
        zoomMap(0.08)
        return
      }
      if (k === settings.keys.graphZoomOut || k === '-' || k === '_') {
        e.preventDefault()
        zoomMap(-0.08)
        return
      }
      if (k === '0') {
        e.preventDefault()
        resetMapView()
      }
    }
  },
  { capture: true }
)
</script>

<template>
  <OverlayShell :open="isOpen" @close="graphStore.closeBuffer()">
    <template #title>
      <div class="buffer-tabs">
        <button :class="['tab-btn', isPinnedBuffer ? 'active' : '']" @click="graphStore.openBuffer('pinned')">
          <Pin :size="14" /> Pinned
        </button>
        <button :class="['tab-btn', isMapBuffer ? 'active' : '']" @click="graphStore.openBuffer('map')">
          <Layers3 :size="14" /> Map
        </button>
      </div>
    </template>
    <template #actions>
      <button class="close-btn" @click="graphStore.closeBuffer()" aria-label="Close"><X :size="15" /></button>
    </template>

    <div v-if="isPinnedBuffer" class="buffer-body">
            <div class="buffer-title">Pinned Nodes</div>
            <div v-if="graphStore.pinnedNodes.length === 0" class="empty">
              No pinned nodes yet. Select node and press {{ settings.keys.pinNode.toUpperCase() }}.
            </div>
            <div v-else>
              <div class="pin-toolbar">
                <button
                  class="pin-mode-btn"
                  :class="{ active: pinnedViewMode === 'cards' }"
                  @click="pinnedViewMode = 'cards'"
                >
                  Cards
                </button>
                <button
                  class="pin-mode-btn"
                  :class="{ active: pinnedViewMode === 'list' }"
                  @click="pinnedViewMode = 'list'"
                >
                  List
                </button>
              </div>
              <div v-if="pinnedViewMode === 'list'" class="pin-list">
                <div v-for="node in graphStore.pinnedNodes" :key="node.id" class="pin-row">
                  <button class="pin-focus" @click="focusNode(node.id)">{{ node.title }}</button>
                  <button class="pin-remove" @click="graphStore.unpinNode(node.id)">Unpin</button>
                </div>
              </div>
              <div v-else class="pin-cards">
                <article
                  v-for="node in graphStore.pinnedNodes"
                  :key="`card-${node.id}`"
                  class="pin-card"
                  @click="focusNode(node.id)"
                >
                  <div class="pin-card-head">
                    <div class="pin-card-title">{{ node.title }}</div>
                    <button class="pin-remove" @click.stop="graphStore.unpinNode(node.id)">Unpin</button>
                  </div>
                  <div class="pin-card-type">
                    {{ node.node_type }} ·
                    {{
                      node.note_type_id
                        ? (graphStore.noteTypes.find(n => n.id === node.note_type_id)?.name ?? 'Note')
                        : 'Note'
                    }}
                  </div>
                  <p class="pin-card-content">{{ node.content_data || 'No content' }}</p>
                  <div class="pin-card-tags">
                    <span v-for="tag in node.tags.slice(0, 3)" :key="`${node.id}-${tag}`">{{ tag }}</span>
                  </div>
                </article>
              </div>
            </div>
    </div>

    <div v-else-if="isMapBuffer" class="buffer-body">
            <div class="buffer-title">Quick Map</div>
            <div
              class="map-wrap"
              @pointerdown="onMapPointerDown"
              @pointermove="onMapPointerMove"
              @pointerup="onMapPointerUp"
              @pointerleave="onMapPointerUp"
            >
              <svg class="map" viewBox="0 0 100 100" preserveAspectRatio="xMidYMid slice" @wheel.prevent="onMapWheel">
                <rect class="map-plane" x="0" y="0" width="100" height="100" />
                <line
                  v-for="e in quickMap.edges"
                  :key="`edge-${e.id}`"
                  :x1="e.x1"
                  :y1="e.y1"
                  :x2="e.x2"
                  :y2="e.y2"
                  :class="['map-edge', e.highlight ? 'highlight' : '']"
                  :style="{ opacity: e.opacity, strokeWidth: `${e.width}px` }"
                />
                <g
                  v-for="n in quickMap.nodes"
                  :key="n.id"
                  :class="[
                    'map-node',
                    n.selected ? 'selected' : '',
                    n.pinned ? 'pinned' : '',
                    n.learned ? 'learned' : '',
                  ]"
                  @mouseenter="hoveredMapNodeId = n.id"
                  @mouseleave="hoveredMapNodeId = null"
                  @click="focusNode(n.id)"
                >
                  <circle class="map-node-halo" :cx="n.sx" :cy="n.sy" :r="n.r + 0.95" />
                  <circle class="map-node-core" :cx="n.sx" :cy="n.sy" :r="Math.max(1.05, n.r)" />
                  <circle class="map-node-ring" :cx="n.sx" :cy="n.sy" :r="n.r + 0.55" />
                </g>
                <text
                  v-for="n in quickMap.nodes.filter(x => x.label && x.inView)"
                  :key="`label-${n.id}`"
                  :x="Math.min(96, n.sx + n.r + 0.9)"
                  :y="Math.max(4, n.sy - (n.r + 0.5))"
                  class="map-label"
                >
                  {{ n.title.length > 26 ? `${n.title.slice(0, 26)}...` : n.title }}
                </text>
              </svg>
              <div v-if="hoveredMapNode" class="map-hover-card">
                <strong>{{ hoveredMapNode.title }}</strong>
                <span>{{ hoveredMapNode.nodeType }} · {{ hoveredMapNode.degree }} links</span>
              </div>
            </div>
            <div class="map-hint-row">
              <div class="map-hint">
                WASD or drag mouse to pan · Mouse wheel or {{ settings.keys.graphZoomIn.toUpperCase() }}/{{
                  settings.keys.graphZoomOut.toUpperCase()
                }}
                to zoom · 0 reset
              </div>
              <div class="map-stats">
                {{ mapStats.nodes }} nodes · {{ mapStats.edges }} links · {{ mapStats.pinned }} pinned
              </div>
            </div>
    </div>
  </OverlayShell>
</template>

<style scoped>
.buffer-tabs {
  display: flex;
  gap: 8px;
}

.tab-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  background: rgba(255, 255, 255, 0.03);
  color: #b8bdd0;
  border-radius: 8px;
  padding: 7px 10px;
  font-size: 12px;
  cursor: pointer;
}

.tab-btn.active {
  border-color: color-mix(in srgb, var(--app-accent) 50%, transparent);
  color: var(--app-accent);
  background: color-mix(in srgb, var(--app-accent) 15%, transparent);
}

.close-btn {
  width: 28px;
  height: 28px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: none;
  border-radius: 8px;
  background: rgba(255, 255, 255, 0.05);
  color: #94a0c0;
  cursor: pointer;
}

.buffer-body {
  flex: 1;
  padding: 14px;
  overflow: auto;
}

.buffer-title {
  font-size: 12px;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--app-text-secondary);
  margin-bottom: 12px;
}

.empty {
  color: #7a8099;
  font-size: 13px;
}

.pin-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.pin-toolbar {
  display: flex;
  gap: 8px;
  margin-bottom: 10px;
}

.pin-mode-btn {
  border: 1px solid rgba(255, 255, 255, 0.12);
  background: rgba(255, 255, 255, 0.04);
  color: #bac0d5;
  border-radius: 7px;
  padding: 5px 9px;
  font-size: 11px;
  cursor: pointer;
}

.pin-mode-btn.active {
  border-color: color-mix(in srgb, var(--app-accent) 45%, transparent);
  color: var(--app-accent);
  background: color-mix(in srgb, var(--app-accent) 16%, transparent);
}

.pin-row {
  display: flex;
  justify-content: space-between;
  gap: 10px;
  align-items: center;
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid rgba(255, 255, 255, 0.07);
  border-radius: 8px;
  padding: 8px;
}

.pin-focus {
  background: none;
  border: none;
  color: #e8eaf0;
  text-align: left;
  cursor: pointer;
  font-size: 13px;
}

.pin-remove {
  border: 1px solid rgba(255, 159, 26, 0.4);
  color: #ff9f1a;
  background: rgba(255, 159, 26, 0.08);
  border-radius: 6px;
  padding: 5px 8px;
  cursor: pointer;
  font-size: 11px;
}

.pin-cards {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
  gap: 10px;
}

.pin-card {
  background: linear-gradient(180deg, rgba(255, 255, 255, 0.05), rgba(255, 255, 255, 0.02));
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 10px;
  padding: 10px;
  cursor: pointer;
  min-height: 150px;
}

.pin-card-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.pin-card-title {
  font-size: 14px;
  font-weight: 600;
  color: #e8eaf0;
}

.pin-card-type {
  margin-top: 4px;
  color: #8090b0;
  font-size: 11px;
}

.pin-card-content {
  margin: 8px 0;
  color: #c8cad8;
  font-size: 12px;
  line-height: 1.45;
  overflow: hidden;
  display: -webkit-box;
  -webkit-line-clamp: 3;
  -webkit-box-orient: vertical;
}

.pin-card-tags {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}

.pin-card-tags span {
  font-size: 10px;
  color: var(--app-accent);
  background: color-mix(in srgb, var(--app-accent) 14%, transparent);
  border: 1px solid color-mix(in srgb, var(--app-accent) 28%, transparent);
  border-radius: 999px;
  padding: 2px 7px;
}

.map {
  width: 100%;
  height: min(68vh, 680px);
  background:
    radial-gradient(circle at 20% 18%, color-mix(in srgb, var(--app-accent) 25%, transparent), transparent 38%),
    radial-gradient(circle at 78% 74%, rgba(61, 214, 140, 0.1), transparent 46%),
    linear-gradient(
      165deg,
      color-mix(in srgb, var(--app-overlay-bg) 88%, #0a0f20),
      color-mix(in srgb, var(--app-canvas-bg) 86%, #04090f)
    );
  border: 1px solid var(--app-overlay-border);
  border-radius: 12px;
}

.map-wrap {
  position: relative;
  overflow: hidden;
  border-radius: 12px;
  cursor: grab;
}

.map-wrap:active {
  cursor: grabbing;
}

.map-plane {
  fill: rgba(12, 16, 30, 0.42);
  stroke: color-mix(in srgb, var(--app-accent) 26%, transparent);
  stroke-width: 0.26px;
}

.map-edge {
  stroke: rgba(126, 140, 176, 0.32);
  stroke-width: 0.24;
  stroke-linecap: round;
}

.map-edge.highlight {
  stroke: color-mix(in srgb, var(--app-accent) 90%, white 10%);
}

.map-node {
  cursor: pointer;
  transition:
    transform 0.12s ease,
    opacity 0.12s ease;
  transform-origin: center;
}

.map-node-halo {
  fill: color-mix(in srgb, var(--app-accent) 8%, transparent);
}

.map-node-core {
  fill: color-mix(in srgb, var(--app-accent) 62%, #9cb6e8 38%);
  stroke: rgba(225, 233, 250, 0.88);
  stroke-width: 0.28px;
}

.map-node-ring {
  fill: none;
  stroke: rgba(215, 226, 247, 0.62);
  stroke-width: 0.14px;
}

.map-node.learned .map-node-halo {
  fill: rgba(61, 214, 140, 0.08);
}

.map-node.learned .map-node-core {
  fill: #58c894;
  stroke: rgba(197, 247, 224, 0.9);
}

.map-node.learned .map-node-ring {
  stroke: rgba(170, 232, 205, 0.62);
}

.map-node.pinned .map-node-halo {
  fill: rgba(255, 159, 26, 0.1);
}

.map-node.pinned .map-node-core {
  fill: #d89b3c;
  stroke: rgba(255, 231, 191, 0.9);
}

.map-node.pinned .map-node-ring {
  stroke: rgba(255, 220, 162, 0.65);
}

.map-node.selected .map-node-halo {
  fill: rgba(255, 255, 255, 0.1);
}

.map-node.selected .map-node-core {
  fill: #ffffff;
  stroke: rgba(180, 210, 255, 0.92);
}

.map-node.selected .map-node-ring {
  stroke: rgba(210, 228, 255, 0.7);
}

.map-hint-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
  margin-top: 10px;
}

.map-hint {
  font-size: 12px;
  color: #7a8099;
}

.map-stats {
  font-size: 11px;
  color: var(--app-text-secondary);
  white-space: nowrap;
}

.map-hover-card {
  position: absolute;
  left: 12px;
  bottom: 12px;
  display: inline-flex;
  align-items: center;
  gap: 10px;
  padding: 6px 10px;
  border-radius: 8px;
  border: 1px solid color-mix(in srgb, var(--app-accent) 30%, transparent);
  background: color-mix(in srgb, var(--app-accent) 12%, transparent);
  pointer-events: none;
}

.map-hover-card strong {
  font-size: 12px;
  font-weight: 700;
  color: #e8eaf0;
}

.map-hover-card span {
  font-size: 11px;
  color: #90a0c8;
}

.map-label {
  fill: color-mix(in srgb, var(--app-text-primary) 92%, #b7c6ea 8%);
  font-size: 2.05px;
  pointer-events: none;
  stroke: rgba(5, 8, 16, 0.82);
  stroke-width: 0.34px;
  paint-order: stroke fill;
  letter-spacing: 0.01em;
  font-family: 'Inter', 'Segoe UI Variable', 'Segoe UI', system-ui, sans-serif;
  font-weight: 500;
  text-rendering: geometricPrecision;
}

@media (max-width: 760px) {
  .map-hint-row {
    flex-direction: column;
    align-items: flex-start;
  }

  .map-stats {
    white-space: normal;
  }
}

</style>
