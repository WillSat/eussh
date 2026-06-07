<script setup>
import { ref, onBeforeUnmount, watch, computed, toRef } from 'vue'
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

const diskUsage = ref(null)
const diskTotal = ref(null)
const timezone = ref(null)
const uptime = ref(null)
const osInfo = ref(null)
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
      fetchExtraInfo()
      clearInterval(timer)
      timer = setInterval(() => fetchExtraInfo(), Math.max(5, settings.monitorRefreshSecs) * 1000)
    }
  },
  { immediate: true }
)

async function fetchExtraInfo() {
  if (!connected.value) return
  const df = await exec("df -h / 2>/dev/null | awk 'NR==2{print $2,$3,$5}'")
  if (df) {
    const parts = df.trim().split(/\s+/)
    diskTotal.value = parts[0] || null
    diskUsage.value = parts[1] || null
  }
  const os = await exec("cat /etc/os-release 2>/dev/null | grep PRETTY_NAME | cut -d'\"' -f2 || uname -sr")
  if (os) osInfo.value = os.trim()
  const tz = await exec("timedatectl show --property=Timezone --value 2>/dev/null || cat /etc/timezone 2>/dev/null")
  if (tz) timezone.value = tz.trim()
  const up = await exec("uptime -p 2>/dev/null | cut -d' ' -f2- || uptime 2>/dev/null | awk -F'up' '{print $2}' | awk -F',' '{print $1}'")
  if (up) uptime.value = up.trim()
}

async function exec(cmd) {
  if (!props.sessionId || !connected.value) return null
  try { return (await invoke('exec_command', { sessionId: props.sessionId, command: cmd })).trim() } catch { return null }
}

onBeforeUnmount(() => { connected.value = false; stop(); clearInterval(timer) })

function barStyle(pct) {
  if (pct === null) return { width: '0%' }
  const w = Math.min(100, Math.max(0, pct))
  const c = w > 80 ? 'var(--color-danger)' : w > 60 ? 'var(--color-warning)' : 'var(--color-accent)'
  return { width: `${w}%`, background: c }
}

function formatMemory(mib) {
  if (mib === null) return '--'
  if (mib >= 1024) return (mib / 1024).toFixed(1) + ' GiB'
  return mib + ' MiB'
}
</script>

<template>
  <div class="h-full overflow-y-auto p-6 bg-[var(--color-bg-primary)]">
    <div class="max-w-4xl mx-auto space-y-4">
      <!-- Header -->
      <div class="flex items-center justify-between">
        <div>
          <h2 class="text-lg font-semibold text-[var(--color-text-primary)]">{{ t('overview.title') }}</h2>
          <p v-if="osInfo" class="text-xs text-[var(--color-text-tertiary)] mt-0.5">{{ osInfo }}</p>
        </div>
        <div class="flex items-center gap-2">
          <button
            @click="serverStore.addTerminalTab(props.serverId)"
            class="px-3 py-1.5 text-xs font-medium rounded-[var(--radius-sm)]
              bg-[var(--color-accent)] text-white hover:bg-[var(--color-accent-hover)]
              transition-colors"
          >{{ t('overview.openTerminal') }}</button>
          <button
            @click="serverStore.addFileManagerTab(props.serverId)"
            class="px-3 py-1.5 text-xs font-medium rounded-[var(--radius-sm)]
              bg-[var(--color-bg-secondary)] text-[var(--color-text-primary)] border border-[var(--color-border)]
              hover:bg-[var(--color-bg-tertiary)] transition-colors"
          >{{ t('overview.openFileManager') }}</button>
        </div>
      </div>

      <!-- Metric Cards -->
      <div class="grid grid-cols-2 lg:grid-cols-3 gap-3">
        <!-- CPU -->
        <div class="rounded-lg border border-[var(--color-border)] p-4 bg-[var(--color-bg-secondary)]">
          <p class="text-[11px] uppercase tracking-wider text-[var(--color-text-tertiary)]">{{ t('overview.cpu') }}</p>
          <p class="text-2xl font-semibold text-[var(--color-text-primary)] mt-1">{{ cpuPercent !== null ? cpuPercent + '%' : '--' }}</p>
          <div class="mt-2 h-1.5 rounded-full bg-[var(--color-bg-tertiary)] overflow-hidden">
            <div class="h-full rounded-full transition-all duration-500" :style="barStyle(cpuPercent)" />
          </div>
        </div>

        <!-- Memory -->
        <div class="rounded-lg border border-[var(--color-border)] p-4 bg-[var(--color-bg-secondary)]">
          <p class="text-[11px] uppercase tracking-wider text-[var(--color-text-tertiary)]">{{ t('overview.memory') }}</p>
          <p class="text-2xl font-semibold text-[var(--color-text-primary)] mt-1">{{ memoryPercent !== null ? memoryPercent + '%' : '--' }}</p>
          <p v-if="memoryUsedMib !== null" class="text-[11px] text-[var(--color-text-tertiary)] mt-0.5">
            {{ formatMemory(memoryUsedMib) }} / {{ formatMemory(memoryTotalMib) }}
          </p>
          <div class="mt-2 h-1.5 rounded-full bg-[var(--color-bg-tertiary)] overflow-hidden">
            <div class="h-full rounded-full transition-all duration-500" :style="barStyle(memoryPercent)" />
          </div>
        </div>

        <!-- Disk -->
        <div class="rounded-lg border border-[var(--color-border)] p-4 bg-[var(--color-bg-secondary)]">
          <p class="text-[11px] uppercase tracking-wider text-[var(--color-text-tertiary)]">{{ t('overview.storage') }}</p>
          <p class="text-2xl font-semibold text-[var(--color-text-primary)] mt-1">{{ diskUsage || '--' }}</p>
          <p class="text-[11px] text-[var(--color-text-tertiary)]">/ {{ diskTotal || '--' }}</p>
        </div>
      </div>

      <!-- System Info -->
      <div class="grid grid-cols-2 gap-3">
        <div class="rounded-lg border border-[var(--color-border)] p-4 bg-[var(--color-bg-secondary)]">
          <p class="text-[11px] uppercase tracking-wider text-[var(--color-text-tertiary)]">{{ t('overview.timezone') }}</p>
          <p class="text-sm text-[var(--color-text-primary)] mt-1">{{ timezone || '--' }}</p>
        </div>
        <div class="rounded-lg border border-[var(--color-border)] p-4 bg-[var(--color-bg-secondary)]">
          <p class="text-[11px] uppercase tracking-wider text-[var(--color-text-tertiary)]">{{ t('overview.uptime') }}</p>
          <p class="text-sm text-[var(--color-text-primary)] mt-1">{{ uptime || '--' }}</p>
        </div>
      </div>
    </div>
  </div>
</template>
