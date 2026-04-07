<script setup lang="ts">
import SideBar from '@/components/nav/SideBar.vue'
import BaseFileDescriptor from '@/components/ui/BaseFileDescriptor.vue'
import UploadButton from '@/components/ui/UploadButton.vue'

function onUpload(file: File) {
  // TODO: send file to backend
  console.log('uploading', file.name)
}
</script>

<template>
  <div class="layout">
    <SideBar />
    <div class="main">
      <header class="topbar">
        <h1 class="page-title">My Files</h1>
      </header>
      <div class="file-grid">
        <BaseFileDescriptor
          fileName="Test"
          :fileSize="1024"
          :created_at="Date.now()"
          :last_modified_at="Date.now()"
          type="text/plain"
        />
      </div>
    </div>
    <UploadButton @upload="onUpload" />
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
  padding: 0 28px;
  height: 56px;
  background: white;
  border-bottom: 1px solid #dde3ed;
  flex-shrink: 0;
}

.page-title {
  font-size: 16px;
  font-weight: 600;
  color: #003580;
  margin: 0;
  letter-spacing: 0.01em;
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
