<script setup>
import { ref, onMounted } from 'vue'
import { getVersion } from '@tauri-apps/api/app'
import { useSettingsStore } from '@/stores/useSettingsStore'
import { useI18n } from '@/composables/useI18n'
import { invoke } from '@/utils/ipc'

const { t } = useI18n()
const settings = useSettingsStore()

const GITHUB_API = 'https://api.github.com/repos/WillSat/eussh/releases/latest'
const DOWNLOAD_URL = 'https://github.com/WillSat/eussh/releases/latest'
const STORAGE_KEY = 'eussh_version_skip' // only stores permanently-skipped versions

const showModal = ref(false)
const latestVersion = ref('')
const latestUrl = ref(DOWNLOAD_URL)
const latestBody = ref('')
const currentVersion = ref('0.0.0')

function shouldCheck() {
  if (!settings.checkUpdates) return false
  return true
}

/** Close dialog — will re-check on next app launch */
function remindLater() {
  showModal.value = false
}

/** Open download page in system browser + permanently skip this version */
function goDownload() {
  invoke('open_url', { url: latestUrl.value }).catch(() => {})
  localStorage.setItem(STORAGE_KEY, JSON.stringify({
    skippedVersion: latestVersion.value,
    skippedAt: Date.now(),
  }))
  showModal.value = false
}

async function checkVersion() {
  if (!shouldCheck()) return
  try {
    const resp = await fetch(GITHUB_API, {
      headers: { Accept: 'application/vnd.github+json' },
    })
    if (!resp.ok) return
    const release = await resp.json()
    const tag = (release.tag_name || '').replace(/^v/, '')
    if (!tag) return

    // Skip if user already permanently skipped this version
    const stored = (() => { try { return JSON.parse(localStorage.getItem(STORAGE_KEY) || '{}') } catch { return {} } })()
    if (stored.skippedVersion === tag) return

    if (compareVersions(tag, currentVersion.value) > 0) {
      latestVersion.value = release.tag_name
      latestUrl.value = release.html_url || DOWNLOAD_URL
      latestBody.value = release.body || ''
      showModal.value = true
    }
  } catch { /* network error – silently ignore */ }
}

function compareVersions(a, b) {
  const pa = a.split('.').map(Number)
  const pb = b.split('.').map(Number)
  for (let i = 0; i < Math.max(pa.length, pb.length); i++) {
    const na = pa[i] || 0; const nb = pb[i] || 0
    if (na > nb) return 1; if (na < nb) return -1
  }
  return 0
}

onMounted(async () => {
  try { currentVersion.value = await getVersion() } catch { currentVersion.value = '0.0.0' }
  setTimeout(checkVersion, 3000)
})
</script>

<template>
  <Teleport to="body">
    <div
      v-if="showModal"
      class="fixed inset-0 z-[9999] flex items-center justify-center bg-black/50"
      @click.self="remindLater"
    >
      <div class="rounded-xl border border-[var(--color-border)] bg-[var(--color-bg-secondary)] shadow-2xl w-[400px] max-w-[90vw] overflow-hidden">
        <div class="px-6 pt-5 pb-3">
          <h3 class="text-base font-bold text-[var(--color-text-primary)]">
            {{ t('versionCheck.newAvailable') }}
          </h3>
          <p class="text-xs text-[var(--color-text-secondary)] mt-1">
            {{ t('versionCheck.description', { version: latestVersion, current: currentVersion }) }}
          </p>
        </div>
        <div v-if="latestBody" class="px-6 pb-2 max-h-[200px] overflow-y-auto">
          <pre class="text-xs text-[var(--color-text-secondary)] font-sans whitespace-pre-wrap">{{ latestBody }}</pre>
        </div>
        <div class="px-6 py-4 flex items-center justify-end gap-3 border-t border-[var(--color-border)]">
          <button @click="remindLater"
            class="px-4 py-1.5 text-xs font-medium rounded-lg bg-[var(--color-bg-tertiary)] text-[var(--color-text-secondary)] hover:bg-[var(--color-border)] transition-colors">
            {{ t('versionCheck.remindLater') }}
          </button>
          <button @click="goDownload"
            class="px-4 py-1.5 text-xs font-medium rounded-lg bg-[var(--color-accent)] text-white hover:brightness-110 transition-all">
            {{ t('versionCheck.download') }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
