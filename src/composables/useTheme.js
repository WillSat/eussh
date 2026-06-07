import { useSettingsStore } from '@/stores/useSettingsStore'
import { watch } from 'vue'

export function useTheme() {
  const settings = useSettingsStore()

  function init() {
    settings.applyTheme()

    // Listen for system theme changes
    window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {
      if (settings.theme === 'system') {
        settings.applyTheme()
      }
    })
  }

  function toggle() {
    const next = { light: 'dark', dark: 'system', system: 'light' }
    settings.setTheme(next[settings.theme])
  }

  watch(() => settings.theme, () => settings.applyTheme())

  return { init, toggle, isDark: settings.isDark }
}
