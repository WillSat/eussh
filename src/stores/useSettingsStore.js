import { defineStore } from 'pinia'
import { invoke } from '@/utils/ipc'

export const useSettingsStore = defineStore('settings', {
  state: () => ({
    theme: 'system',
    language: '',
    fontSize: 16,
    fontFamily: "Consolas, 'Courier New', monospace",
    cursorStyle: 'bar',
    scrollback: 10000,
    sidebarWidth: 260,
    terminalColorPreset: 'default-dark',
    monitorRefreshSecs: 2,
    pingIntervalSecs: 15,
    accentColor: '#007AFF',
    titlebarStyle: 'macos',
    statusbarStyle: 'default',
    showTraffic: true,
  }),

  getters: {
    isDark(state) {
      if (state.theme === 'dark') return true
      if (state.theme === 'light') return false
      return window.matchMedia('(prefers-color-scheme: dark)').matches
    },
  },

  actions: {
    async load() {
      try {
        const config = await invoke('get_config')
        if (config.theme) this.theme = config.theme
        if (config.language) this.language = config.language
        const s = config.settings
        if (s) {
          if (s.font_size) this.fontSize = s.font_size
          if (s.font_family) this.fontFamily = s.font_family
          if (s.cursor_style) this.cursorStyle = s.cursor_style
          if (s.scrollback) this.scrollback = s.scrollback
          if (s.sidebar_width) this.sidebarWidth = s.sidebar_width
          if (s.terminal_color_preset) this.terminalColorPreset = s.terminal_color_preset
          if (s.monitor_refresh_secs) this.monitorRefreshSecs = s.monitor_refresh_secs
          if (s.ping_interval_secs) this.pingIntervalSecs = s.ping_interval_secs
          if (s.accent_color) this.accentColor = s.accent_color
          if (s.titlebar_style) this.titlebarStyle = s.titlebar_style
          if (s.statusbar_style) this.statusbarStyle = s.statusbar_style
          if (s.show_traffic !== undefined) this.showTraffic = s.show_traffic
        }
        try { localStorage.setItem('eussh-theme', this.theme) } catch {}
      } catch (_) {}
    },

    async save() {
      try {
        await invoke('save_config', {
          config: {
            theme: this.theme,
            language: this.language,
            settings: {
              font_size: this.fontSize,
              font_family: this.fontFamily,
              cursor_style: this.cursorStyle,
              scrollback: this.scrollback,
              sidebar_width: this.sidebarWidth,
              terminal_color_preset: this.terminalColorPreset,
              monitor_refresh_secs: this.monitorRefreshSecs,
              ping_interval_secs: this.pingIntervalSecs,
              accent_color: this.accentColor,
              titlebar_style: this.titlebarStyle,
              statusbar_style: this.statusbarStyle,
              show_traffic: this.showTraffic,
            },
          },
        })
      } catch (_) {}
    },

    setTheme(theme) {
      this.theme = theme
      this.applyTheme()
      this.save()
      try { localStorage.setItem('eussh-theme', theme) } catch {}
    },

    setTitlebarStyle(style) {
      this.titlebarStyle = style
      this.save()
    },

    setStatusbarStyle(style) {
      this.statusbarStyle = style
      this.save()
    },

    applyTheme() {
      const dark = this.isDark
      document.documentElement.classList.toggle('dark', dark)
      document.documentElement.classList.toggle('light', !dark)
      document.documentElement.style.setProperty('--color-accent', this.accentColor)
      document.documentElement.style.setProperty('--color-accent-hover', this.accentColor + 'cc')
    },
  },
})
