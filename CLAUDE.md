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
**I18n:** Custom `useI18n` composable; locale files in `src/locales/{en,zh-CN}.js`

### Component Tree

```
App.vue
└── AppShell.vue
    ├── TitleBar.vue              — Window title + debug toggle
    ├── ActivityBar.vue           — VS Code-style icon bar (Servers / Batch / Settings)
    ├── Sidebar.vue               — Thin shell: resize handle + view routing
    │   └── sidebar/
    │       ├── ServersView.vue    — Unified connection list with status-colored rows
    │       ├── BatchView.vue      — Multi-server command execution + per-result copy
    │       └── SettingsView.vue   — All settings with mini activity bar (General / Appearance / Terminal / Monitoring)
    ├── MainTabBar.vue            — Tab strip: overview / terminal / filemanager
    ├── [Content Area]
    │   ├── WelcomeScreen.vue     — Shown when no server connected
    │   ├── ServerOverview.vue    — Donut gauges (CPU/Mem/Disk/Swap) + world map + info cards (useServerData composable)
    │   ├── TerminalContainer.vue — xterm.js terminal (useXterm composable)
    │   └── FileManager.vue       — File browser
    │       ├── BreadcrumbBar.vue
    │       ├── FileListView.vue   — Table: Name, Perms, Owner, Group, Size, Modified
    │       ├── FileIconView.vue   — Grid: icon thumbnails
    │       └── ContextMenu.vue   — Right-click actions
    ├── HostKeyDialog.vue         — TOFU host key fingerprint confirmation
    └── StatusBar.vue             — Connection status, progress bars, clock
```

### State Management (Pinia stores)

- **`useServerStore`** (`src/stores/useServerStore.js`) — Central orchestrator. Manages `servers[]` (each has `tabs[]` and `activeTabId`). `openServer(profile)` creates server entry, connects, starts ping timer. Ping runs `echo 1` every N seconds to measure latency.
- **`useConnectionStore`** (`src/stores/useConnectionStore.js`) — Saved connection profiles CRUD. `profiles[]`, `connecting` Set.
- **`useSettingsStore`** (`src/stores/useSettingsStore.js`) — App settings (theme, font, language, etc.). Persists via `invoke('get_config')` / `invoke('save_config')`.
- **`useFileManagerStore`** (`src/stores/useFileManagerStore.js`) — Per-session file browser state: `paths`, `entries`, `loading`, `errors`, `selections`, `clipboards`, navigation history (`navBack`, `navForward`). All state is keyed by `sessionId` to support multiple concurrent file manager tabs.

### Composables

- **`useServerData`** (`src/composables/useServerData.js`) — Replaced `useMonitor` for the overview page. Split into `fetchDynamic()` (CPU/memory/disk/swap — polled every `monitorRefreshSecs`, min 3s) and `fetchStatic()` (hostname, OS, kernel, uptime, timezone, all IPs, geo — fetched once at connection). Uses `Promise.allSettled` for concurrent execution with per-command timeouts.
- **`useMonitor`** (`src/composables/useMonitor.js`) — Legacy; still available but no longer used by ServerOverview.

### Tauri Commands (Rust)

All in `src-tauri/src/commands/`:

**Config** (`config.rs`): `get_config`, `save_config`, `save_connection`, `delete_connection`
**Connection** (`connection.rs`): `connect`, `disconnect`, `terminal_write`, `terminal_resize`, `exec_command`, `ping`, `server_traffic`, `clipboard_read`, `clipboard_write`
**File** (`file.rs`): `file_list`, `file_mkdir`, `file_remove`, `file_rename`, `file_copy`, `file_exists`, `file_read`, `file_write`, `file_download_dir`, `file_upload_path`, `file_chmod`
**Open** (`open.rs`): `open_url` — opens a URL in the system browser (cross-platform: `cmd /c start` on Windows, `open` on macOS, `xdg-open` on Linux). Uses `CREATE_NO_WINDOW` on Windows.

### SSH Layer (`src-tauri/src/ssh/`)

- **`session.rs`** — `SshSession::connect()` handles DNS → TCP → auth (password or private key with SHA-512 hash). Opens PTY channel, starts shell, spawns 4 tokio tasks: stdin writer, resize handler, exec handler, terminal read loop. **No host key verification** (`check_server_key` returns `Ok(true)` unconditionally).
- **`manager.rs`** — `SshManager` holds `HashMap<String, SshSessionHandle>` behind a `Mutex`. Each handle stores mpsc senders for stdin, exec, resize, and the session handle. `exec_command` uses oneshot channels for request/response pattern.

### File Listing Format

`file_list` uses GNU `find -printf` for machine-parseable output:
```
type\tsize\ttimestamp\tperms\towner\tgroup\tname
```
Fallback to `ls -la` for BSD/macOS. `FileEntry` struct:
```rust
{ name, is_dir, size, modified, perms, owner, group }
```

