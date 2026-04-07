<template>
  <button class="upload-fab" @click="input.click()" title="Upload file">
    <span class="cross">
      <span class="bar bar-h" />
      <span class="bar bar-v" />
    </span>
    <input ref="input" type="file" hidden @change="onFiles" />
  </button>
</template>

<script setup lang="ts">
import { ref } from 'vue'

const input = ref<HTMLInputElement>(null!)

const emit = defineEmits<{
  upload: [file: File]
}>()

function onFiles(e: Event) {
  const file = (e.target as HTMLInputElement).files?.[0]
  if (file) emit('upload', file)
  input.value.value = ''
}
</script>

<style scoped>
.upload-fab {
  position: fixed;
  bottom: 32px;
  right: 32px;
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
  transition: border-color 0.15s ease, background 0.15s ease, box-shadow 0.15s ease, transform 0.15s ease;
}

.upload-fab:hover {
  border-color: #003580;
  background: #f0f4f9;
  box-shadow: 0 6px 24px rgba(0, 53, 128, 0.2);
  transform: translateY(-2px);
}

.cross {
  position: relative;
  width: 20px;
  height: 20px;
}

.bar {
  position: absolute;
  background: #a0b4d6;
  border-radius: 2px;
  transition: background 0.15s ease;
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

.upload-fab:hover .bar {
  background: #003580;
}
</style>
