<script setup>
import { ref, computed, onMounted, onBeforeUnmount } from 'vue'
import { useFileManagerStore } from '@/stores/useFileManagerStore'
import { useI18n } from '@/composables/useI18n'
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow'
import { listen } from '@tauri-apps/api/event'
import BreadcrumbBar from './BreadcrumbBar.vue'
import FileListView from './FileListView.vue'
import FileIconView from './FileIconView.vue'
import ContextMenu from './ContextMenu.vue'
import RoseSpinner from '@/components/common/RoseSpinner.vue'

const props = defineProps({
  sessionId: { type: String, required: true },
})

const fm = useFileManagerStore()
const { t } = useI18n()

const currentDir = computed(() => fm.paths[props.sessionId] || '/')
const files = computed(() => fm.entries[props.sessionId] || [])
const loading = computed(() => fm.loading[props.sessionId])
const error = computed(() => fm.errors[props.sessionId])
const sels = computed(() => fm.selections[props.sessionId] || [])
const canBack = computed(() => (fm.navBack[props.sessionId] || []).length > 0)
const canForward = computed(() => (fm.navForward[props.sessionId] || []).length > 0)
const cbNotEmpty = computed(() => {
  const cb = fm.clipboards[props.sessionId]
  return cb && cb.items && cb.items.length > 0
})

// Context menu
const ctx = ref({ visible: false, x: 0, y: 0, entry: null })
const renameTarget = ref(null)
const newFolderPrompt = ref(false)
const newFolderName = ref('')
const chmodPrompt = ref(null)
const chmodMode = ref('')
const renameName = ref('')
const dragOver = ref(false)

// Cleanup references declared at top level so onBeforeUnmount always fires
let unlistenSftp = null
let unlistenDrag = null

onBeforeUnmount(() => {
  unlistenSftp?.()
  unlistenDrag?.()
  window.removeEventListener('resize', hideContextMenu)
})

onMounted(async () => {
  await fm.loadDir(props.sessionId, currentDir.value)
  unlistenSftp = await listen('sftp-progress', (e) => {
    if (e.payload.session_id === props.sessionId) {
      if (e.payload.total_bytes > 0 && e.payload.bytes_transferred >= e.payload.total_bytes) {
        setTimeout(() => fm.refresh(props.sessionId), 500)
      }
    }
  })

  // Drag-drop from local filesystem
  try {
    const webview = getCurrentWebviewWindow()
    unlistenDrag = await webview.onDragDropEvent((event) => {
      if (event.payload.type === 'enter') { dragOver.value = true }
      else if (event.payload.type === 'leave') { dragOver.value = false }
      else if (event.payload.type === 'drop') {
        dragOver.value = false
        handleDrop(event.payload.paths || [])
      }
    })
  } catch (e) {
    console.warn('FileManager: drag-drop registration failed', e)
  }

  window.addEventListener('resize', hideContextMenu)
})

async function handleDrop(paths) {
  for (const path of paths) {
    try {
      await fm.uploadPath(props.sessionId, path)
    } catch (e) {
      console.error('Drop upload failed:', e)
    }
  }
}

function showContextMenu(entry, event) {
  ctx.value = { visible: true, x: event.clientX, y: event.clientY, entry }
}
function hideContextMenu() { ctx.value.visible = false }

async function triggerDownload(name, isDir) {
  try {
    const result = isDir
      ? await fm.downloadDir(props.sessionId, name)
      : await fm.download(props.sessionId, name)
    const blob = new Blob([result.data])
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url; a.download = result.name
    a.click()
    URL.revokeObjectURL(url)
  } catch {}
}

async function onDoubleClick(entry) {
  if (entry.is_dir) {
    await fm.navigateInto(props.sessionId, entry.name)
  } else {
    await triggerDownload(entry.name, false)
  }
}

