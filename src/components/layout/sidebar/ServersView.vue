<script setup>
import { ref } from 'vue'
import { useConnectionStore } from '@/stores/useConnectionStore'
import { useServerStore } from '@/stores/useServerStore'
import { useToast } from '@/composables/useToast'
import { useI18n } from '@/composables/useI18n'
import ConnectionDialog from '../../connection/ConnectionDialog.vue'

const { t } = useI18n()
const connStore = useConnectionStore()
const serverStore = useServerStore()
const { error: showError } = useToast()

const showDlg = ref(false)
const editing = ref(null)
const showDelete = ref(false)

function newC() { editing.value = null; showDelete.value = false; showDlg.value = true }
function editC(c, e) { e.stopPropagation(); editing.value = { ...c }; showDelete.value = true; showDlg.value = true }
async function delC(id) { try { await connStore.deleteProfile(id); showDlg.value = false } catch (e) { showError(String(e)) } }
function onSaved(p) { connStore.saveProfile(p); showDlg.value = false }

function getServer(c) {
  return serverStore.servers.find(s => s.id === c.id)
}

function statusKey(c) {
  const s = getServer(c)
  if (!s) return 'none'
  const tabs = s.tabs
  if (tabs.some(t => t.status === 'connected')) return 'connected'
  if (tabs.some(t => t.status === 'reconnecting')) return 'reconnecting'
  if (tabs.some(t => t.status === 'reconnect_failed')) return 'reconnect_failed'
  if (tabs.some(t => t.status === 'connecting')) return 'connecting'
  if (tabs.some(t => t.status === 'error')) return 'error'
  return 'none'
}

function statusDot(key) {
  const map = {
    connected: 'bg-[#34C759] shadow-[0_0_8px_rgba(52,199,89,0.5)]',
    connecting: 'bg-[#FF9500] animate-pulse shadow-[0_0_6px_rgba(255,149,0,0.4)]',
    reconnecting: 'bg-[#FF9500] animate-pulse-fast shadow-[0_0_6px_rgba(255,149,0,0.4)]',
    reconnect_failed: 'bg-[#FF3B30] shadow-[0_0_6px_rgba(255,59,48,0.4)]',
    error: 'bg-[#FF3B30] shadow-[0_0_5px_rgba(255,59,48,0.3)]',
    none: 'bg-[var(--color-text-tertiary)]/25',
  }
  return map[key] || map.none
}

function statusText(c) {
  const s = getServer(c)
  if (!s) return ''
  const key = statusKey(c)
  if (key === 'connected') {
    if (s.latency != null) return `${s.latency}ms`
    return ''
  }
  if (key === 'reconnecting') {
    const tab = s.tabs.find(t => t.status === 'reconnecting')
    if (tab?._reconnectInfo) {
      const info = tab._reconnectInfo
      const sec = Math.ceil(Math.min(30000, Math.pow(2, (info.n || 1) - 1) * 1000) / 1000)
      return t('reconnect.trying', { n: info.n, max: info.max, s: sec })
    }
    return ''
  }
  if (key === 'connecting') return t('status.connecting')
  if (key === 'reconnect_failed') return t('reconnect.failed')
  if (key === 'error') return t('status.error')
  return ''
}

function statusTextColor(key) {
  if (key === 'connected') return 'text-[var(--color-text-tertiary)]'
  if (key === 'reconnecting' || key === 'connecting') return 'text-[#FF9500]'
  if (key === 'reconnect_failed' || key === 'error') return 'text-[#FF3B30]'
  return 'text-[var(--color-text-tertiary)]'
}

function isActive(c) {
  return serverStore.activeServerId === c.id
}

function toggleConnect(c) {
  const s = getServer(c)
  if (s && statusKey(c) === 'connected') {
    serverStore.closeServer(c.id)
  } else {
    serverStore.openServer(c)
  }
}
</script>

