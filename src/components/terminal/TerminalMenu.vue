<script setup>
import { ref } from 'vue'

const props = defineProps({
  visible: { type: Boolean, default: false },
  x: { type: Number, default: 0 },
  y: { type: Number, default: 0 },
})

const emit = defineEmits(['close', 'copy', 'paste'])

function onCopy() {
  emit('copy')
  emit('close')
}

function onPaste() {
  emit('paste')
  emit('close')
}
</script>

<template>
  <Teleport to="body">
    <div v-if="visible" class="fixed inset-0 z-[60]" @click="emit('close')" @contextmenu.prevent="emit('close')">
      <div
        class="fixed rounded-[var(--radius-md)] bg-[var(--color-bg-secondary)] border border-[var(--color-border)]
          shadow-[var(--shadow-lg)] py-1 min-w-[120px] z-[61]"
        :style="{ left: x + 'px', top: y + 'px' }"
      >
        <button
          @click="onCopy"
          class="w-full text-left px-3 py-1.5 text-xs text-[var(--color-text-primary)]
            hover:bg-[var(--color-bg-tertiary)] transition-colors flex items-center gap-2"
        >&#x2398; Copy</button>
        <button
          @click="onPaste"
          class="w-full text-left px-3 py-1.5 text-xs text-[var(--color-text-primary)]
            hover:bg-[var(--color-bg-tertiary)] transition-colors flex items-center gap-2"
        >&#x2384; Paste</button>
      </div>
    </div>
  </Teleport>
</template>
