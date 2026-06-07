<script setup>
defineProps({
  modelValue: { type: Boolean, default: false },
  label: { type: String, default: '' },
})

defineEmits(['update:modelValue'])
</script>

<template>
  <label class="inline-flex items-center gap-2 cursor-pointer">
    <span class="text-sm text-[var(--color-text-secondary)]">{{ label }}</span>
    <button
      role="switch"
      :aria-checked="modelValue"
      @click="$emit('update:modelValue', !modelValue)"
      :class="[
        'relative inline-flex h-5 w-9 shrink-0 rounded-full border-2 border-transparent',
        'transition-colors duration-200 ease-in-out',
        'focus:outline-none',
        modelValue ? 'bg-[var(--color-accent)]' : 'bg-[var(--color-bg-tertiary)]',
      ]"
    >
      <span
        :class="[
          'pointer-events-none inline-block h-4 w-4 rounded-full bg-white shadow',
          'transform transition duration-200 ease-in-out',
          modelValue ? 'translate-x-4' : 'translate-x-0',
        ]"
      />
    </button>
  </label>
</template>
