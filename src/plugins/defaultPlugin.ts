import type { AppPlugin } from '@/core/plugin'
import GraphScene from '@/components/scene/GraphScene.vue'
import LayerPanel from '@/components/LayerPanel.vue'
import NodeDetailPanel from '@/components/NodeDetailPanel.vue'
import ModeIndicator from '@/components/ModeIndicator.vue'
import SettingsPanel from '@/components/SettingsPanel.vue'
import NodeSearch from '@/components/NodeSearch.vue'
import CompassHUD from '@/components/CompassHUD.vue'
import BufferOverlay from '@/components/BufferOverlay.vue'

export const defaultPlugin: AppPlugin = {
  id: 'core.default',
  name: 'Core Defaults',
  modules: {
    GraphScene,
    LayerPanel,
    NodeDetailPanel,
    ModeIndicator,
    SettingsPanel,
    NodeSearch,
    CompassHUD,
    BufferOverlay,
  },
}

