<script setup>
import { ref, computed } from 'vue'
import { useServerStore } from '@/stores/useServerStore'
import { useI18n } from '@/composables/useI18n'
import { invoke } from '@/utils/ipc'

const { t } = useI18n()
const serverStore = useServerStore()
defineProps({ width: Number })

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
    <div class="px-3 pt-3 pb-2">
      <span class="text-[11px] font-bold uppercase tracking-widest text-[var(--color-text-tertiary)]">{{ t('batch.title') }}</span>
    </div>

    <!-- Targets -->
    <div class="px-3 pb-2">
      <div class="flex items-center justify-between mb-1.5">
        <span class="text-[10px] font-bold uppercase tracking-wider text-[var(--color-text-tertiary)]">{{ t('batch.targets') }}</span>
        <div class="flex items-center gap-2">
          <span class="text-[10px] tabular-nums text-[var(--color-accent)] font-medium">{{ checked.size }}/{{ connected.length }}</span>
          <button @click="all" class="text-[10px] text-[var(--color-accent)]/60 hover:text-[var(--color-accent)]">{{ t('batch.all') }}</button>
          <button @click="none" class="text-[10px] text-[var(--color-text-tertiary)]/60 hover:text-[var(--color-text-tertiary)]">{{ t('batch.none') }}</button>
        </div>
      </div>
      <div class="space-y-0.5 max-h-28 overflow-y-auto">
        <div v-if="connected.length===0" class="text-[11px] text-[var(--color-text-tertiary)]/40 py-2 px-1">{{ t('batch.noTargets') }}</div>
        <label v-for="srv in connected" :key="srv.id" class="flex items-center gap-2 px-2 py-1 rounded hover:bg-[var(--color-bg-tertiary)] cursor-pointer transition-colors">
          <input type="checkbox" :checked="checked.has(srv.id)" @change="checked.has(srv.id) ? checked.delete(srv.id) : checked.add(srv.id)" class="w-3.5 h-3.5 rounded accent-[var(--color-accent)] shrink-0" />
          <span class="text-[13px] text-[var(--color-text-primary)] truncate">{{ srv.nickname }}</span>
        </label>
      </div>
    </div>

    <!-- Command -->
    <div class="px-3 pb-2">
      <span class="text-[10px] font-bold uppercase tracking-wider text-[var(--color-text-tertiary)]">{{ t('batch.command') }}</span>
      <textarea v-model="cmd" rows="4" :placeholder="t('batch.placeholder')"
        class="w-full mt-1.5 px-2.5 py-2 text-[12px] font-mono rounded-lg border border-[var(--color-border)] bg-[var(--color-bg-primary)] text-[var(--color-text-primary)] resize-none focus:outline-none focus:border-[var(--color-accent)] focus:ring-1 focus:ring-[var(--color-accent)]/20 placeholder:text-[var(--color-text-tertiary)]/30 transition-all" />
      <div class="flex items-center justify-between mt-1.5">
        <span class="text-[10px] text-[var(--color-text-tertiary)]/40">{{ t('batch.hint') }}</span>
        <button @click="run" :disabled="running || checked.size===0 || !cmd.trim()"
          class="px-3 py-1.5 text-[11px] font-bold rounded-lg text-white transition-all bg-[var(--color-accent)] hover:brightness-110 disabled:opacity-30 disabled:cursor-not-allowed flex items-center gap-1">
          <svg v-if="running" class="w-3 h-3 animate-spin" viewBox="0 0 24 24" fill="none"><circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="3" stroke-dasharray="32" stroke-linecap="round"/></svg>
          {{ running ? t('batch.running') : t('batch.run') }}
        </button>
      </div>
    </div>

    <!-- Results -->
    <div v-if="results.length" class="flex-1 flex flex-col min-h-0 px-3 pb-2">
      <span class="text-[10px] font-bold uppercase tracking-wider text-[var(--color-text-tertiary)] mb-1.5">{{ t('batch.results') }} ({{ results.length }})</span>
      <div class="flex-1 overflow-y-auto space-y-1.5">
        <div v-for="r in results" :key="r.id"
          :class="['rounded-lg border overflow-hidden', r.error ? 'border-[var(--color-danger)]/25 bg-[var(--color-danger)]/3' : 'border-[var(--color-border)] bg-[var(--color-bg-primary)]']">
          <div class="flex items-center justify-between px-2.5 py-1.5 border-b border-inherit">
            <div class="flex items-center gap-1.5 min-w-0">
              <span :class="['w-1.5 h-1.5 rounded-full shrink-0', r.error ? 'bg-[var(--color-danger)]' : 'bg-[var(--color-success)]']" />
              <span class="text-[12px] font-semibold text-[var(--color-text-primary)] truncate">{{ r.nickname }}</span>
            </div>
            <div class="flex items-center gap-2 shrink-0 ml-2">
              <span class="text-[10px] tabular-nums text-[var(--color-text-tertiary)]">{{ r.elapsed_ms }}ms</span>
              <button @click="copyOutput(r)"
                :class="['w-5 h-5 flex items-center justify-center rounded transition-colors',
                  copiedServer===r.id ? 'text-[var(--color-success)] bg-[var(--color-success)]/10' : 'text-[var(--color-text-tertiary)] hover:text-[var(--color-accent)] hover:bg-[var(--color-bg-tertiary)]']"
                :title="copiedServer===r.id ? '✓' : t('batch.copyOutput')">
                <svg v-if="copiedServer===r.id" class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round"><polyline points="20 6 9 17 4 12"/></svg>
                <svg v-else class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><rect x="9" y="9" width="13" height="13" rx="2"/><path d="M5 15H4a2 2 0 01-2-2V4a2 2 0 012-2h9a2 2 0 012 2v1"/></svg>
              </button>
            </div>
          </div>
          <pre :class="['px-2.5 py-2 text-[11px] font-mono whitespace-pre-wrap break-all max-h-40 overflow-y-auto', r.error ? 'text-[var(--color-danger)]' : 'text-[var(--color-text-primary)]']">{{ r.error || r.output || t('batch.noOutput') }}</pre>
        </div>
      </div>
    </div>
    <div v-else-if="!running" class="flex-1 flex items-center justify-center">
      <p class="text-[11px] text-[var(--color-text-tertiary)]/40">{{ t('batch.emptyHint') }}</p>
    </div>
  </div>
</template>
