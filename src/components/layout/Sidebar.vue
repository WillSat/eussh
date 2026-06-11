<script setup>
import { ref, onBeforeUnmount } from 'vue'
import { useSettingsStore } from '@/stores/useSettingsStore'
import ServersView from './sidebar/ServersView.vue'
import BatchView from './sidebar/BatchView.vue'
import SettingsView from './sidebar/SettingsView.vue'

const props = defineProps({ view: { type: String, default: 'servers' } })
const emit = defineEmits(['navigate'])
const settings = useSettingsStore()
const MIN=160, MAX=480
const W = ref(settings.sidebarWidth || 260)
const dragging = ref(false)

function onDown(e) { dragging.value=true; document.addEventListener('mousemove',onMove); document.addEventListener('mouseup',onUp); e.preventDefault() }
function onMove(e) { if(!dragging.value) return; W.value=Math.min(MAX,Math.max(MIN,e.clientX-44)) }
function onUp() { dragging.value=false; settings.sidebarWidth=W.value; settings.save(); document.removeEventListener('mousemove',onMove); document.removeEventListener('mouseup',onUp) }
onBeforeUnmount(()=>{ document.removeEventListener('mousemove',onMove); document.removeEventListener('mouseup',onUp) })
</script>
<template>
  <div class="shrink-0 flex flex-col h-full select-none relative bg-[var(--color-bg-secondary)] border-r border-[var(--color-border)]" :style="{width:W+'px'}">
    <ServersView v-if="view==='servers'" :width="W" />
    <BatchView v-else-if="view==='batch'" :width="W" />
    <SettingsView v-else-if="view==='settings'" :width="W" />
    <div @mousedown="onDown" :class="['absolute right-0 top-0 bottom-0 w-1 cursor-col-resize z-20 transition-colors',dragging?'bg-[var(--color-accent)]':'bg-transparent hover:bg-[var(--color-accent)]/30']" />
  </div>
</template>
