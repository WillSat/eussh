import { ref, watch, onBeforeUnmount } from 'vue'
import { invoke } from '@/utils/ipc'
import { useSettingsStore } from '@/stores/useSettingsStore'

function execWithTimeout(sessionId, command, timeoutMs = 6000) {
  if (!sessionId) return Promise.resolve(null)
  const work = invoke('exec_command', { sessionId, command })
    .then(r => (typeof r === 'string' ? r : String(r ?? '')).trim())
    .catch(() => null)
  let tid
  const timeout = new Promise(resolve => { tid = setTimeout(() => resolve(null), timeoutMs) })
  return Promise.race([work, timeout]).finally(() => clearTimeout(tid))
}

async function safeExec(sessionId, command, timeoutMs = 6000) {
  try { return await execWithTimeout(sessionId, command, timeoutMs) } catch { return null }
}

function parseNum(raw) {
  if (raw == null || raw === '') return null
  const n = parseInt(raw.toString().trim(), 10)
  return Number.isFinite(n) ? n : null
}

function parseFloatNum(raw) {
  if (raw == null || raw === '') return null
  const n = parseFloat(raw.toString().trim())
  return Number.isFinite(n) ? n : null
}

/**
 * fetchDynamic() — polled on interval: CPU / memory / disk / swap
 * fetchStatic()  — fired once:       hostname, OS, kernel, uptime, timezone, all IPs, geo
 */
