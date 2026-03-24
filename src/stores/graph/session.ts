import type { Ref } from 'vue'
import type { BufferId, GraphSessionState } from '@/stores/graph/shared'
import { graphTrace } from '@/stores/graph/debug'
import { createWorldFocusState } from '@/scene/model/focusState'

export function createGraphSessionActions(session: GraphSessionState) {
  function selectNode(id: string | null) {
    if (session.selectedNodeId.value === id) {
      graphTrace('session.selectNode.skip', { id, selectedNodeId: session.selectedNodeId.value })
      return
    }
    graphTrace('session.selectNode', {
      from: session.selectedNodeId.value,
      to: id,
      focusViewActive: session.focusViewActive.value,
      focusRootNodeId: session.focusRootNodeId.value,
      focusCursorNodeId: session.focusCursorNodeId.value,
    })
    session.selectedNodeId.value = id
  }

  function clearSelection() {
    if (session.selectedNodeId.value === null && !session.centeredNodePanel.value && !session.nodeEditorOpen.value && !session.focusViewActive.value) {
      graphTrace('session.clearSelection.skip', {
        selectedNodeId: session.selectedNodeId.value,
        centeredNodePanel: session.centeredNodePanel.value,
        nodeEditorOpen: session.nodeEditorOpen.value,
        focusViewActive: session.focusViewActive.value,
      })
      return
    }
    graphTrace('session.clearSelection', {
      selectedNodeId: session.selectedNodeId.value,
      centeredNodePanel: session.centeredNodePanel.value,
      nodeEditorOpen: session.nodeEditorOpen.value,
      focusViewActive: session.focusViewActive.value,
      focusRootNodeId: session.focusRootNodeId.value,
      focusCursorNodeId: session.focusCursorNodeId.value,
    })
    session.selectedNodeId.value = null
    session.centeredNodePanel.value = false
    session.nodeEditorOpen.value = false
    session.focusState.value = createWorldFocusState()
    session.focusOverlayParentSelection.value = null
    session.focusViewVersion.value += 1
  }

  function requestFocus(id: string) {
    graphTrace('session.requestFocus', {
      id,
      from: session.selectedNodeId.value,
      focusVersion: session.focusVersion.value,
    })
    session.selectedNodeId.value = id
    session.focusVersion.value++
  }

  function toggleCenteredNodePanel() {
    if (!session.selectedNodeId.value) return
    session.centeredNodePanel.value = !session.centeredNodePanel.value
  }

  function openNodeEditor() {
    if (!session.selectedNodeId.value) return
    session.nodeEditorOpen.value = true
  }

  function closeNodeEditor() {
    session.nodeEditorOpen.value = false
  }

  function toggleNodeEditor() {
    if (!session.selectedNodeId.value) return
    session.nodeEditorOpen.value = !session.nodeEditorOpen.value
  }

  function isNodePinned(id: string | null | undefined): boolean {
    if (!id) return false
    return session.pinnedNodeIds.value.includes(id)
  }

  function togglePinNode(id: string) {
    const idx = session.pinnedNodeIds.value.indexOf(id)
    if (idx === -1) session.pinnedNodeIds.value.push(id)
    else session.pinnedNodeIds.value.splice(idx, 1)
  }

  function unpinNode(id: string) {
    const idx = session.pinnedNodeIds.value.indexOf(id)
    if (idx !== -1) session.pinnedNodeIds.value.splice(idx, 1)
  }

  function clearPinnedNodes() {
    session.pinnedNodeIds.value = []
  }

  function closeBuffer() {
    session.activeBuffer.value = 'none'
  }

  function openBuffer(buffer: Exclude<BufferId, 'none'>) {
    session.packLibraryOpen.value = false
    session.activeBuffer.value = buffer
  }

  function toggleBuffer(buffer: Exclude<BufferId, 'none'>) {
    session.activeBuffer.value = session.activeBuffer.value === buffer ? 'none' : buffer
  }

  function openProgressOverlay() {
    session.progressOverlayOpen.value = true
  }

  function openPackLibrary() {
    session.activeBuffer.value = 'none'
    session.packLibraryOpen.value = true
  }

  function closePackLibrary() {
    session.packLibraryOpen.value = false
  }

  function togglePackLibrary() {
    session.packLibraryOpen.value = !session.packLibraryOpen.value
  }

  function closeProgressOverlay() {
    session.progressOverlayOpen.value = false
  }

  function toggleProgressOverlay() {
    session.progressOverlayOpen.value = !session.progressOverlayOpen.value
  }

  function openWorldPicker() {
    session.worldPickerOpen.value = true
  }

  function closeWorldPicker(onClose?: () => void) {
    session.worldPickerOpen.value = false
    onClose?.()
  }

  function toggleWorldPicker(onClose?: () => void) {
    session.worldPickerOpen.value = !session.worldPickerOpen.value
    if (!session.worldPickerOpen.value) onClose?.()
  }

  function resetInteractiveState() {
    clearSelection()
    session.pinnedNodeIds.value = []
    session.activeBuffer.value = 'none'
    session.packLibraryOpen.value = false
  }

  return {
    selectNode,
    clearSelection,
    requestFocus,
    toggleCenteredNodePanel,
    openNodeEditor,
    closeNodeEditor,
    toggleNodeEditor,
    isNodePinned,
    togglePinNode,
    unpinNode,
    clearPinnedNodes,
    closeBuffer,
    openBuffer,
    toggleBuffer,
    openProgressOverlay,
    openPackLibrary,
    closePackLibrary,
    togglePackLibrary,
    closeProgressOverlay,
    toggleProgressOverlay,
    openWorldPicker,
    closeWorldPicker,
    toggleWorldPicker,
    resetInteractiveState,
  }
}
