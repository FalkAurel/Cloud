<template>
  <div class="call-to-action">
    <router-link v-bind:to="route">
      <span>{{ text }}</span>
    </router-link>
  </div>
</template>

<script setup lang="ts">
defineProps({
  text: {
    type: String,
    required: true,
  },
  route: {
    type: String,
    required: true,
  },
})
</script>

<style scoped>
.call-to-action {
  text-align: center;
  margin-top: 12px;
}

.call-to-action span {
  color: #42b983;
  font-weight: 500;
  cursor: pointer;
  text-decoration: none;
}

.call-to-action span:hover {
  text-decoration: underline;
}
</style>
