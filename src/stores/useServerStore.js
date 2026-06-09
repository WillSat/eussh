import { defineStore } from 'pinia'
import { useConnectionStore } from './useConnectionStore'
import { useToast } from '@/composables/useToast'
import { invoke } from '@/utils/ipc'
import { useSettingsStore } from './useSettingsStore'

function makeTabId() {
  return 'term-' + Math.random().toString(36).slice(2, 8)
}

const _log = (...args) => { try { console.log('[ServerStore]', ...args) } catch {} }

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

    setTabStatus(sessionId, status) {
      const found = this._findTab(sessionId)
      if (found) found.tab.status = status
    },

    async openServer(profile) {
      _log('openServer', { id: profile.id, name: profile.name, host: profile.host })
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
      _log('entry added, connecting overview tab')

      const connStore = useConnectionStore()
      const toast = useToast()
      try {
        const sessionId = await connStore.connect(profile)
        _log('overview got sessionId, waiting for connection', sessionId)
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
      } catch (e) {
        _log('overview connect FAILED', e?.message || e)
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
      for (const tab of server.tabs) {
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
      _log('terminal tab added, connecting', { tabId: id })

      const toast = useToast()
      try {
        const sessionId = await connStore.connect(profile)
        _log('terminal got sessionId, waiting for connection', sessionId)
        const rTab = server.tabs.find(t => t.id === id)
        if (rTab) {
          rTab.sessionId = sessionId
          rTab.status = 'connected'
        }
      } catch (e) {
        _log('terminal connect FAILED', e?.message || e)
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
      _log('filemanager tab added, connecting', { tabId: id })

      const toast = useToast()
      try {
        const sessionId = await connStore.connect(profile)
        _log('filemanager got sessionId', sessionId)
        const rTab = server.tabs.find(t => t.id === id)
        if (rTab) {
          rTab.sessionId = sessionId
          rTab.status = 'connected'
        }
      } catch (e) {
        _log('filemanager connect FAILED', e?.message || e)
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
