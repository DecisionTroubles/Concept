<script setup lang="ts">
import { COMPASS_RING_R } from '@/composables/useEditorMode'

interface CompassDot {
  id: string
  title: string
  screenX: number
  screenY: number
  edgeType: string
  index: number
}

const props = defineProps<{
  dots: CompassDot[]
  centerX: number
  centerY: number
  activeIndex: number
}>()

function edgeColor(edgeType: string): string {
  switch (edgeType) {
    case 'Prerequisite': return '#5b8fff'
    case 'Semantic':     return '#5a648c'
    case 'UserDefined':  return '#f59e0b'
    case 'Context':
    default:             return '#4a5068'
  }
}
</script>

<template>
  <div
    class="compass-root"
    :style="{ left: `${centerX}px`, top: `${centerY}px` }"
  >
    <!-- SVG ring + lines -->
    <svg class="compass-svg" :width="COMPASS_RING_R * 2 + 40" :height="COMPASS_RING_R * 2 + 40"
         :style="{ left: `${-(COMPASS_RING_R + 20)}px`, top: `${-(COMPASS_RING_R + 20)}px` }">
      <g :transform="`translate(${COMPASS_RING_R + 20}, ${COMPASS_RING_R + 20})`">
        <!-- Faint ring -->
        <circle :r="COMPASS_RING_R" fill="none" stroke="rgba(255,255,255,0.06)" stroke-width="1" />
        <!-- Lines to each dot -->
        <line
          v-for="dot in dots"
          :key="`line-${dot.id}`"
          x1="0" y1="0"
          :x2="dot.screenX - centerX"
          :y2="dot.screenY - centerY"
          :stroke="edgeColor(dot.edgeType)"
          stroke-width="1"
          stroke-opacity="0.5"
        />
      </g>
    </svg>

    <!-- Dot labels -->
    <div
      v-for="(dot, i) in dots"
      :key="`dot-${dot.id}`"
      class="compass-dot-wrapper"
      :style="{ left: `${dot.screenX - centerX}px`, top: `${dot.screenY - centerY}px` }"
    >
      <div :class="['compass-dot', { active: i === activeIndex }]"
           :style="{ borderColor: edgeColor(dot.edgeType) }">
        <span class="dot-num">{{ dot.index }}</span>
      </div>
      <div class="dot-title">{{ dot.title }}</div>
    </div>
  </div>
</template>

<style scoped>
.compass-root {
  position: fixed;
  pointer-events: none;
  z-index: 150;
}

.compass-svg {
  position: absolute;
  overflow: visible;
}

.compass-dot-wrapper {
  position: absolute;
  transform: translate(-50%, -50%);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 3px;
}

.compass-dot {
  width: 20px;
  height: 20px;
  border-radius: 50%;
  background: rgba(10, 13, 24, 0.85);
  border: 1.5px solid #4a5068;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s;
}

.compass-dot.active {
  width: 26px;
  height: 26px;
  box-shadow: 0 0 10px currentColor;
  background: rgba(20, 25, 40, 0.95);
}

.dot-num {
  font-size: 9px;
  font-weight: 700;
  color: #e8eaf0;
  font-family: system-ui, sans-serif;
}

.dot-title {
  font-size: 9px;
  color: #c8cad6;
  white-space: nowrap;
  font-family: system-ui, sans-serif;
  text-shadow: 0 0 6px rgba(0,0,0,0.9);
  max-width: 80px;
  overflow: hidden;
  text-overflow: ellipsis;
}
</style>
