<script setup>
import { watch } from 'vue'
import { useSettingsStore } from '@/stores/useSettingsStore'
import { useI18n } from '@/composables/useI18n'
import { useLogger } from '@/composables/useLogger'
import { COLOR_PRESETS } from '@/utils/theme'
import Toggle from '@/components/common/Toggle.vue'

const log = useLogger('SettingsPanel')
const { t, locale, setLocale, locales } = useI18n()

const props = defineProps({
  visible: { type: Boolean, default: false },
})
const emit = defineEmits(['close'])

const settings = useSettingsStore()
const presets = Object.entries(COLOR_PRESETS)
const accentColors = ['#007AFF','#34C759','#FF9500','#FF3B30','#AF52DE','#5856D6','#00C7BE','#FF2D55']

function selectPreset(key) { settings.terminalColorPreset = key; settings.save() }
function setLanguage(loc) { setLocale(loc); settings.language = loc; settings.save() }

watch(() => props.visible, (v) => {
  log.info('visible changed', v)
}, { immediate: true })

// ---- Reusable style helpers ----
function segBtnClass(active) {
  return active
    ? 'flex-1 py-1.5 text-xs font-medium rounded-md bg-[var(--color-bg-primary)] text-[var(--color-text-primary)] shadow-sm transition-all'
    : 'flex-1 py-1.5 text-xs font-medium rounded-md text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)] transition-all'
}

function inputClass() {
  return 'w-full px-3 py-1.5 text-xs rounded-lg border border-[var(--color-border)] bg-[var(--color-bg-secondary)] text-[var(--color-text-primary)] focus:outline-none focus:border-[var(--color-accent)] focus:ring-1 focus:ring-[var(--color-accent)]/20 transition-all'
}

function settingRowClass() {
  return 'flex items-center justify-between py-2.5'
}
</script>

