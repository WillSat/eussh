import { invoke as tauriInvoke } from '@tauri-apps/api/core'

export async function invoke(command, args = {}) {
  try {
    return await tauriInvoke(command, args)
  } catch (e) {
    console.error(`IPC error [${command}]:`, e)
    throw e
  }
}
