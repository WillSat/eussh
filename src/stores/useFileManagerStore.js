import { defineStore } from 'pinia'
import { invoke } from '@/utils/ipc'

const _log = (...args) => { try { console.log('[FMStore]', ...args) } catch {} }

export const useFileManagerStore = defineStore('filemanager', {
  state: () => ({
    paths: {},
    entries: {},
    loading: {},
    errors: {},
    selections: {},
    clipboards: {},
    navBack: {},
    navForward: {},
    viewMode: 'list',
    lastClicked: {},
  }),

  actions: {
    // --- UI helpers ---
    setViewMode(mode) { this.viewMode = mode },

    // --- State helpers ---
    _path(sid) { return this.paths[sid] || '/' },
    _files(sid) { return this.entries[sid] || [] },
    _sels(sid) { return this.selections[sid] || [] },

    sortFiles(files) {
      if (!files) return []
      return [...files].sort((a, b) => {
        if (a.is_dir !== b.is_dir) return a.is_dir ? -1 : 1
        return a.name.toLowerCase().localeCompare(b.name.toLowerCase())
      })
    },

    // --- Directory loading ---
    async loadDir(sid, path) {
      this.loading[sid] = true
      this.errors[sid] = null
      try {
        const files = await invoke('file_list', { sessionId: sid, path })
        this.paths[sid] = path
        this.entries[sid] = this.sortFiles(files)
        this.selections[sid] = []
        this.lastClicked[sid] = null
      } catch (e) {
        this.errors[sid] = e?.message || String(e)
      } finally {
        this.loading[sid] = false
      }
    },

    async refresh(sid) {
      await this.loadDir(sid, this._path(sid))
    },

    // --- Navigation ---
    async navigateInto(sid, dirName) {
      const current = this._path(sid)
      const newPath = current === '/' ? '/' + dirName : current + '/' + dirName
      const back = this.navBack[sid] || []
      this.navBack[sid] = [...back, current]
      this.navForward[sid] = []
      await this.loadDir(sid, newPath)
    },

    async navigateBack(sid) {
      const back = this.navBack[sid] || []
      if (back.length === 0) return
      const prev = back.pop()
      const forward = this.navForward[sid] || []
      forward.push(this._path(sid))
      this.navBack[sid] = back
      this.navForward[sid] = forward
      await this.loadDir(sid, prev)
    },

    async navigateForward(sid) {
      const forward = this.navForward[sid] || []
      if (forward.length === 0) return
      const next = forward.pop()
      const back = this.navBack[sid] || []
      back.push(this._path(sid))
      this.navBack[sid] = back
      this.navForward[sid] = forward
      await this.loadDir(sid, next)
    },

    async navigateTo(sid, path) {
      const current = this._path(sid)
      if (path === current) return
      if (this.navBack[sid]) {
        this.navBack[sid] = [...this.navBack[sid], current]
      } else {
        this.navBack[sid] = [current]
      }
      this.navForward[sid] = []
      await this.loadDir(sid, path)
    },

    // --- Selection ---
    handleClick(sid, entryName, _event) {
      this.selections[sid] = [entryName]
    },

    clearSelection(sid) {
      this.selections[sid] = []
      this.lastClicked[sid] = null
    },

    // --- Clipboard ---
    copyItems(sid) {
      const sels = this._sels(sid)
      if (sels.length === 0) return
      this.clipboards[sid] = { sourceDir: this._path(sid), items: [...sels], operation: 'copy' }
    },

    cutItems(sid) {
      const sels = this._sels(sid)
      if (sels.length === 0) return
      this.clipboards[sid] = { sourceDir: this._path(sid), items: [...sels], operation: 'cut' }
    },

    async pasteItems(sid) {
      const cb = this.clipboards[sid]
      if (!cb || cb.items.length === 0) return
      const destDir = this._path(sid)
      for (const item of cb.items) {
        const src = cb.sourceDir === '/' ? '/' + item : cb.sourceDir + '/' + item
        const dst = destDir === '/' ? '/' + item : destDir + '/' + item
        try {
          if (cb.operation === 'cut') {
            await invoke('file_rename', { sessionId: sid, oldPath: src, newPath: dst })
          } else {
            await invoke('file_copy', { sessionId: sid, src, dst })
          }
        } catch (e) { _log('paste error', e) }
      }
      if (cb.operation === 'cut') { this.clipboards[sid] = null }
      await this.refresh(sid)
    },

    async duplicateItem(sid, name) {
      const dir = this._path(sid)
      const src = dir === '/' ? '/' + name : dir + '/' + name
      const dotIdx = name.lastIndexOf('.')
      const base = dotIdx > 0 ? name.slice(0, dotIdx) : name
      const ext = dotIdx > 0 ? name.slice(dotIdx) : ''
      let n = 1, newName
      const files = this._files(sid)
      do { newName = `${base} (${n})${ext}`; n++ }
      while (files.some(f => f.name === newName))
      const dst = dir === '/' ? '/' + newName : dir + '/' + newName
      try { await invoke('file_copy', { sessionId: sid, src, dst }) } catch (e) { _log('dup error', e) }
      await this.refresh(sid)
    },

    async renameItem(sid, oldName, newName) {
      const dir = this._path(sid)
      const src = dir === '/' ? '/' + oldName : dir + '/' + oldName
      const dst = dir === '/' ? '/' + newName : dir + '/' + newName
      try { await invoke('file_rename', { sessionId: sid, oldPath: src, newPath: dst }) } catch (e) { _log('rename error', e) }
      await this.refresh(sid)
    },

    async deleteItems(sid) {
      const sels = this._sels(sid)
      if (sels.length === 0) return
      const files = this._files(sid)
      for (const name of sels) {
        const entry = files.find(f => f.name === name)
        const path = this._path(sid) === '/' ? '/' + name : this._path(sid) + '/' + name
        try { await invoke('file_remove', { sessionId: sid, path, isDir: entry?.is_dir || false }) } catch (e) { _log('remove error', e) }
      }
      await this.refresh(sid)
    },

    async mkdir(sid, name) {
      const path = this._path(sid) === '/' ? '/' + name : this._path(sid) + '/' + name
      try { await invoke('file_mkdir', { sessionId: sid, path }) } catch (e) { _log('mkdir error', e) }
      await this.refresh(sid)
    },

    // --- Download/Upload ---
    async download(sid, remoteName) {
      const dir = this._path(sid)
      const remotePath = dir === '/' ? '/' + remoteName : dir + '/' + remoteName
      _log('download', remotePath)
      try {
        const data = await invoke('file_read', { sessionId: sid, remotePath })
        return { name: remoteName, data: new Uint8Array(data) }
      } catch (e) {
        _log('download error', e)
        throw e
      }
    },

    async upload(sid, fileName, data) {
      const dir = this._path(sid)
      const remotePath = dir === '/' ? '/' + fileName : dir + '/' + fileName
      _log('upload', remotePath)
      try {
        await invoke('file_write', { sessionId: sid, remotePath, data: Array.from(data) })
        await this.refresh(sid)
      } catch (e) {
        _log('upload error', e)
        throw e
      }
    },

    async downloadDir(sid, remoteName) {
      const dir = this._path(sid)
      const remotePath = dir === '/' ? '/' + remoteName : dir + '/' + remoteName
      _log('downloadDir', remotePath)
      try {
        const data = await invoke('file_download_dir', { sessionId: sid, remotePath })
        return { name: remoteName + '.tar.gz', data: new Uint8Array(data) }
      } catch (e) {
        _log('downloadDir error', e)
        throw e
      }
    },

    async chmod(sid, name, mode) {
      const dir = this._path(sid)
      const path = dir === '/' ? '/' + name : dir + '/' + name
      try { await invoke('file_chmod', { sessionId: sid, path, mode }) } catch (e) { _log('chmod error', e) }
      await this.refresh(sid)
    },

    async uploadPath(sid, localPath) {
      const dir = this._path(sid)
      _log('uploadPath', localPath, 'to', dir)
      try {
        await invoke('file_upload_path', { sessionId: sid, localPath, remoteDir: dir })
        await this.refresh(sid)
      } catch (e) {
        _log('uploadPath error', e)
        throw e
      }
    },
  },
})
