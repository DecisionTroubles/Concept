import type { Component } from 'vue'

export type ModuleSlot =
  | 'GraphScene'
  | 'LayerPanel'
  | 'NodeDetailPanel'
  | 'ModeIndicator'
  | 'SettingsPanel'
  | 'NodeSearch'
  | 'CompassHUD'
  | 'BufferOverlay'

export type ModuleMap = Partial<Record<ModuleSlot, Component>>

