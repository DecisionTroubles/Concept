<script setup lang="ts">
import { TresCanvas } from '@tresjs/core'
import { EffectComposerPmndrs, BloomPmndrs, VignettePmndrs } from '@tresjs/post-processing'
import GraphScene from '@/components/scene/GraphScene.vue'
import NodeDetailPanel from '@/components/NodeDetailPanel.vue'
import LayerPanel from '@/components/LayerPanel.vue'
import BufferOverlay from '@/components/BufferOverlay.vue'

const graphStore = useGraphStore()
const editorMode = useEditorMode()
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
  <ModeIndicator />
  <SettingsPanel />
  <NodeSearch />
  <CompassHUD
    v-if="editorMode.mode.value === 'graph' && editorMode.compassCenter.value"
    :dots="editorMode.compassDots.value"
    :center-x="editorMode.compassCenter.value?.x ?? 0"
    :center-y="editorMode.compassCenter.value?.y ?? 0"
    :active-index="editorMode.compassIndex.value"
  />
  <BufferOverlay />
</template>
