import { defineStore } from 'pinia'
import { useTauRPC } from '@/composables/useTauRPC'
import { useSettings } from '@/composables/useSettings'
import type {
  AnkiConnectPackSourceInput,
  AnkiDeckInspectInput,
  CreateLocalWorldInput,
  CreateNodeInput,
  GitHubPackSourceInput,
  LocalPackSourceInput,
} from '@/bindings'
import {
  createGraphDerivedState,
  createGraphResourceState,
  createGraphSessionState,
  createGraphStatusState,
  type BufferId,
} from '@/stores/graph/shared'
import { createGraphSessionActions } from '@/stores/graph/session'
import { createGraphResourceActions } from '@/stores/graph/resources'
import { createGraphFocusActions } from '@/stores/graph/focus'
import { createGraphWorldActions } from '@/stores/graph/worlds'

export type { BufferId }

export const useGraphStore = defineStore('graph', () => {
  const settings = useSettings()
  const resources = createGraphResourceState()
  const session = createGraphSessionState()
  const status = createGraphStatusState()
  const derived = createGraphDerivedState(resources.nodes, session.selectedNodeId, session.pinnedNodeIds)

  const sessionActions = createGraphSessionActions(session)
  const resourceActions = createGraphResourceActions({
    state: resources,
    session,
    status,
    settings,
    resetInteractiveState: sessionActions.resetInteractiveState,
  })
  const focusActions = createGraphFocusActions({
    state: resources,
    session,
    settings,
    setConnectionLayerSelection: resourceActions.setConnectionLayerSelection,
  })
  const worldActions = createGraphWorldActions({
    resources,
    session,
    status,
    loadWorldPacks: resourceActions.loadWorldPacks,
    loadWorldConfig: resourceActions.loadWorldConfig,
    loadRelationKinds: resourceActions.loadRelationKinds,
    loadPackRegistry: resourceActions.loadPackRegistry,
    loadConnectionLayers: resourceActions.loadConnectionLayers,
    loadNoteTypes: resourceActions.loadNoteTypes,
    loadSchedulerAlgorithms: resourceActions.loadSchedulerAlgorithms,
    loadNodeProgress: resourceActions.loadNodeProgress,
    loadReviewEvents: resourceActions.loadReviewEvents,
    loadLayers: resourceActions.loadLayers,
    loadNodes: resourceActions.loadNodes,
    resetInteractiveState: sessionActions.resetInteractiveState,
  })

  async function resetGraphData() {
    await worldActions.resetGraphData(() => useTauRPC().reset_data(true))
  }

  async function selectWorld(worldId: string) {
    if (resources.worldConfig.value?.id === worldId) {
      await worldActions.reloadActiveWorld(() => useTauRPC().reload_active_world())
      return
    }
    await worldActions.switchWorld((id: string) => useTauRPC().select_world(id), worldId)
  }

  async function reloadActiveWorld() {
    await worldActions.reloadActiveWorld(() => useTauRPC().reload_active_world())
  }

  async function refreshPackRegistry() {
    await resourceActions.loadPackRegistry()
    await resourceActions.loadWorldPacks()
  }

  async function createLocalWorld(input: CreateLocalWorldInput) {
    const info = await useTauRPC().create_local_world(input)
    await refreshPackRegistry()
    if (info.world_id) {
      await selectWorld(info.world_id)
    }
    return info
  }

  async function createNode(input: CreateNodeInput) {
    return resourceActions.createNode(input)
  }

  async function createEdge(
    sourceId: string,
    targetId: string,
    edgeType = 'Semantic',
    connectionLayerId: string | null = null,
  ) {
    return resourceActions.createEdge(sourceId, targetId, edgeType, connectionLayerId)
  }

  async function createLayer(name: string, displayOrder: number) {
    return resourceActions.createLayer(name, displayOrder)
  }

  async function createConnectionLayer(
    id: string | null,
    name: string,
    displayOrder: number,
    metadata: string | null = null,
  ) {
    return resourceActions.createConnectionLayer(id, name, displayOrder, metadata)
  }

  async function addGitHubPackSource(input: GitHubPackSourceInput) {
    const entry = await useTauRPC().add_github_pack_source(input)
    await refreshPackRegistry()
    return entry
  }

  async function updatePackSource(id: string, input: GitHubPackSourceInput) {
    const entry = await useTauRPC().update_pack_source(id, input)
    await refreshPackRegistry()
    return entry
  }

  async function addLocalPackSource(input: LocalPackSourceInput) {
    const entry = await useTauRPC().add_local_pack_source(input)
    await refreshPackRegistry()
    return entry
  }

  async function listAnkiDecks(baseUrl?: string | null) {
    return useTauRPC().list_anki_decks(baseUrl?.trim() ? baseUrl.trim() : null)
  }

  async function inspectAnkiDeck(input: AnkiDeckInspectInput) {
    return useTauRPC().inspect_anki_deck(input)
  }

  async function addAnkiPackSource(input: AnkiConnectPackSourceInput) {
    const entry = await useTauRPC().add_anki_pack_source(input)
    await refreshPackRegistry()
    return entry
  }

  async function inspectLocalPackPath(path: string) {
    return useTauRPC().inspect_local_pack_path(path)
  }

  async function updateLocalPackSource(id: string, input: LocalPackSourceInput) {
    const entry = await useTauRPC().update_local_pack_source(id, input)
    await refreshPackRegistry()
    return entry
  }

  async function updateAnkiPackSource(id: string, input: AnkiConnectPackSourceInput) {
    const entry = await useTauRPC().update_anki_pack_source(id, input)
    await refreshPackRegistry()
    return entry
  }

  async function removePackSource(id: string) {
    await useTauRPC().remove_pack_source(id)
    await refreshPackRegistry()
  }

  async function deleteLocalWorld(packPath: string) {
    await useTauRPC().delete_local_world(packPath)
    await refreshPackRegistry()
    await resourceActions.loadWorldConfig()
    await resourceActions.loadLayers()
    if (resources.layers.value[0]) await resourceActions.loadNodes(resources.layers.value[0].id)
    else resources.nodes.value = []
  }

  async function installPackSource(id: string) {
    const entry = await useTauRPC().install_pack_source(id)
    await refreshPackRegistry()
    return entry
  }

  async function refreshPackSource(id: string) {
    const entry = await useTauRPC().refresh_pack_source(id)
    await refreshPackRegistry()
    return entry
  }

  async function checkPackSourceUpdates(id: string) {
    const entry = await useTauRPC().check_pack_source_updates(id)
    await refreshPackRegistry()
    return entry
  }

  return {
    layers: resources.layers,
    activeLayerId: resources.activeLayerId,
    nodes: resources.nodes,
    noteTypes: resources.noteTypes,
    nodeProgress: resources.nodeProgress,
    schedulerAlgorithms: resources.schedulerAlgorithms,
    reviewEvents: resources.reviewEvents,
    worldConfig: resources.worldConfig,
    worldPacks: resources.worldPacks,
    packRegistry: resources.packRegistry,
    relationKinds: resources.relationKinds,
    connectionLayers: resources.connectionLayers,
    activeConnectionLayerIds: resources.activeConnectionLayerIds,
    selectedNodeId: session.selectedNodeId,
    centeredNodePanel: session.centeredNodePanel,
    nodeEditorOpen: session.nodeEditorOpen,
    pinnedNodeIds: session.pinnedNodeIds,
    focusState: session.focusState,
    focusViewActive: session.focusViewActive,
    focusRootNodeId: session.focusRootNodeId,
    focusCursorNodeId: session.focusCursorNodeId,
    pinnedNodes: derived.pinnedNodes,
    activeBuffer: session.activeBuffer,
    packLibraryOpen: session.packLibraryOpen,
    progressOverlayOpen: session.progressOverlayOpen,
    worldPickerOpen: session.worldPickerOpen,
    selectedNode: derived.selectedNode,
    dueNodes: derived.dueNodes,
    isLoading: status.isLoading,
    error: status.error,
    loadLayers: resourceActions.loadLayers,
    loadNoteTypes: resourceActions.loadNoteTypes,
    loadSchedulerAlgorithms: resourceActions.loadSchedulerAlgorithms,
    loadNodeProgress: resourceActions.loadNodeProgress,
    loadReviewEvents: resourceActions.loadReviewEvents,
    loadWorldConfig: resourceActions.loadWorldConfig,
    loadWorldPacks: resourceActions.loadWorldPacks,
    loadPackRegistry: resourceActions.loadPackRegistry,
    loadRelationKinds: resourceActions.loadRelationKinds,
    loadConnectionLayers: resourceActions.loadConnectionLayers,
    loadNodes: resourceActions.loadNodes,
    markLearned: resourceActions.markLearned,
    updateNodePosition: resourceActions.updateNodePosition,
    createNode,
    createEdge,
    createLayer,
    createConnectionLayer,
    setNodeNoteType: resourceActions.setNodeNoteType,
    createNoteType: resourceActions.createNoteType,
    updateNoteType: resourceActions.updateNoteType,
    duplicateNoteType: resourceActions.duplicateNoteType,
    updateNodeContent: resourceActions.updateNodeContent,
    setNodeProgressStatus: resourceActions.setNodeProgressStatus,
    reviewNode: resourceActions.reviewNode,
    selectNode: sessionActions.selectNode,
    clearSelection: sessionActions.clearSelection,
    toggleCenteredNodePanel: sessionActions.toggleCenteredNodePanel,
    openNodeEditor: sessionActions.openNodeEditor,
    closeNodeEditor: sessionActions.closeNodeEditor,
    toggleNodeEditor: sessionActions.toggleNodeEditor,
    isNodePinned: sessionActions.isNodePinned,
    togglePinNode: sessionActions.togglePinNode,
    unpinNode: sessionActions.unpinNode,
    clearPinnedNodes: sessionActions.clearPinnedNodes,
    openFocusView: focusActions.openFocusView,
    closeFocusView: focusActions.closeFocusView,
    toggleFocusView: focusActions.toggleFocusView,
    setFocusCursorNode: focusActions.setFocusCursorNode,
    selectFocusNode: focusActions.selectFocusNode,
    closeBuffer: sessionActions.closeBuffer,
    openBuffer: sessionActions.openBuffer,
    toggleBuffer: sessionActions.toggleBuffer,
    openPackLibrary: sessionActions.openPackLibrary,
    closePackLibrary: sessionActions.closePackLibrary,
    togglePackLibrary: sessionActions.togglePackLibrary,
    openProgressOverlay: sessionActions.openProgressOverlay,
    closeProgressOverlay: sessionActions.closeProgressOverlay,
    toggleProgressOverlay: sessionActions.toggleProgressOverlay,
    openWorldPicker: sessionActions.openWorldPicker,
    closeWorldPicker: () => sessionActions.closeWorldPicker(worldActions.markWorldPickerSeen),
    toggleWorldPicker: () => sessionActions.toggleWorldPicker(worldActions.markWorldPickerSeen),
    toggleConnectionLayer: resourceActions.toggleConnectionLayer,
    setConnectionLayerSelection: resourceActions.setConnectionLayerSelection,
    focusVersion: session.focusVersion,
    focusViewVersion: session.focusViewVersion,
    requestFocus: sessionActions.requestFocus,
    initialize: worldActions.initialize,
    resetGraphData,
    createLocalWorld,
    selectWorld,
    reloadActiveWorld,
    addGitHubPackSource,
    addLocalPackSource,
    listAnkiDecks,
    inspectAnkiDeck,
    addAnkiPackSource,
    inspectLocalPackPath,
    updatePackSource,
    updateLocalPackSource,
    updateAnkiPackSource,
    removePackSource,
    deleteLocalWorld,
    installPackSource,
    refreshPackSource,
    checkPackSourceUpdates,
    refreshPackRegistry,
  }
})
