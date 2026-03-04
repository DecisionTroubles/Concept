import type { ModuleMap } from '@/core/modules'
import type { ThemePreset } from '@/core/themes'

export interface AppPlugin {
  id: string
  name: string
  modules?: ModuleMap
  themes?: ThemePreset[]
  setup?: () => void | Promise<void>
}

