<script setup lang="ts">
import { computed, ref } from 'vue'
import { useEventListener } from '@vueuse/core'
import { Layers3, Pin, X } from 'lucide-vue-next'

const graphStore = useGraphStore()
const settings = useSettings()

const isOpen = computed(() => graphStore.activeBuffer !== 'none')
const isPinnedBuffer = computed(() => graphStore.activeBuffer === 'pinned')
const isMapBuffer = computed(() => graphStore.activeBuffer === 'map')
const pinnedViewMode = ref<'list' | 'cards'>('cards')

const mapPoints = computed(() => {
  const items = graphStore.nodes
  if (items.length === 0) return []

  const xs = items.map((n, i) => n.pos_x ?? Math.cos((i * 2 * Math.PI) / Math.max(1, items.length)) * 8)
  const zs = items.map((n, i) => n.pos_z ?? Math.sin((i * 2 * Math.PI) / Math.max(1, items.length)) * 8)
  const minX = Math.min(...xs)
  const maxX = Math.max(...xs)
  const minZ = Math.min(...zs)
  const maxZ = Math.max(...zs)
  const spanX = Math.max(1, maxX - minX)
  const spanZ = Math.max(1, maxZ - minZ)

  return items.map((n, i) => {
    const x = n.pos_x ?? xs[i]
    const z = n.pos_z ?? zs[i]
    return {
      id: n.id,
      title: n.title,
      pinned: graphStore.isNodePinned(n.id),
      selected: graphStore.selectedNodeId === n.id,
      sx: 5 + ((x - minX) / spanX) * 90,
      sy: 5 + ((z - minZ) / spanZ) * 90,
    }
  })
})

function focusNode(id: string) {
  graphStore.requestFocus(id)
}

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

    const k = e.key.toLowerCase()
    if (k === settings.keys.pinnedBuffer) {
      e.preventDefault()
      graphStore.openBuffer('pinned')
      return
    }
    if (k === settings.keys.mapBuffer) {
      e.preventDefault()
      graphStore.openBuffer('map')
    }
  },
  { capture: true },
)
</script>

