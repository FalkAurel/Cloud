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
  border-radius: 14px;

  font-size: 14px;
  font-weight: 500;

  color: white;

  backdrop-filter: blur(12px);
  background: linear-gradient(180deg, #42b983, #2c974b);

  box-shadow:
    0 10px 30px rgba(0, 0, 0, 0.2),
    inset 0 1px 0 rgba(255, 255, 255, 0.25);

  animation: slideIn 0.25s ease;
}

/* types */
.notification.success {
  background: linear-gradient(180deg, #42b983, #2c974b);
}

.notification.error {
  background: linear-gradient(180deg, #ef4444, #b91c1c);
}

.notification.info {
  background: linear-gradient(180deg, #3b82f6, #1d4ed8);
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

/* animation */
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