import { invoke as tauriInvoke } from '@tauri-apps/api/core'

export const IPC = {
  CONNECT: 'connect',
  DISCONNECT: 'disconnect',
  TERMINAL_WRITE: 'terminal_write',
  TERMINAL_RESIZE: 'terminal_resize',
  EXEC_COMMAND: 'exec_command',
  GET_CONFIG: 'get_config',
  SAVE_CONFIG: 'save_config',
  SAVE_CONNECTION: 'save_connection',
  DELETE_CONNECTION: 'delete_connection',
  CLIPBOARD_READ: 'clipboard_read',
  CLIPBOARD_WRITE: 'clipboard_write',
  FILE_LIST: 'file_list',
  FILE_MKDIR: 'file_mkdir',
  FILE_REMOVE: 'file_remove',
  FILE_RENAME: 'file_rename',
  FILE_COPY: 'file_copy',
  FILE_EXISTS: 'file_exists',
  FILE_READ: 'file_read',
  FILE_WRITE: 'file_write',
  FILE_DOWNLOAD_DIR: 'file_download_dir',
  FILE_UPLOAD_PATH: 'file_upload_path',
  FILE_CHMOD: 'file_chmod',
}

export async function invoke(command, args = {}) {
  try {
    return await tauriInvoke(command, args)
  } catch (e) {
    console.error(`IPC error [${command}]:`, e)
    throw e
  }
}
