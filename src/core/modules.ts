import type { Component } from 'vue'

export type ModuleSlot =
  | 'GraphScene'
  | 'LayerPanel'
  | 'NodeDetailPanel'
  | 'NodeEditorOverlay'
  | 'ProgressOverlay'
  | 'ModeIndicator'
  | 'SettingsPanel'
  | 'WorldPickerOverlay'
  | 'NodeSearch'
  | 'CompassHUD'
  | 'BufferOverlay'

export type ModuleMap = Partial<Record<ModuleSlot, Component>>
