import type { Component } from 'vue'

export type ModuleSlot =
  | 'GraphScene'
  | 'LayerPanel'
  | 'NodeDetailPanel'
  | 'NodeEditorOverlay'
  | 'ProgressOverlay'
  | 'ModeIndicator'
  | 'SettingsPanel'
  | 'PackLibraryOverlay'
  | 'WorldPickerOverlay'
  | 'NodeSearch'
  | 'CompassHUD'
  | 'BufferOverlay'

export type ModuleMap = Partial<Record<ModuleSlot, Component>>
