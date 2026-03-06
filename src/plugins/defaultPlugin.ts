import type { AppPlugin } from '@/core/plugin'
import GraphScene from '@/components/scene/GraphScene.vue'
import LayerPanel from '@/components/LayerPanel.vue'
import NodeDetailPanel from '@/components/NodeDetailPanel.vue'
import ProgressOverlay from '@/components/ProgressOverlay.vue'
import ModeIndicator from '@/components/ModeIndicator.vue'
import SettingsPanel from '@/components/SettingsPanel.vue'
import WorldPickerOverlay from '@/components/WorldPickerOverlay.vue'
import NodeSearch from '@/components/NodeSearch.vue'
import CompassHUD from '@/components/CompassHUD.vue'
import BufferOverlay from '@/components/BufferOverlay.vue'
import NodeAssetsExtension from '@/components/node/extensions/NodeAssetsExtension.vue'
import NodeNotesExtension from '@/components/node/extensions/NodeNotesExtension.vue'

export const defaultPlugin: AppPlugin = {
  id: 'core.default',
  name: 'Core Defaults',
  modules: {
    GraphScene,
    LayerPanel,
    NodeDetailPanel,
    ProgressOverlay,
    ModeIndicator,
    SettingsPanel,
    WorldPickerOverlay,
    NodeSearch,
    CompassHUD,
    BufferOverlay,
  },
  nodeWorkspaceExtensions: [
    {
      id: 'ai-assistant',
      title: 'AI Assistant',
      description: 'Future plugin slot for node-specific explanations, summaries, and saved model chats.',
      slot: 'extensions.primary',
      order: 10,
    },
    {
      id: 'node-assets',
      title: 'Node Assets',
      description: 'Saved node-specific assets such as images, links, music, or file references.',
      slot: 'extensions.primary',
      order: 20,
      component: NodeAssetsExtension,
    },
    {
      id: 'node-notes',
      title: 'Node Notes',
      description: 'Saved per-node notes backed by the extension data store.',
      slot: 'extensions.primary',
      order: 30,
      component: NodeNotesExtension,
    },
    {
      id: 'review-tools',
      title: 'Review Tools',
      description: 'Future slot for custom schedulers, drills, and node-specific learning helpers.',
      slot: 'learning.secondary',
      order: 40,
    },
    {
      id: 'history-tools',
      title: 'History Tools',
      description: 'Future slot for analytics, comparison views, and event-derived insights.',
      slot: 'history.secondary',
      order: 50,
    },
  ],
}
