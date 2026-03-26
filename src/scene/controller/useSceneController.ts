import * as THREE from 'three'
import { computed, ref, type ComputedRef } from 'vue'
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
  const placementPreview = ref<{ x: number; y: number; z: number } | null>(null)
  const connectSourceNodeId = ref<string | null>(null)
  const autoConnectNewNodes = ref(false)
  const lastCreatedNodeId = ref<string | null>(null)
  const isEditableWorld = computed(() => {
    const activeWorldId = graphStore.worldConfig?.id
    if (!activeWorldId) return false
    const activeWorld = graphStore.worldPacks.find(world => world.world_id === activeWorldId)
    return activeWorld?.source_kind === 'local'
  })
  const authorActive = computed(() => editorMode.mode.value === 'author' && isEditableWorld.value)
  const activeConnectionLayerId = computed(
    () => graphStore.activeConnectionLayerIds[0] ?? graphStore.connectionLayers[0]?.id ?? null
  )
  const defaultNoteTypeId = computed(
    () => graphStore.noteTypes.find(noteType => noteType.is_default)?.id ?? graphStore.noteTypes[0]?.id ?? null
  )
  const activeLayerId = computed(() => graphStore.activeLayerId ?? graphStore.layers[0]?.id ?? null)

  function requestNodeFocus(nodeId: string | null) {
    if (!nodeId) return
    const node = options.snapshot.value.nodes.find(item => item.id === nodeId)
    if (!node) return
    const target = new THREE.Vector3(node.x, node.y, node.z)
    const position = new THREE.Vector3(node.x + 8, node.y + 6, node.z + 12)
    options.cameraController.requestFocus({ target, position })
  }

  async function clickNode(nodeId: string) {
    graphTrace('sceneController.clickNode', {
      nodeId,
      mode: options.snapshot.value.mode,
      selectedNodeId: graphStore.selectedNodeId,
    })
    if (authorActive.value && connectSourceNodeId.value && connectSourceNodeId.value !== nodeId) {
      await graphStore.createEdge(
        connectSourceNodeId.value,
        nodeId,
        'Semantic',
        activeConnectionLayerId.value
      )
      connectSourceNodeId.value = null
    }
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

  function setPlacementPreview(position: { x: number; y: number; z: number } | null) {
    if (!authorActive.value) {
      placementPreview.value = null
      return
    }
    placementPreview.value = position
  }

  function patchNodePosition(id: string, x: number, y: number, z: number) {
    const node = graphStore.nodes.find(item => item.id === id)
    if (node) {
      node.pos_x = x
      node.pos_y = y
      node.pos_z = z
    }
    graphStore.updateNodePosition(id, x, y, z)
  }

  async function createNodeAt(position: { x: number; y: number; z: number }) {
    if (!authorActive.value || !activeLayerId.value) return
    const node = await graphStore.createNode({
      title: 'New node',
      layer_id: activeLayerId.value,
      parent_node_id: null,
      node_type: 'concept',
      note_type_id: defaultNoteTypeId.value,
      note_fields: {},
      content_data: null,
      tags: [],
      weight: 1.0,
    })
    patchNodePosition(node.id, position.x, position.y, position.z)
    if (autoConnectNewNodes.value && lastCreatedNodeId.value && lastCreatedNodeId.value !== node.id) {
      await graphStore.createEdge(lastCreatedNodeId.value, node.id, 'Semantic', activeConnectionLayerId.value)
    }
    lastCreatedNodeId.value = node.id
    graphStore.selectNode(node.id)
    graphStore.openNodeEditor()
    requestNodeFocus(node.id)
  }

  async function clickBackground(position: { x: number; y: number; z: number }) {
    if (!authorActive.value) return
    await createNodeAt(position)
  }

  function startConnectFromSelected() {
    if (!authorActive.value || !graphStore.selectedNodeId) return
    connectSourceNodeId.value = graphStore.selectedNodeId
  }

  function cancelConnectMode() {
    connectSourceNodeId.value = null
  }

  function toggleAutoConnectNewNodes() {
    autoConnectNewNodes.value = !autoConnectNewNodes.value
  }

  function nudgeSelected(dx: number, dy: number, dz: number, scale = 1) {
    if (!authorActive.value || !graphStore.selectedNodeId) return
    const node = graphStore.nodes.find(item => item.id === graphStore.selectedNodeId)
    if (!node) return
    patchNodePosition(
      node.id,
      (node.pos_x ?? 0) + dx * scale,
      (node.pos_y ?? 0) + dy * scale,
      (node.pos_z ?? 0) + dz * scale
    )
  }

  function spreadSelected(radius = 10, strength = 3) {
    if (!authorActive.value || !graphStore.selectedNodeId) return
    const selected = graphStore.nodes.find(item => item.id === graphStore.selectedNodeId)
    if (!selected) return
    const sx = selected.pos_x ?? 0
    const sy = selected.pos_y ?? 0
    const sz = selected.pos_z ?? 0
    for (const node of graphStore.nodes) {
      if (node.id === selected.id) continue
      const dx = (node.pos_x ?? 0) - sx
      const dy = (node.pos_y ?? 0) - sy
      const dz = (node.pos_z ?? 0) - sz
      const distance = Math.sqrt(dx * dx + dy * dy + dz * dz)
      if (distance < 0.001 || distance > radius) continue
      const push = strength * (1 - distance / radius)
      patchNodePosition(
        node.id,
        (node.pos_x ?? 0) + (dx / distance) * push,
        (node.pos_y ?? 0) + (dy / distance) * push,
        (node.pos_z ?? 0) + (dz / distance) * push
      )
    }
  }

  function expandNeighborSpacing(strength = 2.5) {
    if (!authorActive.value || !graphStore.selectedNodeId) return
    const selected = graphStore.nodes.find(item => item.id === graphStore.selectedNodeId)
    if (!selected) return
    const neighborIds = new Set(selected.connections.map(edge => edge.target_id))
    const neighborhood = graphStore.nodes.filter(node => node.id === selected.id || neighborIds.has(node.id))
    if (neighborhood.length <= 1) return
    const centroid = neighborhood.reduce(
      (acc, node) => {
        acc.x += node.pos_x ?? 0
        acc.y += node.pos_y ?? 0
        acc.z += node.pos_z ?? 0
        return acc
      },
      { x: 0, y: 0, z: 0 }
    )
    centroid.x /= neighborhood.length
    centroid.y /= neighborhood.length
    centroid.z /= neighborhood.length
    for (const node of neighborhood) {
      if (node.id === selected.id) continue
      const dx = (node.pos_x ?? 0) - centroid.x
      const dy = (node.pos_y ?? 0) - centroid.y
      const dz = (node.pos_z ?? 0) - centroid.z
      const distance = Math.sqrt(dx * dx + dy * dy + dz * dz)
      if (distance < 0.001) continue
      patchNodePosition(
        node.id,
        (node.pos_x ?? 0) + (dx / distance) * strength,
        (node.pos_y ?? 0) + (dy / distance) * strength,
        (node.pos_z ?? 0) + (dz / distance) * strength
      )
    }
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
    if (connectSourceNodeId.value) {
      connectSourceNodeId.value = null
      return
    }
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
    authorActive,
    isEditableWorld,
    placementPreview,
    connectSourceNodeId,
    autoConnectNewNodes,
    setPlacementPreview,
    clickBackground,
    createNodeAt,
    startConnectFromSelected,
    cancelConnectMode,
    toggleAutoConnectNewNodes,
    nudgeSelected,
    spreadSelected,
    expandNeighborSpacing,
    handleEscape,
    requestNodeFocus,
  }
}
