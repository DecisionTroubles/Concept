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

export function useEditorMode() {
  const graphStore = useGraphStore()

  function enterFly()   { mode.value = 'fly' }
  function enterNormal() { mode.value = 'normal' }
  function enterGraph() {
    if (graphStore.selectedNodeId) { mode.value = 'graph'; compassIndex.value = 0 }
  }
  function escapeFromCurrentMode() {
    if (mode.value === 'fly') { mode.value = 'normal' }
    else if (mode.value === 'graph') {
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
    const node = graphStore.selectedNode
    if (!node?.connections.length) return null
    compassIndex.value = (compassIndex.value + 1) % node.connections.length
    return node.connections[compassIndex.value].target_id
  }
  function tabPrev(): string | null {
    const node = graphStore.selectedNode
    if (!node?.connections.length) return null
    const len = node.connections.length
    compassIndex.value = (compassIndex.value - 1 + len) % len
    return node.connections[compassIndex.value].target_id
  }
  function jumpToNeighbor(n: number): string | null {
    const node = graphStore.selectedNode
    if (!node) return null
    const idx = n - 1
    if (idx < 0 || idx >= node.connections.length) return null
    compassIndex.value = idx
    return node.connections[idx].target_id
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
    setCompassState,
  }
}
