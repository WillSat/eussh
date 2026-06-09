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

const terminalCount = computed(() => {
  const s = serverStore.activeServer
  if (!s) return 0
  return s.tabs.filter(t => t.type === 'terminal' && t.status === 'connected').length
})

const latencyMs = computed(() => serverStore.activeServer?.latency ?? null)

let completeTimer = null

onMounted(async () => {
  clockTimer = setInterval(() => { now.value = new Date() }, 1000)
  const unlisten = await listen('sftp-progress', (e) => {
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
  onBeforeUnmount(() => unlisten?.())
})

onBeforeUnmount(() => {
  clearInterval(clockTimer)
  if (completeTimer) clearTimeout(completeTimer)
})

const isAccent = computed(() => settingsStore.statusbarStyle === 'accent')

// Status dot color: on accent bg we use a light dot
const dotClass = computed(() => {
  const base = 'w-2 h-2 rounded-full shrink-0'
  if (isAccent.value) {
    return `${base} bg-white/70`
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
    <!-- Progress bar (above status bar) -->
    <div v-if="progress" :class="['absolute bottom-7 left-0 right-0 h-1', getProgressBar()]">
      <div
        :class="[
          'h-full',
          progress.determinate ? getProgressFill() : `${getProgressFill()} animate-pulse`,
        ]"
        :style="{ width: progress.determinate ? progress.percentage + '%' : '60%' }"
      />
    </div>

    <div class="flex items-center gap-2 min-w-0">
      <span :class="dotClass" />
      <!-- Progress label replaces status text during transfer -->
      <span v-if="progress" :class="['truncate', isAccent ? 'text-white' : 'text-[var(--color-accent)]']">{{ progress.label }}</span>
      <span v-else class="truncate">{{ statusText }}</span>
      <span v-if="!progress && terminalCount > 0" :class="['shrink-0', getDimText()]">
        {{ t('status.terminals', { count: terminalCount }) }}
      </span>
      <span v-if="!progress && latencyMs !== null" :class="['shrink-0', getDimText()]">
        {{ t('status.latency', { ms: latencyMs }) }}
      </span>
    </div>
    <div class="flex items-center min-w-[100px] justify-end gap-2">
      <span v-if="completedMessage" :class="['text-xs', isAccent ? 'text-white' : 'text-[var(--color-accent)]']">{{ completedMessage }}</span>
      <span v-else-if="!progress" :class="['text-xs tabular-nums', getDimText()]">
        {{ formattedTime }}
      </span>
    </div>
  </div>
</template>
