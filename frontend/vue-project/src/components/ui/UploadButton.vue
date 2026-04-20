<template>
  <div v-if="open" class="backdrop" @click="close" />

  <div class="fab-container">
    <Transition name="pop">
      <div v-if="open" class="menu">
        <template v-if="!namingFolder">
          <button class="menu-item" @click="triggerFile">
            <span class="menu-icon">📄</span>Upload file
          </button>
          <button class="menu-item" @click="startFolder">
            <span class="menu-icon">📁</span>New folder
          </button>
        </template>
        <template v-else>
          <div class="folder-row">
            <input
              ref="folderInput"
              v-model="folderName"
              class="folder-input"
              placeholder="Folder name"
              @keydown.enter="confirmFolder"
              @keydown.esc="close"
            />
            <button class="confirm-btn" :disabled="!folderName.trim()" @click="confirmFolder">✓</button>
          </div>
        </template>
      </div>
    </Transition>

    <button class="upload-fab" :class="{ active: open }" :disabled="disabled" @click="toggleMenu" title="New">
      <span class="cross" :class="{ rotated: open }">
        <span class="bar bar-h" />
        <span class="bar bar-v" />
      </span>
    </button>

    <input ref="fileInput" type="file" hidden @change="onFiles" />
  </div>
</template>

<script setup lang="ts">
import { ref, nextTick } from 'vue'

defineProps<{ disabled?: boolean }>()

const emit = defineEmits<{
  upload: [file: File]
  'create-folder': [name: string]
}>()

const open = ref(false)
const namingFolder = ref(false)
const folderName = ref('')
const fileInput = ref<HTMLInputElement>(null!)
const folderInput = ref<HTMLInputElement>(null!)

function toggleMenu() {
  open.value ? close() : (open.value = true)
}

function close() {
  open.value = false
  namingFolder.value = false
  folderName.value = ''
}

function triggerFile() {
  close()
  fileInput.value.click()
}

function onFiles(e: Event) {
  const file = (e.target as HTMLInputElement).files?.[0]
  if (file) emit('upload', file)
  fileInput.value.value = ''
}

async function startFolder() {
  namingFolder.value = true
  await nextTick()
  folderInput.value?.focus()
}

function confirmFolder() {
  const name = folderName.value.trim()
  if (!name) return
  emit('create-folder', name)
  close()
}
</script>

<style scoped>
.backdrop {
  position: fixed;
  inset: 0;
  z-index: 99;
}

.fab-container {
  position: fixed;
  bottom: 32px;
  right: 32px;
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 8px;
  z-index: 100;
}

.menu {
  background: white;
  border: 1px solid #dde3ed;
  border-radius: 8px;
  box-shadow: 0 4px 16px rgba(0, 53, 128, 0.12);
  overflow: hidden;
  min-width: 160px;
}

.menu-item {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 10px 14px;
  background: none;
  border: none;
  font-size: 13px;
  font-weight: 500;
  color: #1a2b4b;
  cursor: pointer;
  text-align: left;
  transition: background 0.12s;
}

.menu-item:hover {
  background: #f0f4f9;
}

.menu-icon {
  font-size: 15px;
}

.folder-row {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 10px;
}

.folder-input {
  flex: 1;
  padding: 5px 8px;
  border: 1px solid #c8d3e3;
  border-radius: 5px;
  font-size: 13px;
  color: #1a2b4b;
  outline: none;
}

.folder-input:focus {
  border-color: #003580;
}

.confirm-btn {
  padding: 5px 10px;
  background: #003580;
  color: white;
  border: none;
  border-radius: 5px;
  font-size: 13px;
  cursor: pointer;
  transition: background 0.12s;
}

.confirm-btn:disabled {
  background: #a0b4d6;
  cursor: not-allowed;
}

.confirm-btn:not(:disabled):hover {
  background: #002560;
}

.upload-fab {
  width: 52px;
  height: 52px;
  border-radius: 8px;
  background: white;
  border: 2px solid #c8d3e3;
  box-shadow: 0 4px 16px rgba(0, 53, 128, 0.12);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: border-color 0.15s, background 0.15s, box-shadow 0.15s, transform 0.15s;
}

.upload-fab:hover:not(:disabled) {
  border-color: #003580;
  background: #f0f4f9;
  box-shadow: 0 6px 24px rgba(0, 53, 128, 0.2);
  transform: translateY(-2px);
}

.upload-fab.active {
  border-color: #003580;
  background: #f0f4f9;
}

.upload-fab:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.cross {
  position: relative;
  width: 20px;
  height: 20px;
  transition: transform 0.2s ease;
}

.cross.rotated {
  transform: rotate(45deg);
}

.bar {
  position: absolute;
  background: #a0b4d6;
  border-radius: 2px;
  transition: background 0.15s;
}

.bar-h {
  width: 100%;
  height: 2.5px;
  top: 50%;
  left: 0;
  transform: translateY(-50%);
}

.bar-v {
  width: 2.5px;
  height: 100%;
  left: 50%;
  top: 0;
  transform: translateX(-50%);
}

.upload-fab:hover:not(:disabled) .bar,
.upload-fab.active .bar {
  background: #003580;
}

.pop-enter-active,
.pop-leave-active {
  transition: opacity 0.15s ease, transform 0.15s ease;
}

.pop-enter-from,
.pop-leave-to {
  opacity: 0;
  transform: translateY(6px) scale(0.97);
}
</style>
