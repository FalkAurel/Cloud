<template>
  <div
    class="file-descriptor"
    :class="{ folder: is_folder }"
    @click="is_folder ? emit('open-folder') : undefined"
  >
    <div class="file-actions" @click.stop>
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

    <div class="file-header" :class="{ 'folder-header': is_folder }">
      <div class="file-icon" :class="{ 'folder-icon': is_folder }">
        <img v-if="returnIcon().includes('.svg')" :src="returnIcon()" />
        <span v-else>{{ returnIcon() }}</span>
      </div>
      <div class="file-name">
        <span>{{ fileName }}</span>
      </div>
    </div>

    <div class="file-meta">
      <div class="meta-row" v-if="!is_folder">
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
      <div v-if="is_folder" class="open-hint">Click to open →</div>
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
import { returnBytesFormated } from '@/utils/format'

interface FileProps {
  fileName: string
  fileSize: number
  created_at: number
  last_modified_at: number
  type: string
  is_folder: boolean
}

const props = defineProps<FileProps>()
const emit = defineEmits<{ 'open-folder': [] }>()

function returnIcon(): string {
  if (props.is_folder) return '🗂️'
  const extension = props.fileName.split('.').pop()
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
      return '📄'
  }
}

function returnDateFormated(date: number): string {
  const now = Date.now()
  const diff = now - date

  const minutes = Math.floor(diff / (1000 * 60))
  const hours = Math.floor(diff / (1000 * 60 * 60))
  const days = Math.floor(diff / (1000 * 60 * 60 * 24))

  if (minutes < 60) return plurals(minutes, 'minute')
  if (hours < 24) return plurals(hours, 'hour')
  if (days < 30) return plurals(days, 'day')
  return new Date(date).toLocaleDateString()
}

function plurals(value: number, unit: string): string {
  return `${value} ${unit}${value !== 1 ? 's' : ''} ago`
}
</script>

<style scoped>
/* ── Base card ───────────────────────────────────────────── */
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
  overflow: hidden;
}

.file-descriptor:hover {
  box-shadow: 0 4px 16px rgba(0, 53, 128, 0.12);
  transform: translateY(-2px);
}

/* ── Folder variant ──────────────────────────────────────── */
.file-descriptor.folder {
  cursor: pointer;
  border-top-color: #f59e0b;
}

.file-descriptor.folder:hover {
  box-shadow: 0 4px 16px rgba(245, 158, 11, 0.2);
}

/* Folder "tab" shape — a notch cut from top-left corner */
.file-descriptor.folder::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  width: 56px;
  height: 6px;
  background: #f59e0b;
  border-radius: 0 0 6px 0;
}

.folder-header {
  background: #fffbeb;
}

.folder-icon {
  background: #fde68a !important;
}

/* ── File header ─────────────────────────────────────────── */
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

/* ── Scrolling filename ──────────────────────────────────── */
.file-name {
  font-weight: 600;
  font-size: 13px;
  color: #003580;
  width: 160px;
  overflow: hidden;
  white-space: nowrap;
  text-align: center;
}

.file-name span {
  display: inline-block;
  white-space: nowrap;
}

/* Scroll the text left to reveal the full name on hover.
   min(0px, calc(160px - 100%)) moves only if text overflows:
   if span is 250px wide → shifts -90px; if ≤160px → stays at 0. */
.file-descriptor:hover .file-name span {
  animation: scroll-name 3s ease-in-out 0.4s infinite alternate;
}

@keyframes scroll-name {
  from { transform: translateX(0); }
  to   { transform: translateX(min(0px, calc(160px - 100%))); }
}

/* ── Meta ────────────────────────────────────────────────── */
.file-meta {
  padding: 0 16px 12px;
  font-size: 11px;
  color: #6b7280;
  display: flex;
  flex-direction: column;
  gap: 3px;
  border-top: 1px solid #f0f4f9;
  margin-top: 4px;
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

.open-hint {
  font-size: 10px;
  color: #f59e0b;
  font-weight: 600;
  text-align: right;
  margin-top: 2px;
  opacity: 0;
  transition: opacity 0.15s;
}

.file-descriptor.folder:hover .open-hint {
  opacity: 1;
}

/* ── Action buttons ──────────────────────────────────────── */
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
