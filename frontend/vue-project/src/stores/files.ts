import { ref } from 'vue'
import { defineStore } from 'pinia'

export type FileEntry = {
  id: string
  name: string
  size: number
  created_at: number
  last_modified_at: number
  type: string
  is_folder: boolean
}

type Crumb = { id: string | null; name: string }

export const useFilesStore = defineStore('files', () => {
  const files = ref<FileEntry[]>([])
  const uploading = ref(false)
  const error = ref<string | null>(null)
  const currentFolderId = ref<string | null>(null)
  const folderStack = ref<Crumb[]>([{ id: null, name: 'My Files' }])

  async function fetchFiles(parentId: string | null = null) {
    error.value = null
    const url = parentId
      ? `${import.meta.env.VITE_API_BASE}/files?parent_id=${parentId}`
      : `${import.meta.env.VITE_API_BASE}/files`
    try {
      const res = await fetch(url, { credentials: 'include' })
      if (!res.ok) { error.value = errorMessage(res.status); return }
      const data = await res.json()
      files.value = data.map((entry: {
        id: string; name: string; size_bytes: number;
        is_folder: boolean; created_at: string; modified_at: string
      }) => ({
        id: entry.id,
        name: entry.name,
        size: entry.size_bytes,
        created_at: new Date(entry.created_at).getTime(),
        last_modified_at: new Date(entry.modified_at).getTime(),
        type: '',
        is_folder: entry.is_folder,
      }))
      currentFolderId.value = parentId
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : 'Unknown error'
    }
  }

  function openFolder(entry: FileEntry) {
    folderStack.value.push({ id: entry.id, name: entry.name })
    fetchFiles(entry.id)
  }

  function navigateTo(crumb: Crumb) {
    const idx = folderStack.value.findIndex(c => c.id === crumb.id)
    if (idx >= 0) folderStack.value.splice(idx + 1)
    fetchFiles(crumb.id)
  }

  function goBack() {
    if (folderStack.value.length <= 1) return
    folderStack.value.pop()
    const parent = folderStack.value[folderStack.value.length - 1]
    if (parent) fetchFiles(parent.id)
  }

  async function upload(file: File) {
    if (uploading.value) return
    uploading.value = true
    error.value = null
    try {
      const headers: Record<string, string> = {
        'Content-Type': 'application/octet-stream',
        'X-Filename': file.name,
        'X-IsFolder': 'false',
      }
      if (currentFolderId.value) headers['X-ParentUuid'] = currentFolderId.value
      const res = await fetch(`${import.meta.env.VITE_API_BASE}/upload`, {
        method: 'POST',
        credentials: 'include',
        headers,
        body: file,
      })
      if (!res.ok) {
        error.value = errorMessage(res.status)
        return
      }
      const { id } = await res.json()
      const now = Date.now()
      files.value.unshift({ id, name: file.name, size: file.size, type: file.type, is_folder: false, created_at: now, last_modified_at: now })
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : 'Unknown error'
    } finally {
      uploading.value = false
    }
  }

  async function createFolder(name: string) {
    if (uploading.value) return
    uploading.value = true
    error.value = null
    try {
      const headers: Record<string, string> = {
        'Content-Type': 'application/octet-stream',
        'X-Filename': name,
        'X-IsFolder': 'true',
      }
      if (currentFolderId.value) headers['X-ParentUuid'] = currentFolderId.value
      const res = await fetch(`${import.meta.env.VITE_API_BASE}/upload`, {
        method: 'POST',
        credentials: 'include',
        headers,
      })
      if (!res.ok) {
        error.value = errorMessage(res.status)
        return
      }
      const { id } = await res.json()
      const now = Date.now()
      files.value.unshift({ id, name, size: 0, type: '', is_folder: true, created_at: now, last_modified_at: now })
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : 'Unknown error'
    } finally {
      uploading.value = false
    }
  }

  function errorMessage(status: number): string {
    const messages: Record<number, string> = {
      401: 'Not authorised. Please log in again.',
      413: 'File too large (max 200 MB).',
      415: 'Unsupported file type.',
      500: 'Server error. Please try again.',
    }
    return messages[status] ?? `Upload failed (${status})`
  }

  return { files, uploading, error, currentFolderId, folderStack, fetchFiles, openFolder, navigateTo, goBack, upload, createFolder }
})
