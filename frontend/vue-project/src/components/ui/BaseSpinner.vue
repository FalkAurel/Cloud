<template>
  <div class="spinner" />
</template>

<style scoped>
.spinner {
  width: 32px;
  height: 32px;
  border: 3px solid #e6ecf5;
  border-top-color: #003580;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}
</style>
