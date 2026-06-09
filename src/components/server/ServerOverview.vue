<script setup>
import { ref, onBeforeUnmount, watch, toRef } from 'vue'
import { invoke } from '@/utils/ipc'
import { useSettingsStore } from '@/stores/useSettingsStore'
import { useServerStore } from '@/stores/useServerStore'
import { useMonitor } from '@/composables/useMonitor'
import { useI18n } from '@/composables/useI18n'
import { useLogger } from '@/composables/useLogger'

const props = defineProps({
  serverId: { type: String, required: true },
  sessionId: { type: String, default: null },
  host: { type: String, required: true },
})

const { t } = useI18n()
const log = useLogger('ServerOverview')
const settings = useSettingsStore()
const serverStore = useServerStore()
const sessionIdRef = toRef(props, 'sessionId')
const { cpuPercent, memoryPercent, memoryUsedMib, memoryTotalMib, start, stop } = useMonitor(sessionIdRef, props.host)

const diskPct = ref(null)
const diskUsed = ref(null)
const diskTotal = ref(null)
const timezone = ref(null)
const uptime = ref(null)
const osInfo = ref(null)
const hostname = ref(null)
const kernelVer = ref(null)
const loadAvg = ref(null)
const cpuCores = ref(null)
const connected = ref(false)

let timer = null

watch(
  () => {
    const s = serverStore.servers.find(s => s.id === props.serverId)
    const t = s?.tabs.find(t => t.type === 'overview')
    return t?.status === 'connected'
  },
  (isConnected) => {
    if (isConnected && !connected.value) {
      log.info('overview connected, starting monitor')
      connected.value = true
      start()
      fetchServerInfo()
      clearInterval(timer)
      timer = setInterval(() => fetchServerInfo(), Math.max(5, settings.monitorRefreshSecs) * 1000)
    }
  },
  { immediate: true }
)

async function fetchServerInfo() {
  if (!connected.value) return
  // Disk
  const df = await exec("df -h / 2>/dev/null | awk 'NR==2{print $2,$3,$5}'")
  if (df) {
    const parts = df.trim().split(/\s+/)
    diskTotal.value = parts[0] || null
    diskUsed.value = parts[1] || null
    diskPct.value = parts[2] ? parseInt(parts[2]) : null
  }
  // OS
  const os = await exec("cat /etc/os-release 2>/dev/null | grep PRETTY_NAME | cut -d'\"' -f2 || uname -sr")
  if (os) osInfo.value = os.trim()
  // Hostname
  const hn = await exec("hostname 2>/dev/null")
  if (hn) hostname.value = hn.trim()
  // Kernel
  const kv = await exec("uname -r 2>/dev/null")
  if (kv) kernelVer.value = kv.trim()
  // Timezone
  const tz = await exec("timedatectl show --property=Timezone --value 2>/dev/null || cat /etc/timezone 2>/dev/null")
  if (tz) timezone.value = tz.trim()
  // Uptime
  const up = await exec("uptime -p 2>/dev/null | cut -d' ' -f2- || uptime 2>/dev/null | awk -F'up' '{print $2}' | awk -F',' '{print $1}'")
  if (up) uptime.value = up.trim()
  // Load average
  const la = await exec("cat /proc/loadavg 2>/dev/null | awk '{print $1,$2,$3}'")
  if (la) loadAvg.value = la.trim()
  // CPU cores
  const nc = await exec("nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null")
  if (nc) cpuCores.value = nc.trim()
}

async function exec(cmd) {
  if (!props.sessionId || !connected.value) return null
  try { return (await invoke('exec_command', { sessionId: props.sessionId, command: cmd })).trim() } catch { return null }
}

onBeforeUnmount(() => { connected.value = false; stop(); clearInterval(timer) })
</script>

