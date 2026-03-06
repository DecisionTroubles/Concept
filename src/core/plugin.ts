import type { ModuleMap } from '@/core/modules'
import type { NodeWorkspaceExtension } from '@/core/nodeExtensions'
import type { ThemePreset } from '@/core/themes'

export interface AppPlugin {
  id: string
  name: string
  modules?: ModuleMap
  nodeWorkspaceExtensions?: NodeWorkspaceExtension[]
  themes?: ThemePreset[]
  setup?: () => void | Promise<void>
}

export function definePlugin(plugin: AppPlugin): AppPlugin {
  return plugin
}
