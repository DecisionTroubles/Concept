import * as THREE from 'three'
import { computed, type ComputedRef } from 'vue'
import type { SceneSnapshot } from '@/scene/model/sceneSnapshot'
import type { useCameraController } from '@/scene/controller/useCameraController'
import { useSceneHudState } from '@/scene/controller/useSceneHudState'
import { graphTrace } from '@/stores/graph/debug'

export type SceneDirection = 'left' | 'right' | 'up' | 'down'

interface SceneControllerOptions {
  snapshot: ComputedRef<SceneSnapshot>
  cameraController: ReturnType<typeof useCameraController>
}

export function useSceneController(options: SceneControllerOptions) {
  const graphStore = useGraphStore()
  const editorMode = useEditorMode()
  const hudState = useSceneHudState()

  const activeNodeId = computed(() => options.snapshot.value.activeNodeId)

  function requestNodeFocus(nodeId: string | null) {
    if (!nodeId) return
    const node = options.snapshot.value.nodes.find(item => item.id === nodeId)
    if (!node) return
    const target = new THREE.Vector3(node.x, node.y, node.z)
    const position = new THREE.Vector3(node.x + 8, node.y + 6, node.z + 12)
    options.cameraController.requestFocus({ target, position })
  }

  function clickNode(nodeId: string) {
    graphTrace('sceneController.clickNode', {
      nodeId,
      mode: options.snapshot.value.mode,
      selectedNodeId: graphStore.selectedNodeId,
    })
    if (options.snapshot.value.mode === 'solar') graphStore.selectFocusNode(nodeId)
    else graphStore.selectNode(nodeId)
    hudState.setActiveNode(nodeId)
    requestNodeFocus(nodeId)
  }

  function hoverNode(nodeId: string | null) {
    graphTrace('sceneController.hoverNode', { nodeId })
  }

  function clearSelection() {
    graphStore.clearSelection()
    hudState.clearHud()
  }

  function enterSolar(rootNodeId?: string | null) {
    graphStore.openFocusView(rootNodeId ?? graphStore.selectedNodeId)
  }

  function exitSolar() {
    graphStore.closeFocusView()
  }

  function toggleSolar(rootNodeId?: string | null) {
    graphStore.toggleFocusView(rootNodeId ?? graphStore.selectedNodeId)
  }

  function moveSolarCursor(direction: SceneDirection) {
    const currentId = activeNodeId.value
    if (!currentId) return
    const current = options.snapshot.value.nodes.find(node => node.id === currentId)
    if (!current) return
    const desired =
      direction === 'left'
        ? new THREE.Vector2(-1, 0)
        : direction === 'right'
          ? new THREE.Vector2(1, 0)
          : direction === 'up'
            ? new THREE.Vector2(0, -1)
            : new THREE.Vector2(0, 1)
    let best: { id: string; score: number } | null = null
    for (const node of options.snapshot.value.nodes) {
      if (node.id === currentId) continue
      const delta = new THREE.Vector2(node.x - current.x, node.z - current.z)
      const distance = delta.length()
      if (distance < 0.001) continue
      delta.normalize()
      const alignment = delta.dot(desired)
      if (alignment <= 0.15) continue
      const score = alignment * 10 - distance
      if (!best || score > best.score) best = { id: node.id, score }
    }
    if (!best) return
    graphStore.selectFocusNode(best.id)
    hudState.setActiveNode(best.id)
    requestNodeFocus(best.id)
  }

  function selectSolarCursor(nodeId: string) {
    graphStore.selectFocusNode(nodeId)
    hudState.setActiveNode(nodeId)
    requestNodeFocus(nodeId)
  }

  function cycleNeighbor(next: boolean) {
    const id = hudState.cycleNeighbor(next)
    if (!id) return
    if (graphStore.focusViewActive) graphStore.selectFocusNode(id)
    else graphStore.selectNode(id)
    requestNodeFocus(id)
  }

  function jumpToNeighbor(index1Based: number) {
    const id = hudState.jumpToNeighbor(index1Based)
    if (!id) return
    if (graphStore.focusViewActive) graphStore.selectFocusNode(id)
    else graphStore.selectNode(id)
    requestNodeFocus(id)
  }

  function jumpBack() {
    const id = editorMode.jumpBack()
    if (!id) return
    graphStore.selectNode(id)
    requestNodeFocus(id)
  }

  function openCenteredDetail() {
    graphStore.toggleCenteredNodePanel()
  }

  function closeCenteredDetail() {
    if (graphStore.centeredNodePanel) graphStore.toggleCenteredNodePanel()
  }

  function toggleCenteredDetail() {
    graphStore.toggleCenteredNodePanel()
  }

  function openNodeEditor() {
    graphStore.openNodeEditor()
  }

  function closeNodeEditor() {
    graphStore.closeNodeEditor()
  }

  function togglePin(nodeId: string) {
    graphStore.togglePinNode(nodeId)
  }

  async function switchLayer(layerId: string) {
    if (graphStore.focusViewActive) graphStore.closeFocusView()
    if (graphStore.activeLayerId === layerId) return
    await graphStore.loadNodes(layerId)
  }

  function toggleConnectionLayer(layerId: string) {
    graphStore.toggleConnectionLayer(layerId)
  }

  function setConnectionLayers(ids: string[]) {
    graphStore.setConnectionLayerSelection(ids)
  }

  function handleEscape() {
    if (graphStore.activeBuffer !== 'none') {
      graphStore.closeBuffer()
      return
    }
    if (graphStore.packLibraryOpen) {
      graphStore.closePackLibrary()
      return
    }
    if (graphStore.focusViewActive || graphStore.selectedNodeId) {
      clearSelection()
      return
    }
    editorMode.escapeFromCurrentMode()
  }

  return {
    clickNode,
    hoverNode,
    clearSelection,
    enterSolar,
    exitSolar,
    toggleSolar,
    moveSolarCursor,
    selectSolarCursor,
    cycleNeighbor,
    jumpToNeighbor,
    jumpBack,
    openCenteredDetail,
    closeCenteredDetail,
    toggleCenteredDetail,
    openNodeEditor,
    closeNodeEditor,
    togglePin,
    switchLayer,
    toggleConnectionLayer,
    setConnectionLayers,
    handleEscape,
    requestNodeFocus,
  }
}
