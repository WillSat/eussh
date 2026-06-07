<script setup>
import { ref, watch, onMounted } from 'vue'

const props = defineProps({
  currentDir: { type: String, required: true },
  canGoBack: Boolean,
  canGoForward: Boolean,
})
const emit = defineEmits(['navigate', 'back', 'forward'])

const pathInput = ref(props.currentDir)

watch(() => props.currentDir, (v) => { pathInput.value = v })

function onEnter() {
  let p = pathInput.value.trim()
  if (!p) return
  if (!p.startsWith('/')) p = '/' + p
  // Normalize double slashes
  p = p.replace(/\/+/g, '/')
  emit('navigate', p)
}

function onFocus() {
  pathInput.value = props.currentDir
}
</script>

<template>
  <div class="flex items-center gap-1 h-8 px-2 bg-[var(--color-bg-secondary)] border-b border-[var(--color-border)]">
    <button @click="emit('back')" :disabled="!canGoBack"
      :class="['px-1.5 py-0.5 rounded text-sm', canGoBack ? 'text-[var(--color-text-secondary)] hover:bg-[var(--color-bg-tertiary)] hover:text-[var(--color-text-primary)]' : 'text-[var(--color-text-tertiary)] opacity-40']"
    >&#x2039;</button>
    <button @click="emit('forward')" :disabled="!canGoForward"
      :class="['px-1.5 py-0.5 rounded text-sm', canGoForward ? 'text-[var(--color-text-secondary)] hover:bg-[var(--color-bg-tertiary)] hover:text-[var(--color-text-primary)]' : 'text-[var(--color-text-tertiary)] opacity-40']"
    >&#x203A;</button>
    <input
      v-model="pathInput"
      @keyup.enter="onEnter"
      @focus="onFocus"
      class="flex-1 h-6 px-2 text-[12px] rounded-[var(--radius-sm)]
        bg-[var(--color-bg-primary)] text-[var(--color-text-primary)] font-mono
        focus:outline-none focus:ring-1 focus:ring-[var(--color-accent)]"
      spellcheck="false"
    />
  </div>
</template>
