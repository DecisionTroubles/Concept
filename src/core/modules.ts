import type { Component } from 'vue'

export type ModuleSlot =
  | 'GraphScene'
  | 'LayerPanel'
  | 'NodeDetailPanel'
  | 'ProgressOverlay'
  | 'ModeIndicator'
  | 'SettingsPanel'
  | 'WorldPickerOverlay'
  | 'NodeSearch'
  | 'CompassHUD'
  | 'BufferOverlay'

export type ModuleMap = Partial<Record<ModuleSlot, Component>>
