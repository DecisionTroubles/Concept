import { appKernel } from '@/core/kernel'
import { corePlugins } from '@/plugins/corePlugins'
import { loadUserPlugins } from '@/plugins/userPlugins'

let bootstrapped = false

export async function bootstrapPlugins() {
  if (bootstrapped) return
  bootstrapped = true

  for (const plugin of corePlugins) {
    await appKernel.install(plugin)
  }

  for (const plugin of await loadUserPlugins()) {
    await appKernel.install(plugin)
  }
}