export function useServerData(sessionIdRef) {
  const settings = useSettingsStore()

  const hostname  = ref(null)
  const osInfo    = ref(null)
  const kernelVer = ref(null)
  const uptime    = ref(null)
  const timezone  = ref(null)
  const loadAvg   = ref(null)
  const cpuCores  = ref(null)
  const cpuPercent = ref(null)

  const memoryUsedMib  = ref(null)
  const memoryTotalMib = ref(null)
  const memoryPercent  = ref(null)

  const swapUsedMib  = ref(null)
  const swapTotalMib = ref(null)
  const swapPercent  = ref(null)

  const diskTotal  = ref(null)
  const diskUsed   = ref(null)
  const diskPercent = ref(null)

  const allIps      = ref([])          // string[] — IPv4 + IPv6
  const geoLocation = ref(null)

  const loading       = ref(false)
  const staticLoading = ref(false)
  const firstLoadDone = ref(false)

  let disposed = false
  let timer = null

  // ── fetchDynamic ─────────────────────────────────────────────────
  async function fetchDynamic() {
    if (disposed) return
    const sid = sessionIdRef?.value
    if (!sid) return

    loading.value = true

    const settled = await Promise.allSettled([
      safeExec(sid, "cat /proc/loadavg 2>/dev/null | awk '{print $1,$2,$3}'"),                                     // 0
      safeExec(sid, 'nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 1'),                               // 1
      safeExec(sid, "df -h / 2>/dev/null | awk 'NR==2{print $2,$3,$5}'"),                                          // 2
      safeExec(sid, "free -m 2>/dev/null | awk '/^Mem:/{printf \"%.0f %.0f\", $3, $2}'"),                           // 3
      safeExec(sid, "free -m 2>/dev/null | awk '/^Swap:/{printf \"%.0f %.0f\", $3, $2}'"),                          // 4
      safeExec(sid, "top -bn1 2>/dev/null | grep 'Cpu(s)' | awk '{print $2}' | cut -d'%' -f1", 10000),             // 5
      safeExec(sid, "cat /proc/meminfo 2>/dev/null | grep -E '^(MemTotal|MemAvailable):' | awk '{print $2}'"),      // 6
    ])

    if (disposed) return
    const v = settled.map(r => (r.status === 'fulfilled' ? r.value : null))

    if (v[0]) loadAvg.value = v[0]
    if (v[1]) cpuCores.value = v[1]

    if (v[2]) {
      const p = v[2].split(/\s+/)
      if (p.length >= 2) { diskTotal.value = p[0] || null; diskUsed.value = p[1] || null; diskPercent.value = p[2] ? parseInt(p[2], 10) : null }
    }

    if (v[3]) {
      const p = v[3].split(/\s+/)
      if (p.length >= 2) {
        const used = parseNum(p[0]); const total = parseNum(p[1])
        if (used !== null && total !== null && total > 0) { memoryUsedMib.value = used; memoryTotalMib.value = total; memoryPercent.value = Math.round((used / total) * 100) }
      }
    } else if (v[6]) {
      const lines = v[6].split('\n').filter(Boolean)
      if (lines.length >= 2) {
        const tKb = parseNum(lines[0]); const aKb = parseNum(lines[1])
        if (tKb !== null && aKb !== null && tKb > 0) {
          const uKb = tKb - aKb
          memoryUsedMib.value = Math.round(uKb / 1024); memoryTotalMib.value = Math.round(tKb / 1024); memoryPercent.value = Math.round((uKb / tKb) * 100)
        }
      }
    }

    if (v[4]) {
      const p = v[4].split(/\s+/)
      if (p.length >= 2) {
        const used = parseNum(p[0]); const total = parseNum(p[1])
        if (used !== null && total !== null) { swapUsedMib.value = used; swapTotalMib.value = total; swapPercent.value = total > 0 ? Math.round((used / total) * 100) : 0 }
      }
    }

    if (v[0]) {
      const la = parseFloatNum(String(v[0]).split(/\s+/)[0])
      const nc = parseNum(cpuCores.value) || 1
      if (la !== null && nc > 0) cpuPercent.value = Math.min(100, Math.round((la / nc) * 100))
    }
    if (cpuPercent.value == null && v[5]) { const p = parseFloatNum(v[5]); if (p !== null) cpuPercent.value = Math.round(p) }

    loading.value = false
    if (!firstLoadDone.value) firstLoadDone.value = true
  }

  // ── fetchStatic ──────────────────────────────────────────────────
  async function fetchStatic() {
    if (disposed) return
    const sid = sessionIdRef?.value
    if (!sid) return

    staticLoading.value = true

    // Command 2: gather all IPs (IPv4 + IPv6), one-per-line output
    const ipCmd = "(hostname -I 2>/dev/null; ip -6 addr show scope global 2>/dev/null | grep -oP '(?<=inet6 )[\\da-f:]+(?=/)') | tr ' ' '\\n' | grep -v '^$' | grep -v '^fe80'"

    const settled = await Promise.allSettled([
      safeExec(sid, 'hostname 2>/dev/null'),                                                                        // 0
      safeExec(sid, "cat /etc/os-release 2>/dev/null | grep PRETTY_NAME | cut -d'\"' -f2 || uname -sr 2>/dev/null"), // 1
      safeExec(sid, ipCmd),                                                                                          // 2  all IPs
      safeExec(sid, 'uname -r 2>/dev/null'),                                                                         // 3
      safeExec(sid, "uptime -p 2>/dev/null | cut -d' ' -f2- || uptime 2>/dev/null | awk -F'up ' '{print $2}' | awk -F',' '{print $1}'"), // 4
      safeExec(sid, 'timedatectl show -p Timezone --value 2>/dev/null || cat /etc/timezone 2>/dev/null || readlink /etc/localtime 2>/dev/null | sed "s|.*/zoneinfo/||"'), // 5
      safeExec(sid, 'curl -s --max-time 5 ipinfo.io 2>/dev/null', 10000),                                           // 6
      safeExec(sid, 'curl -s --max-time 5 "http://ip-api.com/json/?fields=country,city,lat,lon" 2>/dev/null', 10000), // 7
    ])

    if (disposed) return
    const v = settled.map(r => (r.status === 'fulfilled' ? r.value : null))

    if (v[0]) hostname.value = v[0]
    if (v[1]) osInfo.value = v[1]
    if (v[2]) allIps.value = v[2].split(/\n/).map(s => s.trim()).filter(Boolean) // split into array
    if (v[3]) kernelVer.value = v[3]
    if (v[4]) uptime.value = v[4]
    if (v[5]) timezone.value = v[5]

    // Geo
    if (v[6]) {
      try {
        const g = JSON.parse(v[6])
        if (g.loc) {
          const [lat, lng] = g.loc.split(',').map(Number)
          if (Number.isFinite(lat) && Number.isFinite(lng)) geoLocation.value = { lat, lng, label: [g.city, g.country].filter(Boolean).join(', ') }
        }
      } catch {}
    }
    if (!geoLocation.value && v[7]) {
      try {
        const g = JSON.parse(v[7])
        if (g.lat != null && g.lon != null) geoLocation.value = { lat: Number(g.lat), lng: Number(g.lon), label: [g.city, g.country].filter(Boolean).join(', ') }
      } catch {}
    }

    staticLoading.value = false
  }

  // ── Polling ─────────────────────────────────────────────────────
  function startPolling() {
    if (disposed) return
    stopPolling()
    fetchDynamic()
    fetchStatic()
    const ms = Math.max(3, settings.monitorRefreshSecs) * 1000
    timer = setInterval(fetchDynamic, ms)
  }

  function stopPolling() { clearInterval(timer); timer = null }

  watch(sessionIdRef, (sid) => { if (sid && !disposed) startPolling(); else if (!sid) stopPolling() }, { immediate: true })
  onBeforeUnmount(() => { disposed = true; stopPolling() })

  return {
    hostname, osInfo, kernelVer, uptime, timezone,
    loadAvg, cpuCores, cpuPercent,
    memoryUsedMib, memoryTotalMib, memoryPercent,
    swapUsedMib, swapTotalMib, swapPercent,
    diskTotal, diskUsed, diskPercent,
    allIps, geoLocation,
    loading, staticLoading, firstLoadDone,
    fetchDynamic, startPolling, stopPolling,
  }
}