<template>
  <div class="flex-1 flex flex-col min-h-0 bg-[var(--color-bg-primary)]">
    <!-- Header -->
    <div class="shrink-0 flex items-center justify-between px-3 py-2.5">
      <span class="text-[10px] font-bold uppercase tracking-[0.15em] text-[var(--color-text-tertiary)]">{{ t('sidebar.connections') }}</span>
      <div class="flex items-center gap-2">
        <span v-if="connStore.profiles.length" class="text-[10px] tabular-nums font-medium px-1.5 py-px rounded bg-[var(--color-accent)]/10 text-[var(--color-accent)]">{{ connStore.profiles.length }}</span>
      </div>
    </div>

    <!-- Server list -->
    <div class="flex-1 overflow-y-auto py-1">
      <!-- Empty state -->
      <div v-if="connStore.profiles.length === 0" class="flex flex-col items-center justify-center py-16 px-4">
        <div class="w-10 h-10 mb-3 rounded-xl bg-[var(--color-accent)]/10 flex items-center justify-center">
          <svg class="w-5 h-5 text-[var(--color-accent)]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><rect x="2" y="2" width="20" height="8" rx="3"/><rect x="2" y="14" width="20" height="8" rx="3"/><circle cx="6" cy="6" r="1" fill="currentColor"/><circle cx="6" cy="18" r="1" fill="currentColor"/></svg>
        </div>
        <p class="text-[12px] font-medium text-[var(--color-text-secondary)]">{{ t('sidebar.noConnections') }}</p>
        <p class="text-[10px] text-[var(--color-text-secondary)] mt-1 leading-relaxed text-center max-w-[180px]">{{ t('sidebar.addHint') }}</p>
        <button @click="newC" class="mt-3 px-4 py-1.5 text-[11px] font-semibold rounded-lg text-white bg-[var(--color-accent)] hover:brightness-110 active:scale-95 transition-all">
          {{ t('sidebar.addServer') }}
        </button>
      </div>

      <!-- Server items -->
      <div v-for="c in connStore.profiles" :key="c.id"
        @click="serverStore.openServer(c)"
        :class="['group relative mx-2 my-0.5 rounded-lg cursor-pointer transition-all duration-200',
          isActive(c) ? 'bg-accent-dim' : 'hover:bg-[var(--color-bg-secondary)]/50']">
        <div class="px-3 py-2.5 flex items-center gap-2.5">
          <!-- Status dot -->
          <span :class="['w-2 h-2 rounded-full shrink-0 transition-all duration-300', statusDot(statusKey(c))]" />

          <!-- Info -->
          <div class="flex-1 min-w-0">
            <div class="text-[12px] font-medium text-[var(--color-text-primary)] truncate leading-tight">{{ serverStore.displayName(c) }}</div>
            <div class="flex items-center gap-1.5 mt-0.5">
              <span class="text-[10px] font-mono text-[var(--color-text-tertiary)]/70 truncate">{{ c.host }}<span v-if="c.port !== 22" class="text-[var(--color-text-tertiary)]/40">:{{ c.port }}</span></span>
              <span v-if="statusText(c)" :class="['text-[10px] font-medium shrink-0', statusTextColor(statusKey(c))]">{{ statusText(c) }}</span>
            </div>
          </div>

          <!-- Actions -->
          <div class="flex items-center gap-0.5 shrink-0 opacity-0 group-hover:opacity-100 transition-opacity duration-150">
            <!-- Cancel reconnect -->
            <button v-if="statusKey(c) === 'reconnecting'" @click.stop="serverStore.cancelReconnect(c.id)"
              class="w-6 h-6 flex items-center justify-center rounded-md text-[var(--color-text-tertiary)] hover:text-[var(--color-danger)] hover:bg-[var(--color-danger)]/10 transition-colors"
              :title="t('reconnect.cancel')">
              <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
            </button>
            <!-- Retry -->
            <button v-if="statusKey(c) === 'reconnect_failed'" @click.stop="serverStore.retryReconnect(c.id)"
              class="w-6 h-6 flex items-center justify-center rounded-md text-[var(--color-accent)] hover:bg-[var(--color-accent)]/10 transition-colors"
              :title="t('reconnect.retry')">
              <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="1 4 1 10 7 10"/><path d="M3.51 15a9 9 0 102.13-9.36L1 10"/></svg>
            </button>
            <!-- Connect / Disconnect -->
            <button @click.stop="toggleConnect(c)"
              :class="['w-6 h-6 flex items-center justify-center rounded-md transition-colors',
                statusKey(c) === 'connected' ? 'text-[var(--color-text-tertiary)] hover:text-[var(--color-danger)] hover:bg-[var(--color-danger)]/10' : 'text-[var(--color-text-tertiary)] hover:text-[var(--color-accent)] hover:bg-[var(--color-accent)]/10']"
              :title="statusKey(c) === 'connected' ? t('sidebar.disconnect') : ''">
              <svg v-if="statusKey(c) === 'connected'" class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M18.36 6.64a9 9 0 11-12.73 0"/><line x1="12" y1="2" x2="12" y2="12"/></svg>
              <svg v-else class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><polygon points="5 3 19 12 5 21 5 3"/></svg>
            </button>
            <!-- Edit -->
            <button @click.stop="editC(c, $event)"
              class="w-6 h-6 flex items-center justify-center rounded-md text-[var(--color-text-tertiary)] hover:text-[var(--color-accent)] hover:bg-[var(--color-accent)]/10 transition-colors"
              :title="t('sidebar.edit')">
              <svg class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M11 4H4a2 2 0 00-2 2v14a2 2 0 002 2h14a2 2 0 002-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 013 3L12 15l-4 1 1-4 9.5-9.5z"/></svg>
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Footer -->
    <div class="shrink-0 px-3 py-2">
      <button @click="newC"
        class="w-full py-1.5 text-[11px] font-medium rounded-lg text-[var(--color-accent)] hover:bg-[var(--color-accent)]/10 active:scale-[0.98] transition-all">
        + {{ t('sidebar.addServer') }}
      </button>
    </div>

    <ConnectionDialog :visible="showDlg" :connection="editing" :show-delete="showDelete" @close="showDlg = false" @saved="onSaved" @delete="delC" />
  </div>
</template>
