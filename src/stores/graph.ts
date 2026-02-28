import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Layer, Node } from '@/bindings'
import { useTauRPC } from '@/composables/useTauRPC'

export const useGraphStore = defineStore('graph', () => {
  const layers = ref<Layer[]>([])
  const activeLayerId = ref<string | null>(null)
  const nodes = ref<Node[]>([])
  const selectedNodeId = ref<string | null>(null)
  const focusVersion = ref(0)
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  const selectedNode = computed(() =>
    selectedNodeId.value ? (nodes.value.find(n => n.id === selectedNodeId.value) ?? null) : null,
  )

  function selectNode(id: string | null) {
    selectedNodeId.value = id
  }

  function requestFocus(id: string) {
    selectedNodeId.value = id
    focusVersion.value++
  }

  async function loadLayers() {
    isLoading.value = true
    try {
      layers.value = await useTauRPC()[''].get_layers()
    } catch (e) {
      error.value = String(e)
    } finally {
      isLoading.value = false
    }
  }

  async function loadNodes(layerId: string) {
    activeLayerId.value = layerId
    selectedNodeId.value = null
    isLoading.value = true
    try {
      nodes.value = await useTauRPC()[''].get_nodes(layerId)
    } catch (e) {
      error.value = String(e)
    } finally {
      isLoading.value = false
    }
  }

  async function markLearned(id: string, learned: boolean = true) {
    try {
      const updated = await useTauRPC()[''].mark_learned(id, learned)
      const idx = nodes.value.findIndex(n => n.id === id)
      if (idx !== -1) nodes.value[idx] = updated
    } catch (e) {
      error.value = String(e)
    }
  }

  async function updateNodePosition(id: string, x: number, y: number, z: number) {
    try {
      await useTauRPC()[''].update_node_position(id, x, y, z)
    } catch {
      // Non-critical — don't surface position errors to the user
    }
  }

  async function initialize() {
    await useTauRPC()[''].seed_sample_data()
    await loadLayers()
    if (layers.value[0]) await loadNodes(layers.value[0].id)
  }

  return {
    layers,
    activeLayerId,
    nodes,
    selectedNodeId,
    selectedNode,
    isLoading,
    error,
    loadLayers,
    loadNodes,
    markLearned,
    updateNodePosition,
    selectNode,
    focusVersion,
    requestFocus,
    initialize,
  }
})
