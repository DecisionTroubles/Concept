import { readonly, ref } from 'vue'

export type EditorMode = 'normal' | 'fly' | 'graph'

const mode = ref<EditorMode>('normal')

const jumpList: string[] = []
let lastNodeId: string | null = null

export function useEditorMode() {
  function enterFly() {
    mode.value = 'fly'
  }

  function enterNormal() {
    mode.value = 'normal'
  }

  function enterGraph() {
    const graphStore = useGraphStore()
    if (graphStore.selectedNodeId) mode.value = 'graph'
  }

  function escapeFromCurrentMode() {
    if (mode.value === 'fly') {
      mode.value = 'normal'
      return
    }
    if (mode.value === 'graph') {
      const graphStore = useGraphStore()
      graphStore.clearSelection()
      mode.value = 'normal'
    }
  }

  function onNodeSelected(id: string | null) {
    if (id) {
      if (lastNodeId && lastNodeId !== id) {
        jumpList.push(lastNodeId)
        if (jumpList.length > 20) jumpList.shift()
      }
      lastNodeId = id
      mode.value = 'graph'
      return
    }
    lastNodeId = null
    if (mode.value === 'graph') mode.value = 'normal'
  }

  function jumpBack(): string | null {
    return jumpList.pop() ?? null
  }

  return {
    mode: readonly(mode),
    enterFly,
    enterNormal,
    enterGraph,
    escapeFromCurrentMode,
    onNodeSelected,
    jumpBack,
  }
}
