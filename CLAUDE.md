# CLAUDE.md

Codebase guidance for Claude Code.

## Build & Run

```bash
npm run dev          # Vite dev server
npx tauri dev        # Full Tauri app
npm run build        # Frontend production build
cd src-tauri && cargo check  # Rust check
```

## Architecture

**Stack:** Tauri v2 + Vue 3 (Composition API, `<script setup>`) + Tailwind CSS + Pinia + xterm.js
**SSH:** `russh` 0.61 with `ring` crypto backend (pure Rust)
**Storage:** AES-256-GCM encrypted JSON config, key derived from machine GUID via PBKDF2

### State Management (Pinia stores)

- **`useServerStore`** — Central store. Open servers array, tabs, activeTabId.
- **`useConnectionStore`** — Saved connection profiles CRUD.
- **`useSettingsStore`** — App settings. Persists via `invoke('get_config')` / `invoke('save_config')`.

### Tauri Commands (Rust)

All in `src-tauri/src/commands/`:
- `config::*` — get/save config, save/delete connection
- `connection::*` — connect, disconnect, terminal_write, terminal_resize, exec_command
- `file::*` — file_list, file_mkdir, file_remove, file_rename, file_copy, file_exists, file_read, file_write, file_download_dir, file_upload_path, file_chmod

### Toolchain

**Rust:** MSVC toolchain (`stable-x86_64-pc-windows-msvc`). All deps are pure Rust — no C compilation needed. `russh` with `ring` backend.
