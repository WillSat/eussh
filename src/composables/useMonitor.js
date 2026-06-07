import { ref, onBeforeUnmount } from 'vue'
import { invoke } from '@/utils/ipc'
import { useSettingsStore } from '@/stores/useSettingsStore'

export function useMonitor(sessionIdRef, host) {
  const cpuPercent = ref(null)
  const memoryPercent = ref(null)
  const memoryUsedMib = ref(null)
  const memoryTotalMib = ref(null)
  const alive = ref(true)

  const settings = useSettingsStore()
  let cpuTimer = null

  async function exec(cmd) {
    if (!alive.value) return null
    const sid = sessionIdRef.value
    if (!sid) return null
    try {
      const out = await invoke('exec_command', { sessionId: sid, command: cmd })
      return out.trim()
    } catch {
      // Session gone — stop silently, happens on disconnect
      alive.value = false
      stop()
      return null
    }
  }

  async function fetchCpu() {
    let out = await exec("cat /proc/loadavg 2>/dev/null | awk '{print $1}'")
    if (out) {
      const load = parseFloat(out)
      if (!isNaN(load)) {
        const ncpuOut = await exec("nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 1")
        const ncpu = parseInt(ncpuOut) || 1
        cpuPercent.value = Math.min(100, Math.round((load / ncpu) * 100))
        return
      }
    }
    out = await exec("top -bn1 2>/dev/null | grep 'Cpu(s)' | awk '{print $2}' | cut -d'%' -f1")
    if (out) {
      const v = parseFloat(out)
      if (!isNaN(v)) { cpuPercent.value = Math.round(v); return }
    }
    cpuPercent.value = null
  }

  async function fetchMemory() {
    let out = await exec("free -m 2>/dev/null | awk '/^Mem:/{printf \"%.0f %.0f\", $3, $2}'")
    if (out) {
      const parts = out.split(/\s+/)
      if (parts.length >= 2) {
        const used = parseFloat(parts[0])
        const total = parseFloat(parts[1])
        if (!isNaN(used) && !isNaN(total) && total > 0) {
          memoryPercent.value = Math.round((used / total) * 100)
          memoryUsedMib.value = Math.round(used)
          memoryTotalMib.value = Math.round(total)
          return
        }
      }
    }
    out = await exec("cat /proc/meminfo 2>/dev/null | grep -E '^(MemTotal|MemAvailable):' | awk '{print $2}'")
    if (out) {
      const lines = out.split('\n')
      if (lines.length >= 2) {
        const total = parseFloat(lines[0])
        const avail = parseFloat(lines[1])
        if (!isNaN(total) && !isNaN(avail) && total > 0) {
          memoryPercent.value = Math.round(((total - avail) / total) * 100)
          memoryUsedMib.value = Math.round((total - avail) / 1024)
          memoryTotalMib.value = Math.round(total / 1024)
          return
        }
      }
    }
    memoryPercent.value = null
  }

  function start() {
    alive.value = true
    const refresh = Math.max(1, settings.monitorRefreshSecs) * 1000

    fetchCpu()
    fetchMemory()

    cpuTimer = setInterval(() => { if (alive.value) { fetchCpu(); fetchMemory() } }, refresh)
  }

  function stop() {
    alive.value = false
    clearInterval(cpuTimer)
    cpuTimer = null
  }

  onBeforeUnmount(stop)

  return { cpuPercent, memoryPercent, memoryUsedMib, memoryTotalMib, start, stop }
}
