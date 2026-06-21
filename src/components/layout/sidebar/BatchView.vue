<script setup>
import { ref, computed } from 'vue'
import { useServerStore } from '@/stores/useServerStore'
import { useI18n } from '@/composables/useI18n'
import { invoke } from '@/utils/ipc'

const { t } = useI18n()
const serverStore = useServerStore()

const connected = computed(() => serverStore.servers.filter(s => s.tabs.some(t => t.status === 'connected')))
const checked = ref(new Set())
function all() { checked.value = new Set(connected.value.map(s => s.id)) }
function none() { checked.value.clear() }

const cmd = ref('')
const running = ref(false)
const results = ref([])
const copiedServer = ref(null)

async function run() {
  if (checked.value.size === 0 || !cmd.value.trim()) return
  running.value = true; results.value = []
  const ids = []; const meta = []
  for (const s of connected.value) {
    if (!checked.value.has(s.id)) continue
    const ov = s.tabs.find(t => t.type === 'overview')
    if (!ov?.sessionId) continue
    ids.push(ov.sessionId); meta.push({ id: s.id, nickname: s.nickname, host: s.host })
  }
  try {
    const all = await invoke('batch_exec', { sessionIds: ids, command: cmd.value })
    results.value = all.map((r, i) => ({ ...meta[i], output: r.output || '', error: r.error || '', elapsed_ms: r.elapsed_ms }))
  } finally { running.value = false }
}

