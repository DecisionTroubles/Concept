<script setup lang="ts">
import { TresCanvas } from '@tresjs/core'
import { EffectComposerPmndrs, BloomPmndrs, VignettePmndrs } from '@tresjs/post-processing'
import { appKernel } from '@/core/kernel'
import { useGraphEditorSync } from '@/composables/useGraphEditorSync'
import { useTheme } from '@/composables/useTheme'
import { useSceneHudState } from '@/scene/controller/useSceneHudState'

const graphStore = useGraphStore()
const editorMode = useEditorMode()
const themeState = useTheme()
const settings = useSettings()
const hudState = useSceneHudState()

useGraphEditorSync()

const GraphSceneModule = appKernel.getModule('GraphScene')
const LayerPanelModule = appKernel.getModule('LayerPanel')
const NodeDetailPanelModule = appKernel.getModule('NodeDetailPanel')
const NodeEditorOverlayModule = appKernel.getModule('NodeEditorOverlay')
const ProgressOverlayModule = appKernel.getModule('ProgressOverlay')
const ModeIndicatorModule = appKernel.getModule('ModeIndicator')
const SettingsPanelModule = appKernel.getModule('SettingsPanel')
const PackLibraryOverlayModule = appKernel.getModule('PackLibraryOverlay')
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
  <component :is="NodeEditorOverlayModule" />
  <component :is="ProgressOverlayModule" />
  <component :is="ModeIndicatorModule" />
  <component :is="SettingsPanelModule" />
  <component :is="PackLibraryOverlayModule" />
  <component :is="WorldPickerOverlayModule" />
  <component :is="NodeSearchModule" />
  <component
    :is="CompassHUDModule"
    v-if="editorMode.mode.value !== 'fly' && hudState.center.value && hudState.dots.value.length > 0"
    :dots="hudState.dots.value"
    :center-x="hudState.center.value?.x ?? 0"
    :center-y="hudState.center.value?.y ?? 0"
    :active-index="hudState.activeIndex.value"
  />
  <component :is="BufferOverlayModule" />
</template>
