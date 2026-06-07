<script setup>
import { ref, computed, onMounted, onBeforeUnmount } from 'vue'
import { useServerStore } from '@/stores/useServerStore'
import { useI18n } from '@/composables/useI18n'
import { listen } from '@tauri-apps/api/event'

const { t } = useI18n()
const serverStore = useServerStore()

const now = ref(new Date())
const uploadProgress = ref(null)
const completedMessage = ref(null)
let clockTimer = null

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

// Modifier key tracking
const pressedModifiers = ref(new Set())
const displayModifier = ref(null)

function onKeyDown(e) {
  if (['Shift', 'Control', 'Alt', 'Meta'].includes(e.key)) {
    const next = new Set(pressedModifiers.value)
    next.add(e.key)
    pressedModifiers.value = next
    displayModifier.value = [...next].join('+')
  }
}
function onKeyUp(e) {
  if (pressedModifiers.value.has(e.key)) {
    const next = new Set(pressedModifiers.value)
    next.delete(e.key)
    pressedModifiers.value = next
    displayModifier.value = next.size > 0 ? [...next].join('+') : null
  }
}

onMounted(async () => {
  clockTimer = setInterval(() => { now.value = new Date() }, 1000)
  window.addEventListener('keydown', onKeyDown)
  window.addEventListener('keyup', onKeyUp)
  const unlisten = await listen('sftp-progress', (e) => {
    const { operation, path, bytes_transferred, total_bytes } = e.payload
    if (total_bytes > 0) {
      const name = (path || '').split('/').pop()
      const pct = Math.round((bytes_transferred / total_bytes) * 100)
      uploadProgress.value = {
        label: `${operation === 'upload' ? '↑' : '↓'} ${name} ${pct}%`,
        percentage: pct,
      }
      if (bytes_transferred >= total_bytes) {
        const op = operation === 'upload' ? 'Uploaded' : 'Downloaded'
        completedMessage.value = `${op} ${name}`
        setTimeout(() => { uploadProgress.value = null }, 1000)
        setTimeout(() => { completedMessage.value = null }, 6000)
      }
    }
  })
  onBeforeUnmount(() => unlisten?.())
})

onBeforeUnmount(() => {
  clearInterval(clockTimer)
  window.removeEventListener('keydown', onKeyDown)
  window.removeEventListener('keyup', onKeyUp)
})
</script>

<template>
  <div class="flex items-center justify-between h-7 px-3 select-none
    bg-[var(--color-bg-secondary)] border-t border-[var(--color-border)]
    text-xs text-[var(--color-text-secondary)]">
    <!-- Upload progress bar -->
    <div v-if="uploadProgress" class="absolute bottom-7 left-0 right-0 h-1 bg-[var(--color-bg-tertiary)]">
      <div class="h-full bg-[var(--color-accent)] transition-all duration-300" :style="{ width: uploadProgress.percentage + '%' }" />
    </div>
    <div class="flex items-center gap-2 min-w-0">
      <span
        :class="[
          'w-2 h-2 rounded-full shrink-0',
          serverStore.activeServer?.tabs.some(t => t.id === serverStore.activeServer?.activeTabId && t.status === 'connected') ? 'bg-[var(--color-success)]' :
          serverStore.activeServer?.tabs.some(t => t.id === serverStore.activeServer?.activeTabId && t.status === 'connecting') ? 'bg-[var(--color-warning)]' :
          serverStore.activeServer?.tabs.some(t => t.status === 'connected') ? 'bg-[var(--color-success)]' :
          'bg-[var(--color-text-tertiary)]',
        ]"
      />
      <span class="truncate">{{ statusText }}</span>
      <span v-if="terminalCount > 0" class="text-[var(--color-text-tertiary)] shrink-0">
        {{ t('status.terminals', { count: terminalCount }) }}
      </span>
      <span v-if="latencyMs !== null" class="text-[var(--color-text-tertiary)] shrink-0">
        {{ t('status.latency', { ms: latencyMs }) }}
      </span>
    </div>
    <div class="flex items-center min-w-[100px] justify-end gap-2">
      <span v-if="completedMessage" class="text-[var(--color-accent)] text-xs">{{ completedMessage }}</span>
      <span v-if="displayModifier" class="text-[var(--color-accent)] text-xs font-medium">
        {{ displayModifier }}
      </span>
      <span v-else-if="!completedMessage" class="text-[var(--color-text-tertiary)] text-xs tabular-nums">
        {{ formattedTime }}
      </span>
    </div>
  </div>
</template>
