import type { AppPlugin } from '@/core/plugin'

type UserPluginModule = {
  default?: AppPlugin
  plugin?: AppPlugin
  plugins?: AppPlugin[]
}

const userPluginModules = import.meta.glob<UserPluginModule>('/user-plugins/**/*.{js,mjs,ts}', {
  eager: true,
})

export async function loadUserPlugins(): Promise<AppPlugin[]> {
  const resolvedPlugins: AppPlugin[] = []

  for (const [path, mod] of Object.entries(userPluginModules)) {
    const candidates = [mod.default, mod.plugin, ...(mod.plugins ?? [])].filter(
      (plugin): plugin is AppPlugin => !!plugin,
    )

    for (const plugin of candidates) {
      resolvedPlugins.push(plugin)
    }

    if (candidates.length === 0) {
      console.warn(`[plugins] Ignored ${path}: no plugin export found`)
    }
  }

  resolvedPlugins.sort((a, b) => a.name.localeCompare(b.name))
  return resolvedPlugins
}
