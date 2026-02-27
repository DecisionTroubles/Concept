<script setup lang="ts">
import { TresCanvas } from '@tresjs/core'
import { EffectComposerPmndrs, BloomPmndrs, VignettePmndrs } from '@tresjs/post-processing'
import GraphScene from '@/components/scene/GraphScene.vue'
import NodeDetailPanel from '@/components/NodeDetailPanel.vue'
import LayerPanel from '@/components/LayerPanel.vue'

const graphStore = useGraphStore()
onMounted(() => graphStore.initialize())
</script>

<template>
  <!-- Full-viewport 3D canvas -->
  <TresCanvas clear-color="#080b14" window-size>
    <GraphScene />

    <!-- Post-processing stack -->
    <EffectComposerPmndrs>
      <BloomPmndrs
        :intensity="0.9"
        :luminance-threshold="0.25"
        :luminance-smoothing="0.6"
        :kernel-size="4"
      />
      <VignettePmndrs :offset="0.45" :darkness="0.55" />
    </EffectComposerPmndrs>
  </TresCanvas>

  <!-- 2D overlay panels (position: fixed inside each component) -->
  <LayerPanel />
  <NodeDetailPanel />
</template>
