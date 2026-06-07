<script setup>
import { ref, onMounted, onBeforeUnmount } from 'vue'
import { useI18n } from '@/composables/useI18n'

const { t } = useI18n()
const W = ref(window.innerWidth)
const H = ref(window.innerHeight)
function onResize() { W.value = window.innerWidth; H.value = window.innerHeight }
onMounted(() => window.addEventListener('resize', onResize))
onBeforeUnmount(() => window.removeEventListener('resize', onResize))
const props = defineProps({
  visible: Boolean,
  x: Number,
  y: Number,
  entry: Object,
  clipboardNotEmpty: Boolean,
})
const emit = defineEmits(['close', 'open', 'download', 'copy', 'cut', 'paste', 'duplicate', 'rename', 'delete', 'newFolder', 'chmod'])

function act(action) { emit(action); emit('close') }
</script>

<template>
  <Teleport to="body">
    <div v-if="visible" class="fixed inset-0 z-[60]" @click="emit('close')" @contextmenu.prevent="emit('close')">
      <div
        class="fixed bg-[var(--color-bg-primary)] border border-[var(--color-border)] rounded-[var(--radius-sm)] shadow-[var(--shadow-md)] py-1 min-w-[160px] text-[13px] select-none"
        :style="{ left: Math.min(x, W - 170) + 'px', top: Math.min(y, H - 300) + 'px' }"
      >
        <!-- File or Folder -->
        <template v-if="entry">
          <button v-if="entry.is_dir" @click="act('open')"
            class="w-full text-left px-3 py-1.5 hover:bg-[var(--color-bg-tertiary)] text-[var(--color-text-primary)]">
            {{ t('filemanager.open') }}
          </button>
          <button @click="act('download')"
            class="w-full text-left px-3 py-1.5 hover:bg-[var(--color-bg-tertiary)] text-[var(--color-text-primary)]">
            {{ entry.is_dir ? t('filemanager.downloadAsArchive') : t('filemanager.download') }}
          </button>
          <div class="border-t border-[var(--color-border)] my-0.5" />
          <button @click="act('chmod')"
            class="w-full text-left px-3 py-1.5 hover:bg-[var(--color-bg-tertiary)] text-[var(--color-text-primary)]">
            {{ t('filemanager.permissions') }}
          </button>
          <div class="border-t border-[var(--color-border)] my-0.5" />
          <button @click="act('copy')"
            class="w-full text-left px-3 py-1.5 hover:bg-[var(--color-bg-tertiary)] text-[var(--color-text-primary)]">
            {{ t('filemanager.copy') }}
          </button>
          <button @click="act('cut')"
            class="w-full text-left px-3 py-1.5 hover:bg-[var(--color-bg-tertiary)] text-[var(--color-text-primary)]">
            {{ t('filemanager.cut') }}
          </button>
          <button v-if="clipboardNotEmpty" @click="act('paste')"
            class="w-full text-left px-3 py-1.5 hover:bg-[var(--color-bg-tertiary)] text-[var(--color-text-primary)]">
            {{ t('filemanager.paste') }}
          </button>
          <div class="border-t border-[var(--color-border)] my-0.5" />
          <button @click="act('duplicate')"
            class="w-full text-left px-3 py-1.5 hover:bg-[var(--color-bg-tertiary)] text-[var(--color-text-primary)]">
            {{ t('filemanager.duplicate') }}
          </button>
          <button @click="act('rename')"
            class="w-full text-left px-3 py-1.5 hover:bg-[var(--color-bg-tertiary)] text-[var(--color-text-primary)]">
            {{ t('filemanager.rename') }}
          </button>
          <div class="border-t border-[var(--color-border)] my-0.5" />
          <button @click="act('delete')"
            class="w-full text-left px-3 py-1.5 hover:bg-[var(--color-bg-tertiary)] text-[var(--color-danger)]">
            {{ t('filemanager.delete') }}
          </button>
        </template>
        <!-- Empty area -->
        <template v-else>
          <button v-if="clipboardNotEmpty" @click="act('paste')"
            class="w-full text-left px-3 py-1.5 hover:bg-[var(--color-bg-tertiary)] text-[var(--color-text-primary)]">
            {{ t('filemanager.paste') }}
          </button>
          <button @click="act('newFolder')"
            class="w-full text-left px-3 py-1.5 hover:bg-[var(--color-bg-tertiary)] text-[var(--color-text-primary)]">
            {{ t('filemanager.newFolder') }}
          </button>
        </template>
      </div>
    </div>
  </Teleport>
</template>
