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

function arraysEqual(a: string[], b: string[]): boolean {
  if (a === b) return true
  if (a.length !== b.length) return false
  for (let i = 0; i < a.length; i += 1) {
    if (a[i] !== b[i]) return false
  }
  return true
}

function compassDotsEqual(a: CompassDot[], b: CompassDot[]): boolean {
  if (a === b) return true
  if (a.length !== b.length) return false
  for (let i = 0; i < a.length; i += 1) {
    const left = a[i]
    const right = b[i]
    if (
      left.id !== right.id ||
      left.title !== right.title ||
      left.edgeType !== right.edgeType ||
      left.index !== right.index ||
      Math.abs(left.screenX - right.screenX) > 0.5 ||
      Math.abs(left.screenY - right.screenY) > 0.5
    ) {
      return false
    }
  }
  return true
}

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
    if (arraysEqual(neighborOrder, ids)) return
    neighborOrder = ids
  }
  function setCompassState(dots: CompassDot[], center: { x: number; y: number } | null) {
    const currentCenter = compassCenter.value
    const sameCenter =
      currentCenter === center ||
      (!!currentCenter && !!center && Math.abs(currentCenter.x - center.x) < 0.5 && Math.abs(currentCenter.y - center.y) < 0.5)
    if (sameCenter && compassDotsEqual(compassDots.value, dots)) return
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
