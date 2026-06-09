<script setup>
import { watch } from 'vue'
import { useSettingsStore } from '@/stores/useSettingsStore'
import { useI18n } from '@/composables/useI18n'
import { useLogger } from '@/composables/useLogger'
import { COLOR_PRESETS } from '@/utils/theme'

const log = useLogger('SettingsPanel')
const { t, locale, setLocale, locales } = useI18n()

const props = defineProps({
  visible: { type: Boolean, default: false },
})
const emit = defineEmits(['close'])

const settings = useSettingsStore()
const presets = Object.entries(COLOR_PRESETS)

function selectPreset(key) {
  settings.terminalColorPreset = key
  settings.save()
}

function setLanguage(loc) {
  setLocale(loc)
  settings.language = loc
  settings.save()
}

watch(() => props.visible, (v) => {
  log.info('visible changed', v)
}, { immediate: true })
</script>

<template>
  <Teleport to="body">
    <Transition name="panel">
      <div v-if="visible" class="fixed inset-0 z-50 flex justify-end">
        <div class="absolute inset-0 bg-black/20" @click="emit('close')" />
        <div class="relative w-80 h-full bg-[var(--color-bg-primary)] border-l border-[var(--color-border)]
          shadow-[var(--shadow-lg)] overflow-y-auto">
          <div class="flex items-center justify-between px-4 py-3 border-b border-[var(--color-border)]">
            <h2 class="text-sm font-semibold text-[var(--color-text-primary)]">{{ t('settings.title') }}</h2>
            <button @click="emit('close')"
              class="w-6 h-6 flex items-center justify-center rounded-md
                text-[var(--color-text-secondary)] hover:bg-[var(--color-bg-tertiary)] transition-colors">&#x2715;</button>
          </div>

          <div class="p-4 space-y-6">
            <!-- Language -->
            <section>
              <h3 class="text-[10px] font-semibold uppercase tracking-wider text-[var(--color-text-tertiary)] mb-3">{{ t('settings.language') }}</h3>
              <div class="flex rounded-[var(--radius-sm)] bg-[var(--color-bg-secondary)] p-0.5">
                <button v-for="loc in locales" :key="loc"
                  @click="setLanguage(loc)"
                  :class="['flex-1 py-1 text-xs font-medium rounded transition-colors',
                    locale === loc
                      ? 'bg-[var(--color-bg-primary)] text-[var(--color-text-primary)] shadow-[var(--shadow-sm)]'
                      : 'text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]']"
                >{{ loc === 'zh-CN' ? '中文' : 'EN' }}</button>
              </div>
            </section>

            <!-- Appearance -->
            <section>
              <h3 class="text-[10px] font-semibold uppercase tracking-wider text-[var(--color-text-tertiary)] mb-3">{{ t('settings.appearance') }}</h3>
              <div class="space-y-3">
                <div>
                  <label class="text-xs text-[var(--color-text-secondary)] mb-1.5 block">{{ t('settings.theme') }}</label>
                  <div class="flex rounded-[var(--radius-sm)] bg-[var(--color-bg-secondary)] p-0.5">
                    <button v-for="th in ['light','system','dark']" :key="th"
                      @click="settings.setTheme(th)"
                      :class="['flex-1 py-1 text-xs font-medium rounded transition-colors',
                        settings.theme === th
                          ? 'bg-[var(--color-bg-primary)] text-[var(--color-text-primary)] shadow-[var(--shadow-sm)]'
                          : 'text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]']"
                    >{{ t(`settings.${th}`) }}</button>
                  </div>
                </div>
                <!-- Title Bar Style -->
                <div>
                  <label class="text-xs text-[var(--color-text-secondary)] mb-1.5 block">{{ t('settings.titlebarStyle') }}</label>
                  <div class="flex rounded-[var(--radius-sm)] bg-[var(--color-bg-secondary)] p-0.5">
                    <button
                      @click="settings.setTitlebarStyle('macos')"
                      :class="['flex-1 py-1 text-xs font-medium rounded transition-colors',
                        settings.titlebarStyle === 'macos'
                          ? 'bg-[var(--color-bg-primary)] text-[var(--color-text-primary)] shadow-[var(--shadow-sm)]'
                          : 'text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]']"
                    >{{ t('settings.titlebarMacos') }}</button>
                    <button
                      @click="settings.setTitlebarStyle('win11')"
                      :class="['flex-1 py-1 text-xs font-medium rounded transition-colors',
                        settings.titlebarStyle === 'win11'
                          ? 'bg-[var(--color-bg-primary)] text-[var(--color-text-primary)] shadow-[var(--shadow-sm)]'
                          : 'text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]']"
                    >{{ t('settings.titlebarWin11') }}</button>
                  </div>
                </div>
                <div>
                  <label class="text-xs text-[var(--color-text-secondary)] mb-1.5 block">{{ t('settings.statusbarStyle') }}</label>
                  <div class="flex rounded-[var(--radius-sm)] bg-[var(--color-bg-secondary)] p-0.5">
                    <button
                      @click="settings.setStatusbarStyle('default')"
                      :class="['flex-1 py-1 text-xs font-medium rounded transition-colors',
                        settings.statusbarStyle === 'default'
                          ? 'bg-[var(--color-bg-primary)] text-[var(--color-text-primary)] shadow-[var(--shadow-sm)]'
                          : 'text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]']"
                    >{{ t('settings.statusbarDefault') }}</button>
                    <button
                      @click="settings.setStatusbarStyle('accent')"
                      :class="['flex-1 py-1 text-xs font-medium rounded transition-colors',
                        settings.statusbarStyle === 'accent'
                          ? 'bg-[var(--color-bg-primary)] text-[var(--color-text-primary)] shadow-[var(--shadow-sm)]'
                          : 'text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]']"
                    >{{ t('settings.statusbarAccent') }}</button>
                  </div>
                </div>
                <div>
                  <label class="text-xs text-[var(--color-text-secondary)] mb-1.5 block">{{ t('settings.accentColor') }}</label>
                  <div class="grid grid-cols-4 gap-2">
                    <button v-for="c in ['#007AFF','#34C759','#FF9500','#FF3B30','#AF52DE','#5856D6','#00C7BE','#FF2D55']" :key="c"
                      @click="settings.accentColor = c; settings.applyTheme(); settings.save()"
                      :class="['w-8 h-8 rounded-full transition-transform',
                        settings.accentColor === c ? 'ring-2 ring-offset-2 ring-offset-[var(--color-bg-primary)] scale-110' : 'hover:scale-105']"
                      :style="{ background: c, ringColor: c }" />
                  </div>
                </div>
                <div>
                  <label class="text-xs text-[var(--color-text-secondary)] mb-1.5 block">{{ t('settings.terminalColors') }}</label>
                  <div class="grid grid-cols-2 gap-2">
                    <button v-for="[key, p] in presets" :key="key"
                      @click="selectPreset(key)"
                      :class="['flex flex-col gap-1 p-2 rounded-[var(--radius-sm)] transition-colors text-left',
                        settings.terminalColorPreset === key
                          ? 'ring-2 ring-[var(--color-accent)]'
                          : 'hover:ring-1 hover:ring-[var(--color-border)]']">
                      <div class="h-4 rounded-sm border border-[var(--color-border)] flex overflow-hidden">
                        <div class="flex-1" :style="{ background: p.background }" />
                        <div class="w-3" :style="{ background: p.cursor }" />
                      </div>
                      <span class="text-[10px] text-[var(--color-text-secondary)]">{{ p.label }}</span>
                    </button>
                  </div>
                </div>
              </div>
            </section>

            <!-- Terminal -->
            <section>
              <h3 class="text-[10px] font-semibold uppercase tracking-wider text-[var(--color-text-tertiary)] mb-3">{{ t('settings.terminal') }}</h3>
              <div class="space-y-3">
                <div>
                  <label class="text-xs text-[var(--color-text-secondary)] mb-1 block">{{ t('settings.fontFamily') }}</label>
                  <input v-model="settings.fontFamily" @change="settings.save()"
                    class="w-full px-3 py-1.5 text-xs rounded-[var(--radius-sm)] border border-[var(--color-border)]
                      bg-[var(--color-bg-secondary)] text-[var(--color-text-primary)]
                      focus:outline-none focus:border-[var(--color-accent)]" />
                </div>
                <div>
                  <label class="text-xs text-[var(--color-text-secondary)] mb-1 block">{{ t('settings.fontSize') }}</label>
                  <div class="flex items-center gap-2">
                    <input type="range" min="10" max="24" :value="settings.fontSize"
                      @input="settings.fontSize = Number($event.target.value); settings.save()"
                      class="flex-1 accent-[var(--color-accent)]" />
                    <span class="text-xs text-[var(--color-text-primary)] w-8 text-right">{{ settings.fontSize }}px</span>
                  </div>
                </div>
                <div>
                  <label class="text-xs text-[var(--color-text-secondary)] mb-1.5 block">{{ t('settings.cursor') }}</label>
                  <div class="flex rounded-[var(--radius-sm)] bg-[var(--color-bg-secondary)] p-0.5">
                    <button v-for="c in ['block','bar','underline']" :key="c"
                      @click="settings.cursorStyle = c; settings.save()"
                      :class="['flex-1 py-1 text-xs font-medium rounded transition-colors',
                        settings.cursorStyle === c
                          ? 'bg-[var(--color-bg-primary)] text-[var(--color-text-primary)] shadow-[var(--shadow-sm)]'
                          : 'text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]']"
                    >{{ t(`settings.${c}`) }}</button>
                  </div>
                </div>
              </div>
            </section>

            <!-- Monitoring -->
            <section>
              <h3 class="text-[10px] font-semibold uppercase tracking-wider text-[var(--color-text-tertiary)] mb-3">{{ t('settings.monitoring') }}</h3>
              <div class="space-y-3">
                <div class="flex items-center justify-between">
                  <label class="text-xs text-[var(--color-text-secondary)]">{{ t('settings.sysRefresh') }}</label>
                  <div class="flex items-center gap-1">
                    <input type="number" min="1" max="10" :value="settings.monitorRefreshSecs"
                      @change="settings.monitorRefreshSecs = Number($event.target.value); settings.save()"
                      class="w-12 px-2 py-0.5 text-xs text-right rounded-[var(--radius-sm)] border border-[var(--color-border)]
                        bg-[var(--color-bg-secondary)] text-[var(--color-text-primary)]
                        focus:outline-none focus:border-[var(--color-accent)]" />
                    <span class="text-xs text-[var(--color-text-tertiary)]">s</span>
                  </div>
                </div>
                <div class="flex items-center justify-between">
                  <label class="text-xs text-[var(--color-text-secondary)]">{{ t('settings.pingInterval') }}</label>
                  <div class="flex items-center gap-1">
                    <input type="number" min="5" max="120" :value="settings.pingIntervalSecs"
                      @change="settings.pingIntervalSecs = Number($event.target.value); settings.save()"
                      class="w-12 px-2 py-0.5 text-xs text-right rounded-[var(--radius-sm)] border border-[var(--color-border)]
                        bg-[var(--color-bg-secondary)] text-[var(--color-text-primary)]
                        focus:outline-none focus:border-[var(--color-accent)]" />
                    <span class="text-xs text-[var(--color-text-tertiary)]">s</span>
                  </div>
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
