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
  border-radius: 999px;
  border: none;

  font-size: 14px;
  font-weight: 500;
  letter-spacing: 0.01em;

  color: white;
  background: linear-gradient(180deg, #42b983, #2c974b);

  box-shadow:
    0 6px 16px rgba(0, 0, 0, 0.15),
    inset 0 1px 0 rgba(255, 255, 255, 0.25);

  cursor: pointer;

  transition:
    transform 0.15s ease,
    box-shadow 0.15s ease,
    inset 0 1px 0 rgba(255, 255, 255, 0.3);
}

.apple-btn:disabled {
  background: linear-gradient(180deg, #a3d9b1, #81c68b); /* lighter green */
  color: #e0e0e0; /* light text color */
  cursor: not-allowed; /* show the "not clickable" cursor */
  box-shadow: none; /* optional: remove shadow */
  transform: none; /* no hover/active movement */
  opacity: 0.7; /* optional: make it slightly transparent */
}

/* hover effect */
.apple-btn:hover {
  transform: translateY(-1px);
  box-shadow:
    0 10px 22px rgba(0, 0, 0, 0.18),
    inset 0 1px 0 rgba(255, 255, 255, 0.3);
}

/* click */
.apple-btn:active {
  transform: translateY(1px);
  box-shadow:
    0 4px 10px rgba(0, 0, 0, 0.15),
    inset 0 2px 4px rgba(0, 0, 0, 0.2);
}

/* focus ring for accessibility */
.apple-btn:focus-visible {
  outline: none;
  box-shadow:
    0 0 0 3px rgba(66, 185, 131, 0.4),
    /* lighter Vue green */ 0 6px 16px rgba(0, 0, 0, 0.15);
}
</style>
