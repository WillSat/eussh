import { reactive } from 'vue'

const state = reactive({
  message: '',
  type: 'info',
  visible: false,
})

let timer = null

export function useToast() {
  function show(message, type = 'info', duration = 4000) {
    clearTimeout(timer)
    state.message = message
    state.type = type
    state.visible = true
    timer = setTimeout(() => {
      state.visible = false
    }, duration)
  }

  function error(message) { show(message, 'error', 5000) }
  function success(message) { show(message, 'success', 3000) }
  function info(message) { show(message, 'info', 3000) }
  function close() { state.visible = false }

  return { state, show, error, close }
}
