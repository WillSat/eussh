import { ref, watch, onBeforeUnmount } from 'vue'
import { invoke } from '@/utils/ipc'
import { useSettingsStore } from '@/stores/useSettingsStore'
import { useLogger } from '@/composables/useLogger'

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

  const log = useLogger('ServerData')

  let disposed = false
  let timer = null
  let inflightDynamic = false
  let inflightStatic = false
  let staticCache = null   // cached hostname/os/kernel/timezone/ips/geo across reconnects
  const dataStale = ref(false)  // true when a command failed ≥2 consecutive times
  // Per-command consecutive failure counters
  const cmdFails = {
    loadavg: 0, nproc: 0, df: 0, mem: 0, swap: 0,
    hostname: 0, os: 0, ips: 0, kernel: 0, uptime: 0, tz: 0,
  }

  function markFail(key) { cmdFails[key]++; if (cmdFails[key] >= 2) dataStale.value = true }
  function markOk(key)   { cmdFails[key] = 0 }
  function clearStaleIfAllOk() {
    if (Object.values(cmdFails).every(v => v === 0)) dataStale.value = false
  }

  // ── Client-side geo lookup (replaces slow SSH curls) ─────────────
  function isPrivateIpv4(ip) {
    const parts = ip.split('.').map(Number)
    if (parts.length !== 4 || parts.some(n => !Number.isInteger(n) || n < 0 || n > 255)) return true
    const [a, b] = parts
    return a === 10 ||
      a === 127 ||
      (a === 172 && b >= 16 && b <= 31) ||
      (a === 192 && b === 168) ||
      (a === 169 && b === 254)
  }

  async function fetchGeoLocation() {
    if (!settings.showGeoLookup) {
      geoLocation.value = null
      if (staticCache) staticCache.geoLocation = null
      return
    }

    const ips = allIps.value
    if (!ips.length) return
    // Find first public IPv4 (skip loopback, private, link-local)
    const publicIp = ips.find(ip => ip.includes('.') && !ip.includes(':') && !isPrivateIpv4(ip))
    if (!publicIp) return
    try {
      const res = await fetch(`http://ip-api.com/json/${publicIp}?fields=country,city,lat,lon`)
      if (!res.ok) return
      const g = await res.json()
      if (g.lat != null && g.lon != null) {
        geoLocation.value = { lat: Number(g.lat), lng: Number(g.lon), label: [g.city, g.country].filter(Boolean).join(', ') }
        // Update cache so geo persists across reconnects
        if (staticCache) staticCache.geoLocation = geoLocation.value
      }
    } catch { /* geo is non-critical */ }
  }

  // ── fetchDynamic ─────────────────────────────────────────────────
  // 5 commands: loadavg, nproc, df, mem, swap. top -bn1 and /proc/meminfo are conditional fallbacks.
  async function fetchDynamic() {
    if (disposed || inflightDynamic) return
    const sid = sessionIdRef?.value
    if (!sid) return
    inflightDynamic = true
    const t0 = performance.now()
    const sidTag = sid.slice(0, 8)
    try {
      loading.value = true

      const cmdNames = ['loadavg', 'nproc', 'df', 'mem', 'swap']
      const t1 = performance.now()
      const settled = await Promise.allSettled([
        safeExec(sid, "cat /proc/loadavg 2>/dev/null | awk '{print $1,$2,$3}'"),                                     // 0
        safeExec(sid, 'nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 1'),                               // 1
        safeExec(sid, "df -h / 2>/dev/null | awk 'NR==2{print $2,$3,$5}'"),                                          // 2
        safeExec(sid, "free -m 2>/dev/null | awk '/^Mem:/{printf \"%.0f %.0f\", $3, $2}'"),                           // 3 mem
        safeExec(sid, "free -m 2>/dev/null | awk '/^Swap:/{printf \"%.0f %.0f\", $3, $2}'"),                          // 4 swap
      ])
      const elapsed = Math.round(performance.now() - t1)

      if (disposed) return
      const v = settled.map(r => (r.status === 'fulfilled' ? r.value : null))

      // Log each command result
      settled.forEach((r, i) => {
        const ok = r.status === 'fulfilled' && r.value !== null
        if (ok) markOk(cmdNames[i]); else markFail(cmdNames[i])
        log.debug(`[${sidTag}] ${cmdNames[i]}: ${ok ? 'OK' : 'FAIL'}`, ok ? undefined : { status: r.status })
      })

      if (v[0]) loadAvg.value = v[0]
      if (v[1]) cpuCores.value = v[1]

      // Disk
      if (v[2]) {
        const p = v[2].split(/\s+/)
        if (p.length >= 2) { diskTotal.value = p[0] || null; diskUsed.value = p[1] || null; diskPercent.value = p[2] ? parseInt(p[2], 10) : null }
      }

      // Memory (independent command)
      if (v[3]) {
        const p = v[3].split(/\s+/)
        if (p.length >= 2) {
          const used = parseNum(p[0]); const total = parseNum(p[1])
          if (used !== null && total !== null && total > 0) { memoryUsedMib.value = used; memoryTotalMib.value = total; memoryPercent.value = Math.round((used / total) * 100) }
        }
      }
      // Fallback: /proc/meminfo only if free -m mem failed
      if (memoryUsedMib.value == null && v[3] === null) {
        const mi = await safeExec(sid, "cat /proc/meminfo 2>/dev/null | grep -E '^(MemTotal|MemAvailable):' | awk '{print $2}'")
        if (mi) {
          const miLines = mi.split('\n').filter(Boolean)
          if (miLines.length >= 2) {
            const tKb = parseNum(miLines[0]); const aKb = parseNum(miLines[1])
            if (tKb !== null && aKb !== null && tKb > 0) {
              const uKb = tKb - aKb
              memoryUsedMib.value = Math.round(uKb / 1024); memoryTotalMib.value = Math.round(tKb / 1024); memoryPercent.value = Math.round((uKb / tKb) * 100)
              markOk('mem')  // fallback succeeded
            }
          }
        }
      }

      // Swap (independent command)
      if (v[4]) {
        const p = v[4].split(/\s+/)
        if (p.length >= 2) {
          const used = parseNum(p[0]); const total = parseNum(p[1])
          if (used !== null && total !== null) { swapUsedMib.value = used; swapTotalMib.value = total; swapPercent.value = total > 0 ? Math.round((used / total) * 100) : 0 }
        }
      }

      // CPU percent from 1-min load average
      if (v[0]) {
        const la = parseFloatNum(String(v[0]).split(/\s+/)[0])
        const nc = parseNum(cpuCores.value) || 1
        if (la !== null && nc > 0) cpuPercent.value = Math.min(100, Math.round((la / nc) * 100))
      }
      // Fallback: only run top -bn1 when load-avg path failed
      if (cpuPercent.value == null) {
        log.debug(`[${sidTag}] cpu: load-avg failed, trying top -bn1 fallback`)
        const topResult = await safeExec(sid, "top -bn1 2>/dev/null | grep 'Cpu(s)' | awk '{print $2}' | cut -d'%' -f1", 10000)
        if (topResult) { const p = parseFloatNum(topResult); if (p !== null) cpuPercent.value = Math.round(p) }
      }

      const totalMs = Math.round(performance.now() - t0)
      log.info(`[${sidTag}] dynamic fetched in ${totalMs}ms (cmds:${elapsed}ms) CPU=${cpuPercent.value}% Mem=${memoryPercent.value}% Disk=${diskPercent.value}% Swap=${swapPercent.value ?? '-'}`)
      clearStaleIfAllOk()
    } finally {
      loading.value = false
      if (!firstLoadDone.value) firstLoadDone.value = true
      inflightDynamic = false
    }
  }

  // ── fetchStatic ──────────────────────────────────────────────────
  // 5 commands: hostname, os, ips, kernel, uptime, timezone. Geo is client-side.
  // Cached after first successful fetch.
  async function fetchStatic() {
    if (disposed || inflightStatic) return
    const sid = sessionIdRef?.value
    if (!sid) return

    // Return cached static data if available (same session reconnect)
    if (staticCache) {
      hostname.value = staticCache.hostname
      osInfo.value = staticCache.osInfo
      kernelVer.value = staticCache.kernelVer
      timezone.value = staticCache.timezone
      allIps.value = staticCache.allIps
      geoLocation.value = staticCache.geoLocation
      staticLoading.value = false
      return
    }

    inflightStatic = true
    staticLoading.value = true
    const t0 = performance.now()
    const sidTag = sid.slice(0, 8)
    try {
      // Gather all IPs (IPv4 + IPv6), one-per-line output
      const ipCmd = "(hostname -I 2>/dev/null; ip -6 addr show scope global 2>/dev/null | grep -oP '(?<=inet6 )[\\da-f:]+(?=/)') | tr ' ' '\\n' | grep -v '^$' | grep -v '^fe80'"

      const cmdNames = ['hostname', 'os', 'ips', 'kernel', 'uptime', 'tz']
      const t1 = performance.now()
      const settled = await Promise.allSettled([
        safeExec(sid, 'hostname 2>/dev/null'),                                                                        // 0
        safeExec(sid, "cat /etc/os-release 2>/dev/null | grep PRETTY_NAME | cut -d'\"' -f2 || uname -sr 2>/dev/null"), // 1
        safeExec(sid, ipCmd),                                                                                          // 2  all IPs
        safeExec(sid, 'uname -r 2>/dev/null'),                                                                         // 3
        safeExec(sid, "uptime -p 2>/dev/null | cut -d' ' -f2- || uptime 2>/dev/null | awk -F'up ' '{print $2}' | awk -F',' '{print $1}'"), // 4
        safeExec(sid, 'timedatectl show -p Timezone --value 2>/dev/null || cat /etc/timezone 2>/dev/null || readlink /etc/localtime 2>/dev/null | sed "s|.*/zoneinfo/||"'), // 5
      ])
      const elapsed = Math.round(performance.now() - t1)

      if (disposed) return
      const v = settled.map(r => (r.status === 'fulfilled' ? r.value : null))

      // Log each command result
      settled.forEach((r, i) => {
        const ok = r.status === 'fulfilled' && r.value !== null
        if (ok) markOk(cmdNames[i]); else markFail(cmdNames[i])
        log.debug(`[${sidTag}] ${cmdNames[i]}: ${ok ? 'OK' : 'FAIL'}`, ok ? undefined : { status: r.status })
      })

      if (v[0]) hostname.value = v[0]
      if (v[1]) osInfo.value = v[1]
      if (v[2]) allIps.value = v[2].split(/\n/).map(s => s.trim()).filter(Boolean)
      if (v[3]) kernelVer.value = v[3]
      if (v[4]) uptime.value = v[4]
      if (v[5]) timezone.value = v[5]

      // Populate cache for reconnects
      staticCache = {
        hostname: hostname.value,
        osInfo: osInfo.value,
        kernelVer: kernelVer.value,
        timezone: timezone.value,
        allIps: allIps.value,
        geoLocation: geoLocation.value,
      }

      const totalMs = Math.round(performance.now() - t0)
      log.info(`[${sidTag}] static fetched in ${totalMs}ms (cmds:${elapsed}ms) host=${hostname.value} ips=${allIps.value.length}`)
    } finally {
      staticLoading.value = false
      inflightStatic = false
    }

    // Fire-and-forget client-side geo (non-blocking)
    fetchGeoLocation()
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
  watch(() => settings.showGeoLookup, (enabled) => {
    if (enabled) fetchGeoLocation()
    else {
      geoLocation.value = null
      if (staticCache) staticCache.geoLocation = null
    }
  })
  onBeforeUnmount(() => { disposed = true; stopPolling(); staticCache = null })

  return {
    hostname, osInfo, kernelVer, uptime, timezone,
    cpuCores, cpuPercent,
    memoryUsedMib, memoryTotalMib, memoryPercent,
    swapUsedMib, swapTotalMib, swapPercent,
    diskTotal, diskUsed, diskPercent,
    allIps, geoLocation,
    staticLoading, firstLoadDone, dataStale,
  }
}
