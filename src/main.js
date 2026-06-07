import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import './assets/css/main.css'

document.addEventListener('contextmenu', e => e.preventDefault())

const app = createApp(App)
const pinia = createPinia()

app.use(pinia)
app.mount('#app')