<template>
  <Teleport to="body">
    <Transition name="buffer">
      <div v-if="isOpen" class="buffer-backdrop" @click.self="graphStore.closeBuffer()">
        <div class="buffer-shell">
          <div class="buffer-head">
            <div class="buffer-tabs">
              <button :class="['tab-btn', isPinnedBuffer ? 'active' : '']" @click="graphStore.openBuffer('pinned')">
                <Pin :size="14" /> Pinned
              </button>
              <button :class="['tab-btn', isMapBuffer ? 'active' : '']" @click="graphStore.openBuffer('map')">
                <Layers3 :size="14" /> Map
              </button>
            </div>
            <button class="close-btn" @click="graphStore.closeBuffer()" aria-label="Close"><X :size="15" /></button>
          </div>

          <div v-if="isPinnedBuffer" class="buffer-body">
            <div class="buffer-title">Pinned Nodes</div>
            <div v-if="graphStore.pinnedNodes.length === 0" class="empty">No pinned nodes yet. Select node and press {{ settings.keys.pinNode.toUpperCase() }}.</div>
            <div v-else>
              <div class="pin-toolbar">
                <button class="pin-mode-btn" :class="{ active: pinnedViewMode === 'cards' }" @click="pinnedViewMode = 'cards'">Cards</button>
                <button class="pin-mode-btn" :class="{ active: pinnedViewMode === 'list' }" @click="pinnedViewMode = 'list'">List</button>
              </div>
              <div v-if="pinnedViewMode === 'list'" class="pin-list">
                <div v-for="node in graphStore.pinnedNodes" :key="node.id" class="pin-row">
                  <button class="pin-focus" @click="focusNode(node.id)">{{ node.title }}</button>
                  <button class="pin-remove" @click="graphStore.unpinNode(node.id)">Unpin</button>
                </div>
              </div>
              <div v-else class="pin-cards">
                <article v-for="node in graphStore.pinnedNodes" :key="`card-${node.id}`" class="pin-card" @click="focusNode(node.id)">
                  <div class="pin-card-head">
                    <div class="pin-card-title">{{ node.title }}</div>
                    <button class="pin-remove" @click.stop="graphStore.unpinNode(node.id)">Unpin</button>
                  </div>
                  <div class="pin-card-type">{{ node.node_type }} · {{ node.note_type_id ? (graphStore.noteTypes.find((n) => n.id === node.note_type_id)?.name ?? 'Note') : 'Note' }}</div>
                  <p class="pin-card-content">{{ node.content_data || 'No content' }}</p>
                  <div class="pin-card-tags">
                    <span v-for="tag in node.tags.slice(0, 3)" :key="`${node.id}-${tag}`">{{ tag }}</span>
                  </div>
                </article>
              </div>
            </div>
          </div>

          <div v-else-if="isMapBuffer" class="buffer-body">
            <div class="buffer-title">Context Map</div>
            <svg class="map" viewBox="0 0 100 100" preserveAspectRatio="none">
              <circle
                v-for="p in mapPoints"
                :key="p.id"
                :cx="p.sx"
                :cy="p.sy"
                :r="p.selected ? 1.9 : 1.3"
                :fill="p.pinned ? '#ff9f1a' : (p.selected ? '#e8eaf0' : '#5b8fff')"
                @click="focusNode(p.id)"
              />
              <text
                v-for="p in mapPoints"
                :key="`label-${p.id}`"
                :x="Math.min(96, p.sx + 1.6)"
                :y="Math.max(4, p.sy - 1.2)"
                class="map-label"
              >
                {{ p.title.length > 24 ? `${p.title.slice(0, 24)}...` : p.title }}
              </text>
            </svg>
            <div class="map-hint">Click a node dot to focus it in 3D.</div>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.buffer-backdrop {
  position: fixed;
  inset: 0;
  z-index: 520;
  background: rgba(4, 6, 12, 0.72);
  backdrop-filter: blur(8px);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px;
}

.buffer-shell {
  width: min(1100px, 100%);
  height: min(86vh, 860px);
  background: rgba(9, 12, 20, 0.96);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 14px;
  box-shadow: 0 16px 40px rgba(0, 0, 0, 0.55);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.buffer-head {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 12px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
}

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
  border-color: rgba(91, 143, 255, 0.45);
  color: #5b8fff;
  background: rgba(91, 143, 255, 0.12);
}

.close-btn {
  width: 28px;
  height: 28px;
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
  color: #6f7999;
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
  border-color: rgba(91, 143, 255, 0.4);
  color: #5b8fff;
  background: rgba(91, 143, 255, 0.14);
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
  color: #5b8fff;
  background: rgba(91, 143, 255, 0.12);
  border: 1px solid rgba(91, 143, 255, 0.24);
  border-radius: 999px;
  padding: 2px 7px;
}

.map {
  width: 100%;
  height: min(68vh, 680px);
  background:
    radial-gradient(circle at 18% 16%, rgba(91, 143, 255, 0.08), transparent 36%),
    radial-gradient(circle at 80% 70%, rgba(255, 159, 26, 0.08), transparent 42%),
    rgba(6, 9, 17, 0.9);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 12px;
}

.map-hint {
  margin-top: 10px;
  font-size: 12px;
  color: #7a8099;
}

.map-label {
  fill: #d8deef;
  font-size: 2.4px;
  pointer-events: none;
  paint-order: stroke;
  stroke: rgba(5, 8, 16, 0.9);
  stroke-width: 0.5px;
  font-family: system-ui, sans-serif;
}

.buffer-enter-active,
.buffer-leave-active {
  transition: opacity 0.16s ease;
}

.buffer-enter-from,
.buffer-leave-to {
  opacity: 0;
}
</style>
