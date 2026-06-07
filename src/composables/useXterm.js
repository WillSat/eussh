import { ref, onBeforeUnmount, watch } from 'vue'
import { Terminal } from '@xterm/xterm'
import { FitAddon } from '@xterm/addon-fit'
import { WebLinksAddon } from '@xterm/addon-web-links'
import { WebglAddon } from '@xterm/addon-webgl'
import { invoke } from '@/utils/ipc'
import { getTerminalTheme } from '@/utils/theme'
import { listen } from '@tauri-apps/api/event'
import { useSettingsStore } from '@/stores/useSettingsStore'
import '@xterm/xterm/css/xterm.css'

// Wait for browser layout pass — rAF fires after layout, nextTick does not
function afterLayout() {
  return new Promise(resolve => requestAnimationFrame(resolve))
}

export function useXterm(sessionIdRef) {
  const term = ref(null)
  const fitAddon = new FitAddon()
  const containerRef = ref(null)
  const unlisteners = []
  const sessionId = ref(sessionIdRef.value)
  let resizeObserver = null

  watch(sessionIdRef, (newId) => {
    sessionId.value = newId
  })

  async function init() {
    const el = containerRef.value
    if (!el) return

    const settings = useSettingsStore()

    term.value = new Terminal({
      fontFamily: settings.fontFamily,
      fontSize: settings.fontSize,
      cursorBlink: true,
      cursorStyle: settings.cursorStyle,
      theme: getTerminalTheme(settings.terminalColorPreset),
      scrollback: settings.scrollback,
      allowProposedApi: true,
    })

    term.value.loadAddon(fitAddon)
    term.value.loadAddon(new WebLinksAddon())

    try { term.value.loadAddon(new WebglAddon()) } catch {}

    term.value.open(el)

    // rAF ensures browser has done layout — nextTick is too early (microtask)
    await afterLayout()
    fitAddon.fit()
    term.value.focus()

    term.value.onData(async (data) => {
      const sid = sessionId.value
      if (!sid) return
      try {
        const encoder = new TextEncoder()
        await invoke('terminal_write', { sessionId: sid, data: Array.from(encoder.encode(data)) })
      } catch {}
    })

    unlisteners.push(await listen('terminal-data', (event) => {
      if (event.payload.session_id === sessionId.value) {
        const decoder = new TextDecoder()
        const text = decoder.decode(new Uint8Array(event.payload.data))
        term.value?.write(text)
      }
    }))

    resizeObserver = new ResizeObserver(() => {
      fitAddon.fit()
      if (term.value && sessionId.value) {
        invoke('terminal_resize', {
          sessionId: sessionId.value,
          cols: term.value.cols,
          rows: term.value.rows,
        }).catch(() => {})
      }
    })
    resizeObserver.observe(el)
  }

  function fit() { fitAddon.fit() }

  async function refitAndFocus() {
    await afterLayout()
    fitAddon.fit()
    term.value?.focus()
  }

  function focus() { term.value?.focus() }

  function getSelection() { return term.value?.getSelection() || '' }

  function destroy() {
    unlisteners.forEach(fn => fn?.())
    resizeObserver?.disconnect()
    try { term.value?.dispose() } catch {}
    term.value = null
  }

  onBeforeUnmount(destroy)

  return { term, containerRef, init, fit, focus, refitAndFocus, getSelection, destroy }
}
