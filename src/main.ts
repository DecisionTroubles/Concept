import { devtools } from '@vue/devtools'
import { createPinia } from 'pinia'
import { createApp } from 'vue'
import App from './App.vue'
import './assets/main.css'
import { bootstrapPlugins } from '@/plugins/bootstrap'
import { useTheme } from '@/composables/useTheme'

if (process.env.NODE_ENV === 'development') {
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
app.mount('#app')
