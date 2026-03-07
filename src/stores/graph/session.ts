import type { Ref } from 'vue'
import type { BufferId, GraphSessionState } from '@/stores/graph/shared'

export function createGraphSessionActions(session: GraphSessionState) {
  function selectNode(id: string | null) {
    session.selectedNodeId.value = id
    if (!id) {
      session.centeredNodePanel.value = false
      session.nodeEditorOpen.value = false
    }
  }

  function requestFocus(id: string) {
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
    session.activeBuffer.value = buffer
  }

  function toggleBuffer(buffer: Exclude<BufferId, 'none'>) {
    session.activeBuffer.value = session.activeBuffer.value === buffer ? 'none' : buffer
  }

  function openProgressOverlay() {
    session.progressOverlayOpen.value = true
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
    session.selectedNodeId.value = null
    session.centeredNodePanel.value = false
    session.nodeEditorOpen.value = false
    session.pinnedNodeIds.value = []
    session.activeBuffer.value = 'none'
  }

  return {
    selectNode,
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
    closeProgressOverlay,
    toggleProgressOverlay,
    openWorldPicker,
    closeWorldPicker,
    toggleWorldPicker,
    resetInteractiveState,
  }
}
