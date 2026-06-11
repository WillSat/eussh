<script setup>
import { computed } from 'vue'
import { useI18n } from '@/composables/useI18n'
const { t } = useI18n()
const props = defineProps({ active: { type: String, default: 'servers' } })
const emit = defineEmits(['select'])
const items = computed(() => [
  { id: 'servers',  icon: 'servers',  label: t('activity.servers') },
  { id: 'batch',    icon: 'batch',    label: t('activity.batch') },
  { id: 'settings', icon: 'settings', label: t('activity.settings') },
])
</script>
<template>
  <div class="w-11 shrink-0 flex flex-col items-center gap-0.5 pt-2 pb-3 select-none bg-[var(--color-bg-secondary)] border-r border-[var(--color-border)]">
    <button v-for="item in items" :key="item.id" @click="emit('select', item.id)" :title="item.label"
      :class="['w-8 h-8 flex items-center justify-center rounded-lg transition-all',
        active === item.id ? 'text-[var(--color-accent)] bg-[var(--color-accent)]/10' : 'text-[var(--color-text-tertiary)] hover:text-[var(--color-text-secondary)] hover:bg-[var(--color-bg-tertiary)]']">
      <svg v-if="item.icon==='servers'" class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><rect x="2" y="2" width="20" height="8" rx="2"/><rect x="2" y="14" width="20" height="8" rx="2"/><line x1="6" y1="6" x2="6.01" y2="6"/><line x1="6" y1="18" x2="6.01" y2="18"/></svg>
      <svg v-else-if="item.icon==='batch'" class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><polyline points="4 17 10 11 4 5"/><line x1="12" y1="19" x2="20" y2="19"/></svg>
      <svg v-else class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 010 2.83 2 2 0 01-2.83 0l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-4 0v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83-2.83l.06-.06A1.65 1.65 0 004.68 15a1.65 1.65 0 00-1.51-1H3a2 2 0 010-4h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 012.83-2.83l.06.06A1.65 1.65 0 009 4.68a1.65 1.65 0 001-1.51V3a2 2 0 014 0v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 2.83l-.06.06A1.65 1.65 0 0019.4 9a1.65 1.65 0 001.51 1H21a2 2 0 010 4h-.09a1.65 1.65 0 00-1.51 1z"/></svg>
    </button>
    <div class="flex-1" />
  </div>
</template>
