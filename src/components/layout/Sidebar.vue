<script setup>
import { ref } from 'vue'
import { useConnectionStore } from '@/stores/useConnectionStore'
import { useServerStore } from '@/stores/useServerStore'
import { useToast } from '@/composables/useToast'
import { useLogger } from '@/composables/useLogger'
import { useI18n } from '@/composables/useI18n'
import ConnectionDialog from '../connection/ConnectionDialog.vue'

const { t } = useI18n()
const log = useLogger('Sidebar')
const connectionStore = useConnectionStore()
const serverStore = useServerStore()
const { error: showError } = useToast()

const showDialog = ref(false)
const editingConnection = ref(null)

function newConnection() {
  log.info('newConnection dialog')
  editingConnection.value = null
  showDialog.value = true
}
function editConnection(conn, e) {
  e.stopPropagation()
  log.info('editConnection', conn.name)
  editingConnection.value = { ...conn }
  showDialog.value = true
}
async function deleteConnection(conn, e) {
  e.stopPropagation()
  try {
    await connectionStore.deleteProfile(conn.id)
  } catch (err) {
    showError(`Failed to delete: ${err}`)
  }
}
function onSaved(profile) {
  log.info('connection saved', profile.name)
  connectionStore.saveProfile(profile)
  showDialog.value = false
}

function openOrSwitch(profile) {
  log.info('openOrSwitch', { id: profile.id, name: profile.name, host: profile.host })
  serverStore.openServer(profile)
}

defineExpose({ newConnection })
</script>

<template>
  <div class="w-44 shrink-0 flex flex-col h-full select-none
    bg-[var(--color-bg-secondary)] border-r border-[var(--color-border)]">

    <!-- TOP: Open Servers -->
    <div class="flex-1 flex flex-col min-h-0">
      <div class="flex items-center px-3 py-2">
        <span class="text-[10px] font-semibold tracking-widest text-[var(--color-text-tertiary)] uppercase">
          {{ t('sidebar.openServers') }}
        </span>
      </div>
      <div class="flex-1 overflow-y-auto">
        <div v-if="serverStore.servers.length === 0" class="px-3 py-6 text-center">
          <p class="text-[10px] text-[var(--color-text-tertiary)]">{{ t('sidebar.noServers') }}</p>
        </div>
        <button
          v-for="srv in serverStore.servers"
          :key="srv.id"
          @click="serverStore.activeServerId = srv.id"
          :class="[
            'group w-full flex items-center gap-2 px-3 py-2 text-[13px] text-left transition-colors',
            srv.id === serverStore.activeServerId
              ? 'bg-[var(--color-bg-tertiary)] text-[var(--color-text-primary)]'
              : 'text-[var(--color-text-secondary)] hover:bg-[var(--color-bg-tertiary)]',
          ]"
        >
          <span class="w-2 h-2 rounded-full shrink-0"
            :class="srv.tabs.some(t => t.status === 'connected') ? 'bg-[var(--color-success)]' :
              srv.tabs.some(t => t.status === 'connecting') ? 'bg-[var(--color-warning)] animate-pulse' :
              'bg-[var(--color-text-tertiary)]'"
          />
          <span class="truncate flex-1">{{ srv.nickname }}</span>
          <span
            @click.stop="serverStore.closeServer(srv.id)"
            class="w-4 h-4 flex items-center justify-center rounded
              text-[var(--color-text-tertiary)] hover:text-[var(--color-danger)]
              hover:bg-[var(--color-bg-tertiary)] opacity-0 group-hover:opacity-100
              transition-opacity text-[10px] shrink-0"
          >&#x2715;</span>
        </button>
      </div>
    </div>

    <!-- Divider -->
    <div class="border-t-2 border-[var(--color-border)]" />

    <!-- BOTTOM: Saved Servers -->
    <div class="flex flex-col" style="flex: 0 1 50%; min-height: 0;">
      <div class="flex items-center justify-between px-3 py-2">
        <span class="text-[10px] font-semibold tracking-widest text-[var(--color-text-tertiary)] uppercase">
          {{ t('sidebar.saved') }}
        </span>
        <button @click="newConnection"
          class="w-6 h-6 flex items-center justify-center rounded text-[var(--color-text-secondary)]
            hover:text-[var(--color-accent)] hover:bg-[var(--color-bg-tertiary)] transition-colors text-base"
          :title="t('sidebar.addServer')">+</button>
      </div>
      <div class="flex-1 overflow-y-auto">
        <div v-if="connectionStore.profiles.length === 0" class="px-3 py-6 text-center">
          <p class="text-[10px] text-[var(--color-text-tertiary)]">{{ t('sidebar.noSaved') }}</p>
        </div>
        <button
          v-for="conn in connectionStore.profiles"
          :key="conn.id"
          @click="openOrSwitch(conn)"
          class="group w-full flex items-center gap-2 px-3 py-2 text-[13px] text-left
            text-[var(--color-text-secondary)] hover:bg-[var(--color-bg-tertiary)]
            transition-colors"
        >
          <span class="w-2 h-2 rounded-full shrink-0"
            :class="serverStore.servers.some(s => s.id === conn.id && s.tabs.some(t => t.status === 'connected'))
              ? 'bg-[var(--color-success)]' : 'bg-[var(--color-text-tertiary)]'"
          />
          <span class="truncate flex-1">{{ serverStore.displayName(conn) }}</span>
          <span
            @click.stop="deleteConnection(conn, $event)"
            class="w-4 h-4 flex items-center justify-center rounded
              text-[var(--color-text-tertiary)] hover:text-[var(--color-danger)]
              hover:bg-[var(--color-bg-tertiary)] opacity-0 group-hover:opacity-100
              transition-opacity text-[10px] shrink-0"
            :title="t('sidebar.delete')">&#x2715;</span>
          <span
            @click.stop="editConnection(conn, $event)"
            class="w-4 h-4 flex items-center justify-center rounded
              text-[var(--color-text-tertiary)] hover:text-[var(--color-text-primary)]
              hover:bg-[var(--color-bg-tertiary)] opacity-0 group-hover:opacity-100
              transition-opacity text-[10px] shrink-0"
          >&#x270E;</span>
        </button>
      </div>
    </div>

    <ConnectionDialog
      :visible="showDialog"
      :connection="editingConnection"
      @close="showDialog = false"
      @saved="onSaved"
    />
  </div>
</template>
