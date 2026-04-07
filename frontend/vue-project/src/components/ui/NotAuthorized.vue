<template>
  <div class="error-state">
    <span class="error-icon">!</span>
    <span>{{ message }}</span>
    <button class="retry-btn" @click="emit('retry')">Erneut versuchen</button>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  message: string
}>()

const emit = defineEmits<{
  retry: []
}>()
</script>

<style scoped>
.error-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  flex: 1;
  gap: 12px;
  color: #6b7a90;
  font-size: 14px;
}

.error-icon {
  width: 36px;
  height: 36px;
  border-radius: 50%;
  background: #fef2f2;
  border: 2px solid #c0392b;
  color: #c0392b;
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 700;
  font-size: 18px;
}

.retry-btn {
  padding: 6px 16px;
  font-size: 13px;
  background: #003580;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
}

.retry-btn:hover {
  background: #0050b3;
}
</style>
