<script setup lang="ts">
import SideBar from '@/components/nav/SideBar.vue'
import BaseFileDescriptor from '@/components/ui/BaseFileDescriptor.vue'
</script>

<template>
  <div class="layout">
    <SideBar />
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
</template>

<style scoped>
.layout {
  display: flex;
}

.file-grid {
  display: grid;
  padding: 1%;
  flex: 1;
  grid-template-columns: repeat(auto-fill, 200px);
  gap: 20px;
}
</style>
