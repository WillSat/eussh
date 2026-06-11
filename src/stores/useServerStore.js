import { defineStore } from 'pinia'
import { useConnectionStore } from './useConnectionStore'
import { useToast } from '@/composables/useToast'
import { invoke } from '@/utils/ipc'
import { useSettingsStore } from './useSettingsStore'
import { useLogger } from '@/composables/useLogger'

function makeTabId() {
  return 'term-' + Math.random().toString(36).slice(2, 8)
}

const log = useLogger('ServerStore')

// Non-reactive module-level state for reconnect management
const _reconnectTimers = new Map()    // tabId → setTimeout id
const _reconnectCancel = new Set()    // sessionIds being manually disconnected

export const useServerStore = defineStore('servers', {
  state: () => ({
    servers: [],
    activeServerId: null,
  }),

  getters: {
    activeServer(state) {
      return state.servers.find(s => s.id === state.activeServerId) || null
    },
    hasServers(state) {
      return state.servers.length > 0
    },
  },

  actions: {
    displayName(profile) {
      return profile.name || profile.host
    },

    _findTab(sessionId) {
      for (const s of this.servers) {
        const tab = s.tabs.find(t => t.sessionId === sessionId)
        if (tab) return { server: s, tab }
      }
      return null
    },

    // ── Reconnect state (non-reactive module-level) ─────────────────
    // _reconnectTimers: Map<sessionId, timerId> — cleared on manual close
    // _reconnectAttempts: Map<sessionId, { profile, serverId, tabId, attempt }>
    // _reconnectCancel: Set<sessionId> — cancel signal

    setTabStatus(sessionId, status) {
      const found = this._findTab(sessionId)
      if (found) {
        found.tab.status = status
        if (status === 'disconnected') {
          this._onDisconnected(sessionId)
        }
      }
    },

    // ── Auto-reconnect ────────────────────────────────────────────
    _onDisconnected(sessionId) {
      // Skip if this session was cancelled (manual close)
      if (_reconnectCancel.has(sessionId)) {
        _reconnectCancel.delete(sessionId)
        return
      }
      const found = this._findTab(sessionId)
      if (!found) return
      const { server, tab } = found
      if (tab.type === 'overview' && tab.status !== 'disconnected') return

      const connStore = useConnectionStore()
      const profile = connStore.profiles.find(p => p.id === server.id)
      if (!profile) return

      this._startReconnect(profile, server.id, tab.id, 1)
    },

    _startReconnect(profile, serverId, tabId, attempt) {
      const MAX = 5
      // Exponential backoff: 1s, 2s, 4s, 8s, 16s, max 30s
      const delay = Math.min(30000, Math.pow(2, attempt - 1) * 1000)

      const server = this.servers.find(s => s.id === serverId)
      const tab = server?.tabs.find(t => t.id === tabId)
      if (!tab) return

      tab.status = 'reconnecting'
      tab._reconnectInfo = { n: attempt, max: MAX }

      const timerId = setTimeout(async () => {
        const srv = this.servers.find(s => s.id === serverId)
        const t = srv?.tabs.find(t => t.id === tabId)
        if (!t || t.status !== 'reconnecting') return

        const connStore = useConnectionStore()
        const fresh = connStore.profiles.find(p => p.id === serverId)
        if (!fresh) { t.status = 'error'; return }

        try {
          const newSessionId = await connStore.connect(fresh)
          t.sessionId = newSessionId
          t.status = 'connected'
          t._reconnectInfo = undefined
        } catch {
          if (attempt >= MAX) {
            t.status = 'error'
            t._reconnectInfo = undefined
            log.warn('reconnect gave up', { tabId, attempts: attempt })
          } else {
            this._startReconnect(fresh, serverId, tabId, attempt + 1)
          }
        }
      }, delay)

      // Track timer for cancellation on manual close
      _reconnectTimers.set(tabId, timerId)
    },

    _cancelReconnect(tab) {
      if (tab.sessionId) {
        _reconnectCancel.add(tab.sessionId)
      }
      if (tab._reconnectInfo) {
        const timerId = _reconnectTimers.get(tab.id)
        if (timerId) { clearTimeout(timerId); _reconnectTimers.delete(tab.id) }
        tab._reconnectInfo = undefined
        tab.status = 'disconnected'
      }
    },

    async openServer(profile) {
      log.info('openServer', { id: profile.id, name: profile.name, host: profile.host })
      const existing = this.servers.find(s => s.id === profile.id)
      if (existing) {
        this.activeServerId = profile.id
        return
      }

      const entry = {
        id: profile.id,
        nickname: this.displayName(profile),
        host: profile.host,
        tabs: [{ id: 'overview', type: 'overview', title: 'Overview', sessionId: null, status: 'connecting' }],
        activeTabId: 'overview',
        previousActiveTabId: null,
        latency: null,
      }
      this.servers.push(entry)
      this.activeServerId = profile.id
      log.info('entry added, connecting overview tab')

      const connStore = useConnectionStore()
      const toast = useToast()
      try {
        const sessionId = await connStore.connect(profile)
        log.info('overview got sessionId, waiting for connection', sessionId)
        const srv = this.servers.find(s => s.id === profile.id)
        srv.tabs[0].sessionId = sessionId
        srv.tabs[0].status = 'connected'
        connStore.updateLastConnected(profile.id)
        const pingMs = Math.max(5, useSettingsStore().pingIntervalSecs) * 1000
        srv._pingPending = false
        srv._pingTimer = setInterval(async () => {
          if (srv._pingPending) return  // skip if previous ping still in-flight
          srv._pingPending = true
          try {
            const start = performance.now()
            await invoke('ping', { sessionId })
            srv.latency = Math.round(performance.now() - start)
          } catch { srv.latency = null }
          finally { srv._pingPending = false }
        }, pingMs)

        // Server-side traffic polling (reads /proc/net/dev, zero client overhead)
        // Only start if enabled in settings
        const settings = useSettingsStore()
        srv.trafficUp = 0
        srv.trafficDown = 0
        if (settings.showTraffic) {
          const trafficMs = 5000
          srv._trafficPrev = null
          srv._trafficTimer = setInterval(async () => {
          try {
            const stats = await invoke('server_traffic', { sessionId })
            const now = performance.now()
            if (srv._trafficPrev) {
              const elapsed = (now - srv._trafficPrev.time) / 1000
              if (elapsed > 0) {
                srv.trafficUp = Math.max(0, Math.round((stats.tx_bytes - srv._trafficPrev.tx) / elapsed))
                srv.trafficDown = Math.max(0, Math.round((stats.rx_bytes - srv._trafficPrev.rx) / elapsed))
              }
            }
            srv._trafficPrev = { time: now, rx: stats.rx_bytes, tx: stats.tx_bytes }
          } catch {
            srv.trafficUp = 0
            srv.trafficDown = 0
            srv._trafficPrev = null
          }
        }, trafficMs)
        }
      } catch (e) {
        log.error('overview connect FAILED', e?.message || e)
        const srv = this.servers.find(s => s.id === profile.id)
        if (srv) srv.tabs[0].status = 'error'
        toast.error(`Failed to connect: ${e}`)
      }
    },

    closeServer(profileId) {
      const idx = this.servers.findIndex(s => s.id === profileId)
      if (idx < 0) return
      const server = this.servers[idx]
      const connStore = useConnectionStore()
      clearInterval(server._pingTimer)
      clearInterval(server._trafficTimer)
      for (const tab of server.tabs) {
        this._cancelReconnect(tab)
        if (tab.sessionId) connStore.disconnect(tab.sessionId).catch(() => {})
      }
      this.servers.splice(idx, 1)
      if (this.activeServerId === profileId) {
        this.activeServerId = this.servers[this.servers.length - 1]?.id || null
      }
    },

    switchServer(profileId) {
      if (this.servers.some(s => s.id === profileId)) {
        this.activeServerId = profileId
      }
    },

    async addTerminalTab(profileId) {
      const server = this.servers.find(s => s.id === profileId)
      if (!server) return

      const connStore = useConnectionStore()
      const profile = connStore.profiles.find(p => p.id === profileId)
      if (!profile) return

      const id = makeTabId()
      const num = server.tabs.filter(t => t.type === 'terminal').length + 1
      const tab = { id, type: 'terminal', title: `Terminal ${num}`, sessionId: null, status: 'connecting' }
      server.tabs.push(tab)
      server.previousActiveTabId = server.activeTabId
      server.activeTabId = id
      log.info('terminal tab added, connecting', { tabId: id })

      const toast = useToast()
      try {
        const sessionId = await connStore.connect(profile)
        log.info('terminal got sessionId, waiting for connection', sessionId)
        const rTab = server.tabs.find(t => t.id === id)
        if (rTab) {
          rTab.sessionId = sessionId
          rTab.status = 'connected'
        }
      } catch (e) {
        log.error('terminal connect FAILED', e?.message || e)
        const rTab = server.tabs.find(t => t.id === id)
        if (rTab) rTab.status = 'error'
        toast.error(`Failed to open terminal: ${e}`)
      }
    },

    async addFileManagerTab(profileId) {
      const server = this.servers.find(s => s.id === profileId)
      if (!server) return

      const connStore = useConnectionStore()
      const profile = connStore.profiles.find(p => p.id === profileId)
      if (!profile) return

      const id = makeTabId()
      const num = server.tabs.filter(t => t.type === 'filemanager').length + 1
      const tab = { id, type: 'filemanager', title: `Files ${num}`, sessionId: null, status: 'connecting' }
      server.tabs.push(tab)
      server.previousActiveTabId = server.activeTabId
      server.activeTabId = id
      log.info('filemanager tab added, connecting', { tabId: id })

      const toast = useToast()
      try {
        const sessionId = await connStore.connect(profile)
        log.info('filemanager got sessionId', sessionId)
        const rTab = server.tabs.find(t => t.id === id)
        if (rTab) {
          rTab.sessionId = sessionId
          rTab.status = 'connected'
        }
      } catch (e) {
        log.error('filemanager connect FAILED', e?.message || e)
        const rTab = server.tabs.find(t => t.id === id)
        if (rTab) rTab.status = 'error'
        toast.error(`Failed to open file manager: ${e}`)
      }
    },

    removeTab(profileId, tabId) {
      const server = this.servers.find(s => s.id === profileId)
      if (!server) return
      const idx = server.tabs.findIndex(t => t.id === tabId)
      if (idx < 0) return
      const tab = server.tabs[idx]
      if (tab.type === 'overview') return
      this._cancelReconnect(tab)
      const connStore = useConnectionStore()
      if (tab.sessionId) connStore.disconnect(tab.sessionId).catch(() => {})
      server.tabs.splice(idx, 1)
      if (server.activeTabId === tabId) {
        server.previousActiveTabId = server.activeTabId
        server.activeTabId = server.tabs[server.tabs.length - 1]?.id || 'overview'
      }
    },

    setActiveTab(profileId, tabId) {
      const server = this.servers.find(s => s.id === profileId)
      if (!server) return
      if (server.tabs.some(t => t.id === tabId)) {
        server.previousActiveTabId = server.activeTabId
        server.activeTabId = tabId
      }
    },
  },
})
