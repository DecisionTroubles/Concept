<script setup lang="ts">
import { TresCanvas } from '@tresjs/core'
import { EffectComposerPmndrs, BloomPmndrs, VignettePmndrs } from '@tresjs/post-processing'
import { appKernel } from '@/core/kernel'
import { useTheme } from '@/composables/useTheme'

const graphStore = useGraphStore()
const editorMode = useEditorMode()
const themeState = useTheme()
const settings = useSettings()

const GraphSceneModule = appKernel.getModule('GraphScene')
const LayerPanelModule = appKernel.getModule('LayerPanel')
const NodeDetailPanelModule = appKernel.getModule('NodeDetailPanel')
const ProgressOverlayModule = appKernel.getModule('ProgressOverlay')
const ModeIndicatorModule = appKernel.getModule('ModeIndicator')
const SettingsPanelModule = appKernel.getModule('SettingsPanel')
const WorldPickerOverlayModule = appKernel.getModule('WorldPickerOverlay')
const NodeSearchModule = appKernel.getModule('NodeSearch')
const CompassHUDModule = appKernel.getModule('CompassHUD')
const BufferOverlayModule = appKernel.getModule('BufferOverlay')

onMounted(() => graphStore.initialize())
</script>

<template>
  <!-- Full-viewport 3D canvas -->
  <TresCanvas :clear-color="themeState.canvasColor.value" window-size>
    <component :is="GraphSceneModule" />

    <!-- Post-processing stack -->
    <EffectComposerPmndrs>
      <BloomPmndrs
        v-if="settings.graphics.bloomEnabled"
        :intensity="settings.graphics.bloomIntensity"
        :luminance-threshold="settings.graphics.bloomThreshold"
        :luminance-smoothing="settings.graphics.bloomSmoothing"
        :kernel-size="4"
      />
      <VignettePmndrs v-if="settings.graphics.vignetteEnabled" :offset="0.45" :darkness="settings.graphics.vignetteDarkness" />
    </EffectComposerPmndrs>
  </TresCanvas>

  <!-- 2D overlay panels (position: fixed inside each component) -->
  <component :is="LayerPanelModule" />
  <component :is="NodeDetailPanelModule" />
  <component :is="ProgressOverlayModule" />
  <component :is="ModeIndicatorModule" />
  <component :is="SettingsPanelModule" />
  <component :is="WorldPickerOverlayModule" />
  <component :is="NodeSearchModule" />
  <component
    :is="CompassHUDModule"
    v-if="editorMode.mode.value === 'graph' && editorMode.compassCenter.value"
    :dots="editorMode.compassDots.value"
    :center-x="editorMode.compassCenter.value?.x ?? 0"
    :center-y="editorMode.compassCenter.value?.y ?? 0"
    :active-index="editorMode.compassIndex.value"
  />
  <component :is="BufferOverlayModule" />
</template>
