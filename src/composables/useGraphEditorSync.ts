import { watch } from 'vue'

export function useGraphEditorSync() {
  const graphStore = useGraphStore()
  const editorMode = useEditorMode()

  watch(
    () => graphStore.selectedNodeId,
    id => {
      editorMode.onNodeSelected(id)
    },
    { flush: 'post' }
  )
}
