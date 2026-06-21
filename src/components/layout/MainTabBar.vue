<script setup>
import { ref, computed } from 'vue'
import { useServerStore } from '@/stores/useServerStore'
import { useI18n } from '@/composables/useI18n'
const { t } = useI18n()

const serverStore = useServerStore()

const server = computed(() => serverStore.activeServer)
const tabs = computed(() => server.value?.tabs || [])
const activeTabId = computed(() => server.value?.activeTabId || 'overview')

function selectTab(tabId) {
  if (!server.value) return
  serverStore.setActiveTab(server.value.id, tabId)
}

function closeTab(tab) {
  if (!server.value) return
  serverStore.removeTab(server.value.id, tab.id)
}

function addTerminal() {
  if (!server.value) return
  serverStore.addTerminalTab(server.value.id)
  dropdownVisible.value = false
}

function addFileManager() {
  if (!server.value) return
  serverStore.addFileManagerTab(server.value.id)
  dropdownVisible.value = false
}

const dropdownVisible = ref(false)
const dropdownX = ref(0)
const dropdownY = ref(0)

const winW = computed(() => window.innerWidth)

function toggleDropdown(e) {
  const rect = e.currentTarget.getBoundingClientRect()
  dropdownX.value = rect.left
  dropdownY.value = rect.bottom + 4
  dropdownVisible.value = !dropdownVisible.value
}
</script>

<template>
  <div
    v-if="server"
    class="flex items-center h-9 shrink-0 select-none
      bg-[var(--color-bg-secondary)] shadow-[0_1px_3px_rgba(0,0,0,0.04)]"
  >
    <div class="flex items-center h-full overflow-x-auto">
      <button
        v-for="tab in tabs"
        :key="tab.id"
        @click="selectTab(tab.id)"
        :class="[
          'group flex items-center gap-2 h-full px-3 text-[13px] font-medium whitespace-nowrap transition-colors',
          tab.id === activeTabId
            ? 'bg-[var(--color-bg-primary)] text-[var(--color-text-primary)]'
            : 'text-[var(--color-text-secondary)] hover:bg-[var(--color-bg-tertiary)]',
        ]"
      >
        <span>{{ tab.title }}</span>
        <span
          v-if="tab.type !== 'overview'"
          @click.stop="closeTab(tab)"
          class="w-4 h-4 flex items-center justify-center rounded text-[var(--color-text-tertiary)]
            hover:bg-[var(--color-bg-tertiary)] hover:text-[var(--color-text-primary)]
            transition-colors text-[10px] ml-0.5"
        >&#x2715;</span>
      </button>
    </div>
    <div class="relative">
      <button
        @click="toggleDropdown"
        class="w-6 h-6 flex items-center justify-center rounded text-[var(--color-text-secondary)]
          hover:text-[var(--color-text-primary)] hover:bg-[var(--color-bg-tertiary)] transition-colors shrink-0 mx-1 text-sm"
        title="Add"
      >+</button>
      <Teleport to="body">
        <div v-if="dropdownVisible" class="fixed inset-0 z-[55]" @click="dropdownVisible = false" @contextmenu.prevent="dropdownVisible = false">
          <div
            class="fixed bg-[var(--color-bg-primary)] rounded-[var(--radius-sm)] shadow-[var(--shadow-lg)] py-1 min-w-[160px] text-[13px] select-none"
            :style="{ left: Math.min(dropdownX, winW - 170) + 'px', top: dropdownY + 'px' }"
          >
            <button @click="addTerminal"
              class="w-full text-left px-3 py-1.5 hover:bg-[var(--color-bg-tertiary)] text-[var(--color-text-primary)]">
              {{ t('tabbar.newTerminal') }}
            </button>
            <button @click="addFileManager"
              class="w-full text-left px-3 py-1.5 hover:bg-[var(--color-bg-tertiary)] text-[var(--color-text-primary)]">
              {{ t('tabbar.newFileManager') }}
            </button>
          </div>
        </div>
      </Teleport>
    </div>
  </div>
</template>
