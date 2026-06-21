<script setup>
import { ref } from 'vue'
import { invoke } from '@/utils/ipc'
import { useI18n } from '@/composables/useI18n'

const { t } = useI18n()

const state = ref({
  visible: false,
  requestId: '',
  host: '',
  fingerprint: '',
  isKeyChanged: false,
})

function show(data) {
  state.value = { visible: true, ...data }
}

function dismiss() {
  state.value.visible = false
}

async function respond(accepted, remember) {
  try {
    await invoke('confirm_host_key', {
      requestId: state.value.requestId,
      accepted,
      remember,
    })
  } catch {
    // If the verification request is already gone, ignore
  }
  dismiss()
}

defineExpose({ show })
</script>

<template>
  <Teleport to="body">
    <div
      v-if="state.visible"
      class="fixed inset-0 z-[80] flex items-center justify-center"
      @click.self="respond(false, false)"
    >
      <!-- Overlay -->
      <div class="absolute inset-0 bg-black/40 backdrop-blur-sm" />

      <!-- Dialog -->
      <div
        class="relative w-[420px] max-w-[92vw] bg-[var(--color-bg-primary)] rounded-xl
          shadow-[var(--shadow-lg)]
          overflow-hidden select-none"
        @click.stop
      >
        <!-- Header -->
        <div
          :class="[
            'px-5 py-4 flex items-center gap-3',
            state.isKeyChanged
              ? 'bg-[#FF3B30]/10'
              : 'bg-[var(--color-bg-secondary)]'
          ]"
        >
          <svg
            v-if="state.isKeyChanged"
            class="w-6 h-6 text-[#FF3B30] shrink-0"
            viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"
            stroke-linecap="round"
          >
            <path d="M10.29 3.86L1.82 18a2 2 0 001.71 3h16.94a2 2 0 001.71-3L13.71 3.86a2 2 0 00-3.42 0z"/>
            <line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/>
          </svg>
          <svg
            v-else
            class="w-6 h-6 text-[var(--color-accent)] shrink-0"
            viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"
            stroke-linecap="round"
          >
            <rect x="3" y="11" width="18" height="11" rx="2"/>
            <path d="M7 11V7a5 5 0 0110 0v4"/>
          </svg>
          <h2 class="text-sm font-bold text-[var(--color-text-primary)]">
            {{ state.isKeyChanged ? t('hostKey.titleChanged') : t('hostKey.title') }}
          </h2>
        </div>

        <!-- Body -->
        <div class="px-5 py-4 space-y-4">
          <p
            class="text-[13px] leading-relaxed text-[var(--color-text-secondary)]"
            v-html="
              state.isKeyChanged
                ? t('hostKey.changed', { host: state.host })
                : t('hostKey.unknown', { host: state.host })
            "
          />

          <div
            :class="[
              'rounded-lg p-3 space-y-1 shadow-[var(--shadow-sm)]',
              state.isKeyChanged
                ? 'bg-[#FF3B30]/5'
                : 'bg-[var(--color-bg-secondary)]'
            ]"
          >
            <span class="text-[10px] font-bold uppercase tracking-wider text-[var(--color-text-tertiary)]">
              {{ t('hostKey.changedDetail') }}
            </span>
            <p class="text-xs font-mono break-all text-[var(--color-text-primary)] leading-relaxed">
              {{ state.fingerprint }}
            </p>
          </div>
        </div>

        <!-- Footer -->
        <div class="px-5 py-3 bg-[var(--color-bg-secondary)]
          flex items-center gap-2 justify-end">
          <button
            @click="respond(false, false)"
            class="px-4 py-1.5 text-xs font-medium rounded-lg
              text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]
              hover:bg-[var(--color-bg-tertiary)] transition-all"
          >
            {{ t('hostKey.cancel') }}
          </button>
          <button
            @click="respond(true, false)"
            class="px-4 py-1.5 text-xs font-medium rounded-lg
              bg-[var(--color-bg-tertiary)]
              text-[var(--color-text-primary)]
              hover:brightness-105 transition-all"
          >
            {{ t('hostKey.acceptOnce') }}
          </button>
          <button
            @click="respond(true, true)"
            class="px-4 py-1.5 text-xs font-bold rounded-lg
              text-white bg-[var(--color-accent)]
              hover:brightness-110 transition-all"
          >
            {{ t('hostKey.acceptAlways') }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
