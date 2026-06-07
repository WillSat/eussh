<script setup>
import { watch } from 'vue'

const props = defineProps({
  visible: { type: Boolean, default: false },
  title: { type: String, default: '' },
})

const emit = defineEmits(['close'])

function onKeydown(e) {
  if (e.key === 'Escape') emit('close')
}

watch(() => props.visible, (v) => {
  if (v) {
    document.addEventListener('keydown', onKeydown)
  } else {
    document.removeEventListener('keydown', onKeydown)
  }
})
</script>

<template>
  <Teleport to="body">
    <div
      v-if="visible"
      class="fixed inset-0 z-50 flex items-center justify-center"
      @click.self="emit('close')"
    >
      <!-- Backdrop -->
      <div class="absolute inset-0 bg-black/30 backdrop-blur-sm" />

      <!-- Panel -->
      <div class="relative bg-[var(--color-bg-primary)] rounded-[var(--radius-lg)]
        shadow-[var(--shadow-lg)] border border-[var(--color-border)]
        w-full max-w-md mx-4 overflow-hidden">
        <div class="flex items-center justify-between px-5 py-4 border-b border-[var(--color-border)]">
          <h2 class="text-sm font-semibold text-[var(--color-text-primary)]">{{ title }}</h2>
          <button
            @click="emit('close')"
            class="w-5 h-5 flex items-center justify-center rounded-full
              text-[var(--color-text-tertiary)] hover:bg-[var(--color-bg-tertiary)]
              transition-colors text-xs"
          >&#x2715;</button>
        </div>
        <div class="p-5">
          <slot />
        </div>
      </div>
    </div>
  </Teleport>
</template>
