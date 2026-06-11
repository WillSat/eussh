<script setup>
import { ref, onMounted, onErrorCaptured, watch, reactive } from 'vue'
import TitleBar from './TitleBar.vue'
import ActivityBar from './ActivityBar.vue'
import Sidebar from './Sidebar.vue'
import MainTabBar from './MainTabBar.vue'
import StatusBar from './StatusBar.vue'
import WelcomeScreen from './WelcomeScreen.vue'
import ServerOverview from '../server/ServerOverview.vue'
import TerminalContainer from '../terminal/TerminalContainer.vue'
import FileManager from '../filemanager/FileManager.vue'
import Toast from '../common/Toast.vue'
import DebugPanel from './DebugPanel.vue'
import HostKeyDialog from '../connection/HostKeyDialog.vue'
import VersionCheck from './VersionCheck.vue'
import { useConnectionStore } from '@/stores/useConnectionStore'
import { useServerStore } from '@/stores/useServerStore'
import { useSettingsStore } from '@/stores/useSettingsStore'
import { useTheme } from '@/composables/useTheme'
import { useToast } from '@/composables/useToast'
import { useLogger } from '@/composables/useLogger'
import { useI18n } from '@/composables/useI18n'
import { listen } from '@tauri-apps/api/event'

const log = useLogger('AppShell')
const { init: initI18n, t } = useI18n()

const connectionStore = useConnectionStore()
const serverStore = useServerStore()
const settingsStore = useSettingsStore()
const { init: initTheme } = useTheme()
const { state: toast, close: closeToast, error: showError } = useToast()

const activeView = ref('servers')
const hostKeyDialog = ref(null)
const vueError = ref(null)

log.info('setup start')

function getOverviewSessionId(srv) {
  const overview = srv?.tabs?.find(t => t.type === 'overview')
  return overview?.sessionId || null
}

// --- Per-server tab slide animation state ---
// Track which server+tab is currently entering (for animation CSS class)
const enteringState = reactive({}) // { key: 'serverId::tabId' -> direction: 'right'|'left' }

watch(
  () => serverStore.servers.map(s => ({ id: s.id, tabId: s.activeTabId })),
  (newStates, oldStates) => {
    if (!oldStates) return
    for (const ns of newStates) {
      const os = oldStates.find(o => o.id === ns.id)
      if (!os || ns.tabId === os.tabId) continue
      // Tab changed within this server — compute animation direction
      const srv = serverStore.servers.find(s => s.id === ns.id)
      if (!srv) continue
      const oldIdx = srv.tabs.findIndex(t => t.id === os.tabId)
      const newIdx = srv.tabs.findIndex(t => t.id === ns.tabId)
      const dir = (oldIdx >= 0 && newIdx > oldIdx) ? 'right' : 'left'
      enteringState[ns.id + '::' + ns.tabId] = dir
    }
  },
  { deep: true },
)

function paneClasses(srv, tabId) {
  const isActive = tabId === srv.activeTabId
  if (!isActive) return 'pane-hidden'

  const key = srv.id + '::' + tabId
  if (enteringState[key]) {
    return enteringState[key] === 'right' ? 'pane-enter-right' : 'pane-enter-left'
  }
  return 'pane-visible'
}

function handleAnimationEnd(_e) {
  // Clear all entering states on any animation end
  // (only one pane animates at a time, so clearing all is safe)
  for (const key of Object.keys(enteringState)) {
    delete enteringState[key]
  }
}

// Catch Vue rendering errors
onErrorCaptured((err, instance, info) => {
  const msg = err instanceof Error ? err.message : String(err)
  log.error(`Vue error in ${instance?.$options?.name || instance?.type?.name || '?'}: ${msg}`, { info, stack: err?.stack })
  vueError.value = msg
  showError(`Render error: ${msg}`)
  return false // prevent propagation
})

onMounted(async () => {
  log.info('onMounted start')
  try {
    log.info('loading settings')
    await settingsStore.load()
    log.info('settings loaded', settingsStore.theme)

    // Apply debug logging toggle
    log.setLoggingEnabled(settingsStore.showDebug)
    watch(() => settingsStore.showDebug, (val) => {
      log.setLoggingEnabled(val)
    })

    log.info('initI18n')
    initI18n(settingsStore.language)

    log.info('initTheme')
    initTheme()

    log.info('applyTheme')
    settingsStore.applyTheme()

    log.info('loading profiles')
    await connectionStore.loadProfiles()
    log.info('profiles loaded', connectionStore.profiles.length)

    log.info('listening for connection-status')
    await listen('connection-status', (event) => {
      const { session_id, status, message } = event.payload
      log.info(`connection-status: ${status}`, { session_id, message })
      serverStore.setTabStatus(session_id, status)
      if (status === 'error' && message) {
        showError(message)
      }
    })

    log.info('listening for debug-event')
    await listen('debug-event', (event) => {
      const { session_id, level, source, message, elapsed_ms } = event.payload
      const tag = session_id ? `[${session_id.slice(0, 8)}] ${source}` : source
      const msg = elapsed_ms != null ? `${message} (${elapsed_ms}ms)` : message
      switch (level) {
        case 'error': log.error(`[Rust] ${msg}`, { tag }); break
        case 'warn':  log.warn(`[Rust] ${msg}`, { tag }); break
        default:      log.info(`[Rust] ${msg}`, { tag }); break
      }
    })
    log.info('listening for host-key-verify')
    await listen('host-key-verify', (event) => {
      log.info('host-key-verify', event.payload)
      hostKeyDialog.value?.show(event.payload)
    })

    log.info('onMounted complete')
  } catch (e) {
    log.error('onMounted crashed', { message: e?.message, stack: e?.stack })
    vueError.value = `Startup error: ${e?.message || e}`
  }
})
</script>

