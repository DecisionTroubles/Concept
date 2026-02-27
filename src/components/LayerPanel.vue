<script setup lang="ts">
import { Layers, ChevronLeft, ChevronRight } from 'lucide-vue-next'
import { ref } from 'vue'

const graphStore = useGraphStore()
const collapsed = ref(false)
</script>

<template>
  <div :class="['layer-panel', collapsed ? 'collapsed' : '']">
    <!-- Collapse toggle -->
    <button class="collapse-btn" @click="collapsed = !collapsed" :aria-label="collapsed ? 'Expand layers' : 'Collapse layers'">
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

        <!-- Layer list -->
        <ul class="layer-list">
          <li
            v-for="layer in graphStore.layers"
            :key="layer.id"
            :class="['layer-item', graphStore.activeLayerId === layer.id ? 'active' : '']"
            @click="graphStore.loadNodes(layer.id)"
          >
            <span class="layer-dot" />
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
  transition: color 0.15s, background 0.15s;
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
  0%, 100% { opacity: 0.3; }
  50%       { opacity: 1; }
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
