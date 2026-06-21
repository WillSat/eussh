<script setup>
import { ref, watch } from 'vue'

const props = defineProps({
  visible: { type: Boolean, default: false },
  title: { type: String, default: '' },
})

const emit = defineEmits(['close'])

const pointerDownOnBackdrop = ref(false)

function onKeydown(e) {
  if (e.key === 'Escape') emit('close')
}

function onPointerDown(e) {
  // Track whether pointerdown started on the backdrop itself (not on the panel)
  pointerDownOnBackdrop.value = (e.target === e.currentTarget)
}

function onClickBackdrop(e) {
  // Only close if both pointerdown and click happened on the backdrop
  // This prevents closing when the user starts a drag inside the panel
  // and releases outside (e.g., text selection in an input field)
  if (pointerDownOnBackdrop.value && e.target === e.currentTarget) {
    emit('close')
  }
  pointerDownOnBackdrop.value = false
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
      @pointerdown="onPointerDown"
      @click="onClickBackdrop"
    >
      <!-- Backdrop -->
      <div class="absolute inset-0 bg-black/30 backdrop-blur-sm" />

      <!-- Panel -->
      <div class="relative bg-[var(--color-bg-primary)] rounded-[var(--radius-lg)]
        shadow-[var(--shadow-lg)]
        w-full max-w-md mx-4 overflow-hidden">
        <div class="flex items-center justify-between px-5 py-4 bg-[var(--color-bg-secondary)]">
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
