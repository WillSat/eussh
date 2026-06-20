<script setup>
import { watch } from 'vue'

const props = defineProps({
  message: { type: String, default: '' },
  type: { type: String, default: 'error' }, // 'error' | 'success' | 'info'
  visible: { type: Boolean, default: false },
  duration: { type: Number, default: 4000 },
})

const emit = defineEmits(['close'])

let timer = null

watch(() => props.visible, (v) => {
  if (v) {
    clearTimeout(timer)
    timer = setTimeout(() => emit('close'), props.duration)
  }
})

const bgMap = {
  error: 'bg-red-500/90',
  success: 'bg-green-500/90',
  info: 'bg-[var(--color-accent)]/90',
}
</script>

<template>
  <Teleport to="body">
    <Transition
      enter-active-class="transition-all duration-300 ease-out"
      enter-from-class="opacity-0 translate-y-2"
      enter-to-class="opacity-100 translate-y-0"
      leave-active-class="transition-all duration-200 ease-in"
      leave-from-class="opacity-100 translate-y-0"
      leave-to-class="opacity-0 translate-y-2"
    >
      <div
        v-if="visible"
        :class="[
          'fixed bottom-16 left-1/2 -translate-x-1/2 z-[100]',
          'px-4 py-2 rounded-[var(--radius-md)] shadow-[var(--shadow-lg)]',
          'text-white text-sm font-medium backdrop-blur-sm',
          bgMap[type] || bgMap.info,
        ]"
      >
        {{ message }}
      </div>
    </Transition>
  </Teleport>
</template>
