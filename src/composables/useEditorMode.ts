import { ref, readonly, shallowRef } from 'vue'

export type EditorMode = 'normal' | 'fly' | 'graph'
export const COMPASS_RING_R = 110   // px — shared with CompassHUD

export interface CompassDot {
  id: string
  title: string
  screenX: number
  screenY: number
  edgeType: string
  index: number
}

const mode = ref<EditorMode>('normal')
const compassIndex = ref(0)         // Tab position in connections[]
const compassDots = shallowRef<CompassDot[]>([])
const compassCenter = shallowRef<{ x: number; y: number } | null>(null)

// Jump list — module-level, not reactive (no need to render)
const jumpList: string[] = []   // max 20 entries
let lastNodeId: string | null = null

// Ordered neighbor IDs matching compass dot indices (1-based).
// Set every frame by GraphScene from the filtered, sequential compass dots.
let neighborOrder: string[] = []

export function useEditorMode() {
  function enterFly()   { mode.value = 'fly' }
  function enterNormal() { mode.value = 'normal' }
  function enterGraph() {
    const graphStore = useGraphStore()
    if (graphStore.selectedNodeId) { mode.value = 'graph'; compassIndex.value = 0 }
  }
  function escapeFromCurrentMode() {
    if (mode.value === 'fly') { mode.value = 'normal' }
    else if (mode.value === 'graph') {
      const graphStore = useGraphStore()
      graphStore.selectNode(null); mode.value = 'normal'; compassIndex.value = 0
    }
  }
  function onNodeSelected(id: string | null) {
    if (id) {
      if (lastNodeId && lastNodeId !== id) {
        jumpList.push(lastNodeId)
        if (jumpList.length > 20) jumpList.shift()
      }
      lastNodeId = id
      mode.value = 'graph'; compassIndex.value = 0
    } else if (mode.value === 'graph') {
      lastNodeId = null
      mode.value = 'normal'; compassIndex.value = 0
    }
  }
  function jumpBack(): string | null {
    return jumpList.pop() ?? null
  }
  function tabNext(): string | null {
    if (!neighborOrder.length) return null
    compassIndex.value = (compassIndex.value + 1) % neighborOrder.length
    return neighborOrder[compassIndex.value] ?? null
  }
  function tabPrev(): string | null {
    if (!neighborOrder.length) return null
    const len = neighborOrder.length
    compassIndex.value = (compassIndex.value - 1 + len) % len
    return neighborOrder[compassIndex.value] ?? null
  }
  function jumpToNeighbor(n: number): string | null {
    return neighborOrder[n - 1] ?? null
  }
  function setNeighborOrder(ids: string[]) {
    neighborOrder = ids
  }
  function setCompassState(dots: CompassDot[], center: { x: number; y: number } | null) {
    compassDots.value = dots
    compassCenter.value = center
  }

  return {
    mode: readonly(mode),
    compassIndex: readonly(compassIndex),
    compassDots,
    compassCenter,
    enterFly, enterNormal, enterGraph, escapeFromCurrentMode,
    onNodeSelected, tabNext, tabPrev, jumpToNeighbor, jumpBack,
    setNeighborOrder, setCompassState,
  }
}
