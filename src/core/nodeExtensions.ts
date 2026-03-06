import type { Component } from 'vue'

export type NodeWorkspaceSlot =
  | 'overview.secondary'
  | 'learning.secondary'
  | 'history.secondary'
  | 'extensions.primary'

export interface NodeWorkspaceExtension {
  id: string
  title: string
  description: string
  slot: NodeWorkspaceSlot
  order?: number
  component?: Component
}
