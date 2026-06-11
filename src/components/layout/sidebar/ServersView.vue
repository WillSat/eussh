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

function rowBg(c) {
  const s = serverStore.servers.find(s => s.id === c.id)
  if (!s) return ''
  if (s.tabs.some(t => t.status === 'connected')) return 'bg-[#34C759]/15'
  if (s.tabs.some(t => t.status === 'connecting' || t.status === 'reconnecting')) return 'bg-[#FF9500]/15'
  if (s.tabs.some(t => t.status === 'error')) return 'bg-[#FF3B30]/12'
  return ''
}
function statusLabel(c) {
  const s = serverStore.servers.find(s => s.id === c.id)
  if (!s) return ''
  if (s.tabs.some(t => t.status === 'connected')) return ''
  if (s.tabs.some(t => t.status === 'connecting')) return t('status.connecting')
  if (s.tabs.some(t => t.status === 'reconnecting')) return t('reconnect.trying').replace('{s}','...')
  if (s.tabs.some(t => t.status === 'error')) return t('status.disconnected')
  return ''
}
function isConnected(c) {
  return serverStore.servers.some(s => s.id === c.id && s.tabs.some(t => t.status === 'connected'))
}
</script>
<template>
  <div class="flex-1 flex flex-col min-h-0">
    <!-- Sticky header with background to prevent overlap -->
    <div class="shrink-0 flex items-center justify-between px-3 pt-2.5 pb-1.5 bg-[var(--color-bg-secondary)] z-10 border-b border-[var(--color-border)]">
      <span class="text-[11px] font-bold uppercase tracking-widest text-[var(--color-text-tertiary)]">{{ t('sidebar.connections') }}</span>
      <div class="flex items-center gap-1">
        <span v-if="connStore.profiles.length" class="text-[10px] tabular-nums text-[var(--color-text-tertiary)]">{{ connStore.profiles.length }}</span>
        <button @click="newC" class="w-5 h-5 flex items-center justify-center rounded text-[var(--color-text-secondary)] hover:text-[var(--color-accent)] hover:bg-[var(--color-bg-tertiary)] transition-colors" :title="t('sidebar.addServer')">+</button>
      </div>
    </div>
    <div class="flex-1 overflow-y-auto px-2.5 pb-2 pt-1.5">
      <div v-if="connStore.profiles.length===0" class="py-10 text-center">
        <svg class="w-10 h-10 mx-auto mb-2 text-[var(--color-text-tertiary)]/25" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><rect x="2" y="2" width="20" height="8" rx="2"/><rect x="2" y="14" width="20" height="8" rx="2"/></svg>
        <p class="text-[12px] text-[var(--color-text-tertiary)]">{{ t('sidebar.noConnections') }}</p>
        <p class="text-[10px] text-[var(--color-text-tertiary)]/40 mt-1">{{ t('sidebar.addHint') }}</p>
        <button @click="newC" class="mt-3 px-3 py-1.5 text-[11px] font-bold rounded-lg text-white bg-[var(--color-accent)] hover:brightness-110 transition-all">{{ t('sidebar.addServer') }}</button>
      </div>
      <div v-for="c in connStore.profiles" :key="c.id" class="mb-1">
        <button @click="serverStore.openServer(c)"
          :class="['group w-full text-left px-2 py-1.5 rounded-lg transition-all relative', rowBg(c),
            serverStore.activeServerId===c.id ? 'ring-1 ring-[var(--color-accent)]/30' : 'hover:brightness-95']">
          <div class="flex items-center gap-2">
            <span class="text-[14px] leading-tight truncate flex-1 font-medium text-[var(--color-text-primary)]">{{ serverStore.displayName(c) }}</span>
            <span class="text-[10px] shrink-0 ml-1 text-[var(--color-text-tertiary)]/50">{{ statusLabel(c) }}</span>
            <span v-if="isConnected(c)" @click.stop="serverStore.closeServer(c.id)" class="hidden group-hover:flex w-4 h-4 items-center justify-center rounded text-[var(--color-text-tertiary)] hover:text-[var(--color-warning)] shrink-0" :title="t('sidebar.disconnect')">
              <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M18.36 6.64a9 9 0 11-12.73 0"/><line x1="12" y1="2" x2="12" y2="12"/></svg>
            </span>
            <span @click.stop="editC(c,$event)" class="hidden group-hover:flex w-4 h-4 items-center justify-center rounded text-[var(--color-text-tertiary)] hover:text-[var(--color-text-primary)] shrink-0 text-[11px]" :title="t('sidebar.edit')">&#x270E;</span>
          </div>
          <div class="text-[11px] mt-0.5 truncate text-[var(--color-text-secondary)]">{{ c.host }}</div>
        </button>
      </div>
    </div>
    <ConnectionDialog :visible="showDlg" :connection="editing" :show-delete="showDelete" @close="showDlg=false" @saved="onSaved" @delete="delC" />
  </div>
</template>
