<script setup>
import { ref, computed, onMounted, onBeforeUnmount } from 'vue'
import { useSettingsStore } from '@/stores/useSettingsStore'
import { useI18n } from '@/composables/useI18n'
import { COLOR_PRESETS } from '@/utils/theme'

const { t, locale, setLocale } = useI18n()
const settings = useSettingsStore()
const emit = defineEmits(['close'])

const sTab = ref('general')
const sTabs = computed(() => [
  { id: 'general',    label: t('settings.general') },
  { id: 'appearance', label: t('settings.appearance') },
  { id: 'terminal',   label: t('settings.terminal') },
  { id: 'monitoring', label: t('settings.monitoring') },
])

function onKey(e) { if (e.key === 'Escape') emit('close') }
onMounted(() => document.addEventListener('keydown', onKey))
onBeforeUnmount(() => document.removeEventListener('keydown', onKey))

function seg(active) {
  return active
    ? 'flex-1 py-1.5 text-[12px] font-medium rounded bg-[var(--color-accent)] text-white shadow-sm'
    : 'flex-1 py-1.5 text-[12px] font-medium rounded text-[var(--color-text-tertiary)]/50 hover:text-[var(--color-text-secondary)]'
}
function inp() { return 'w-full px-2 py-1.5 text-[12px] rounded border border-[var(--color-border)] bg-[var(--color-bg-primary)] text-[var(--color-text-primary)] focus:outline-none focus:border-[var(--color-accent)]' }

