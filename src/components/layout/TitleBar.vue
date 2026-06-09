<script setup>
import { computed, watch } from 'vue'
import { useServerStore } from '@/stores/useServerStore'
import { useSettingsStore } from '@/stores/useSettingsStore'
import { useLogger } from '@/composables/useLogger'
import { useI18n } from '@/composables/useI18n'
import { getCurrentWindow } from '@tauri-apps/api/window'

const log = useLogger('TitleBar')
const { t } = useI18n()

const props = defineProps({
  showSettings: { type: Boolean, default: false },
})
const emit = defineEmits(['update:showSettings'])

const serverStore = useServerStore()
const settingsStore = useSettingsStore()
const win = getCurrentWindow()
const { logState } = log

const title = computed(() => {
  const s = serverStore.activeServer
  return s ? `${s.nickname} - Eussh` : 'Eussh'
})

watch(title, (t) => { try { win.setTitle(t) } catch {} }, { immediate: true })

function toggleSettings() {
  log.info('toggle settings', { from: props.showSettings, to: !props.showSettings })
  emit('update:showSettings', !props.showSettings)
}

function toggleDebug() {
  logState.showPanel = !logState.showPanel
}

function minimize() { win.minimize() }
function toggleMaximize() { win.toggleMaximize() }
function close() { win.close() }
</script>

<template>
  <div
    data-tauri-drag-region
    class="flex items-center justify-between h-8 shrink-0 select-none
      bg-[var(--color-bg-secondary)] border-b border-[var(--color-border)]"
  >
    <!-- Left: debug + settings -->
    <div class="flex items-center gap-1 pl-2">
      <button
        @click="toggleDebug"
        class="px-2 py-1 text-[11px] font-medium rounded
          text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]
          hover:bg-[var(--color-bg-tertiary)] transition-colors"
      >{{ t('titlebar.debug') }}</button>
      <button
        @click="toggleSettings"
        class="px-2 py-1 text-[11px] font-medium rounded
          text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]
          hover:bg-[var(--color-bg-tertiary)] transition-colors"
      >{{ t('titlebar.settings') }}</button>
    </div>

    <!-- Center title -->
    <span class="absolute left-1/2 -translate-x-1/2 text-[11px] font-medium
      text-[var(--color-text-secondary)] pointer-events-none">
      {{ title }}
    </span>

    <!-- Right: window controls -->
    <!-- macOS style: colored dots with icons -->
    <div v-if="settingsStore.titlebarStyle === 'macos'" class="flex items-center gap-1.5 pr-2">
      <button @click="minimize"
        class="group w-3 h-3 rounded-full bg-[#FEBC2E] hover:bg-[#F5A623] transition-colors flex items-center justify-center"
        title="Minimize">
        <svg class="w-1.5 h-1.5 opacity-0 group-hover:opacity-100 transition-opacity" viewBox="0 0 10 10">
          <line x1="2" y1="5" x2="8" y2="5" stroke="#995700" stroke-width="1.2" stroke-linecap="round" />
        </svg>
      </button>
      <button @click="toggleMaximize"
        class="group w-3 h-3 rounded-full bg-[#28C840] hover:bg-[#1DAF35] transition-colors flex items-center justify-center"
        title="Maximize">
        <svg class="w-1.5 h-1.5 opacity-0 group-hover:opacity-100 transition-opacity" viewBox="0 0 10 10">
          <rect x="2" y="2" width="6" height="6" fill="none" stroke="#006500" stroke-width="1.2" rx="1" />
        </svg>
      </button>
      <button @click="close"
        class="group w-3 h-3 rounded-full bg-[#FF5F57] hover:bg-[#EE433B] transition-colors flex items-center justify-center"
        title="Close">
        <svg class="w-1.5 h-1.5 opacity-0 group-hover:opacity-100 transition-opacity" viewBox="0 0 10 10">
          <line x1="2.5" y1="2.5" x2="7.5" y2="7.5" stroke="#520000" stroke-width="1.2" stroke-linecap="round" />
          <line x1="7.5" y1="2.5" x2="2.5" y2="7.5" stroke="#520000" stroke-width="1.2" stroke-linecap="round" />
        </svg>
      </button>
    </div>

    <!-- Windows 11 style: square buttons -->
    <div v-else class="flex items-center h-full">
      <button @click="minimize"
        class="w-[46px] h-full flex items-center justify-center
          text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]
          hover:bg-[var(--color-bg-tertiary)] transition-colors"
        title="Minimize">
        <svg class="w-2.5 h-2.5" viewBox="0 0 10 10">
          <line x1="2" y1="5" x2="8" y2="5" stroke="currentColor" stroke-width="1" stroke-linecap="round" />
        </svg>
      </button>
      <button @click="toggleMaximize"
        class="w-[46px] h-full flex items-center justify-center
          text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]
          hover:bg-[var(--color-bg-tertiary)] transition-colors"
        title="Maximize">
        <svg class="w-2.5 h-2.5" viewBox="0 0 10 10">
          <rect x="1.5" y="1.5" width="7" height="7" fill="none" stroke="currentColor" stroke-width="1" rx="1" />
        </svg>
      </button>
      <button @click="close"
        class="w-[46px] h-full flex items-center justify-center
          text-[var(--color-text-secondary)] hover:text-white
          hover:bg-[#E81123] transition-colors"
        title="Close">
        <svg class="w-2.5 h-2.5" viewBox="0 0 10 10">
          <line x1="2.5" y1="2.5" x2="7.5" y2="7.5" stroke="currentColor" stroke-width="1" stroke-linecap="round" />
          <line x1="7.5" y1="2.5" x2="2.5" y2="7.5" stroke="currentColor" stroke-width="1" stroke-linecap="round" />
        </svg>
      </button>
    </div>
  </div>
</template>
