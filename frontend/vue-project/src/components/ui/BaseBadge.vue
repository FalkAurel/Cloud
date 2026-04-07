<template>
  <span class="badge" :class="`badge-${variant}`">
    <slot />
  </span>
</template>

<script setup lang="ts">
defineProps<{
  variant: 'user' | 'admin'
}>()
</script>

<style scoped>
.badge {
  display: inline-block;
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  width: fit-content;
}

.badge-user {
  background: #e8effa;
  color: #003580;
}

.badge-admin {
  background: #fff3cd;
  color: #856404;
}
</style>
