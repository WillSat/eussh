<script setup>
const props = defineProps({
  files: Array,
  selections: Array,
})
const emit = defineEmits(['select', 'doubleClick', 'contextMenu'])

function isSelected(name) {
  return (props.selections || []).includes(name)
}
</script>

<template>
  <div class="grid p-3 gap-3" style="grid-template-columns: repeat(auto-fill, minmax(90px, 1fr));">
    <div v-for="f in files" :key="f.name"
      :class="[
        'flex flex-col items-center p-2 rounded-lg cursor-pointer select-none transition-colors',
        isSelected(f.name) ? 'bg-accent-dim' : 'hover:bg-[var(--color-bg-tertiary)]',
      ]"
      @click="emit('select', f, $event)"
      @dblclick="emit('doubleClick', f)"
      @contextmenu.prevent.stop="emit('contextMenu', f, $event)">
      <span class="text-3xl mb-1" :class="f.is_dir ? 'text-[var(--color-accent)]' : ''">{{ f.is_dir ? '&#x1F4C1;' : '&#x1F4C4;' }}</span>
      <span class="text-[11px] text-center break-all w-full line-clamp-2"
        :class="f.name.startsWith('.') ? 'text-[var(--color-text-tertiary)]' : 'text-[var(--color-text-primary)]'"
      >{{ f.name }}</span>
    </div>
  </div>
</template>