// Context menu actions — operate on right-clicked entry
function ensureSelected(name) {
  fm.selections[props.sessionId] = [name]
  fm.lastClicked[props.sessionId] = name
}
function ctxOpen() { const e = ctx.value.entry; if (e?.is_dir) { fm.navigateInto(props.sessionId, e.name); hideContextMenu() } }
function ctxDownload() { const e = ctx.value.entry; if (e) { triggerDownload(e.name, e.is_dir); hideContextMenu() } }
function ctxCopy() { const e = ctx.value.entry; if (e) { ensureSelected(e.name); fm.copyItems(props.sessionId); hideContextMenu() } }
function ctxCut() { const e = ctx.value.entry; if (e) { ensureSelected(e.name); fm.cutItems(props.sessionId); hideContextMenu() } }
function ctxPaste() { fm.pasteItems(props.sessionId); hideContextMenu() }
function ctxDuplicate() { const e = ctx.value.entry; if (e) { fm.duplicateItem(props.sessionId, e.name); hideContextMenu() } }
function ctxDelete() { const e = ctx.value.entry; if (e) { ensureSelected(e.name); fm.deleteItems(props.sessionId); hideContextMenu() } }
function ctxRename() { const e = ctx.value.entry; if (e) { renameTarget.value = e.name; renameName.value = e.name; hideContextMenu() } }
function ctxNewFolder() { newFolderPrompt.value = true; newFolderName.value = ''; hideContextMenu() }
function ctxChmod() { const e = ctx.value.entry; if (e) { chmodPrompt.value = e.name; chmodMode.value = e.perms || '755'; hideContextMenu() } }
async function submitChmod() {
  if (chmodPrompt.value && chmodMode.value) {
    await fm.chmod(props.sessionId, chmodPrompt.value, chmodMode.value)
  }
  chmodPrompt.value = null
}

async function submitRename() {
  if (renameTarget.value && renameName.value && renameName.value !== renameTarget.value) {
    await fm.renameItem(props.sessionId, renameTarget.value, renameName.value)
  }
  renameTarget.value = null
}
async function submitNewFolder() {
  if (newFolderName.value.trim()) {
    await fm.mkdir(props.sessionId, newFolderName.value.trim())
  }
  newFolderPrompt.value = false
}

function formatSize(bytes) {
  if (bytes === 0) return '0 B'
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(1024))
  return (bytes / Math.pow(1024, i)).toFixed(i === 0 ? 0 : 1) + ' ' + units[i]
}
</script>