<template>
  <Teleport to="body">
    <Transition name="panel">
      <div v-if="visible" class="fixed inset-0 z-50 flex justify-end">
        <div class="absolute inset-0 bg-black/20" @click="emit('close')" />
        <div class="relative w-[370px] h-full bg-[var(--color-bg-primary)] border-l border-[var(--color-border)]
          shadow-[var(--shadow-lg)] overflow-y-auto">

          <!-- Header -->
          <div class="sticky top-0 z-10 flex items-center justify-between px-5 py-4
            bg-[var(--color-bg-primary)]/95 backdrop-blur-sm border-b border-[var(--color-border)]">
            <h2 class="text-sm font-bold text-[var(--color-text-primary)] tracking-tight">
              &#x2699; {{ t('settings.title') }}
            </h2>
            <button @click="emit('close')"
              class="w-7 h-7 flex items-center justify-center rounded-lg
                text-[var(--color-text-tertiary)] hover:bg-[var(--color-bg-tertiary)] hover:text-[var(--color-text-primary)]
                transition-all text-sm">&#x2715;</button>
          </div>

          <div class="p-5 space-y-6">

            <!-- ===== LANGUAGE ===== -->
            <section>
              <h3 class="text-[10px] font-bold uppercase tracking-widest text-[var(--color-text-tertiary)] mb-3">
                &#x1F310; {{ t('settings.language') }}
              </h3>
              <div class="flex rounded-lg bg-[var(--color-bg-secondary)] p-0.5">
                <button v-for="loc in locales" :key="loc" @click="setLanguage(loc)"
                  :class="segBtnClass(locale === loc)"
                >{{ loc === 'zh-CN' ? '中文' : 'EN' }}</button>
              </div>
            </section>

            <!-- ===== APPEARANCE ===== -->
            <section>
              <h3 class="text-[10px] font-bold uppercase tracking-widest text-[var(--color-text-tertiary)] mb-3">
                &#x1F3A8; {{ t('settings.appearance') }}
              </h3>
              <div class="space-y-4">

                <!-- Theme -->
                <div>
                  <label class="text-[11px] font-medium text-[var(--color-text-secondary)] mb-1.5 block">
                    {{ t('settings.theme') }}
                  </label>
                  <div class="flex rounded-lg bg-[var(--color-bg-secondary)] p-0.5">
                    <button v-for="th in ['light','system','dark']" :key="th"
                      @click="settings.setTheme(th)"
                      :class="segBtnClass(settings.theme === th)"
                    >{{ t(`settings.${th}`) }}</button>
                  </div>
                </div>

                <!-- Accent color -->
                <div>
                  <label class="text-[11px] font-medium text-[var(--color-text-secondary)] mb-1.5 block">
                    {{ t('settings.accentColor') }}
                  </label>
                  <div class="flex gap-2">
                    <button v-for="c in accentColors" :key="c"
                      @click="settings.accentColor = c; settings.applyTheme(); settings.save()"
                      :class="['w-8 h-8 rounded-full transition-all hover:scale-110',
                        settings.accentColor === c
                          ? 'ring-2 ring-offset-2 ring-offset-[var(--color-bg-primary)] scale-110'
                          : '']"
                      :style="{ background: c, ringColor: c }" />
                  </div>
                </div>

                <!-- Title bar style -->
                <div>
                  <label class="text-[11px] font-medium text-[var(--color-text-secondary)] mb-1.5 block">
                    {{ t('settings.titlebarStyle') }}
                  </label>
                  <div class="flex rounded-lg bg-[var(--color-bg-secondary)] p-0.5">
                    <button @click="settings.setTitlebarStyle('macos')"
                      :class="segBtnClass(settings.titlebarStyle === 'macos')">
                      <span class="flex items-center justify-center gap-1">
                        <span class="flex gap-1">
                          <span class="w-2 h-2 rounded-full bg-[#FF5F57]"/><span class="w-2 h-2 rounded-full bg-[#FFBD2E]"/><span class="w-2 h-2 rounded-full bg-[#28C840]"/>
                        </span>
                        {{ t('settings.titlebarMacos') }}
                      </span>
                    </button>
                    <button @click="settings.setTitlebarStyle('win11')"
                      :class="segBtnClass(settings.titlebarStyle === 'win11')">
                      <span class="flex items-center justify-center gap-1">
                        <span class="text-[10px]">&#x2750;</span>
                        {{ t('settings.titlebarWin11') }}
                      </span>
                    </button>
                  </div>
                </div>

                <!-- Status bar style -->
                <div>
                  <label class="text-[11px] font-medium text-[var(--color-text-secondary)] mb-1.5 block">
                    {{ t('settings.statusbarStyle') }}
                  </label>
                  <div class="flex rounded-lg bg-[var(--color-bg-secondary)] p-0.5">
                    <button @click="settings.setStatusbarStyle('default')"
                      :class="segBtnClass(settings.statusbarStyle === 'default')"
                    >{{ t('settings.statusbarDefault') }}</button>
                    <button @click="settings.setStatusbarStyle('accent')"
                      :class="segBtnClass(settings.statusbarStyle === 'accent')"
                    >{{ t('settings.statusbarAccent') }}</button>
                  </div>
                </div>

                <!-- Terminal colors -->
                <div>
                  <label class="text-[11px] font-medium text-[var(--color-text-secondary)] mb-1.5 block">
                    {{ t('settings.terminalColors') }}
                  </label>
                  <div class="grid grid-cols-2 gap-2">
                    <button v-for="[key, p] in presets" :key="key" @click="selectPreset(key)"
                      :class="['flex items-center gap-2 p-2 rounded-lg transition-all text-left',
                        settings.terminalColorPreset === key
                          ? 'ring-2 ring-[var(--color-accent)] bg-[var(--color-bg-secondary)]'
                          : 'hover:bg-[var(--color-bg-secondary)]']">
                      <div class="w-8 h-5 rounded-sm border border-[var(--color-border)] overflow-hidden shrink-0"
                        :style="{ background: p.background }">
                        <div class="w-1.5 h-full ml-auto" :style="{ background: p.cursor }" />
                      </div>
                      <span class="text-[10px] text-[var(--color-text-secondary)] leading-tight">{{ p.label }}</span>
                    </button>
                  </div>
                </div>

              </div>
            </section>

            <!-- ===== TERMINAL ===== -->
            <section>
              <h3 class="text-[10px] font-bold uppercase tracking-widest text-[var(--color-text-tertiary)] mb-3">
                &#x1F4BB; {{ t('settings.terminal') }}
              </h3>
              <div class="space-y-3">

                <!-- Font family -->
                <div>
                  <label class="text-[11px] font-medium text-[var(--color-text-secondary)] mb-1 block">
                    {{ t('settings.fontFamily') }}
                  </label>
                  <input v-model="settings.fontFamily" @change="settings.save()" :class="inputClass()"
                    placeholder="Consolas, monospace" />
                </div>

                <!-- Font size -->
                <div :class="settingRowClass()">
                  <label class="text-[11px] font-medium text-[var(--color-text-secondary)]">
                    {{ t('settings.fontSize') }}
                  </label>
                  <div class="flex items-center gap-2.5 min-w-0">
                    <input type="range" min="10" max="24" :value="settings.fontSize"
                      @input="settings.fontSize = Number($event.target.value); settings.save()"
                      class="w-24 accent-[var(--color-accent)]" />
                    <span class="text-xs tabular-nums text-[var(--color-text-primary)] w-9 text-right font-mono">
                      {{ settings.fontSize }}px
                    </span>
                  </div>
                </div>

                <!-- Cursor style -->
                <div>
                  <label class="text-[11px] font-medium text-[var(--color-text-secondary)] mb-1.5 block">
                    {{ t('settings.cursor') }}
                  </label>
                  <div class="flex rounded-lg bg-[var(--color-bg-secondary)] p-0.5">
                    <button v-for="c in ['block','bar','underline']" :key="c"
                      @click="settings.cursorStyle = c; settings.save()"
                      :class="segBtnClass(settings.cursorStyle === c)"
                    >{{ t(`settings.${c}`) }}</button>
                  </div>
                </div>

              </div>
            </section>

            <!-- ===== MONITORING ===== -->
            <section>
              <h3 class="text-[10px] font-bold uppercase tracking-widest text-[var(--color-text-tertiary)] mb-3">
                &#x1F4CA; {{ t('settings.monitoring') }}
              </h3>
              <div class="space-y-2.5">

                <!-- System refresh -->
                <div :class="settingRowClass()">
                  <label class="text-[11px] font-medium text-[var(--color-text-secondary)]">
                    {{ t('settings.sysRefresh') }}
                  </label>
                  <div class="flex items-center gap-1.5">
                    <input type="number" min="1" max="10" :value="settings.monitorRefreshSecs"
                      @change="settings.monitorRefreshSecs = Number($event.target.value); settings.save()"
                      class="w-12 px-2 py-1 text-xs text-right rounded-lg border border-[var(--color-border)]
                        bg-[var(--color-bg-secondary)] text-[var(--color-text-primary)] font-mono
                        focus:outline-none focus:border-[var(--color-accent)] transition-all" />
                    <span class="text-[10px] text-[var(--color-text-tertiary)]">s</span>
                  </div>
                </div>

                <!-- Ping interval -->
                <div :class="settingRowClass()">
                  <label class="text-[11px] font-medium text-[var(--color-text-secondary)]">
                    {{ t('settings.pingInterval') }}
                  </label>
                  <div class="flex items-center gap-1.5">
                    <input type="number" min="5" max="120" :value="settings.pingIntervalSecs"
                      @change="settings.pingIntervalSecs = Number($event.target.value); settings.save()"
                      class="w-12 px-2 py-1 text-xs text-right rounded-lg border border-[var(--color-border)]
                        bg-[var(--color-bg-secondary)] text-[var(--color-text-primary)] font-mono
                        focus:outline-none focus:border-[var(--color-accent)] transition-all" />
                    <span class="text-[10px] text-[var(--color-text-tertiary)]">s</span>
                  </div>
                </div>

                <!-- Traffic monitor toggle -->
                <div :class="settingRowClass()">
                  <label class="text-[11px] font-medium text-[var(--color-text-secondary)]">
                    {{ t('settings.showTraffic') }}
                  </label>
                  <button role="switch" :aria-checked="settings.showTraffic"
                    @click="settings.showTraffic = !settings.showTraffic; settings.save()"
                    :class="['relative inline-flex h-5 w-9 shrink-0 rounded-full border-2 border-transparent transition-colors duration-200',
                      settings.showTraffic ? 'bg-[var(--color-accent)]' : 'bg-[var(--color-bg-tertiary)]']">
                    <span :class="['pointer-events-none inline-block h-4 w-4 rounded-full bg-white shadow transition-transform duration-200',
                      settings.showTraffic ? 'translate-x-4' : 'translate-x-0']" />
                  </button>
                </div>

              </div>
            </section>

          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.panel-enter-active,
.panel-leave-active {
  transition: opacity 0.2s ease;
}
.panel-enter-active > div:last-child,
.panel-leave-active > div:last-child {
  transition: transform 0.2s ease;
}
.panel-enter-from,
.panel-leave-to {
  opacity: 0;
}
.panel-enter-from > div:last-child {
  transform: translateX(100%);
}
.panel-leave-to > div:last-child {
  transform: translateX(100%);
}
</style>
