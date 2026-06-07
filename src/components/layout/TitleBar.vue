<script setup>
import { computed, watch } from 'vue'
import { useServerStore } from '@/stores/useServerStore'
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

    <!-- Right: window controls (rounded squares) -->
    <div class="flex items-center gap-1.5 pr-2">
      <button @click="minimize"
        class="w-3 h-3 rounded-full bg-[#FEBC2E] hover:bg-[#FF9500] transition-colors" />
      <button @click="toggleMaximize"
        class="w-3 h-3 rounded-full bg-[#28C840] hover:bg-[#34C759] transition-colors" />
      <button @click="close"
        class="w-3 h-3 rounded-full bg-[#FF5F57] hover:bg-[#FF3B30] transition-colors" />
    </div>
  </div>
</template>
