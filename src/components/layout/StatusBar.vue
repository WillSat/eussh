<script setup>
import { ref, computed, onMounted, onBeforeUnmount } from 'vue'
import { useServerStore } from '@/stores/useServerStore'
import { useSettingsStore } from '@/stores/useSettingsStore'
import { useI18n } from '@/composables/useI18n'
import { listen } from '@tauri-apps/api/event'

const { t } = useI18n()
const serverStore = useServerStore()
const settingsStore = useSettingsStore()

const now = ref(new Date())
const progress = ref(null)
const completedMessage = ref(null)
let clockTimer = null

function fmtBytes(b) {
  if (b < 1024) return b + ' B'
  if (b < 1048576) return (b / 1024).toFixed(1) + ' KB'
  if (b < 1073741824) return (b / 1048576).toFixed(1) + ' MB'
  return (b / 1073741824).toFixed(2) + ' GB'
}

function fmtSpeed(bytesPerSec) {
  if (bytesPerSec < 1024) return bytesPerSec + ' B/s'
  if (bytesPerSec < 1048576) return (bytesPerSec / 1024).toFixed(1) + ' KB/s'
  if (bytesPerSec < 1073741824) return (bytesPerSec / 1048576).toFixed(1) + ' MB/s'
  return (bytesPerSec / 1073741824).toFixed(2) + ' GB/s'
}

const formattedTime = computed(() => {
  return now.value.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit', hour12: false })
})

const statusText = computed(() => {
  const s = serverStore.activeServer
  if (!s) return t('status.noServer')
  const activeTab = s.tabs.find(t => t.id === s.activeTabId)
  if (activeTab?.status === 'connecting') return t('status.connecting')
  if (activeTab?.status === 'connected') return s.nickname
  if (activeTab?.status === 'error') return t('status.error')
  const anyConnected = s.tabs.some(t => t.status === 'connected')
  if (anyConnected) return `${s.nickname} (${t('status.connected')})`
  return t('status.disconnected')
})

const trafficText = computed(() => {
  if (!settingsStore.showTraffic) return ''
  const srv = serverStore.activeServer
  if (!srv || (!srv.trafficUp && !srv.trafficDown)) return ''
  const parts = []
  if (srv.trafficDown > 0) parts.push('↓ ' + fmtSpeed(srv.trafficDown))
  if (srv.trafficUp > 0) parts.push('↑ ' + fmtSpeed(srv.trafficUp))
  return parts.join('  ')
})

const latencyMs = computed(() => serverStore.activeServer?.latency ?? null)

// Unified notification slot on the right side.
// Priority: completed message > progress label.
// Add future notification sources here.
const notification = computed(() => {
  if (completedMessage.value) return completedMessage.value
  if (progress.value) return progress.value.label
  return null
})

let completeTimer = null

onMounted(async () => {
  clockTimer = setInterval(() => { now.value = new Date() }, 1000)

  const unlistenSftp = await listen('sftp-progress', (e) => {
    const { operation, path, bytes_transferred, total_bytes } = e.payload
    const name = (path || '').split('/').pop() || path || '?'
    const arrow = operation === 'upload' ? '↑' : '↓'

    if (total_bytes > 0) {
      const pct = Math.min(Math.round((bytes_transferred / total_bytes) * 100), 100)
      progress.value = { arrow, name, pct, label: `${arrow} ${name} ${pct}%`, percentage: pct, determinate: true }
      if (bytes_transferred >= total_bytes) {
        const key = operation === 'upload' ? 'status.uploaded' : 'status.downloaded'
        completedMessage.value = t(key, { name })
        if (completeTimer) clearTimeout(completeTimer)
        completeTimer = setTimeout(() => {
          progress.value = null
          completeTimer = setTimeout(() => { completedMessage.value = null }, 5000)
        }, 1000)
      }
    } else {
      // Indeterminate progress (e.g. directory archive download)
      progress.value = { arrow, name, pct: 0, label: `${arrow} ${name} ${fmtBytes(bytes_transferred)}`, percentage: 0, determinate: false }
    }
  })

  onBeforeUnmount(() => {
    unlistenSftp?.()
  })
})

