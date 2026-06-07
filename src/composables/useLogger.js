import { reactive, ref } from 'vue'

const MAX_LOGS = 200

const state = reactive({
  entries: [],
  showPanel: false,
})

const errorCount = ref(0)

function add(level, source, message, data) {
  const entry = {
    id: ++errorCount.value,
    timestamp: new Date().toISOString().slice(11, 23),
    level,
    source,
    message,
    data: data !== undefined ? (typeof data === 'object' ? JSON.stringify(data) : String(data)) : '',
  }
  state.entries.push(entry)
  if (state.entries.length > MAX_LOGS) {
    state.entries.shift()
  }

  const prefix = `[${entry.timestamp}][${source}]`
  const args = [prefix, message]
  if (data !== undefined) args.push(data)
  switch (level) {
    case 'error': console.error(...args); break
    case 'warn':  console.warn(...args); break
    default:      console.log(...args); break
  }

  if (level === 'error' || level === 'warn') {
    state.showPanel = true
  }
}

export function useLogger(source) {
  function debug(msg, data) { add('debug', source, msg, data) }
  function info(msg, data)  { add('info', source, msg, data) }
  function warn(msg, data)  { add('warn', source, msg, data) }
  function error(msg, data) { add('error', source, msg, data) }
  function trace(fn, label) {
    return async (...args) => {
      info(`${label || fn.name} called`, args.length ? args : undefined)
      try {
        const result = await fn(...args)
        info(`${label || fn.name} OK`)
        return result
      } catch (e) {
        error(`${label || fn.name} FAILED: ${e?.message || e}`, e)
        throw e
      }
    }
  }

  function togglePanel() { state.showPanel = !state.showPanel }

  return { debug, info, warn, error, trace, logState: state, togglePanel }
}
