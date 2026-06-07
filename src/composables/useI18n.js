import { reactive, computed } from 'vue'
import en from '@/locales/en'
import zhCN from '@/locales/zh-CN'

const locales = { en, 'zh-CN': zhCN }

function detectSystem() {
  try {
    const lang = (navigator.language || 'en').toLowerCase()
    if (lang.startsWith('zh')) return 'zh-CN'
    return 'en'
  } catch { return 'en' }
}

const state = reactive({
  locale: 'en',
  systemLocale: detectSystem(),
})

function t(key, params) {
  const localeData = locales[state.locale] || locales.en
  const keys = key.split('.')
  let val = localeData
  for (const k of keys) {
    if (!val) break
    val = val[k]
  }
  let text = val || key
  if (params) {
    Object.entries(params).forEach(([k, v]) => {
      text = text.replace(`{${k}}`, v)
    })
  }
  return text
}

export function useI18n() {
  const locale = computed(() => state.locale)
  const systemLocale = computed(() => state.systemLocale)

  function setLocale(loc) {
    state.locale = loc in locales ? loc : 'en'
  }

  function init(savedLocale) {
    if (savedLocale && savedLocale in locales) {
      state.locale = savedLocale
    } else {
      state.locale = detectSystem()
    }
  }

  return { t, locale, systemLocale, setLocale, init, locales: Object.keys(locales) }
}
