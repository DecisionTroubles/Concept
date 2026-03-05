import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { ConnectionLayer, Layer, Node, NoteType, RelationKind, WorldConfig } from '@/bindings'
import { useTauRPC } from '@/composables/useTauRPC'

export type BufferId = 'none' | 'pinned' | 'map'

export const useGraphStore = defineStore('graph', () => {
  const layers = ref<Layer[]>([])
  const activeLayerId = ref<string | null>(null)
  const nodes = ref<Node[]>([])
  const selectedNodeId = ref<string | null>(null)
  const noteTypes = ref<NoteType[]>([])
  const worldConfig = ref<WorldConfig | null>(null)
  const relationKinds = ref<RelationKind[]>([])
  const connectionLayers = ref<ConnectionLayer[]>([])
  const activeConnectionLayerIds = ref<string[]>([])
  const centeredNodePanel = ref(false)
  const pinnedNodeIds = ref<string[]>([])
  const activeBuffer = ref<BufferId>('none')
  const focusVersion = ref(0)
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  const selectedNode = computed(() =>
    selectedNodeId.value ? (nodes.value.find(n => n.id === selectedNodeId.value) ?? null) : null
  )
  const pinnedNodes = computed(() => {
    const set = new Set(pinnedNodeIds.value)
    return nodes.value.filter(n => set.has(n.id))
  })

  function selectNode(id: string | null) {
    selectedNodeId.value = id
    if (!id) {
      centeredNodePanel.value = false
    }
  }

  function requestFocus(id: string) {
    selectedNodeId.value = id
    focusVersion.value++
  }

  function toggleCenteredNodePanel() {
    if (!selectedNodeId.value) return
    centeredNodePanel.value = !centeredNodePanel.value
  }

  function isNodePinned(id: string | null | undefined): boolean {
    if (!id) return false
    return pinnedNodeIds.value.includes(id)
  }

  function togglePinNode(id: string) {
    const idx = pinnedNodeIds.value.indexOf(id)
    if (idx === -1) pinnedNodeIds.value.push(id)
    else pinnedNodeIds.value.splice(idx, 1)
  }

  function unpinNode(id: string) {
    const idx = pinnedNodeIds.value.indexOf(id)
    if (idx !== -1) pinnedNodeIds.value.splice(idx, 1)
  }

  function clearPinnedNodes() {
    pinnedNodeIds.value = []
  }

  function closeBuffer() {
    activeBuffer.value = 'none'
  }

  function openBuffer(buffer: Exclude<BufferId, 'none'>) {
    activeBuffer.value = buffer
  }

  function toggleBuffer(buffer: Exclude<BufferId, 'none'>) {
    activeBuffer.value = activeBuffer.value === buffer ? 'none' : buffer
  }

  async function loadLayers() {
    isLoading.value = true
    try {
      layers.value = await useTauRPC().get_layers()
    } catch (e) {
      error.value = String(e)
    } finally {
      isLoading.value = false
    }
  }

  async function loadWorldConfig() {
    try {
      worldConfig.value = await useTauRPC().get_world_config()
    } catch (e) {
      error.value = String(e)
    }
  }

  async function loadRelationKinds() {
    try {
      relationKinds.value = await useTauRPC().get_relation_kinds()
    } catch (e) {
      error.value = String(e)
    }
  }

  async function loadNoteTypes() {
    try {
      noteTypes.value = await useTauRPC().get_note_types()
    } catch (e) {
      error.value = String(e)
    }
  }

  async function loadConnectionLayers() {
    try {
      connectionLayers.value = await useTauRPC().get_connection_layers()
      if (activeConnectionLayerIds.value.length === 0) {
        activeConnectionLayerIds.value = connectionLayers.value.map(l => l.id)
      } else {
        const valid = new Set(connectionLayers.value.map(l => l.id))
        activeConnectionLayerIds.value = activeConnectionLayerIds.value.filter(id => valid.has(id))
      }
    } catch (e) {
      error.value = String(e)
    }
  }

  function toggleConnectionLayer(id: string) {
    const idx = activeConnectionLayerIds.value.indexOf(id)
    if (idx === -1) activeConnectionLayerIds.value.push(id)
    else activeConnectionLayerIds.value.splice(idx, 1)
  }

  async function loadNodes(layerId: string) {
    activeLayerId.value = layerId
    selectedNodeId.value = null
    centeredNodePanel.value = false
    isLoading.value = true
    try {
      nodes.value = await useTauRPC().get_nodes(layerId)
    } catch (e) {
      error.value = String(e)
    } finally {
      isLoading.value = false
    }
  }

  async function markLearned(id: string, learned: boolean = true) {
    try {
      const updated = await useTauRPC().mark_learned(id, learned)
      const idx = nodes.value.findIndex(n => n.id === id)
      if (idx !== -1) nodes.value[idx] = updated
    } catch (e) {
      error.value = String(e)
    }
  }

  async function updateNodePosition(id: string, x: number, y: number, z: number) {
    try {
      await useTauRPC().update_node_position(id, x, y, z)
    } catch {
      // Non-critical — don't surface position errors to the user
    }
  }

  async function setNodeNoteType(nodeId: string, noteTypeId: string | null) {
    try {
      const updated = await useTauRPC().set_node_note_type(nodeId, noteTypeId)
      const idx = nodes.value.findIndex(n => n.id === nodeId)
      if (idx !== -1) nodes.value[idx] = updated
    } catch (e) {
      error.value = String(e)
    }
  }

  async function initialize() {
    await useTauRPC().seed_sample_data()
    await loadWorldConfig()
    await loadRelationKinds()
    await loadConnectionLayers()
    await loadNoteTypes()
    await loadLayers()
    if (layers.value[0]) await loadNodes(layers.value[0].id)
  }

  async function resetGraphData() {
    isLoading.value = true
    try {
      await useTauRPC().reset_data()
      selectedNodeId.value = null
      centeredNodePanel.value = false
      pinnedNodeIds.value = []
      await loadWorldConfig()
      await loadRelationKinds()
      await loadConnectionLayers()
      await loadNoteTypes()
      await loadLayers()
      if (layers.value[0]) await loadNodes(layers.value[0].id)
    } catch (e) {
      error.value = String(e)
    } finally {
      isLoading.value = false
    }
  }

  return {
    layers,
    activeLayerId,
    nodes,
    noteTypes,
    worldConfig,
    relationKinds,
    connectionLayers,
    activeConnectionLayerIds,
    selectedNodeId,
    centeredNodePanel,
    pinnedNodeIds,
    pinnedNodes,
    activeBuffer,
    selectedNode,
    isLoading,
    error,
    loadLayers,
    loadNoteTypes,
    loadWorldConfig,
    loadRelationKinds,
    loadConnectionLayers,
    loadNodes,
    markLearned,
    updateNodePosition,
    setNodeNoteType,
    selectNode,
    toggleCenteredNodePanel,
    isNodePinned,
    togglePinNode,
    unpinNode,
    clearPinnedNodes,
    closeBuffer,
    openBuffer,
    toggleBuffer,
    toggleConnectionLayer,
    focusVersion,
    requestFocus,
    initialize,
    resetGraphData,
  }
})
