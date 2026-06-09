<script setup>
import { ref, onMounted, onBeforeUnmount, toRef, watch } from 'vue'
import { useXterm } from '@/composables/useXterm'
import { invoke } from '@/utils/ipc'
import TerminalMenu from './TerminalMenu.vue'

const props = defineProps({
  sessionId: { type: String, required: true },
  isActive: { type: Boolean, default: false },
})

const sessionIdRef = toRef(props, 'sessionId')
const { term, containerRef, init, refitAndFocus, getSelection } = useXterm(sessionIdRef)

const menuVisible = ref(false)
const menuX = ref(0)
const menuY = ref(0)

// Auto-focus when this tab becomes active (e.g., user clicks tab in MainTabBar)
watch(() => props.isActive, async (active) => {
  if (active) {
    await refitAndFocus()
  }
})

function onContextMenu(e) {
  e.preventDefault()
  menuX.value = e.clientX
  menuY.value = e.clientY
  menuVisible.value = true
}

async function handleCopy() {
  const text = getSelection()
  if (text) {
    try { await invoke('clipboard_write', { text }) } catch {}
  }
}

async function handlePaste() {
  try {
    const text = await invoke('clipboard_read')
    if (text) {
      const encoder = new TextEncoder()
      await invoke('terminal_write', {
        sessionId: props.sessionId,
        data: Array.from(encoder.encode(text)),
      })
    }
  } catch {}
  // Refocus the terminal so the user can continue typing immediately
  term.value?.focus()
}

onMounted(async () => {
  await init()
  containerRef.value?.addEventListener('contextmenu', onContextMenu)
})

onBeforeUnmount(() => {
  containerRef.value?.removeEventListener('contextmenu', onContextMenu)
})
</script>

<template>
  <div ref="containerRef" class="terminal-container h-full w-full p-2" />
  <TerminalMenu
    :visible="menuVisible"
    :x="menuX"
    :y="menuY"
    @close="menuVisible = false"
    @copy="handleCopy"
    @paste="handlePaste"
  />
</template>
