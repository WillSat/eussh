# CLAUDE.md

Codebase guidance for Claude Code.

## Build & Run

```bash
npm run dev              # Vite dev server only (no Tauri)
npx tauri dev            # Full Tauri app (dev mode: compiles Rust + starts Vite + opens window)
npm run build            # Frontend production build → dist/
cd src-tauri && cargo check              # Rust type-check only (fast, no binary)
cd src-tauri && cargo build              # Rust debug build → target/debug/eussh.exe
cd src-tauri && cargo build --release    # Rust release build (won't embed frontend correctly)
npx tauri build          # Full production build with frontend embedding → installers
```

**Critical:** `cargo check` only type-checks — it does NOT produce a binary. Always use `cargo build` or `npx tauri dev/build` when testing runtime changes. `cargo build --release` alone does NOT embed frontend files; only `npx tauri build` does a complete production build.

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

## Windows Console Window Suppression

Three mechanisms prevent black terminal windows from flashing on Windows:

### 1. Main binary subsystem (`src-tauri/src/main.rs:1`)
```rust
#![windows_subsystem = "windows"]
```
Marks the `.exe` as a GUI-subsystem application. Without this, Windows allocates a console (`conhost.exe`) at startup. Must be unconditional — the original `cfg_attr(not(debug_assertions), ...)` wrapper only applied it in release builds, causing debug builds to flash.

### 2. Child process suppression (`src-tauri/src/storage/encrypt.rs:22,30`)
```rust
use std::os::windows::process::CommandExt;
Command::new("reg")
    .creation_flags(0x08000000)  // CREATE_NO_WINDOW
    .output()
```
Any `std::process::Command` on Windows must set `creation_flags(0x08000000)` to prevent the child process from opening a console. `get_machine_id()` also uses `std::sync::OnceLock` to cache the machine GUID — the cache eliminates redundant calls during config load/save.

### 3. Dev server (`src-tauri/tauri.conf.json:6`)
```json
"beforeDevCommand": "node node_modules/vite/bin/vite.js"
```
Bypasses `npm.cmd` and `vite.cmd` batch files. On Windows, `.cmd` files require `cmd.exe` (which is console-subsystem), causing a flash. Calling `node.exe` directly avoids the `cmd.exe` intermediaries entirely.

## Config Encryption Flow

`get_machine_id()` is called from `derive_key()` → called by both `encrypt_config()` and `decrypt_config()`:

- **Startup:** `AppShell.vue` → `settingsStore.load()` + `connectionStore.loadProfiles()` → `get_config` → `decrypt_config()` → `derive_key()` → `get_machine_id()` (cached after first call)
- **Connection:** `serverStore.openServer()` → `updateLastConnected()` → `saveProfile()` → `save_connection` → `load()` + `save()` → both go through `derive_key()` → cache hit
- **Settings save:** `settingsStore.save()` → `save_config` → `load()` + `save()` → cache hit

The `OnceLock` cache in `get_machine_id()` means `reg.exe` runs at most once per process lifetime.
