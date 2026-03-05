<script setup lang="ts">
import { Layers, ChevronLeft, ChevronRight } from 'lucide-vue-next'
import { computed, ref } from 'vue'

const graphStore = useGraphStore()
const collapsed = ref(false)

const visibleNodeLayers = computed(() => {
  const seen = new Set<string>()
  return graphStore.layers.filter(layer => {
    const key = layer.name.trim().toLowerCase()
    if (seen.has(key)) return false
    seen.add(key)
    return true
  })
})

function parseJson(raw: string | null | undefined): Record<string, unknown> {
  if (!raw) return {}
  try {
    const parsed = JSON.parse(raw)
    return parsed && typeof parsed === 'object' ? (parsed as Record<string, unknown>) : {}
  } catch {
    return {}
  }
}

function nodeLayerColor(layer: { metadata: string }): string {
  const md = parseJson(layer.metadata)
  const style = md.node_style
  if (style && typeof style === 'object' && typeof (style as Record<string, unknown>).color === 'string') {
    return (style as Record<string, unknown>).color as string
  }
  return '#5b8fff'
}

function connectionLayerLineStyle(layer: { metadata: string }): Record<string, string> {
  const md = parseJson(layer.metadata)
  const style = md.edge_style && typeof md.edge_style === 'object' ? (md.edge_style as Record<string, unknown>) : {}
  const color = typeof style.color === 'string' ? style.color : '#8fa2d6'
  const width = typeof style.width === 'number' ? `${Math.max(1, Math.min(4, style.width))}px` : '2px'
  const dashed = typeof style.dash_size === 'number' && style.dash_size > 0
  const opacity = typeof style.opacity === 'number' ? String(Math.max(0.2, Math.min(1, style.opacity))) : '1'
  return {
    borderColor: color,
    borderTopWidth: width,
    borderTopStyle: dashed ? 'dashed' : 'solid',
    opacity,
  }
}

function isConnectionLayerActive(id: string): boolean {
  return graphStore.activeConnectionLayerIds.includes(id)
}
</script>

<template>
  <div :class="['layer-panel', collapsed ? 'collapsed' : '']">
    <!-- Collapse toggle -->
    <button
      class="collapse-btn"
      @click="collapsed = !collapsed"
      :aria-label="collapsed ? 'Expand layers' : 'Collapse layers'"
    >
      <ChevronLeft v-if="!collapsed" :size="13" />
      <ChevronRight v-else :size="13" />
    </button>

    <Transition name="panel-content">
      <div v-if="!collapsed" class="panel-inner">
        <!-- Header -->
        <div class="panel-header">
          <Layers :size="13" />
          <span class="header-text">Layers</span>
        </div>

        <div class="divider" />

        <div class="section-title">Node Layers</div>
        <ul class="layer-list">
          <li
            v-for="layer in visibleNodeLayers"
            :key="layer.id"
            :class="['layer-item', graphStore.activeLayerId === layer.id ? 'active' : '']"
            @click="graphStore.loadNodes(layer.id)"
          >
            <span class="layer-dot" :style="{ background: nodeLayerColor(layer) }" />
            <span class="layer-name">{{ layer.name }}</span>
          </li>
        </ul>

        <div class="divider" />

        <div class="section-title">Connection Layers</div>
        <ul class="layer-list">
          <li
            v-for="layer in graphStore.connectionLayers"
            :key="`connection-${layer.id}`"
            :class="['layer-item', isConnectionLayerActive(layer.id) ? 'active' : '']"
            @click="graphStore.toggleConnectionLayer(layer.id)"
          >
            <span class="layer-line" :style="connectionLayerLineStyle(layer)" />
            <span class="layer-name">{{ layer.name }}</span>
          </li>
        </ul>

        <!-- Loading indicator -->
        <div v-if="graphStore.isLoading" class="loading-row">
          <span class="loading-dot" />
        </div>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.layer-panel {
  position: fixed;
  top: 50%;
  left: 16px;
  transform: translateY(-50%);
  background: rgba(12, 16, 28, 0.82);
  backdrop-filter: blur(16px);
  -webkit-backdrop-filter: blur(16px);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 12px;
  color: #e8eaf0;
  font-family: system-ui, sans-serif;
  font-size: 13px;
  z-index: 100;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
  min-width: 36px;
  transition: min-width 0.2s ease;
  display: flex;
  align-items: flex-start;
}

.layer-panel.collapsed {
  min-width: 36px;
}

.collapse-btn {
  flex-shrink: 0;
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  background: none;
  color: #7a8099;
  cursor: pointer;
  border-radius: 12px;
  transition:
    color 0.15s,
    background 0.15s;
  align-self: center;
}

.collapse-btn:hover {
  color: #e8eaf0;
  background: rgba(255, 255, 255, 0.05);
}

.panel-inner {
  flex: 1;
  padding: 12px 14px 12px 0;
  min-width: 140px;
}

.panel-header {
  display: flex;
  align-items: center;
  gap: 6px;
  padding-bottom: 8px;
  color: #7a8099;
}

.header-text {
  font-size: 10px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.08em;
}

.section-title {
  margin: 8px 0 6px;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.07em;
  text-transform: uppercase;
  color: #8a91ad;
}

.divider {
  height: 1px;
  background: rgba(255, 255, 255, 0.06);
  margin-bottom: 8px;
}

.layer-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.layer-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 8px;
  border-radius: 7px;
  cursor: pointer;
  transition: background 0.12s;
  color: #7a8099;
}

.layer-item:hover {
  background: rgba(255, 255, 255, 0.05);
  color: #c8cad6;
}

.layer-item.active {
  background: rgba(91, 143, 255, 0.12);
  color: #e8eaf0;
}

.layer-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: currentColor;
  flex-shrink: 0;
}

.layer-line {
  width: 14px;
  height: 0;
  border-top: 2px solid #8fa2d6;
  border-radius: 999px;
  flex-shrink: 0;
}

.layer-item.active .layer-dot {
  background: #5b8fff;
  box-shadow: 0 0 6px rgba(91, 143, 255, 0.7);
}

.layer-name {
  font-size: 13px;
  line-height: 1;
  white-space: nowrap;
}

.loading-row {
  display: flex;
  justify-content: center;
  padding: 6px 0;
}

.loading-dot {
  width: 5px;
  height: 5px;
  border-radius: 50%;
  background: #5b8fff;
  animation: pulse 1s ease-in-out infinite;
}

@keyframes pulse {
  0%,
  100% {
    opacity: 0.3;
  }
  50% {
    opacity: 1;
  }
}

/* Content transition */
.panel-content-enter-active,
.panel-content-leave-active {
  transition: opacity 0.18s ease;
  overflow: hidden;
}

.panel-content-enter-from,
.panel-content-leave-to {
  opacity: 0;
}
</style>