onBeforeUnmount(() => {
  clearInterval(clockTimer)
  if (completeTimer) clearTimeout(completeTimer)
})

const isAccent = computed(() => settingsStore.statusbarStyle === 'accent')

// Status dot color: on accent bg we use white variants to stay visible
const dotClass = computed(() => {
  const base = 'w-2 h-2 rounded-full shrink-0'
  if (isAccent.value) {
    if (serverStore.activeServer?.tabs.some(t => t.id === serverStore.activeServer?.activeTabId && t.status === 'connected')) return `${base} bg-white`
    if (serverStore.activeServer?.tabs.some(t => t.id === serverStore.activeServer?.activeTabId && t.status === 'connecting')) return `${base} bg-white/70 animate-pulse`
    if (serverStore.activeServer?.tabs.some(t => t.status === 'connected')) return `${base} bg-white/80`
    return `${base} bg-white/40`
  }
  if (serverStore.activeServer?.tabs.some(t => t.id === serverStore.activeServer?.activeTabId && t.status === 'connected')) return `${base} bg-[var(--color-success)]`
  if (serverStore.activeServer?.tabs.some(t => t.id === serverStore.activeServer?.activeTabId && t.status === 'connecting')) return `${base} bg-[var(--color-warning)]`
  if (serverStore.activeServer?.tabs.some(t => t.status === 'connected')) return `${base} bg-[var(--color-success)]`
  return `${base} bg-[var(--color-text-tertiary)]`
})

function getContainerClass() {
  const base = 'flex items-center justify-between h-7 px-3 select-none border-t text-xs'
  if (isAccent.value) {
    return `${base} bg-[var(--color-accent)] text-white border-[var(--color-accent-hover)]`
  }
  return `${base} bg-[var(--color-bg-secondary)] border-[var(--color-border)] text-[var(--color-text-secondary)]`
}

function getDimText() {
  return isAccent.value ? 'text-white/60' : 'text-[var(--color-text-tertiary)]'
}

function getProgressBar() {
  return isAccent.value ? 'bg-white/30' : 'bg-[var(--color-bg-tertiary)]'
}

function getProgressFill() {
  return isAccent.value ? 'bg-white' : 'bg-[var(--color-accent)]'
}
</script>

<template>
  <div :class="getContainerClass()">
    <!-- Progress bar (thin line above status bar) -->
    <div v-if="progress" :class="['absolute bottom-7 left-0 right-0 h-0.5', getProgressBar()]">
      <div
        :class="[
          'h-full transition-[width] duration-300',
          progress.determinate ? getProgressFill() : `${getProgressFill()} animate-pulse`,
        ]"
        :style="{ width: progress.determinate ? progress.percentage + '%' : '60%' }"
      />
    </div>

    <div class="flex items-center gap-2 min-w-0">
      <span :class="dotClass" />
      <span class="truncate">{{ statusText }}</span>
      <span v-if="latencyMs !== null" :class="['shrink-0', getDimText()]">
        {{ t('status.latency', { ms: latencyMs }) }}
      </span>
      <!-- Network traffic (always visible, even during transfers) -->
      <span v-if="trafficText" :class="['shrink-0', getDimText()]">{{ trafficText }}</span>
    </div>
    <div class="flex items-center min-w-[100px] justify-end gap-2">
      <!-- Unified notification slot (progress, completion, future notices) -->
      <span v-if="notification" :class="['text-xs truncate max-w-[180px]', isAccent ? 'text-white' : 'text-[var(--color-accent)]']">{{ notification }}</span>
      <span :class="['text-xs tabular-nums shrink-0', getDimText()]">
        {{ formattedTime }}
      </span>
    </div>
  </div>
</template>
