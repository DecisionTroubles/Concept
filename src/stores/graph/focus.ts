import type { GraphResourceState, GraphSessionState } from '@/stores/graph/shared'
import type { ReturnTypeUseSettings } from '@/stores/graph/types'
import { graphTrace } from '@/stores/graph/debug'

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
    if (session.focusViewActive.value && session.focusRootNodeId.value === id) {
      graphTrace('focus.open.skip', {
        id,
        focusViewActive: session.focusViewActive.value,
        focusRootNodeId: session.focusRootNodeId.value,
      })
      return
    }

    graphTrace('focus.open', {
      id,
      selectedNodeId: session.selectedNodeId.value,
      focusViewActive: session.focusViewActive.value,
      focusRootNodeId: session.focusRootNodeId.value,
      activeConnectionLayerIds: [...state.activeConnectionLayerIds.value],
    })

    session.focusOverlayParentSelection.value = [...state.activeConnectionLayerIds.value]
    session.focusViewActive.value = true
    session.focusRootNodeId.value = id
    session.focusCursorNodeId.value = id
    if (session.selectedNodeId.value !== id) session.selectedNodeId.value = id
    session.focusViewVersion.value += 1
    setConnectionLayerSelection(focusOverlaySelectionForEntry())
  }

  function closeFocusView() {
    if (!session.focusViewActive.value && session.focusRootNodeId.value === null) {
      graphTrace('focus.close.skip', {
        focusViewActive: session.focusViewActive.value,
        focusRootNodeId: session.focusRootNodeId.value,
      })
      return
    }

    const previousSelection = session.focusOverlayParentSelection.value
    const selected = session.selectedNodeId.value
      ? (state.nodes.value.find(node => node.id === session.selectedNodeId.value) ?? null)
      : null
    const parentId = focusParentId(selected)
    const nextSelection =
      selected && isSublayerNode(selected) && parentId ? parentId : session.selectedNodeId.value

    graphTrace('focus.close', {
      selectedNodeId: session.selectedNodeId.value,
      selectedParentId: parentId,
      nextSelection,
      focusRootNodeId: session.focusRootNodeId.value,
      focusCursorNodeId: session.focusCursorNodeId.value,
      restoreOverlaySelectionOnExit: currentWorldSettings().restoreOverlaySelectionOnExit,
      previousSelection,
    })

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
    graphTrace('focus.toggle', {
      id,
      focusViewActive: session.focusViewActive.value,
      focusRootNodeId: session.focusRootNodeId.value,
      selectedNodeId: session.selectedNodeId.value,
    })
    if (session.focusViewActive.value) {
      closeFocusView()
      return
    }
    openFocusView(id)
  }

  function setFocusCursorNode(id: string | null) {
    if (session.focusCursorNodeId.value === id) {
      graphTrace('focus.setCursor.skip', { id })
      return
    }
    graphTrace('focus.setCursor', {
      from: session.focusCursorNodeId.value,
      to: id,
      focusRootNodeId: session.focusRootNodeId.value,
      selectedNodeId: session.selectedNodeId.value,
    })
    session.focusCursorNodeId.value = id
  }

  function selectFocusNode(id: string) {
    graphTrace('focus.selectNode', {
      id,
      selectedNodeId: session.selectedNodeId.value,
      focusCursorNodeId: session.focusCursorNodeId.value,
      focusRootNodeId: session.focusRootNodeId.value,
    })
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
