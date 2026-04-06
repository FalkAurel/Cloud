<template>
  <div class="file-descriptor">
    <div class="file-actions">
      <button class="icon-btn" title="Download">
        <DownloadIcon />
      </button>
      <button class="icon-btn" title="Share">
        <ShareIcon />
      </button>
      <button class="icon-btn danger" title="Delete">
        <TrashIcon />
      </button>
    </div>

    <div class="file-header">
      <div class="file-icon">
        <img v-if="returnIcon(fileName).includes('.svg')" :src="returnIcon(fileName)" />
        <span v-else>{{ returnIcon(fileName) }}</span>
      </div>
      <div class="file-name">{{ fileName }}</div>
    </div>

    <div class="file-meta">
      <div class="meta-row">
        <span class="label">Size</span>
        <span class="value">{{ returnBytesFormated(fileSize) }}</span>
      </div>
      <div class="meta-row">
        <span class="label">Created</span>
        <span class="value">{{ returnDateFormated(created_at) }}</span>
      </div>
      <div class="meta-row">
        <span class="label">Modified</span>
        <span class="value">{{ returnDateFormated(last_modified_at) }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import DOCXIcon from '@/assets/fileDescriptors/docx.svg?url'
// @ts-ignore — installed in container via postCreateCommand
import DownloadIcon from '@/assets/icons/download.svg?component'
// @ts-ignore
import ShareIcon from '@/assets/icons/share.svg?component'
// @ts-ignore
import TrashIcon from '@/assets/icons/trash.svg?component'
interface FileProps {
  fileName: string
  fileSize: number // in Bytes
  created_at: number // A timestamp representing the creation date in milliseconds
  last_modified_at: number // A timestamp representing the last modified date in milliseconds
  type: string
}

defineProps<FileProps>()

function returnIcon(fileName: string): string {
  const extension = fileName.split('.').pop()
  switch (extension) {
    case 'txt':
      return '📄'
    case 'docx':
    case 'doc':
      return DOCXIcon
    case 'jpg':
    case 'png':
      return '🖼️'
    case 'mp4':
    case 'mov':
    case 'avi':
      return '🎥'
    default:
      return '📁'
  }
}

function returnBytesFormated(bytes: number): string {
  const units = ['Bytes', 'KB', 'MB', 'GB', 'TB']
  let i = 0
  while (bytes >= 1024 && i < units.length - 1) {
    bytes /= 1024
    i++
  }
  return `${bytes.toFixed(1)} ${units[i]}`
}

function returnDateFormated(date: number): string {
  const now = Date.now()
  const diff = now - date

  const minutes = Math.floor(diff / (1000 * 60))
  const hours = Math.floor(diff / (1000 * 60 * 60))
  const days = Math.floor(diff / (1000 * 60 * 60 * 24))

  if (minutes < 60) {
    return plurals(minutes, 'minute')
  }

  if (hours < 24) {
    return plurals(hours, 'hour')
  }

  if (days < 30) {
    return plurals(days, 'day')
  }

  return new Date(date).toLocaleDateString()
}

function plurals(value: number, unit: string): string {
  return `${value} ${unit}${value !== 1 ? 's' : ''} ago`
}
</script>

<style scoped>
.file-descriptor {
  position: relative;
  display: flex;
  flex-direction: column;

  width: 200px;

  background: white;
  border: 1px solid #dde3ed;
  border-radius: 4px;
  border-top: 3px solid #003580;

  transition: box-shadow 0.15s ease, transform 0.15s ease;
}

.file-descriptor:hover {
  box-shadow: 0 4px 16px rgba(0, 53, 128, 0.12);
  transform: translateY(-2px);
}

.file-header {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 20px 16px 12px;
  gap: 8px;
}

.file-icon {
  width: 48px;
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #f0f4f9;
  border-radius: 4px;
  font-size: 24px;
}

.file-icon img {
  width: 26px;
}

.file-name {
  font-weight: 600;
  font-size: 13px;
  color: #003580;
  max-width: 160px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  text-align: center;
}

.file-meta {
  padding: 0 16px 12px;
  font-size: 11px;
  color: #6b7280;
  display: flex;
  flex-direction: column;
  gap: 3px;
  border-bottom: 1px solid #f0f4f9;
}

.meta-row {
  display: flex;
  justify-content: space-between;
}

.label {
  color: #9ca3af;
}

.value {
  color: #374151;
  font-weight: 500;
}

.file-actions {
  position: absolute;
  top: 8px;
  right: 8px;
  display: flex;
  gap: 4px;
  opacity: 0;
  transition: opacity 0.15s;
}

.file-descriptor:hover .file-actions {
  opacity: 1;
}

.icon-btn {
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: white;
  border: 1px solid #dde3ed;
  border-radius: 4px;
  cursor: pointer;
  padding: 5px;
  color: #003580;
  transition: background 0.15s, border-color 0.15s;
}

.icon-btn svg {
  width: 14px;
  height: 14px;
}

.icon-btn:hover {
  background: #f0f4f9;
  border-color: #003580;
}

.icon-btn.danger {
  color: #c0392b;
}

.icon-btn.danger:hover {
  background: #fef2f2;
  border-color: #c0392b;
}
</style>
