import { defineStore } from 'pinia'
import { invoke } from '@/utils/ipc'
import { useLogger } from '@/composables/useLogger'

const log = useLogger('ConnStore')

export const useConnectionStore = defineStore('connections', {
  state: () => ({
    profiles: [],
    connecting: new Set(),
    loadError: null,
  }),

  getters: {
    isConnecting: (state) => (id) => state.connecting.has(id),
  },

  actions: {
    async loadProfiles() {
      try {
        const config = await invoke('get_config')
        this.profiles = config.connections || []
      } catch (e) {
        this.loadError = e
      }
    },

    async saveProfile(profile) {
      const saved = await invoke('save_connection', { profile })
      const idx = this.profiles.findIndex(p => p.id === saved.id)
      if (idx >= 0) {
        this.profiles[idx] = saved
      } else {
        this.profiles.push(saved)
      }
      return saved
    },

    async deleteProfile(id) {
      await invoke('delete_connection', { id })
      this.profiles = this.profiles.filter(p => p.id !== id)
    },

    async connect(profile) {
      log.info('connect', { id: profile.id, host: profile.host })
      this.connecting.add(profile.id)
      try {
        const sessionId = await invoke('connect', { profile })
        log.info('connect OK, sessionId:', sessionId)
        this.connecting.delete(profile.id)
        return sessionId
      } catch (e) {
        log.error('connect FAILED:', e?.message || e)
        this.connecting.delete(profile.id)
        throw e
      }
    },

    async disconnect(sessionId) {
      await invoke('disconnect', { sessionId })
    },

    updateLastConnected(id) {
      const profile = this.profiles.find(p => p.id === id)
      if (profile) {
        profile.last_connected = Date.now() / 1000
        this.saveProfile(profile)
      }
    },
  },
})