async function copyOutput(r) {
  const text = r.error || r.output
  try { await invoke('clipboard_write', { text }) } catch {}
  copiedServer.value = r.id
  setTimeout(() => { copiedServer.value = null }, 1500)
}
</script>
<template>
  <div class="flex-1 flex flex-col min-h-0 overflow-y-auto">
    <!-- Header -->
    <div class="shrink-0 px-3 pt-3 pb-2">
      <span class="text-[11px] font-bold uppercase tracking-widest text-[var(--color-accent)]">{{ t('batch.title') }}</span>
    </div>

    <!-- Targets -->
    <div class="px-3 pt-2.5 pb-2">
      <div class="flex items-center justify-between mb-2">
        <span class="text-[10px] font-bold uppercase tracking-wider text-[var(--color-text-tertiary)]">{{ t('batch.targets') }}</span>
        <div class="flex items-center gap-2">
          <span class="text-[10px] tabular-nums font-medium px-1.5 py-0.5 rounded-md bg-[var(--color-accent)]/10 text-[var(--color-accent)]">{{ checked.size }}/{{ connected.length }}</span>
          <button @click="all" class="text-[10px] font-medium text-[var(--color-accent)]/70 hover:text-[var(--color-accent)] transition-colors">{{ t('batch.all') }}</button>
          <button @click="none" class="text-[10px] font-medium text-[var(--color-text-tertiary)] hover:text-[var(--color-text-secondary)] transition-colors">{{ t('batch.none') }}</button>
        </div>
      </div>
      <div class="space-y-0.5 max-h-28 overflow-y-auto">
        <div v-if="connected.length === 0" class="text-[11px] text-[var(--color-text-secondary)] py-3 px-1 flex items-center gap-2">
          <svg class="w-3.5 h-3.5 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/></svg>
          {{ t('batch.noTargets') }}
        </div>
        <label v-for="srv in connected" :key="srv.id"
          class="flex items-center gap-2.5 px-2 py-1.5 rounded-lg hover:bg-[var(--color-bg-tertiary)]/50 cursor-pointer transition-colors group">
          <span class="relative flex items-center justify-center shrink-0">
            <input type="checkbox" :checked="checked.has(srv.id)" @change="checked.has(srv.id) ? checked.delete(srv.id) : checked.add(srv.id)"
              class="peer w-3.5 h-3.5 rounded bg-[var(--color-bg-tertiary)] checked:bg-[var(--color-accent)] transition-colors cursor-pointer" />
            <svg class="absolute w-2.5 h-2.5 text-white opacity-0 peer-checked:opacity-100 pointer-events-none" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="4" stroke-linecap="round"><polyline points="20 6 9 17 4 12"/></svg>
          </span>
          <span class="text-[12px] text-[var(--color-text-primary)] truncate group-hover:text-[var(--color-accent)] transition-colors">{{ srv.nickname }}</span>
        </label>
      </div>
    </div>

    <!-- Command -->
    <div class="px-3 pb-2.5">
      <span class="text-[10px] font-bold uppercase tracking-wider text-[var(--color-text-tertiary)]">{{ t('batch.command') }}</span>
      <textarea v-model="cmd" rows="4" :placeholder="t('batch.placeholder')"
        class="w-full mt-1.5 px-2.5 py-2 text-[12px] font-mono rounded-lg bg-[var(--color-bg-tertiary)] text-[var(--color-text-primary)] resize-none focus:outline-none focus:ring-1 focus:ring-[var(--color-accent)]/20 placeholder:text-[var(--color-text-tertiary)]/50 transition-all" />
      <div class="flex items-center justify-between mt-1.5">
        <span class="text-[10px] text-[var(--color-text-tertiary)]/70">{{ t('batch.hint') }}</span>
        <button @click="run" :disabled="running || checked.size === 0 || !cmd.trim()"
          class="px-3.5 py-1.5 text-[11px] font-semibold rounded-lg text-white transition-all bg-[var(--color-accent)] hover:brightness-110 active:scale-95 disabled:opacity-30 disabled:cursor-not-allowed flex items-center gap-1.5 shadow-sm shadow-[var(--color-accent)]/20">
          <svg v-if="running" class="w-3 h-3 animate-spin" viewBox="0 0 24 24" fill="none"><circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="3" stroke-dasharray="32" stroke-linecap="round"/></svg>
          <svg v-else class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round"><polygon points="5 3 19 12 5 21 5 3"/></svg>
          {{ running ? t('batch.running') : t('batch.run') }}
        </button>
      </div>
    </div>

    <!-- Results -->
    <div v-if="results.length" class="flex-1 flex flex-col min-h-0 px-3 pb-3">
      <span class="text-[10px] font-bold uppercase tracking-wider text-[var(--color-text-tertiary)] mb-2 flex items-center gap-1.5">
        {{ t('batch.results') }}
        <span class="px-1.5 py-0.5 rounded-md text-[9px] font-medium bg-[var(--color-accent)]/10 text-[var(--color-accent)]">{{ results.length }}</span>
      </span>
      <div class="flex-1 overflow-y-auto space-y-1.5">
        <div v-for="r in results" :key="r.id"
          :class="['rounded-lg overflow-hidden shadow-[var(--shadow-sm)]',
            r.error
              ? 'bg-[color-mix(in_srgb,#FF3B30_4%,transparent)] ring-1 ring-[#FF3B30]/25'
              : 'bg-[var(--color-bg-primary)]']">
          <!-- Card header -->
          <div class="flex items-center justify-between px-2.5 py-1.5">
            <div class="flex items-center gap-1.5 min-w-0">
              <span :class="['w-1.5 h-1.5 rounded-full shrink-0', r.error ? 'bg-[#FF3B30]' : 'bg-[#34C759]']" />
              <span :class="['text-[12px] font-semibold truncate', r.error ? 'text-[var(--color-danger)]' : 'text-[var(--color-text-primary)]']">{{ r.nickname }}</span>
            </div>
            <div class="flex items-center gap-2 shrink-0 ml-2">
              <span class="text-[10px] tabular-nums text-[var(--color-text-tertiary)]/70">{{ r.elapsed_ms }}ms</span>
              <button @click="copyOutput(r)"
                :class="['w-5 h-5 flex items-center justify-center rounded transition-all duration-200',
                  copiedServer === r.id
                    ? 'text-[var(--color-accent)] bg-[var(--color-accent)]/10 scale-110'
                    : 'text-[var(--color-text-tertiary)]/70 hover:text-[var(--color-accent)] hover:bg-[var(--color-accent)]/10']"
                :title="copiedServer === r.id ? 'Copied' : t('batch.copyOutput')">
                <svg v-if="copiedServer === r.id" class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round"><polyline points="20 6 9 17 4 12"/></svg>
                <svg v-else class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><rect x="9" y="9" width="13" height="13" rx="2"/><path d="M5 15H4a2 2 0 01-2-2V4a2 2 0 012-2h9a2 2 0 012 2v1"/></svg>
              </button>
            </div>
          </div>
          <!-- Card body -->
          <pre :class="['px-2.5 py-2 text-[11px] font-mono whitespace-pre-wrap break-all max-h-40 overflow-y-auto', r.error ? 'text-[var(--color-danger)]/90' : 'text-[var(--color-text-primary)]']">{{ r.error || r.output || t('batch.noOutput') }}</pre>
        </div>
      </div>
    </div>

    <!-- Empty results hint -->
    <div v-else-if="!running" class="flex-1 flex items-center justify-center pb-4">
      <div class="flex flex-col items-center gap-2 py-8">
        <svg class="w-7 h-7 text-[var(--color-text-tertiary)]/50" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><polyline points="4 17 10 11 4 5"/><line x1="12" y1="19" x2="20" y2="19"/></svg>
        <p class="text-[11px] text-[var(--color-text-tertiary)]/70">{{ t('batch.emptyHint') }}</p>
      </div>
    </div>
  </div>
</template>