<template>
  <div class="h-full overflow-y-auto bg-[var(--color-bg-primary)]">
    <div class="max-w-5xl mx-auto p-6 space-y-5">

      <!-- === HEADER === -->
      <div class="flex items-start justify-between flex-wrap gap-3">
        <div class="min-w-0">
          <h2 class="text-lg font-bold text-[var(--color-text-primary)] tracking-tight">
            {{ hostname || props.host }}
          </h2>
          <div class="flex flex-wrap items-center gap-x-3 gap-y-0.5 mt-1">
            <span v-if="osInfo" class="text-xs text-[var(--color-text-secondary)]">{{ osInfo }}</span>
            <span v-if="kernelVer" class="text-xs text-[var(--color-text-tertiary)] font-mono">{{ kernelVer }}</span>
          </div>
        </div>
        <div class="flex items-center gap-2 shrink-0">
          <button @click="serverStore.addTerminalTab(props.serverId)"
            class="inline-flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium rounded-lg
              bg-[var(--color-accent)] text-white hover:brightness-110 transition-all">
            <span class="text-sm">&#x25B6;</span> {{ t('overview.openTerminal') }}
          </button>
          <button @click="serverStore.addFileManagerTab(props.serverId)"
            class="inline-flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium rounded-lg
              bg-[var(--color-bg-secondary)] text-[var(--color-text-primary)] border border-[var(--color-border)]
              hover:bg-[var(--color-bg-tertiary)] transition-all">
            <span class="text-sm">&#x1F4C1;</span> {{ t('overview.fileManager') }}
          </button>
        </div>
      </div>

      <!-- === RESOURCE CARDS === -->
      <div class="grid grid-cols-1 sm:grid-cols-3 gap-4">
        <!-- CPU -->
        <div class="rounded-xl border border-[var(--color-border)] bg-[var(--color-bg-secondary)] p-5
          hover:border-[var(--color-accent)]/30 transition-colors">
          <div class="flex items-center gap-2 mb-3">
            <span class="text-base">&#x1F5A5;</span>
            <span class="text-xs font-semibold text-[var(--color-text-secondary)] uppercase tracking-wide">{{ t('overview.cpu') }}</span>
            <span v-if="cpuCores" class="text-[10px] text-[var(--color-text-tertiary)] ml-auto">{{ cpuCores }} {{ t('overview.cores') }}</span>
          </div>
          <p class="text-3xl font-bold text-[var(--color-text-primary)] tabular-nums">
            {{ cpuPercent !== null ? cpuPercent : '--' }}<span class="text-lg font-normal text-[var(--color-text-tertiary)]">%</span>
          </p>
          <div class="mt-3 h-2 rounded-full bg-[var(--color-bg-tertiary)] overflow-hidden">
            <div class="h-full rounded-full transition-all duration-700 ease-out"
              :style="{
                width: cpuPercent !== null ? Math.min(100, cpuPercent) + '%' : '0%',
                background: cpuPercent > 80 ? 'var(--color-danger)' : cpuPercent > 60 ? 'var(--color-warning)' : 'var(--color-accent)'
              }" />
          </div>
          <p v-if="loadAvg" class="mt-2 text-[10px] text-[var(--color-text-tertiary)] font-mono">
            load: {{ loadAvg }}
          </p>
        </div>

        <!-- Memory -->
        <div class="rounded-xl border border-[var(--color-border)] bg-[var(--color-bg-secondary)] p-5
          hover:border-[var(--color-accent)]/30 transition-colors">
          <div class="flex items-center gap-2 mb-3">
            <span class="text-base">&#x1F9E0;</span>
            <span class="text-xs font-semibold text-[var(--color-text-secondary)] uppercase tracking-wide">{{ t('overview.memory') }}</span>
          </div>
          <p class="text-3xl font-bold text-[var(--color-text-primary)] tabular-nums">
            {{ memoryPercent !== null ? memoryPercent : '--' }}<span class="text-lg font-normal text-[var(--color-text-tertiary)]">%</span>
          </p>
          <div class="mt-3 h-2 rounded-full bg-[var(--color-bg-tertiary)] overflow-hidden">
            <div class="h-full rounded-full transition-all duration-700 ease-out"
              :style="{
                width: memoryPercent !== null ? Math.min(100, memoryPercent) + '%' : '0%',
                background: memoryPercent > 80 ? 'var(--color-danger)' : memoryPercent > 60 ? 'var(--color-warning)' : 'var(--color-accent)'
              }" />
          </div>
          <p v-if="memoryUsedMib !== null" class="mt-2 text-[10px] text-[var(--color-text-tertiary)] font-mono">
            {{ memoryUsedMib >= 1024 ? (memoryUsedMib / 1024).toFixed(1) + ' GiB' : memoryUsedMib + ' MiB' }}
            /
            {{ memoryTotalMib >= 1024 ? (memoryTotalMib / 1024).toFixed(1) + ' GiB' : memoryTotalMib + ' MiB' }}
          </p>
        </div>

        <!-- Disk -->
        <div class="rounded-xl border border-[var(--color-border)] bg-[var(--color-bg-secondary)] p-5
          hover:border-[var(--color-accent)]/30 transition-colors">
          <div class="flex items-center gap-2 mb-3">
            <span class="text-base">&#x1F4BE;</span>
            <span class="text-xs font-semibold text-[var(--color-text-secondary)] uppercase tracking-wide">{{ t('overview.storage') }}</span>
            <span class="text-[10px] text-[var(--color-text-tertiary)] ml-auto">/</span>
          </div>
          <p class="text-3xl font-bold text-[var(--color-text-primary)] tabular-nums">
            {{ diskUsed || '--' }}
          </p>
          <div class="mt-3 h-2 rounded-full bg-[var(--color-bg-tertiary)] overflow-hidden">
            <div class="h-full rounded-full transition-all duration-700 ease-out"
              :style="{
                width: diskPct !== null ? Math.min(100, diskPct) + '%' : '0%',
                background: diskPct > 80 ? 'var(--color-danger)' : diskPct > 60 ? 'var(--color-warning)' : 'var(--color-accent)'
              }" />
          </div>
          <p v-if="diskTotal" class="mt-2 text-[10px] text-[var(--color-text-tertiary)] font-mono">
            {{ diskPct }}% &middot; {{ diskTotal }} total
          </p>
        </div>
      </div>

      <!-- === SERVER INFO === -->
      <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3">
        <!-- Uptime -->
        <div class="rounded-xl border border-[var(--color-border)] bg-[var(--color-bg-secondary)] p-4">
          <div class="flex items-center gap-2 mb-1.5">
            <span class="text-sm">&#x23F1;</span>
            <span class="text-[10px] font-semibold text-[var(--color-text-tertiary)] uppercase tracking-wide">{{ t('overview.uptime') }}</span>
          </div>
          <p class="text-sm font-medium text-[var(--color-text-primary)]">{{ uptime || '--' }}</p>
        </div>

        <!-- Timezone -->
        <div class="rounded-xl border border-[var(--color-border)] bg-[var(--color-bg-secondary)] p-4">
          <div class="flex items-center gap-2 mb-1.5">
            <span class="text-sm">&#x1F310;</span>
            <span class="text-[10px] font-semibold text-[var(--color-text-tertiary)] uppercase tracking-wide">{{ t('overview.timezone') }}</span>
          </div>
          <p class="text-sm font-medium text-[var(--color-text-primary)] truncate" :title="timezone">{{ timezone || '--' }}</p>
        </div>

        <!-- Latency -->
        <div class="rounded-xl border border-[var(--color-border)] bg-[var(--color-bg-secondary)] p-4">
          <div class="flex items-center gap-2 mb-1.5">
            <span class="text-sm">&#x1F4E1;</span>
            <span class="text-[10px] font-semibold text-[var(--color-text-tertiary)] uppercase tracking-wide">{{ t('overview.latency') }}</span>
          </div>
          <p class="text-sm font-medium text-[var(--color-text-primary)] tabular-nums">
            {{ serverStore.activeServer?.latency !== null && serverStore.activeServer?.latency !== undefined
              ? serverStore.activeServer.latency + ' ms'
              : '--' }}
          </p>
        </div>

        <!-- IP / Host -->
        <div class="rounded-xl border border-[var(--color-border)] bg-[var(--color-bg-secondary)] p-4">
          <div class="flex items-center gap-2 mb-1.5">
            <span class="text-sm">&#x1F4BB;</span>
            <span class="text-[10px] font-semibold text-[var(--color-text-tertiary)] uppercase tracking-wide">{{ t('overview.host') }}</span>
          </div>
          <p class="text-sm font-medium text-[var(--color-text-primary)] font-mono truncate">{{ props.host }}</p>
        </div>
      </div>

    </div>
  </div>
</template>
