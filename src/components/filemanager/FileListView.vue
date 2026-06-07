<script setup>
import { useI18n } from '@/composables/useI18n'
const { t } = useI18n()

const props = defineProps({
  files: Array,
  selections: Array,
})
const emit = defineEmits(['select', 'doubleClick', 'contextMenu'])

function isSelected(name) {
  return (props.selections || []).includes(name)
}

function formatSize(bytes) {
  if (!bytes) return '--'
  if (bytes < 1024) return bytes + ' B'
  if (bytes < 1048576) return (bytes / 1024).toFixed(1) + ' KB'
  if (bytes < 1073741824) return (bytes / 1048576).toFixed(1) + ' MB'
  return (bytes / 1073741824).toFixed(2) + ' GB'
}

function formatDate(ts) {
  if (!ts || ts === 0) return '--'
  try {
    const d = new Date(ts * 1000)
    if (isNaN(d.getTime())) return '--'
    return d.toLocaleDateString([], { year: 'numeric', month: 'short', day: 'numeric' })
      + ' ' + d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
  } catch { return '--' }
}

function rowClass(name) {
  return isSelected(name)
    ? 'bg-accent-dim hover-bg-accent-dim'
    : 'hover:bg-[var(--color-bg-tertiary)]'
}
</script>

<template>
  <table class="w-full text-sm select-none">
    <thead>
      <tr class="border-b border-[var(--color-border)] text-[11px] text-[var(--color-text-tertiary)] uppercase tracking-wider">
        <th class="text-left py-1.5 px-3 font-medium">{{ t('filemanager.name') }}</th>
        <th class="text-right py-1.5 px-3 font-medium w-24">{{ t('filemanager.size') }}</th>
        <th class="text-right py-1.5 px-3 font-medium w-40">{{ t('filemanager.modified') }}</th>
      </tr>
    </thead>
    <tbody>
      <tr v-for="f in files" :key="f.name"
        :class="['cursor-pointer select-none transition-colors', rowClass(f.name)]"
        @click="emit('select', f, $event)"
        @dblclick="emit('doubleClick', f)"
        @contextmenu.prevent.stop="emit('contextMenu', f, $event)">
        <td class="py-1 px-3 flex items-center gap-2">
          <span class="text-base shrink-0" :class="f.is_dir ? 'text-[var(--color-accent)]' : ''">{{ f.is_dir ? '&#x1F4C1;' : '&#x1F4C4;' }}</span>
          <span class="truncate" :class="f.name.startsWith('.') ? 'opacity-70' : ''">{{ f.name }}</span>
        </td>
        <td class="py-1 px-3 text-right text-xs text-[var(--color-text-tertiary)] tabular-nums">{{ f.is_dir ? '--' : formatSize(f.size) }}</td>
        <td class="py-1 px-3 text-right text-xs text-[var(--color-text-tertiary)]">{{ formatDate(f.modified) }}</td>
      </tr>
    </tbody>
  </table>
</template>
