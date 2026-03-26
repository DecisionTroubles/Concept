import type { CreateNodeInput, NoteTypeInput } from '@/bindings'
import type { GraphResourceState, GraphSessionState, GraphStatusState } from '@/stores/graph/shared'
import { useTauRPC } from '@/composables/useTauRPC'
import type { ReturnTypeUseSettings } from '@/stores/graph/types'

const CONNECTION_LAYER_SELECTION_KEY = 'concept:connection-layer-selection'

export interface GraphResourceActionsOptions {
  state: GraphResourceState
  session: GraphSessionState
  status: GraphStatusState
  settings: ReturnTypeUseSettings
  resetInteractiveState: () => void
}

export function createGraphResourceActions(options: GraphResourceActionsOptions) {
  const { state, session, status, settings, resetInteractiveState } = options

  async function loadLayers() {
    status.isLoading.value = true
    try {
      state.layers.value = await useTauRPC().get_layers()
    } catch (e) {
      status.error.value = String(e)
    } finally {
      status.isLoading.value = false
    }
  }

  async function loadWorldConfig() {
    try {
      state.worldConfig.value = await useTauRPC().get_world_config()
    } catch (e) {
      status.error.value = String(e)
    }
  }

  async function loadWorldPacks() {
    try {
      state.worldPacks.value = await useTauRPC().get_world_packs()
    } catch (e) {
      status.error.value = String(e)
    }
  }

  async function loadPackRegistry() {
    try {
      state.packRegistry.value = await useTauRPC().get_pack_registry()
    } catch (e) {
      status.error.value = String(e)
    }
  }

  async function loadRelationKinds() {
    try {
      state.relationKinds.value = await useTauRPC().get_relation_kinds()
    } catch (e) {
      status.error.value = String(e)
    }
  }

  async function loadNoteTypes() {
    try {
      state.noteTypes.value = await useTauRPC().get_note_types()
    } catch (e) {
      status.error.value = String(e)
    }
  }

  async function loadNodeProgress() {
    try {
      state.nodeProgress.value = await useTauRPC().get_node_progress()
    } catch (e) {
      status.error.value = String(e)
    }
  }

  async function loadSchedulerAlgorithms() {
    try {
      state.schedulerAlgorithms.value = await useTauRPC().get_scheduler_algorithms()
      if (
        !state.schedulerAlgorithms.value.some(x => x.key === settings.learning.defaultSchedulerKey) &&
        state.schedulerAlgorithms.value[0]
      ) {
        settings.setDefaultSchedulerKey(state.schedulerAlgorithms.value[0].key)
      }
    } catch (e) {
      status.error.value = String(e)
    }
  }

  async function loadReviewEvents() {
    try {
      state.reviewEvents.value = await useTauRPC().get_review_events()
    } catch (e) {
      status.error.value = String(e)
    }
  }

  async function loadConnectionLayers() {
    try {
      state.connectionLayers.value = await useTauRPC().get_connection_layers()
      const valid = new Set(state.connectionLayers.value.map(l => l.id))

      if (!state.connectionLayerSelectionInitialized.value) {
        state.connectionLayerSelectionInitialized.value = true
        const saved = loadConnectionLayerSelection()
        if (saved) {
          state.activeConnectionLayerIds.value = saved.filter(id => valid.has(id))
          if (state.activeConnectionLayerIds.value.length === 0 && state.connectionLayers.value[0]) {
            state.activeConnectionLayerIds.value = [state.connectionLayers.value[0].id]
          }
        } else {
          state.activeConnectionLayerIds.value = state.connectionLayers.value[0] ? [state.connectionLayers.value[0].id] : []
        }
      } else {
        state.activeConnectionLayerIds.value = state.activeConnectionLayerIds.value.filter(id => valid.has(id))
      }

      saveConnectionLayerSelection(state.activeConnectionLayerIds.value)
    } catch (e) {
      status.error.value = String(e)
    }
  }

  function toggleConnectionLayer(id: string) {
    const ids = state.activeConnectionLayerIds.value
    const idx = ids.indexOf(id)
    if (idx === -1) {
      state.activeConnectionLayerIds.value = [...ids, id]
    } else {
      state.activeConnectionLayerIds.value = ids.filter(x => x !== id)
    }
    saveConnectionLayerSelection(state.activeConnectionLayerIds.value)
  }

  function setConnectionLayerSelection(ids: string[]) {
    const valid = new Set(state.connectionLayers.value.map(layer => layer.id))
    const next = ids.filter(id => valid.has(id))
    const current = state.activeConnectionLayerIds.value
    if (next.length === current.length && next.every((id, index) => id === current[index])) return
    state.activeConnectionLayerIds.value = next
    saveConnectionLayerSelection(state.activeConnectionLayerIds.value)
  }

  async function loadNodes(layerId: string) {
    state.activeLayerId.value = layerId
    resetInteractiveState()
    status.isLoading.value = true
    try {
      state.nodes.value = await useTauRPC().get_nodes(layerId)
    } catch (e) {
      status.error.value = String(e)
    } finally {
      status.isLoading.value = false
    }
  }

  async function markLearned(id: string, learned = true) {
    try {
      const updated = await useTauRPC().mark_learned(id, learned)
      const idx = state.nodes.value.findIndex(n => n.id === id)
      if (idx !== -1) state.nodes.value[idx] = updated
    } catch (e) {
      status.error.value = String(e)
    }
  }

  async function updateNodePosition(id: string, x: number, y: number, z: number) {
    try {
      await useTauRPC().update_node_position(id, x, y, z)
    } catch {
      // Non-critical — don't surface position errors to the user.
    }
  }

  async function createNode(input: CreateNodeInput) {
    try {
      const created = await useTauRPC().create_node(input)
      if (state.activeLayerId.value === created.layer_id) {
        state.nodes.value = [...state.nodes.value, created]
      }
      return created
    } catch (e) {
      status.error.value = String(e)
      throw e
    }
  }

  async function createLayer(name: string, displayOrder: number) {
    try {
      const created = await useTauRPC().create_layer(name, displayOrder)
      state.layers.value = [...state.layers.value, created].sort((a, b) => a.display_order - b.display_order)
      return created
    } catch (e) {
      status.error.value = String(e)
      throw e
    }
  }

  async function createConnectionLayer(
    id: string | null,
    name: string,
    displayOrder: number,
    metadata: string | null = null,
  ) {
    try {
      const created = await useTauRPC().create_connection_layer(id, name, displayOrder, metadata)
      state.connectionLayers.value = [...state.connectionLayers.value, created].sort(
        (a, b) => a.display_order - b.display_order || a.name.localeCompare(b.name)
      )
      return created
    } catch (e) {
      status.error.value = String(e)
      throw e
    }
  }

  async function createEdge(sourceId: string, targetId: string, edgeType: string, connectionLayerId: string | null = null) {
    try {
      const created = await useTauRPC().create_edge(sourceId, targetId, edgeType, connectionLayerId)
      if (state.activeLayerId.value) {
        state.nodes.value = await useTauRPC().get_nodes(state.activeLayerId.value)
      }
      return created
    } catch (e) {
      status.error.value = String(e)
      throw e
    }
  }

  async function setNodeNoteType(nodeId: string, noteTypeId: string | null) {
    try {
      const updated = await useTauRPC().set_node_note_type(nodeId, noteTypeId)
      const idx = state.nodes.value.findIndex(n => n.id === nodeId)
      if (idx !== -1) state.nodes.value[idx] = updated
    } catch (e) {
      status.error.value = String(e)
    }
  }

  async function createNoteType(input: NoteTypeInput) {
    try {
      const created = await useTauRPC().create_note_type(input)
      state.noteTypes.value.push(created)
      state.noteTypes.value.sort((a, b) => a.name.localeCompare(b.name))
      return created
    } catch (e) {
      status.error.value = String(e)
      throw e
    }
  }

  async function updateNoteType(id: string, input: NoteTypeInput) {
    try {
      const updated = await useTauRPC().update_note_type(id, input)
      const idx = state.noteTypes.value.findIndex(n => n.id === id)
      if (idx !== -1) state.noteTypes.value[idx] = updated
      return updated
    } catch (e) {
      status.error.value = String(e)
      throw e
    }
  }

  async function duplicateNoteType(sourceId: string, name: string, worldId: string | null = null) {
    try {
      const duplicated = await useTauRPC().duplicate_note_type(sourceId, name, worldId)
      state.noteTypes.value.push(duplicated)
      state.noteTypes.value.sort((a, b) => a.name.localeCompare(b.name))
      return duplicated
    } catch (e) {
      status.error.value = String(e)
      throw e
    }
  }

  async function updateNodeContent(
    nodeId: string,
    title: string,
    noteFields: Record<string, string>,
    contentData: string | null,
    tags: string[],
  ) {
    try {
      const updated = await useTauRPC().update_node_content(nodeId, title, noteFields, contentData, tags)
      const idx = state.nodes.value.findIndex(n => n.id === nodeId)
      if (idx !== -1) state.nodes.value[idx] = updated
      return updated
    } catch (e) {
      status.error.value = String(e)
      throw e
    }
  }

  async function setNodeProgressStatus(nodeId: string, statusValue: string) {
    try {
      const updated = await useTauRPC().set_node_progress_status(nodeId, statusValue)
      const idx = state.nodes.value.findIndex(n => n.id === nodeId)
      if (idx !== -1) state.nodes.value[idx] = updated
      await loadNodeProgress()
      await loadReviewEvents()
    } catch (e) {
      status.error.value = String(e)
    }
  }

  async function reviewNode(nodeId: string, grade: string, schedulerKey: string | null = null) {
    try {
      const updated = await useTauRPC().review_node(
        nodeId,
        grade,
        schedulerKey ?? settings.learning.defaultSchedulerKey,
      )
      const idx = state.nodes.value.findIndex(n => n.id === nodeId)
      if (idx !== -1) state.nodes.value[idx] = updated
      await loadNodeProgress()
      await loadReviewEvents()
    } catch (e) {
      status.error.value = String(e)
    }
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

  return {
    loadLayers,
    loadWorldConfig,
    loadWorldPacks,
    loadPackRegistry,
    loadRelationKinds,
    loadNoteTypes,
    loadNodeProgress,
    loadSchedulerAlgorithms,
    loadReviewEvents,
    loadConnectionLayers,
    toggleConnectionLayer,
    setConnectionLayerSelection,
    loadNodes,
    markLearned,
    updateNodePosition,
    createNode,
    createLayer,
    createConnectionLayer,
    createEdge,
    setNodeNoteType,
    createNoteType,
    updateNoteType,
    duplicateNoteType,
    updateNodeContent,
    setNodeProgressStatus,
    reviewNode,
  }
}
