<template>
  <div v-if="visible" class="notification" :class="type">
    <span class="message">
      <slot></slot>
    </span>

    <button class="close-btn" @click="close">×</button>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'

const props = defineProps<{
  show: boolean
  duration?: number
  type?: 'success' | 'error' | 'info'
}>()

const emit = defineEmits(['update:show'])

const visible = ref(props.show)

// sync with parent
watch(() => props.show, (val) => {
  visible.value = val

  if (val && props.duration) {
    setTimeout(() => close(), props.duration)
  }
})

function close() {
  visible.value = false
  emit('update:show', false)
}
</script>

<style scoped>
.notification {
  position: fixed;
  top: 20px;
  right: 20px;

  display: flex;
  align-items: center;
  gap: 12px;

  padding: 12px 18px;
  border-radius: 4px;

  font-size: 14px;
  font-weight: 500;

  color: white;

  background: #003580;

  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.15);

  animation: slideIn 0.25s ease;
}

.notification.success {
  background: #003580;
}

.notification.error {
  background: #c0392b;
}

.notification.info {
  background: #0070d2;
}

.close-btn {
  background: transparent;
  border: none;
  color: white;
  font-size: 16px;
  cursor: pointer;
  opacity: 0.8;
}

.close-btn:hover {
  opacity: 1;
}

@keyframes slideIn {
  from {
    opacity: 0;
    transform: translateY(-10px) translateX(10px);
  }
  to {
    opacity: 1;
    transform: translateY(0) translateX(0);
  }
}
</style>