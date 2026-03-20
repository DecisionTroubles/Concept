import { devtools } from '@vue/devtools'
import { createPinia } from 'pinia'
import { createApp } from 'vue'
import App from './App.vue'
import './assets/main.css'
import { bootstrapPlugins } from '@/plugins/bootstrap'
import { useTheme } from '@/composables/useTheme'
import { getGraphTraceBuffer } from '@/stores/graph/debug'

if (process.env.NODE_ENV === 'development' && import.meta.env.VITE_ENABLE_DEVTOOLS === '1') {
  try {
    devtools.connect('http://localhost', 8098)
  } catch {
    // Devtools server is optional in tauri dev.
  }
}
const app = createApp(App)
const pinia = createPinia()

await bootstrapPlugins()
const theme = useTheme()
theme.syncThemesFromKernel()
theme.initializeTheme()

app.use(pinia)

app.config.errorHandler = (error, instance, info) => {
  console.error(error)
  const message = error instanceof Error ? error.message : String(error)
  if (message.includes('Maximum recursive updates exceeded')) {
    const trace = getGraphTraceBuffer().slice(-24)
    console.groupCollapsed('[graph-trace dump] recursive update')
    console.log('component:', instance?.type)
    console.log('info:', info)
    console.table(trace.map(entry => ({
      seq: entry.seq,
      event: entry.event,
      at: entry.at,
      payload: JSON.stringify(entry.payload ?? {}),
    })))
    console.groupEnd()
  }
}

app.mount('#app')
