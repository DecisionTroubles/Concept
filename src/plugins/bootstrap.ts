import { appKernel } from '@/core/kernel'
import { defaultPlugin } from '@/plugins/defaultPlugin'
import { userPlugins } from '@/plugins/userPlugins'

let bootstrapped = false

export async function bootstrapPlugins() {
  if (bootstrapped) return
  bootstrapped = true

  await appKernel.install(defaultPlugin)
  for (const plugin of userPlugins) {
    await appKernel.install(plugin)
  }
}