<template>
  <div class="app-shell h-screen flex flex-col bg-[var(--color-bg-primary)] text-[var(--color-text-primary)]">
    <TitleBar />
    <div class="flex flex-1 overflow-hidden min-h-0">
      <ActivityBar :active="activeView" @select="activeView = $event" />
      <Sidebar :view="activeView" @navigate="e => activeView = e.view" />
      <div class="flex flex-col flex-1 overflow-hidden min-w-0">
        <MainTabBar />
        <div class="flex-1 relative min-h-0 bg-[var(--color-bg-primary)]">
          <WelcomeScreen v-if="serverStore.servers.length === 0" />
          <!-- Render all servers, keep them alive, only show the active one -->
          <div
            v-for="srv in serverStore.servers"
            :key="srv.id"
            v-show="srv.id === serverStore.activeServerId"
            class="absolute inset-0"
          >
            <!-- Overview pane -->
            <div
              :class="['absolute inset-0', paneClasses(srv, 'overview')]"
              @animationend="handleAnimationEnd"
            >
              <ServerOverview
                :server-id="srv.id"
                :session-id="getOverviewSessionId(srv)"
                :host="srv.host"
              />
            </div>
            <!-- Terminal panes -->
            <div
              v-for="tab in srv.tabs.filter(t => t.type === 'terminal')"
              :key="tab.id"
              :class="['absolute inset-0', paneClasses(srv, tab.id)]"
              @animationend="handleAnimationEnd"
            >
              <TerminalContainer
                v-if="tab.sessionId"
                :session-id="tab.sessionId"
                :is-active="srv.id === serverStore.activeServerId && tab.id === srv.activeTabId"
              />
              <div v-else class="flex flex-col items-center justify-center h-full gap-2">
                <p v-if="tab.status === 'reconnecting' && tab._reconnectInfo" class="text-sm text-[var(--color-accent)]">
                  {{ t('reconnect.trying', { s: Math.ceil(Math.min(30000, Math.pow(2, (tab._reconnectInfo.n || 1) - 1) * 1000) / 1000) }) }}
                </p>
                <p v-if="tab.status === 'error'" class="text-sm text-[var(--color-danger)]">{{ t('status.error') }}</p>
                <p v-else-if="tab.status !== 'reconnecting'" class="text-sm text-[var(--color-text-tertiary)] animate-pulse">{{ t('status.connecting') }}</p>
              </div>
              <!-- Reconnect overlay on terminal -->
              <div
                v-if="tab.sessionId && tab.status === 'reconnecting' && tab._reconnectInfo"
                class="absolute inset-0 bg-[var(--color-bg-primary)]/60 flex items-center justify-center z-10 pointer-events-none"
              >
                <span class="text-xs font-medium text-[var(--color-accent)] bg-[var(--color-bg-primary)] px-3 py-1.5 rounded-lg shadow-sm">
                  {{ t('reconnect.trying', { s: Math.ceil(Math.min(30000, Math.pow(2, (tab._reconnectInfo.n || 1) - 1) * 1000) / 1000) }) }}
                </span>
              </div>
            </div>

            <!-- File Manager panes -->
            <div
              v-for="tab in srv.tabs.filter(t => t.type === 'filemanager')"
              :key="tab.id"
              :class="['absolute inset-0', paneClasses(srv, tab.id)]"
              @animationend="handleAnimationEnd"
            >
              <FileManager
                v-if="tab.sessionId"
                :session-id="tab.sessionId"
              />
              <div v-else class="flex flex-col items-center justify-center h-full gap-2">
                <p v-if="tab.status === 'reconnecting' && tab._reconnectInfo" class="text-sm text-[var(--color-accent)]">
                  {{ t('reconnect.trying', { s: Math.ceil(Math.min(30000, Math.pow(2, (tab._reconnectInfo.n || 1) - 1) * 1000) / 1000) }) }}
                </p>
                <p v-if="tab.status === 'error'" class="text-sm text-[var(--color-danger)]">{{ t('filemanager.connectionFailed') }}</p>
                <p v-else-if="tab.status !== 'reconnecting'" class="text-sm text-[var(--color-text-tertiary)] animate-pulse">{{ t('filemanager.connecting') }}</p>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
    <StatusBar />
    <Toast
      :visible="toast.visible"
      :message="toast.message"
      :type="toast.type"
      @close="closeToast"
    />
    <HostKeyDialog ref="hostKeyDialog" />
    <DebugPanel />
    <VersionCheck />
  </div>
</template>
