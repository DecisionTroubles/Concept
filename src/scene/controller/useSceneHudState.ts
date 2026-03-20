import { computed, readonly, ref, shallowRef } from 'vue'
import type { CompassHudModel } from '@/scene/model/hudModel'

export const COMPASS_RING_R = 110

const dots = shallowRef<CompassHudModel['dots']>([])
const center = shallowRef<CompassHudModel['center']>(null)
const neighborOrder = ref<string[]>([])
const activeIndex = ref(0)

function resetActiveIndex() {
  activeIndex.value = 0
}

function clearHud() {
  dots.value = []
  center.value = null
  neighborOrder.value = []
  activeIndex.value = 0
}

function updateHud(model: CompassHudModel) {
  dots.value = model.dots
  center.value = model.center
  neighborOrder.value = model.neighborOrder
  activeIndex.value = Math.max(0, Math.min(model.activeIndex, Math.max(model.neighborOrder.length - 1, 0)))
}

function setActiveNode(nodeId: string | null) {
  if (!nodeId) {
    activeIndex.value = 0
    return
  }
  const idx = neighborOrder.value.indexOf(nodeId)
  if (idx >= 0) activeIndex.value = idx
}

function cycleNeighbor(next: boolean): string | null {
  if (!neighborOrder.value.length) return null
  const len = neighborOrder.value.length
  activeIndex.value = next ? (activeIndex.value + 1) % len : (activeIndex.value - 1 + len) % len
  return neighborOrder.value[activeIndex.value] ?? null
}

function jumpToNeighbor(index1Based: number): string | null {
  const idx = index1Based - 1
  const id = neighborOrder.value[idx] ?? null
  if (id) activeIndex.value = idx
  return id
}

const hasHud = computed(() => !!center.value && dots.value.length > 0)

export function useSceneHudState() {
  return {
    dots,
    center,
    neighborOrder: readonly(neighborOrder),
    activeIndex: readonly(activeIndex),
    hasHud,
    clearHud,
    updateHud,
    setActiveNode,
    resetActiveIndex,
    cycleNeighbor,
    jumpToNeighbor,
  }
}
