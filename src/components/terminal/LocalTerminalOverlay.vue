<script setup>
import { ref, watch, onMounted, onBeforeUnmount } from 'vue'
import { useXterm } from '@/composables/useXterm'
import { invoke } from '@/utils/ipc'
import { useI18n } from '@/composables/useI18n'

const { t } = useI18n()
const props = defineProps({
  visible: { type: Boolean, default: true },
})
const emit = defineEmits(['close'])

const sessionIdRef = ref('')
const { term, containerRef, init, destroy, refitAndFocus } = useXterm(sessionIdRef, {
  writeCmd: 'local_terminal_write',
  resizeCmd: 'local_terminal_resize',
  dataEvent: 'local-terminal-data',
})

function onKey(e) { if (e.key === 'Escape') emit('close') }

async function handleClose() {
  const sid = sessionIdRef.value
  if (sid) {
    try { await invoke('local_terminal_kill', { sessionId: sid }) } catch {}
  }
  destroy()
  emit('close')
}

watch(() => props.visible, async (v) => {
  if (v) await refitAndFocus()
})

onMounted(async () => {
  document.addEventListener('keydown', onKey)

  await init()

  const cols = term.value?.cols || 80
  const rows = term.value?.rows || 24

  try {
    const id = await invoke('local_terminal_spawn', { cols, rows })
    sessionIdRef.value = id
  } catch (e) {
    destroy()
  }
})

onBeforeUnmount(() => {
  document.removeEventListener('keydown', onKey)
})
</script>
<template>
  <div v-show="visible" class="absolute inset-0 z-50 bg-[var(--color-bg-primary)] flex flex-col">
    <div class="shrink-0 flex items-center justify-between px-4 h-10 bg-[var(--color-bg-secondary)] select-none">
      <div class="flex items-center gap-2">
        <span class="text-[12px] font-semibold text-[var(--color-text-primary)]">{{ t('activity.terminal') }}</span>
      </div>
      <button @click="handleClose"
        class="w-7 h-7 flex items-center justify-center rounded-md text-[var(--color-text-tertiary)] hover:text-[var(--color-text-primary)] hover:bg-[var(--color-bg-tertiary)] transition-colors">
        <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
      </button>
    </div>
    <div ref="containerRef" class="flex-1 min-h-0 p-2" />
  </div>
</template>