<template>
  <div class="h-full flex flex-col bg-[var(--color-bg-primary)] select-none" :class="{ 'ring-2 ring-[var(--color-accent)]': dragOver }">
    <!-- Toolbar -->
    <div class="flex items-center h-8 px-2 gap-1 bg-[var(--color-bg-secondary)] shadow-[0_1px_2px_rgba(0,0,0,0.03)]">
      <button @click="fm.setViewMode('list')"
        :class="['px-2 py-0.5 text-[11px] rounded transition-colors', fm.viewMode === 'list' ? 'bg-[var(--color-bg-tertiary)] text-[var(--color-text-primary)]' : 'text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]']"
      >&#x2630;</button>
      <button @click="fm.setViewMode('icon')"
        :class="['px-2 py-0.5 text-[11px] rounded transition-colors', fm.viewMode === 'icon' ? 'bg-[var(--color-bg-tertiary)] text-[var(--color-text-primary)]' : 'text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]']"
      >&#x25A6;</button>
      <span class="w-px h-4 bg-[var(--color-bg-tertiary)] mx-1" />
      <button @click="fm.refresh(props.sessionId)" class="px-2 py-0.5 text-[11px] text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)] rounded transition-colors">
        &#x21BB;
      </button>
      <button @click="ctxNewFolder()" class="px-2 py-0.5 text-[11px] text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)] rounded transition-colors">
        + {{ t('filemanager.newFolder') }}
      </button>
    </div>

    <!-- Breadcrumb -->
    <BreadcrumbBar
      :current-dir="currentDir"
      :can-go-back="canBack"
      :can-go-forward="canForward"
      @navigate="(p) => fm.navigateTo(props.sessionId, p)"
      @back="fm.navigateBack(props.sessionId)"
      @forward="fm.navigateForward(props.sessionId)"
    />

    <!-- Content -->
    <div class="flex-1 overflow-auto relative"
      @contextmenu.prevent="showContextMenu(null, $event)"
      @click="fm.clearSelection(props.sessionId)">
      <div v-if="loading" class="absolute inset-0 flex items-center justify-center bg-[var(--color-bg-primary)]/50 z-10">
        <RoseSpinner :rose-scale="2.0" :text="t('filemanager.loading')" />
      </div>
      <div v-if="error" class="text-sm text-[var(--color-danger)] p-4">{{ error }}</div>
      <div v-if="!loading && files.length === 0 && !error" class="flex items-center justify-center h-full">
        <p class="text-sm text-[var(--color-text-tertiary)]">{{ t('filemanager.emptyDir') }}</p>
      </div>

      <FileListView v-if="fm.viewMode === 'list' && files.length > 0"
        :files="files" :selections="sels"
        @select="(entry, ev) => fm.handleClick(props.sessionId, entry.name, ev)"
        @double-click="onDoubleClick"
        @context-menu="(entry, ev) => showContextMenu(entry, ev)" />
      <FileIconView v-if="fm.viewMode === 'icon' && files.length > 0"
        :files="files" :selections="sels"
        @select="(entry, ev) => fm.handleClick(props.sessionId, entry.name, ev)"
        @double-click="onDoubleClick"
        @context-menu="(entry, ev) => showContextMenu(entry, ev)" />
    </div>

    <!-- Context Menu -->
    <ContextMenu
      :visible="ctx.visible"
      :x="ctx.x" :y="ctx.y"
      :entry="ctx.entry"
      :clipboard-not-empty="cbNotEmpty"
      @close="hideContextMenu"
      @open="ctxOpen" @download="ctxDownload" @copy="ctxCopy" @cut="ctxCut"
      @paste="ctxPaste" @duplicate="ctxDuplicate" @rename="ctxRename"
      @delete="ctxDelete" @new-folder="ctxNewFolder" @chmod="ctxChmod"
    />

    <!-- Rename Input (inline modal) -->
    <Teleport to="body">
      <div v-if="renameTarget" class="fixed inset-0 z-[70] flex items-center justify-center bg-black/20" @click="renameTarget = null">
        <div class="bg-[var(--color-bg-primary)] rounded-[var(--radius-md)] shadow-[var(--shadow-lg)] p-4 w-72 select-none" @click.stop>
          <p class="text-sm text-[var(--color-text-primary)] mb-2">{{ t('filemanager.renamePrompt', { name: renameTarget }) }}</p>
          <input v-model="renameName" @keyup.enter="submitRename()" @keyup.escape="renameTarget = null"
            ref="renameInput" class="w-full px-2 py-1 text-sm rounded-[var(--radius-sm)] bg-[var(--color-bg-tertiary)] text-[var(--color-text-primary)]" />
          <div class="flex justify-end gap-2 mt-3">
            <button @click="renameTarget = null" class="px-3 py-1 text-xs text-[var(--color-text-secondary)]">{{ t('filemanager.cancel') }}</button>
            <button @click="submitRename()" class="px-3 py-1 text-xs bg-[var(--color-accent)] text-white rounded">{{ t('filemanager.rename') }}</button>
          </div>
        </div>
      </div>
    </Teleport>

    <!-- New Folder Input -->
    <Teleport to="body">
      <div v-if="newFolderPrompt" class="fixed inset-0 z-[70] flex items-center justify-center bg-black/20" @click="newFolderPrompt = false">
        <div class="bg-[var(--color-bg-primary)] rounded-[var(--radius-md)] shadow-[var(--shadow-lg)] p-4 w-72 select-none" @click.stop>
          <p class="text-sm text-[var(--color-text-primary)] mb-2">{{ t('filemanager.newFolderPrompt') }}</p>
          <input v-model="newFolderName" @keyup.enter="submitNewFolder()" @keyup.escape="newFolderPrompt = false"
            class="w-full px-2 py-1 text-sm rounded-[var(--radius-sm)] bg-[var(--color-bg-tertiary)] text-[var(--color-text-primary)]" />
          <div class="flex justify-end gap-2 mt-3">
            <button @click="newFolderPrompt = false" class="px-3 py-1 text-xs text-[var(--color-text-secondary)]">{{ t('filemanager.cancel') }}</button>
            <button @click="submitNewFolder()" class="px-3 py-1 text-xs bg-[var(--color-accent)] text-white rounded">{{ t('filemanager.create') }}</button>
          </div>
        </div>
      </div>
    </Teleport>

    <!-- Chmod Input -->
    <Teleport to="body">
      <div v-if="chmodPrompt" class="fixed inset-0 z-[70] flex items-center justify-center bg-black/20" @click="chmodPrompt = null">
        <div class="bg-[var(--color-bg-primary)] rounded-[var(--radius-md)] shadow-[var(--shadow-lg)] p-4 w-72 select-none" @click.stop>
          <p class="text-sm text-[var(--color-text-primary)] mb-2">{{ t('filemanager.permissionsFor', { name: chmodPrompt }) }}</p>
          <input v-model="chmodMode" @keyup.enter="submitChmod()" @keyup.escape="chmodPrompt = null"
            class="w-full px-2 py-1 text-sm rounded-[var(--radius-sm)] bg-[var(--color-bg-tertiary)] text-[var(--color-text-primary)] font-mono" />
          <p class="text-[10px] text-[var(--color-text-tertiary)] mt-1">{{ t('filemanager.chmodHint') }}</p>
          <div class="flex justify-end gap-2 mt-3">
            <button @click="chmodPrompt = null" class="px-3 py-1 text-xs text-[var(--color-text-secondary)]">{{ t('filemanager.cancel') }}</button>
            <button @click="submitChmod()" class="px-3 py-1 text-xs bg-[var(--color-accent)] text-white rounded">{{ t('filemanager.apply') }}</button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>
