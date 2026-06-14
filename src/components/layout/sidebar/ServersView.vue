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
defineProps({ width: Number })

const showDlg = ref(false)
const editing = ref(null)
const showDelete = ref(false)
function newC() { editing.value = null; showDelete.value = false; showDlg.value = true }
function editC(c, e) { e.stopPropagation(); editing.value = { ...c }; showDelete.value = true; showDlg.value = true }
async function delC(id) { try { await connStore.deleteProfile(id); showDlg.value = false } catch (e) { showError(String(e)) } }
function onSaved(p) { connStore.saveProfile(p); showDlg.value = false }

function rowClasses(c) {
  const s = serverStore.servers.find(s => s.id === c.id)
  const isActive = serverStore.activeServerId === c.id
  if (!s) return { bg: '', status: '', active: isActive ? 'border-l-[3px] border-l-[var(--color-accent)]' : '' }
  if (s.tabs.some(t => t.status === 'connected'))
    return { bg: 'bg-[color-mix(in_srgb,#34C759_12%,transparent)]', status: 'connected', active: isActive ? 'border-l-[3px] border-l-[var(--color-accent)]' : '' }
  if (s.tabs.some(t => t.status === 'connecting' || t.status === 'reconnecting'))
    return { bg: 'bg-[color-mix(in_srgb,#FF9500_12%,transparent)]', status: 'connecting', active: isActive ? 'border-l-[3px] border-l-[var(--color-accent)]' : '' }
  if (s.tabs.some(t => t.status === 'error'))
    return { bg: 'bg-[color-mix(in_srgb,#FF3B30_8%,transparent)]', status: 'error', active: isActive ? 'border-l-[3px] border-l-[var(--color-accent)]' : '' }
  return { bg: '', status: '', active: isActive ? 'border-l-[3px] border-l-[var(--color-accent)]' : '' }
}
function statusLabel(c) {
  const s = serverStore.servers.find(s => s.id === c.id)
  if (!s) return ''
  if (s.tabs.some(t => t.status === 'connected')) return ''
  if (s.tabs.some(t => t.status === 'connecting')) return t('status.connecting')
  if (s.tabs.some(t => t.status === 'reconnecting')) return t('reconnect.trying').replace('{s}', '...')
  if (s.tabs.some(t => t.status === 'error')) return t('status.disconnected')
  return ''
}
function isConnected(c) {
  return serverStore.servers.some(s => s.id === c.id && s.tabs.some(t => t.status === 'connected'))
}
function statusDot(status) {
  if (status === 'connected') return 'bg-[#34C759] shadow-[0_0_6px_rgba(52,199,89,0.4)]'
  if (status === 'connecting') return 'bg-[#FF9500] animate-pulse shadow-[0_0_6px_rgba(255,149,0,0.4)]'
  if (status === 'error') return 'bg-[#FF3B30] shadow-[0_0_4px_rgba(255,59,48,0.3)]'
  return 'bg-[var(--color-text-tertiary)]/30'
}
</script>
<template>
  <div class="flex-1 flex flex-col min-h-0">
    <!-- Header -->
    <div class="shrink-0 flex items-center justify-between px-3 pt-3 pb-2 bg-[var(--color-bg-secondary)] z-10 border-b border-[var(--color-border)]">
      <span class="text-[11px] font-bold uppercase tracking-widest text-[var(--color-accent)]">{{ t('sidebar.connections') }}</span>
      <div class="flex items-center gap-1.5">
        <span v-if="connStore.profiles.length" class="text-[10px] tabular-nums font-medium px-1.5 py-0.5 rounded-md bg-[var(--color-accent)]/10 text-[var(--color-accent)]">{{ connStore.profiles.length }}</span>
        <button @click="newC"
          class="w-6 h-6 flex items-center justify-center rounded-lg text-[var(--color-accent)] hover:bg-[var(--color-accent)]/10 active:scale-95 transition-all"
          :title="t('sidebar.addServer')">
          <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
        </button>
      </div>
    </div>

    <!-- Server list -->
    <div class="flex-1 overflow-y-auto px-2 py-2 space-y-1.5">
      <!-- Empty state -->
      <div v-if="connStore.profiles.length === 0" class="flex flex-col items-center justify-center py-12 px-4">
        <div class="w-12 h-12 mb-3 rounded-2xl bg-[var(--color-accent)]/10 flex items-center justify-center">
          <svg class="w-6 h-6 text-[var(--color-accent)]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><rect x="2" y="2" width="20" height="8" rx="3"/><rect x="2" y="14" width="20" height="8" rx="3"/><circle cx="6" cy="6" r="1" fill="currentColor"/><circle cx="6" cy="18" r="1" fill="currentColor"/></svg>
        </div>
        <p class="text-[12px] font-medium text-[var(--color-text-secondary)]">{{ t('sidebar.noConnections') }}</p>
        <p class="text-[10px] text-[var(--color-text-secondary)] mt-1 leading-relaxed text-center max-w-[180px]">{{ t('sidebar.addHint') }}</p>
        <button @click="newC" class="mt-4 px-4 py-1.5 text-[11px] font-semibold rounded-lg text-white bg-[var(--color-accent)] hover:brightness-110 active:scale-95 transition-all">
          {{ t('sidebar.addServer') }}
        </button>
      </div>

      <!-- Server rows -->
      <div v-for="c in connStore.profiles" :key="c.id">
        <button @click="serverStore.openServer(c)"
          :class="[
            'group w-full text-left px-2 py-1.5 rounded-lg transition-all duration-150 relative border-l-[3px] border-l-transparent',
            rowClasses(c).bg,
            rowClasses(c).active,
            !rowClasses(c).active && 'hover:bg-[var(--color-bg-tertiary)]/60'
          ]">
          <div class="flex items-center gap-2">
            <!-- Status dot -->
            <span :class="['w-1.5 h-1.5 rounded-full shrink-0 transition-colors duration-300', statusDot(rowClasses(c).status)]" />
            <!-- Name + status label -->
            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-1.5">
                <span class="text-[13px] leading-tight font-medium truncate text-[var(--color-text-primary)]">{{ serverStore.displayName(c) }}</span>
                <span v-if="statusLabel(c)" class="text-[9px] text-[var(--color-text-secondary)] shrink-0">{{ statusLabel(c) }}</span>
              </div>
              <div class="text-[10px] mt-0.5 truncate text-[var(--color-text-secondary)]">{{ c.host }}</div>
            </div>
            <!-- Action buttons (show on hover) -->
            <div class="flex items-center gap-0.5 shrink-0 opacity-0 group-hover:opacity-100 transition-opacity duration-150">
              <span v-if="isConnected(c)" @click.stop="serverStore.closeServer(c.id)"
                class="w-5 h-5 flex items-center justify-center rounded text-[var(--color-text-tertiary)]/70 hover:text-[#FF3B30] hover:bg-[#FF3B30]/10 transition-colors"
                :title="t('sidebar.disconnect')">
                <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M18.36 6.64a9 9 0 11-12.73 0"/><line x1="12" y1="2" x2="12" y2="12"/></svg>
              </span>
              <span @click.stop="editC(c, $event)"
                class="w-5 h-5 flex items-center justify-center rounded text-[var(--color-text-tertiary)]/70 hover:text-[var(--color-accent)] hover:bg-[var(--color-accent)]/10 transition-colors"
                :title="t('sidebar.edit')">
                <svg class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M11 4H4a2 2 0 00-2 2v14a2 2 0 002 2h14a2 2 0 002-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 013 3L12 15l-4 1 1-4 9.5-9.5z"/></svg>
              </span>
            </div>
          </div>
        </button>
      </div>
    </div>

    <ConnectionDialog :visible="showDlg" :connection="editing" :show-delete="showDelete" @close="showDlg = false" @saved="onSaved" @delete="delC" />
  </div>
</template>