### File Transfer & Progress Reporting

All file transfers emit `sftp-progress` Tauri events:
```json
{
  "session_id": "...",
  "operation": "upload|download",
  "path": "filename",
  "bytes_transferred": 12345,
  "total_bytes": 67890
}
```

- **Single file download (`file_read`):** Runs `stat -c %s` first to get file size → streams via `cat` with determinate progress (percentage bar)
- **Single file upload (`file_write`):** Streams 32KB chunks via `cat >` with determinate progress
- **Directory download (`file_download_dir`):** `tar czf -` streaming with indeterminate progress (bytes-only, pulsing bar). Emits final completion event after loop ends so StatusBar can show success.
- **Directory upload (`file_upload_path`):** Recursively walks local filesystem, uploads per-file with individual progress events

**StatusBar.vue** listens for `sftp-progress` globally:
- `total_bytes > 0` → determinate progress bar with percentage + filename
- `total_bytes == 0` → indeterminate pulsing bar with bytes transferred
- Completion detected when `bytes_transferred >= total_bytes` or via final completion event
- Shows localized success message (e.g., "↑ file.txt uploaded" / "↓ file.tar.gz downloaded") for ~5s

**FileManager.vue** also listens for `sftp-progress` to auto-refresh the directory listing after uploads complete.

### Config Encryption Flow

`get_machine_id()` is called from `derive_key()` → called by both `encrypt_config()` and `decrypt_config()`:

- **Startup:** `AppShell.vue` → `settingsStore.load()` + `connectionStore.loadProfiles()` → `get_config` → `decrypt_config()` → `derive_key()` → `get_machine_id()` (cached after first call)
- **Connection:** `serverStore.openServer()` → `updateLastConnected()` → `saveProfile()` → `save_connection` → `load()` + `save()` → both go through `derive_key()` → cache hit
- **Settings save:** `settingsStore.save()` → `save_config` → `load()` + `save()` → cache hit

The `OnceLock` cache in `get_machine_id()` means `reg.exe` runs at most once per process lifetime.

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

## Key Behaviours

- **Version check:** `VersionCheck.vue` fetches GitHub releases API on startup (delayed 3s). Uses `getVersion()` from Tauri API for current version. Only `goDownload()` permanently skips a version; `remindLater()` re-prompts on next launch. Gated by `settings.checkUpdates` (default: true). Opens URLs via `open_url` Tauri command.
- **Settings defaults:** `monitorRefreshSecs=10` (min 3s), `pingIntervalSecs=5` (min 3s), `fontFamily="Consolas, 'Courier New'"`, `checkUpdates=true`.
- **ServerOverview data flow:** `fetchDynamic()` runs first → `firstLoadDone=true` → skeleton fades out, gauges appear. `fetchStatic()` runs concurrently — info cards (hostname, OS, uptime, timezone, IPs) fill in as data arrives. Only dynamic metrics poll on the interval.

## Toolchain

**Rust:** MSVC toolchain (`stable-x86_64-pc-windows-msvc`). All deps are pure Rust — no C compilation needed. `russh` with `ring` backend.

## Sidebar Design Specification

All sidebar views are in `src/components/layout/sidebar/` (3 views: ServersView, BatchView, SettingsView). The `Sidebar.vue` shell handles resize, view routing, and passes `:width` to child views.

### Typography Scale

| Role | Classes |
|------|---------|
| Section headers | `text-[11px] font-bold uppercase tracking-widest text-[var(--color-text-tertiary)]` |
| Item titles (server name) | `text-[14px] leading-tight font-medium text-[var(--color-text-primary)]` |
| Item subtitles / host | `text-[11px] text-[var(--color-text-secondary)]` |
| Descriptive hints | `text-[10px] text-[var(--color-text-tertiary)]/40` |
| Code / monospace | `text-[11px] font-mono text-[var(--color-text-primary)]` |
| Form labels | `text-[11px] font-medium text-[var(--color-text-secondary)]` |
| Small badges / counts | `text-[10px] tabular-nums` |
| Segmented buttons | `text-[12px]` |

### Color Tokens

| Purpose | Classes |
|---------|---------|
| Segmented btn active | `bg-[var(--color-accent)] text-white shadow-sm` |
| Segmented btn inactive | `text-[var(--color-text-tertiary)]/50 hover:text-[var(--color-text-secondary)]` |
| Connected status bg | `bg-[#34C759]/12` |
| Reconnecting status bg | `bg-[#FF9500]/12` |
| Error/disconnected bg | `bg-[#FF3B30]/8` |
| Primary action btn | `bg-[var(--color-accent)] text-white hover:brightness-110 disabled:opacity-30` |
| Danger btn | `text-[var(--color-danger)] hover:bg-[var(--color-danger)]/10` |

### Interactive Controls
(Same as before.)

### Layout Rules
(Same as before.)

### Component Pattern
Each view receives `:width` Number prop and self-contained with its own `<script setup>`.
