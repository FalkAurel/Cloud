<script setup lang="ts">
import { onMounted, watch } from 'vue'
import SideBar from '@/components/nav/SideBar.vue'
import BaseFileDescriptor from '@/components/ui/BaseFileDescriptor.vue'
import BaseNotification from '@/components/ui/BaseNotification.vue'
import UploadButton from '@/components/ui/UploadButton.vue'
import { useFilesStore, type FileEntry } from '@/stores/files'

const filesStore = useFilesStore()

onMounted(() => filesStore.fetchFiles())

watch(() => filesStore.error, (val) => {
  if (val) setTimeout(() => { filesStore.error = null }, 5000)
})

function clearError() { filesStore.error = null }
</script>

<template>
  <div class="layout">
    <SideBar />
    <div class="main">
      <header class="topbar">
        <nav class="breadcrumb">
          <template v-for="(crumb, idx) in filesStore.folderStack" :key="crumb.id ?? 'root'">
            <span v-if="idx > 0" class="crumb-sep">/</span>
            <button
              v-if="idx < filesStore.folderStack.length - 1"
              class="crumb-btn"
              @click="filesStore.navigateTo(crumb)"
            >{{ crumb.name }}</button>
            <span v-else class="crumb-current">{{ crumb.name }}</span>
          </template>
        </nav>
        <span v-if="filesStore.uploading" class="uploading-label">Uploading…</span>
      </header>
      <div class="file-grid">
        <BaseFileDescriptor
          v-for="file in filesStore.files"
          :key="file.id"
          :fileName="file.name"
          :fileSize="file.size"
          :created_at="file.created_at"
          :last_modified_at="file.last_modified_at"
          :type="file.type"
          :is_folder="file.is_folder"
          @open-folder="filesStore.openFolder(file as FileEntry)"
        />
      </div>
    </div>

    <BaseNotification
      :show="!!filesStore.error"
      type="error"
      :duration="5000"
      @update:show="clearError"
    >
      {{ filesStore.error }}
    </BaseNotification>

    <UploadButton
      :disabled="filesStore.uploading"
      @upload="filesStore.upload"
      @create-folder="filesStore.createFolder"
    />
  </div>
</template>

<style scoped>
.layout {
  display: flex;
  height: 100vh;
  background: #f0f4f9;
}

.main {
  display: flex;
  flex-direction: column;
  flex: 1;
  overflow: hidden;
}

.topbar {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 0 28px;
  height: 56px;
  background: white;
  border-bottom: 1px solid #dde3ed;
  flex-shrink: 0;
}

.breadcrumb {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 14px;
}

.crumb-btn {
  background: none;
  border: none;
  padding: 2px 4px;
  cursor: pointer;
  color: #003580;
  font-size: 14px;
  font-weight: 500;
  border-radius: 3px;
}

.crumb-btn:hover {
  background: #f0f4f9;
}

.crumb-sep {
  color: #9ca3af;
  font-size: 13px;
}

.crumb-current {
  font-size: 14px;
  font-weight: 600;
  color: #003580;
}

.uploading-label {
  font-size: 12px;
  color: #6b7a90;
}

.file-grid {
  display: grid;
  padding: 24px 28px;
  grid-template-columns: repeat(auto-fill, 200px);
  gap: 16px;
  overflow-y: auto;
  align-content: start;
}
</style>
