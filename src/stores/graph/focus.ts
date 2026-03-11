import type { GraphResourceState, GraphSessionState } from '@/stores/graph/shared'
import type { ReturnTypeUseSettings } from '@/stores/graph/types'

interface GraphFocusActionsOptions {
  state: GraphResourceState
  session: GraphSessionState
  settings: ReturnTypeUseSettings
  setConnectionLayerSelection: (ids: string[]) => void
}

function focusParentId(node: { parent_node_id: string | null } | null | undefined): string | null {
  return node?.parent_node_id ?? null
}

function isSublayerNode(node: { parent_node_id: string | null } | null | undefined): boolean {
  return typeof node?.parent_node_id === 'string' && node.parent_node_id.length > 0
}

export function createGraphFocusActions(options: GraphFocusActionsOptions) {
  const { state, session, settings, setConnectionLayerSelection } = options

  function currentWorldSettings() {
    return settings.getWorldSettings(state.worldConfig.value?.id ?? null)
  }

  function focusOverlaySelectionForEntry(): string[] {
    const mode = currentWorldSettings().focusOverlayEntryMode
    if (mode === 'inherit') return [...state.activeConnectionLayerIds.value]
    if (mode === 'none') return []
    return state.connectionLayers.value.map(layer => layer.id)
  }

  function openFocusView(rootId?: string | null) {
    const id = rootId ?? session.selectedNodeId.value
    if (!id) return
    if (session.focusViewActive.value && session.focusRootNodeId.value === id) return

    session.focusOverlayParentSelection.value = [...state.activeConnectionLayerIds.value]
    session.focusViewActive.value = true
    session.focusRootNodeId.value = id
    session.focusCursorNodeId.value = id
    if (session.selectedNodeId.value !== id) session.selectedNodeId.value = id
    session.focusViewVersion.value += 1
    setConnectionLayerSelection(focusOverlaySelectionForEntry())
  }

  function closeFocusView() {
    if (!session.focusViewActive.value && session.focusRootNodeId.value === null) return

    const previousSelection = session.focusOverlayParentSelection.value
    const selected = session.selectedNodeId.value
      ? (state.nodes.value.find(node => node.id === session.selectedNodeId.value) ?? null)
      : null
    const parentId = focusParentId(selected)
    const nextSelection =
      selected && isSublayerNode(selected) && parentId ? parentId : session.selectedNodeId.value

    session.focusViewActive.value = false
    session.focusRootNodeId.value = null
    session.focusCursorNodeId.value = null
    session.focusOverlayParentSelection.value = null
    session.focusViewVersion.value += 1

    if (nextSelection !== session.selectedNodeId.value) session.selectedNodeId.value = nextSelection
    if (currentWorldSettings().restoreOverlaySelectionOnExit && previousSelection) {
      setConnectionLayerSelection(previousSelection)
    }
  }

  function toggleFocusView(rootId?: string | null) {
    const id = rootId ?? session.selectedNodeId.value
    if (!id) return
    if (session.focusViewActive.value) {
      closeFocusView()
      return
    }
    openFocusView(id)
  }

  function setFocusCursorNode(id: string | null) {
    if (session.focusCursorNodeId.value === id) return
    session.focusCursorNodeId.value = id
  }

  function selectFocusNode(id: string) {
    setFocusCursorNode(id)
    if (session.selectedNodeId.value !== id) session.selectedNodeId.value = id
  }

  return {
    openFocusView,
    closeFocusView,
    toggleFocusView,
    setFocusCursorNode,
    selectFocusNode,
  }
}
