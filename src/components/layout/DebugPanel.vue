<script setup>
import { useLogger } from '@/composables/useLogger'

const { logState } = useLogger('DebugPanel')

const levelColors = {
  error: 'var(--color-danger)',
  warn: 'var(--color-warning)',
  info: 'var(--color-accent)',
  debug: 'var(--color-text-tertiary)',
}
</script>

<template>
  <div
    v-if="logState.showPanel"
    class="fixed bottom-0 left-0 right-0 z-[100] bg-[var(--color-bg-primary)] shadow-[0_-1px_3px_rgba(0,0,0,0.06)] shadow-[var(--shadow-lg)] max-h-64 overflow-y-auto font-mono text-[11px]"
  >
    <div class="flex items-center justify-between px-3 py-1.5 bg-[var(--color-bg-tertiary)] sticky top-0">
      <span class="text-xs font-semibold text-[var(--color-text-primary)]">Debug Log</span>
      <button
        @click="logState.showPanel = false"
        class="text-[var(--color-text-tertiary)] hover:text-[var(--color-text-primary)] text-xs"
      >Close</button>
    </div>
    <div class="p-1 space-y-0.5">
      <div
        v-for="entry in [...logState.entries].reverse()"
        :key="entry.id"
        class="flex gap-2 px-1 py-0.5 rounded hover:bg-[var(--color-bg-secondary)]"
      >
        <span class="text-[var(--color-text-tertiary)] shrink-0">{{ entry.timestamp }}</span>
        <span :style="{ color: levelColors[entry.level] || 'inherit' }" class="shrink-0 w-9">{{ entry.level }}</span>
        <span class="text-[var(--color-text-tertiary)] shrink-0 max-w-24 truncate">{{ entry.source }}</span>
        <span class="text-[var(--color-text-primary)] break-all">{{ entry.message }}</span>
        <span v-if="entry.data" class="text-[var(--color-text-tertiary)] break-all">{{ entry.data }}</span>
      </div>
    </div>
  </div>
</template>
