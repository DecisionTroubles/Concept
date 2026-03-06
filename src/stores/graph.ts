import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { ConnectionLayer, Layer, Node, NodeProgress, NoteType, NoteTypeInput, RelationKind, ReviewEvent, SchedulerDescriptor, WorldConfig } from '@/bindings'
import { useTauRPC } from '@/composables/useTauRPC'
import { useSettings } from '@/composables/useSettings'

export type BufferId = 'none' | 'pinned' | 'map'
const CONNECTION_LAYER_SELECTION_KEY = 'concept:connection-layer-selection'

export const useGraphStore = defineStore('graph', () => {
  const settings = useSettings()
  const layers = ref<Layer[]>([])
  const activeLayerId = ref<string | null>(null)
  const nodes = ref<Node[]>([])
  const selectedNodeId = ref<string | null>(null)
  const noteTypes = ref<NoteType[]>([])
  const nodeProgress = ref<NodeProgress[]>([])
  const schedulerAlgorithms = ref<SchedulerDescriptor[]>([])
  const reviewEvents = ref<ReviewEvent[]>([])
  const worldConfig = ref<WorldConfig | null>(null)
  const relationKinds = ref<RelationKind[]>([])
  const connectionLayers = ref<ConnectionLayer[]>([])
  const activeConnectionLayerIds = ref<string[]>([])
  const connectionLayerSelectionInitialized = ref(false)
  const centeredNodePanel = ref(false)
  const pinnedNodeIds = ref<string[]>([])
  const activeBuffer = ref<BufferId>('none')
  const progressOverlayOpen = ref(false)
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
  const dueNodes = computed(() =>
    nodes.value.filter(n => {
      if (n.progress_status === 'mastered') {
        return n.progress_next_review_at ? Number(n.progress_next_review_at) <= Date.now() / 1000 : false
      }
      if (n.progress_status === 'new') return true
      if (!n.progress_next_review_at) return true
      return Number(n.progress_next_review_at) <= Date.now() / 1000
    })
  )

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

  function openProgressOverlay() {
    progressOverlayOpen.value = true
  }

  function closeProgressOverlay() {
    progressOverlayOpen.value = false
  }

  function toggleProgressOverlay() {
    progressOverlayOpen.value = !progressOverlayOpen.value
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

  async function loadNodeProgress() {
    try {
      nodeProgress.value = await useTauRPC().get_node_progress()
    } catch (e) {
      error.value = String(e)
    }
  }

  async function loadSchedulerAlgorithms() {
    try {
      schedulerAlgorithms.value = await useTauRPC().get_scheduler_algorithms()
      if (!schedulerAlgorithms.value.some(x => x.key === settings.learning.defaultSchedulerKey) && schedulerAlgorithms.value[0]) {
        settings.setDefaultSchedulerKey(schedulerAlgorithms.value[0].key)
      }
    } catch (e) {
      error.value = String(e)
    }
  }

  async function loadReviewEvents() {
    try {
      reviewEvents.value = await useTauRPC().get_review_events()
    } catch (e) {
      error.value = String(e)
    }
  }

  async function loadConnectionLayers() {
    try {
      connectionLayers.value = await useTauRPC().get_connection_layers()
      const valid = new Set(connectionLayers.value.map(l => l.id))

      if (!connectionLayerSelectionInitialized.value) {
        connectionLayerSelectionInitialized.value = true
        const saved = loadConnectionLayerSelection()
        if (saved) {
          activeConnectionLayerIds.value = saved.filter(id => valid.has(id))
          if (activeConnectionLayerIds.value.length === 0 && connectionLayers.value[0]) {
            activeConnectionLayerIds.value = [connectionLayers.value[0].id]
          }
        } else {
          // First boot default: only the first connection layer is visible.
          activeConnectionLayerIds.value = connectionLayers.value[0] ? [connectionLayers.value[0].id] : []
        }
      } else {
        // Preserve explicit user choices, including empty selection.
        activeConnectionLayerIds.value = activeConnectionLayerIds.value.filter(id => valid.has(id))
      }

      saveConnectionLayerSelection(activeConnectionLayerIds.value)
    } catch (e) {
      error.value = String(e)
    }
  }

  function toggleConnectionLayer(id: string) {
    const ids = activeConnectionLayerIds.value
    const idx = ids.indexOf(id)
    if (idx === -1) {
      activeConnectionLayerIds.value = [...ids, id]
    } else {
      activeConnectionLayerIds.value = ids.filter(x => x !== id)
    }
    saveConnectionLayerSelection(activeConnectionLayerIds.value)
  }

  function loadConnectionLayerSelection(): string[] | null {
    try {
      const raw = localStorage.getItem(CONNECTION_LAYER_SELECTION_KEY)
      if (!raw) return null
      const parsed = JSON.parse(raw)
      if (!Array.isArray(parsed)) return null
      return parsed.filter((x): x is string => typeof x === 'string')
    } catch {
      return null
    }
  }

  function saveConnectionLayerSelection(ids: string[]) {
    try {
      localStorage.setItem(CONNECTION_LAYER_SELECTION_KEY, JSON.stringify(ids))
    } catch {
      // ignore localStorage failures
    }
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

  async function createNoteType(input: NoteTypeInput) {
    try {
      const created = await useTauRPC().create_note_type(input)
      noteTypes.value.push(created)
      noteTypes.value.sort((a, b) => a.name.localeCompare(b.name))
      return created
    } catch (e) {
      error.value = String(e)
      throw e
    }
  }

  async function updateNoteType(id: string, input: NoteTypeInput) {
    try {
      const updated = await useTauRPC().update_note_type(id, input)
      const idx = noteTypes.value.findIndex(n => n.id === id)
      if (idx !== -1) noteTypes.value[idx] = updated
      return updated
    } catch (e) {
      error.value = String(e)
      throw e
    }
  }

  async function duplicateNoteType(sourceId: string, name: string, worldId: string | null = null) {
    try {
      const duplicated = await useTauRPC().duplicate_note_type(sourceId, name, worldId)
      noteTypes.value.push(duplicated)
      noteTypes.value.sort((a, b) => a.name.localeCompare(b.name))
      return duplicated
    } catch (e) {
      error.value = String(e)
      throw e
    }
  }

  async function updateNodeContent(
    nodeId: string,
    title: string,
    noteFields: Record<string, string>,
    contentData: string | null,
    tags: string[]
  ) {
    try {
      const updated = await useTauRPC().update_node_content(nodeId, title, noteFields, contentData, tags)
      const idx = nodes.value.findIndex(n => n.id === nodeId)
      if (idx !== -1) nodes.value[idx] = updated
      return updated
    } catch (e) {
      error.value = String(e)
      throw e
    }
  }

  async function setNodeProgressStatus(nodeId: string, status: string) {
    try {
      const updated = await useTauRPC().set_node_progress_status(nodeId, status)
      const idx = nodes.value.findIndex(n => n.id === nodeId)
      if (idx !== -1) nodes.value[idx] = updated
      await loadNodeProgress()
      await loadReviewEvents()
    } catch (e) {
      error.value = String(e)
    }
  }

  async function reviewNode(nodeId: string, grade: string, schedulerKey: string | null = null) {
    try {
      const updated = await useTauRPC().review_node(nodeId, grade, schedulerKey ?? settings.learning.defaultSchedulerKey)
      const idx = nodes.value.findIndex(n => n.id === nodeId)
      if (idx !== -1) nodes.value[idx] = updated
      await loadNodeProgress()
      await loadReviewEvents()
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
    await loadSchedulerAlgorithms()
    await loadNodeProgress()
    await loadReviewEvents()
    await loadLayers()
    if (layers.value[0]) await loadNodes(layers.value[0].id)
  }

  async function resetGraphData() {
    isLoading.value = true
    try {
      await useTauRPC().reset_data(true)
      selectedNodeId.value = null
      centeredNodePanel.value = false
      pinnedNodeIds.value = []
      await loadWorldConfig()
      await loadRelationKinds()
      await loadConnectionLayers()
      await loadNoteTypes()
      await loadSchedulerAlgorithms()
      await loadNodeProgress()
      await loadReviewEvents()
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
    nodeProgress,
    schedulerAlgorithms,
    reviewEvents,
    worldConfig,
    relationKinds,
    connectionLayers,
    activeConnectionLayerIds,
    selectedNodeId,
    centeredNodePanel,
    pinnedNodeIds,
    pinnedNodes,
    activeBuffer,
    progressOverlayOpen,
    selectedNode,
    dueNodes,
    isLoading,
    error,
    loadLayers,
    loadNoteTypes,
    loadSchedulerAlgorithms,
    loadNodeProgress,
    loadReviewEvents,
    loadWorldConfig,
    loadRelationKinds,
    loadConnectionLayers,
    loadNodes,
    markLearned,
    updateNodePosition,
    setNodeNoteType,
    createNoteType,
    updateNoteType,
    duplicateNoteType,
    updateNodeContent,
    setNodeProgressStatus,
    reviewNode,
    selectNode,
    toggleCenteredNodePanel,
    isNodePinned,
    togglePinNode,
    unpinNode,
    clearPinnedNodes,
    closeBuffer,
    openBuffer,
    toggleBuffer,
    openProgressOverlay,
    closeProgressOverlay,
    toggleProgressOverlay,
    toggleConnectionLayer,
    focusVersion,
    requestFocus,
    initialize,
    resetGraphData,
  }
})
