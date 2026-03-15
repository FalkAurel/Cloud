<template>
  <div class="file-descriptor">
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

    <div class="file-actions">
      <button class="btn">Open</button>
      <button class="btn">Download</button>
      <button class="btn">Share</button>
      <button class="btn danger">Delete</button>
    </div>
  </div>
</template>

<script setup lang="ts">
import DOCXIcon from '@/assets/fileDescriptors/docx.svg'
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
  display: flex;
  flex-direction: column;
  justify-content: space-between;

  width: 200px;
  height: 200px; /* same as width → square tile */

  padding: 16px;

  background: white;
  border: 1px solid #e5e7eb;
  border-radius: 12px;

  transition: all 0.2s ease;
}

.file-descriptor:hover {
  border-color: #d1d5db;
  box-shadow: 0 8px 18px rgba(0, 0, 0, 0.06);
  transform: translateY(-2px);
}

/* header */

.file-header {
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
}

.file-icon {
  width: 56px;
  height: 56px;

  display: flex;
  align-items: center;
  justify-content: center;

  background: #f3f4f6;
  border-radius: 10px;

  margin-bottom: 6px;
}

.file-icon img {
  width: 28px;
}

.file-name {
  font-weight: 600;
  font-size: 14px;
  color: #111827;

  max-width: 160px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* metadata */

.file-meta {
  margin-top: 10px;
  font-size: 12px;
  color: #6b7280;
  flex: 1; /* take available space → push actions to bottom */
  display: flex;
  flex-direction: column;
  justify-content: center; /* vertically center metadata */
  gap: 4px;
}

.meta-row {
  display: flex;
  justify-content: space-between;
}

.label {
  font-weight: 500;
}

.value {
  color: #374151;
}

/* actions */

.file-actions {
  display: flex;
  justify-content: center;
  gap: 6px;
  opacity: 0;
  transition: opacity 0.15s;
}

.file-descriptor:hover .file-actions {
  opacity: 1;
}

/* buttons */

.btn {
  border: none;
  background: #f3f4f6;

  padding: 5px 8px;
  border-radius: 6px;

  font-size: 11px;
  cursor: pointer;

  transition: background 0.15s;
}

.btn:hover {
  background: #e5e7eb;
}

.btn.danger:hover {
  background: #fee2e2;
}
</style>
