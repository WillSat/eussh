<script setup>
defineProps({
  tooltip: { type: String, default: '' },
  size: { type: String, default: 'md' },
})

defineEmits(['click'])
</script>

<template>
  <button
    @click="$emit('click')"
    :title="tooltip"
    :class="[
      'inline-flex items-center justify-center rounded-md',
      'text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]',
      'hover:bg-[var(--color-bg-tertiary)] transition-colors',
      'focus:outline-none',
      size === 'sm' ? 'w-7 h-7 text-xs' : 'w-8 h-8 text-sm',
    ]"
  >
    <slot />
  </button>
</template>
