import type { GraphResourceState, GraphSessionState, GraphStatusState } from '@/stores/graph/shared'

const WORLD_PICKER_SEEN_KEY = 'concept:world-picker-startup-seen'

export interface GraphWorldActionsOptions {
  resources: GraphResourceState
  session: GraphSessionState
  status: GraphStatusState
  loadWorldPacks: () => Promise<void>
  loadWorldConfig: () => Promise<void>
  loadRelationKinds: () => Promise<void>
  loadConnectionLayers: () => Promise<void>
  loadNoteTypes: () => Promise<void>
  loadSchedulerAlgorithms: () => Promise<void>
  loadNodeProgress: () => Promise<void>
  loadReviewEvents: () => Promise<void>
  loadLayers: () => Promise<void>
  loadNodes: (layerId: string) => Promise<void>
  resetInteractiveState: () => void
}

export function createGraphWorldActions(options: GraphWorldActionsOptions) {
  const {
    resources,
    session,
    status,
    loadWorldPacks,
    loadWorldConfig,
    loadRelationKinds,
    loadConnectionLayers,
    loadNoteTypes,
    loadSchedulerAlgorithms,
    loadNodeProgress,
    loadReviewEvents,
    loadLayers,
    loadNodes,
    resetInteractiveState,
  } = options

  async function refreshLoadedWorld() {
    await loadWorldPacks()
    await loadWorldConfig()
    await loadRelationKinds()
    await loadConnectionLayers()
    await loadNoteTypes()
    await loadSchedulerAlgorithms()
    await loadNodeProgress()
    await loadReviewEvents()
    await loadLayers()
    if (resources.layers.value[0]) await loadNodes(resources.layers.value[0].id)
    else resources.nodes.value = []
  }

  async function initialize() {
    await refreshLoadedWorld()
    maybeOpenWorldPickerOnStartup()
  }

  async function resetGraphData(resetData: () => Promise<void>) {
    status.isLoading.value = true
    try {
      await resetData()
      resetInteractiveState()
      await refreshLoadedWorld()
    } catch (e) {
      status.error.value = String(e)
    } finally {
      status.isLoading.value = false
    }
  }

  async function switchWorld(selectWorldRpc: (worldId: string) => Promise<void>, worldId: string) {
    status.isLoading.value = true
    try {
      await selectWorldRpc(worldId)
      resetInteractiveState()
      session.worldPickerOpen.value = false
      markWorldPickerSeen()
      await refreshLoadedWorld()
    } catch (e) {
      status.error.value = String(e)
    } finally {
      status.isLoading.value = false
    }
  }

  async function reloadActiveWorld(reloadWorldRpc: () => Promise<void>) {
    status.isLoading.value = true
    try {
      await reloadWorldRpc()
      resetInteractiveState()
      session.worldPickerOpen.value = false
      await refreshLoadedWorld()
    } catch (e) {
      status.error.value = String(e)
    } finally {
      status.isLoading.value = false
    }
  }

  function maybeOpenWorldPickerOnStartup() {
    const validCount = resources.worldPacks.value.filter(world => world.valid).length
    if (validCount <= 1) return
    try {
      if (localStorage.getItem(WORLD_PICKER_SEEN_KEY) === '1') return
    } catch {
      // ignore storage failures
    }
    session.worldPickerOpen.value = true
  }

  function markWorldPickerSeen() {
    try {
      localStorage.setItem(WORLD_PICKER_SEEN_KEY, '1')
    } catch {
      // ignore storage failures
    }
  }

  return {
    initialize,
    resetGraphData,
    switchWorld,
    reloadActiveWorld,
    markWorldPickerSeen,
  }
}
