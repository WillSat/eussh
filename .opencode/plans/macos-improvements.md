# macOS Window Improvements Plan

## 1. macOS Rounded Corners

### 1a. Enable transparent window background
**File:** `src-tauri/tauri.conf.json` line 23
**Change:** `"transparent": false` → `"transparent": true`

### 1b. Add macOS-specific rounded corners in CSS
**File:** `src/assets/css/main.css`

Add after the `body` rule (around line 45-49):

```css
/* macOS rounded corners (when using custom titlebar) */
html.is-macos body.custom-titlebar {
  border-radius: 10px;
  overflow: hidden;
  box-shadow: 0 0 0 1px rgba(0, 0, 0, 0.06), 0 4px 30px rgba(0, 0, 0, 0.3);
}

html.is-macos.dark body.custom-titlebar {
  box-shadow: 0 0 0 1px rgba(255, 255, 255, 0.06), 0 4px 30px rgba(0, 0, 0, 0.5);
}
```

### 1c. Detect macOS platform and apply CSS classes
**File:** `src/components/layout/AppShell.vue`

In `<script setup>`, add import:
```js
import { platform } from '@tauri-apps/api/os'
```

In `onMounted`, add after settings load:
```js
// Detect macOS for rounded corners
try {
  const os = await platform()
  if (os === 'macos') {
    document.documentElement.classList.add('is-macos')
  }
} catch {}
// Apply custom-titlebar class initially (default: no native decorations)
document.body.classList.add('custom-titlebar')
```

Add a watcher to toggle the class when nativeDecorations changes:
```js
watch(() => settingsStore.nativeDecorations, (val) => {
  document.body.classList.toggle('custom-titlebar', !val)
})
```

---

## 2. Native Titlebar Toggle (Settings Option)

### 2a. Rust config model
**File:** `src-tauri/src/models/config.rs`

In `AppSettings` struct, add after `show_geo_lookup`:
```rust
#[serde(default)]
pub native_decorations: bool,
```

In `impl Default for AppSettings`, add:
```rust
native_decorations: false,
```

### 2b. Tauri capabilities permission
**File:** `src-tauri/capabilities/default.json`

Add permission (after `core:window:allow-start-dragging`):
```json
"core:window:allow-set-decorations"
```

### 2c. Frontend settings store
**File:** `src/stores/useSettingsStore.js`

In `state` object, add:
```js
nativeDecorations: false,
```

In `load()` action, after `show_geo_lookup` block, add:
```js
if (s.native_decorations !== undefined) this.nativeDecorations = s.native_decorations
```

In `save()` action, inside the `settings` object, add:
```js
native_decorations: this.nativeDecorations,
```

Add new action `toggleNativeDecorations()`:
```js
async toggleNativeDecorations(val) {
  this.nativeDecorations = val
  try {
    const { getCurrentWindow } = await import('@tauri-apps/api/window')
    const win = getCurrentWindow()
    await win.setDecorations(val)
    document.body.classList.toggle('custom-titlebar', !val)
  } catch {}
  this.save()
},
```

### 2d. Settings UI toggle
**File:** `src/components/layout/sidebar/SettingsView.vue`

In the Appearance tab (`sTab === 'appearance'` section), add BEFORE the Titlebar Style selector (around line 125):

```html
<!-- Native Titlebar -->
<label class="flex items-center justify-between cursor-pointer group">
  <div class="flex-1 mr-3">
    <span class="text-[12px] font-medium text-[var(--color-text-secondary)] group-hover:text-[var(--color-text-primary)] transition-colors">{{ t('settings.nativeTitlebar') }}</span>
    <p class="text-[10px] text-[var(--color-text-tertiary)]/70 mt-0.5 leading-relaxed">{{ t('settingsDesc.nativeTitlebar') }}</p>
  </div>
  <button role="switch" :aria-checked="settings.nativeDecorations" @click="settings.toggleNativeDecorations(!settings.nativeDecorations)"
    :class="['inline-flex h-5 w-8 shrink-0 rounded-full border-2 border-transparent transition-all duration-200', settings.nativeDecorations ? 'bg-[var(--color-accent)]' : 'bg-[var(--color-bg-tertiary)]']">
    <span :class="['inline-block h-3.5 w-3.5 rounded-full bg-white shadow-sm transition-transform duration-200 mt-px', settings.nativeDecorations ? 'translate-x-3.5' : 'translate-x-0.5']" />
  </button>
</label>
```

Then wrap the existing Titlebar Style section with a `v-if="!settings.nativeDecorations"`:

```html
<!-- Titlebar (hidden when native titlebar is on) -->
<div v-if="!settings.nativeDecorations">
  <span class="text-[12px] font-medium text-[var(--color-text-secondary)]">{{ t('settings.titlebarStyle') }}</span>
  <p class="text-[10px] text-[var(--color-text-tertiary)]/70 mt-0.5 leading-relaxed">{{ t('settingsDesc.titlebar') }}</p>
  <div class="flex rounded-lg bg-[var(--color-bg-tertiary)]/40 p-0.5 mt-2">
    <button @click="settings.setTitlebarStyle('macos')" :class="seg(settings.titlebarStyle === 'macos')">{{ t('settings.titlebarMacos') }}</button>
    <button @click="settings.setTitlebarStyle('win11')" :class="seg(settings.titlebarStyle === 'win11')">{{ t('settings.titlebarWin11') }}</button>
  </div>
</div>
```

(Replace the existing Titlebar Style block at lines 126-133)

### 2e. Conditional TitleBar rendering
**File:** `src/components/layout/AppShell.vue`

Change line 172 from:
```html
<TitleBar />
```
to:
```html
<TitleBar v-if="!settingsStore.nativeDecorations" />
```

### 2f. i18n strings

**File:** `src/locales/en.js`

In `settings`, add:
```js
nativeTitlebar: 'Native Title Bar',
```
In `settingsDesc`, add:
```js
nativeTitlebar: 'Use the operating system default title bar instead of the custom one',
```

**File:** `src/locales/zh-CN.js`

In `settings`, add:
```js
nativeTitlebar: '原生标题栏',
```
In `settingsDesc`, add:
```js
nativeTitlebar: '使用操作系统的原生标题栏，替代自定义样式',
```

---

## Summary of All File Changes

| # | File | Type |
|---|---|---|
| 1 | `src-tauri/tauri.conf.json:23` | Change `transparent: false` → `true` |
| 2 | `src-tauri/capabilities/default.json` | Add permission |
| 3 | `src-tauri/src/models/config.rs` | Add `native_decorations` field |
| 4 | `src/stores/useSettingsStore.js` | Add state + load/save + toggle action |
| 5 | `src/components/layout/sidebar/SettingsView.vue` | Add toggle + conditional hide |
| 6 | `src/components/layout/AppShell.vue` | Conditional TitleBar + macOS detection + watcher |
| 7 | `src/assets/css/main.css` | macOS rounded corners CSS |
| 8 | `src/locales/en.js` | i18n strings |
| 9 | `src/locales/zh-CN.js` | i18n strings |