const presets = Object.entries(COLOR_PRESETS)
const accentColors = ['#007AFF','#34C759','#FF9500','#FF3B30','#AF52DE','#5856D6','#00C7BE','#FF2D55']
const langList = ['en','zh-CN']
function setLang(l) { setLocale(l); settings.language=l; settings.save() }
</script>
<template>
  <div class="absolute inset-0 z-50 bg-[var(--color-bg-primary)] flex flex-col">
    <!-- Header bar -->
    <div class="shrink-0 flex items-center justify-between px-4 h-10 border-b border-[var(--color-border)] bg-[var(--color-bg-secondary)] select-none">
      <span class="text-[12px] font-semibold text-[var(--color-text-primary)]">{{ t('settings.title') }}</span>
      <button @click="emit('close')"
        class="w-7 h-7 flex items-center justify-center rounded-md text-[var(--color-text-tertiary)] hover:text-[var(--color-text-primary)] hover:bg-[var(--color-bg-tertiary)] transition-colors">
        <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
      </button>
    </div>
    <!-- Tab bar -->
    <div class="shrink-0 flex items-center gap-1 px-3 py-2 border-b border-[var(--color-border)] bg-[var(--color-bg-primary)]/30">
      <button v-for="st in sTabs" :key="st.id" @click="sTab=st.id"
        :class="['px-3 py-1.5 text-[12px] font-medium rounded transition-all', sTab===st.id ? 'text-[var(--color-accent)] bg-[var(--color-accent)]/10' : 'text-[var(--color-text-tertiary)] hover:text-[var(--color-text-secondary)] hover:bg-[var(--color-bg-tertiary)]']">{{ st.label }}</button>
    </div>
    <!-- Body -->
    <div class="flex-1 overflow-y-auto">

      <!-- GENERAL -->
      <div v-show="sTab==='general'" class="px-3 py-3 space-y-3">
        <span class="text-[10px] font-bold uppercase tracking-widest text-[var(--color-accent)]">{{ t('settings.updates') }}</span>
        <div>
          <div class="flex items-center justify-between">
            <span class="text-[11px] font-medium text-[var(--color-text-secondary)]">{{ t('settings.checkUpdates') }}</span>
            <button role="switch" :aria-checked="settings.checkUpdates" @click="settings.checkUpdates=!settings.checkUpdates; settings.save()"
              :class="['inline-flex h-4 w-7 shrink-0 rounded-full border-2 border-transparent transition-colors duration-200', settings.checkUpdates?'bg-[var(--color-accent)]':'bg-[var(--color-bg-tertiary)]']">
              <span :class="['inline-block h-3 w-3 rounded-full bg-white shadow transition-transform duration-200', settings.checkUpdates?'translate-x-3':'translate-x-0']" /></button>
          </div>
          <p class="text-[10px] text-[var(--color-text-tertiary)]/40 mt-0.5">{{ t('settingsDesc.checkUpdates') }}</p>
        </div>
        <div>
          <div class="flex items-center justify-between">
            <span class="text-[11px] font-medium text-[var(--color-text-secondary)]">{{ t('settings.showDebug') }}</span>
            <button role="switch" :aria-checked="settings.showDebug" @click="settings.showDebug=!settings.showDebug; settings.save()"
              :class="['inline-flex h-4 w-7 shrink-0 rounded-full border-2 border-transparent transition-colors duration-200', settings.showDebug?'bg-[var(--color-accent)]':'bg-[var(--color-bg-tertiary)]']">
              <span :class="['inline-block h-3 w-3 rounded-full bg-white shadow transition-transform duration-200', settings.showDebug?'translate-x-3':'translate-x-0']" /></button>
          </div>
          <p class="text-[10px] text-[var(--color-text-tertiary)]/40 mt-0.5">{{ t('settingsDesc.showDebug') }}</p>
        </div>
        <div class="pt-3 mt-1 border-t border-[var(--color-border)] space-y-1">
          <div class="flex justify-between text-[11px]"><span class="text-[var(--color-text-tertiary)]">Version</span><span class="text-[var(--color-text-primary)] font-mono">1.3.3</span></div>
          <div class="flex justify-between text-[11px]"><span class="text-[var(--color-text-tertiary)]">License</span><span class="text-[var(--color-text-primary)]">MIT</span></div>
        </div>
      </div>

      <!-- APPEARANCE -->
      <div v-show="sTab==='appearance'" class="px-3 py-3 space-y-3">
        <span class="text-[10px] font-bold uppercase tracking-widest text-[var(--color-accent)]">{{ t('settings.appearance') }}</span>
        <div>
          <span class="text-[11px] font-medium text-[var(--color-text-secondary)]">{{ t('settings.theme') }}</span>
          <p class="text-[10px] text-[var(--color-text-tertiary)]/40 mt-0.5">{{ t('settingsDesc.theme') }}</p>
          <div class="flex rounded-lg bg-[var(--color-bg-primary)] p-0.5 mt-1.5">
            <button v-for="th in ['light','system','dark']" :key="th" @click="settings.setTheme(th)" :class="seg(settings.theme===th)">{{ t(`settings.${th}`) }}</button>
          </div>
        </div>
        <div>
          <span class="text-[11px] font-medium text-[var(--color-text-secondary)]">{{ t('settings.accentColor') }}</span>
          <p class="text-[10px] text-[var(--color-text-tertiary)]/40 mt-0.5">{{ t('settingsDesc.accent') }}</p>
          <div class="flex gap-1.5 mt-1.5 flex-wrap">
            <button v-for="c in accentColors" :key="c" @click="settings.accentColor=c; settings.applyTheme(); settings.save()"
              :class="['w-7 h-7 rounded-full transition-all hover:scale-115', settings.accentColor===c ? 'ring-2 ring-offset-2 ring-offset-[var(--color-bg-secondary)] scale-110' : '']" :style="{background:c,ringColor:c}" />
          </div>
        </div>
        <div>
          <span class="text-[11px] font-medium text-[var(--color-text-secondary)]">{{ t('settings.titlebarStyle') }}</span>
          <p class="text-[10px] text-[var(--color-text-tertiary)]/40 mt-0.5">{{ t('settingsDesc.titlebar') }}</p>
          <div class="flex rounded-lg bg-[var(--color-bg-primary)] p-0.5 mt-1.5">
            <button @click="settings.setTitlebarStyle('macos')" :class="seg(settings.titlebarStyle==='macos')">{{ t('settings.titlebarMacos') }}</button>
            <button @click="settings.setTitlebarStyle('win11')" :class="seg(settings.titlebarStyle==='win11')">{{ t('settings.titlebarWin11') }}</button>
          </div>
        </div>
        <div>
          <span class="text-[11px] font-medium text-[var(--color-text-secondary)]">{{ t('settings.statusbarStyle') }}</span>
          <p class="text-[10px] text-[var(--color-text-tertiary)]/40 mt-0.5">{{ t('settingsDesc.statusbar') }}</p>
          <div class="flex rounded-lg bg-[var(--color-bg-primary)] p-0.5 mt-1.5">
            <button @click="settings.setStatusbarStyle('default')" :class="seg(settings.statusbarStyle==='default')">{{ t('settings.statusbarDefault') }}</button>
            <button @click="settings.setStatusbarStyle('accent')" :class="seg(settings.statusbarStyle==='accent')">{{ t('settings.statusbarAccent') }}</button>
          </div>
        </div>
        <div>
          <span class="text-[11px] font-medium text-[var(--color-text-secondary)]">{{ t('settings.language') }}</span>
          <p class="text-[10px] text-[var(--color-text-tertiary)]/40 mt-0.5">{{ t('settingsDesc.language') }}</p>
          <div class="flex rounded-lg bg-[var(--color-bg-primary)] p-0.5 mt-1.5">
            <button v-for="loc in langList" :key="loc" @click="setLang(loc)" :class="seg(locale===loc)">{{ loc==='zh-CN'?'中文':'English' }}</button>
          </div>
        </div>
      </div>

      <!-- TERMINAL -->
      <div v-show="sTab==='terminal'" class="px-3 py-3 space-y-3">
        <span class="text-[10px] font-bold uppercase tracking-widest text-[var(--color-accent)]">{{ t('settings.terminal') }}</span>
        <div>
          <span class="text-[11px] font-medium text-[var(--color-text-secondary)]">{{ t('settings.fontFamily') }}</span>
          <p class="text-[10px] text-[var(--color-text-tertiary)]/40 mt-0.5">{{ t('settingsDesc.fontFamily') }}</p>
          <input :value="settings.fontFamily" @change="settings.fontFamily=$event.target.value; settings.save()" :class="inp()" />
        </div>
        <div>
          <span class="text-[11px] font-medium text-[var(--color-text-secondary)]">{{ t('settings.fontSize') }}</span>
          <p class="text-[10px] text-[var(--color-text-tertiary)]/40 mt-0.5">{{ t('settingsDesc.fontSize') }}</p>
          <div class="flex items-center gap-2 mt-1.5">
            <input type="range" min="10" max="24" :value="settings.fontSize" @input="settings.fontSize=Number($event.target.value); settings.save()" class="flex-1 accent-[var(--color-accent)] h-1" />
            <span class="text-xs tabular-nums text-[var(--color-text-primary)] font-mono w-8 text-right">{{ settings.fontSize }}</span>
          </div>
        </div>
        <div>
          <span class="text-[11px] font-medium text-[var(--color-text-secondary)]">{{ t('settings.cursor') }}</span>
          <p class="text-[10px] text-[var(--color-text-tertiary)]/40 mt-0.5">{{ t('settingsDesc.cursor') }}</p>
          <div class="flex rounded-lg bg-[var(--color-bg-primary)] p-0.5 mt-1.5">
            <button v-for="c in ['block','bar','underline']" :key="c" @click="settings.cursorStyle=c; settings.save()" :class="seg(settings.cursorStyle===c)">{{ t(`settings.${c}`) }}</button>
          </div>
        </div>
        <div>
          <span class="text-[11px] font-medium text-[var(--color-text-secondary)]">{{ t('settings.terminalColors') }}</span>
          <p class="text-[10px] text-[var(--color-text-tertiary)]/40 mt-0.5">{{ t('settingsDesc.terminalColors') }}</p>
          <div class="grid grid-cols-2 gap-1.5 mt-1.5">
            <button v-for="[key,p] in presets" :key="key" @click="settings.terminalColorPreset=key; settings.save()"
              :class="['flex items-center gap-2 p-2 rounded-lg transition-all text-left', settings.terminalColorPreset===key ? 'ring-2 ring-[var(--color-accent)] bg-[var(--color-bg-primary)]' : 'hover:bg-[var(--color-bg-primary)]']">
              <div class="w-8 h-5 rounded-sm border border-[var(--color-border)] shrink-0 relative" :style="{background:p.background}"><div class="absolute right-0 top-0 bottom-0 w-1.5" :style="{background:p.cursor}" /></div>
              <span class="text-[11px] text-[var(--color-text-secondary)] truncate">{{ p.label }}</span>
            </button>
          </div>
        </div>
      </div>

      <!-- MONITORING -->
      <div v-show="sTab==='monitoring'" class="px-3 py-3 space-y-3">
        <span class="text-[10px] font-bold uppercase tracking-widest text-[var(--color-accent)]">{{ t('settings.monitoring') }}</span>
        <div>
          <span class="text-[11px] font-medium text-[var(--color-text-secondary)]">{{ t('settings.sysRefresh') }}</span>
          <p class="text-[10px] text-[var(--color-text-tertiary)]/40 mt-0.5">{{ t('settingsDesc.sysRefresh') }}</p>
          <div class="flex items-center gap-1.5 mt-1.5">
            <input type="number" min="3" max="30" :value="settings.monitorRefreshSecs" @change="settings.monitorRefreshSecs=Number($event.target.value); settings.save()"
              class="w-14 px-2 py-1.5 text-[12px] text-right rounded-lg border border-[var(--color-border)] bg-[var(--color-bg-primary)] text-[var(--color-text-primary)] font-mono focus:outline-none focus:border-[var(--color-accent)]" />
            <span class="text-[11px] text-[var(--color-text-tertiary)]">s</span>
          </div>
        </div>
        <div>
          <span class="text-[11px] font-medium text-[var(--color-text-secondary)]">{{ t('settings.pingInterval') }}</span>
          <p class="text-[10px] text-[var(--color-text-tertiary)]/40 mt-0.5">{{ t('settingsDesc.pingInterval') }}</p>
          <div class="flex items-center gap-1.5 mt-1.5">
            <input type="number" min="3" max="120" :value="settings.pingIntervalSecs" @change="settings.pingIntervalSecs=Number($event.target.value); settings.save()"
              class="w-14 px-2 py-1.5 text-[12px] text-right rounded-lg border border-[var(--color-border)] bg-[var(--color-bg-primary)] text-[var(--color-text-primary)] font-mono focus:outline-none focus:border-[var(--color-accent)]" />
            <span class="text-[11px] text-[var(--color-text-tertiary)]">s</span>
          </div>
        </div>
        <div>
          <div class="flex items-center justify-between">
            <span class="text-[11px] font-medium text-[var(--color-text-secondary)]">{{ t('settings.showTraffic') }}</span>
            <button role="switch" :aria-checked="settings.showTraffic" @click="settings.showTraffic=!settings.showTraffic; settings.save()"
              :class="['inline-flex h-4 w-7 shrink-0 rounded-full border-2 border-transparent transition-colors duration-200', settings.showTraffic?'bg-[var(--color-accent)]':'bg-[var(--color-bg-tertiary)]']">
              <span :class="['inline-block h-3 w-3 rounded-full bg-white shadow transition-transform duration-200', settings.showTraffic?'translate-x-3':'translate-x-0']" /></button>
          </div>
          <p class="text-[10px] text-[var(--color-text-tertiary)]/40 mt-0.5">{{ t('settingsDesc.traffic') }}</p>
        </div>
        <div>
          <div class="flex items-center justify-between">
            <span class="text-[11px] font-medium text-[var(--color-text-secondary)]">{{ t('settings.showGeoLookup') }}</span>
            <button role="switch" :aria-checked="settings.showGeoLookup" @click="settings.showGeoLookup=!settings.showGeoLookup; settings.save()"
              :class="['inline-flex h-4 w-7 shrink-0 rounded-full border-2 border-transparent transition-colors duration-200', settings.showGeoLookup?'bg-[var(--color-accent)]':'bg-[var(--color-bg-tertiary)]']">
              <span :class="['inline-block h-3 w-3 rounded-full bg-white shadow transition-transform duration-200', settings.showGeoLookup?'translate-x-3':'translate-x-0']" /></button>
          </div>
          <p class="text-[10px] text-[var(--color-text-tertiary)]/40 mt-0.5">{{ t('settingsDesc.geoLookup') }}</p>
        </div>
      </div>

    </div>
  </div>
</template>
