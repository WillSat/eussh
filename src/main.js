import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import './assets/css/main.css'

// Disable browser context menu in the app container only (not in devtools/overlays)
document.getElementById('app')?.addEventListener('contextmenu', e => {
  // Allow right-click in input fields and textareas
  const tag = e.target?.tagName?.toLowerCase()
  if (tag === 'input' || tag === 'textarea') return
  e.preventDefault()
})

const app = createApp(App)
const pinia = createPinia()

app.use(pinia)
app.mount('#app')
