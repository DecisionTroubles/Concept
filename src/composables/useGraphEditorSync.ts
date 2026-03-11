import { watch } from 'vue'
import { graphTrace } from '@/stores/graph/debug'

export function useGraphEditorSync() {
  const graphStore = useGraphStore()
  const editorMode = useEditorMode()

  watch(
    () => graphStore.selectedNodeId,
    id => {
      graphTrace('editorSync.selectedNodeId', {
        selectedNodeId: id,
        focusViewActive: graphStore.focusViewActive,
        focusRootNodeId: graphStore.focusRootNodeId,
        focusCursorNodeId: graphStore.focusCursorNodeId,
        editorMode: editorMode.mode.value,
      })
      editorMode.onNodeSelected(id)
    },
    { flush: 'post' }
  )
}
