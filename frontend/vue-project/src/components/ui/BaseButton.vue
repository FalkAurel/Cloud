<template>
  <button class="apple-btn" @click="$emit('click')">
    <slot></slot>
  </button>
</template>

<script setup lang="ts">
defineEmits(['click'])
</script>

<style scoped>
.apple-btn {
  padding: 10px 18px;
  border-radius: 4px;
  border: none;

  font-size: 14px;
  font-weight: 600;
  letter-spacing: 0.03em;
  text-transform: uppercase;

  color: white;
  background: #003580;

  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.15);

  cursor: pointer;

  transition:
    background 0.15s ease,
    box-shadow 0.15s ease,
    transform 0.1s ease;
}

.apple-btn:disabled {
  background: #a0b4d6;
  color: #e8edf5;
  cursor: not-allowed;
  box-shadow: none;
  transform: none;
}

.apple-btn:hover:not(:disabled) {
  background: #0050b3;
  box-shadow: 0 3px 8px rgba(0, 53, 128, 0.25);
}

.apple-btn:active:not(:disabled) {
  background: #002a6e;
  transform: translateY(1px);
  box-shadow: none;
}

.apple-btn:focus-visible {
  outline: none;
  box-shadow: 0 0 0 3px rgba(0, 112, 210, 0.4);
}
</style>
