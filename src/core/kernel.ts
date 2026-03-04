import type { Component } from 'vue'
import type { AppPlugin } from '@/core/plugin'
import type { ModuleSlot } from '@/core/modules'
import { DEFAULT_THEMES, type ThemePreset } from '@/core/themes'

class AppKernel {
  private modules = new Map<ModuleSlot, Component>()
  private themes = new Map<string, ThemePreset>()
  private plugins = new Map<string, AppPlugin>()

  constructor() {
    for (const theme of DEFAULT_THEMES) this.themes.set(theme.id, theme)
  }

  async install(plugin: AppPlugin): Promise<void> {
    if (this.plugins.has(plugin.id)) return
    this.plugins.set(plugin.id, plugin)

    if (plugin.modules) {
      for (const [slot, module] of Object.entries(plugin.modules)) {
        if (!module) continue
        this.modules.set(slot as ModuleSlot, module)
      }
    }

    if (plugin.themes) {
      for (const theme of plugin.themes) this.themes.set(theme.id, theme)
    }

    await plugin.setup?.()
  }

  getModule(slot: ModuleSlot): Component {
    const component = this.modules.get(slot)
    if (!component) {
      throw new Error(`[kernel] Missing module for slot: ${slot}`)
    }
    return component
  }

  listThemes(): ThemePreset[] {
    return Array.from(this.themes.values())
  }

  hasTheme(id: string): boolean {
    return this.themes.has(id)
  }
}

export const appKernel = new AppKernel()

