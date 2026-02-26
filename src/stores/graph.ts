import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { Layer, Node } from '@/bindings'
import { useTauRPC } from '@/composables/useTauRPC'

export const useGraphStore = defineStore('graph', () => {
  const layers = ref<Layer[]>([])
  const activeLayerId = ref<string | null>(null)
  const nodes = ref<Node[]>([])
  const isLoading = ref(false)
  const error = ref<string | null>(null)

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
    isLoading.value = true
    try {
      nodes.value = await useTauRPC()[''].get_nodes(layerId)
    } catch (e) {
      error.value = String(e)
    } finally {
      isLoading.value = false
    }
  }

  async function initialize() {
    await useTauRPC()[''].seed_sample_data()
    await loadLayers()
    if (layers.value[0]) await loadNodes(layers.value[0].id)
  }

  return { layers, activeLayerId, nodes, isLoading, error, loadLayers, loadNodes, initialize }
})
