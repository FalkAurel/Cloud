<template>
  <div class="input-wrapper">
    <input
      class="input-field"
      v-bind:type="computedType"
      v-bind:placeholder="placeholder"
      v-bind:value="modelValue"
      v-on:input="updateValue"
    />

    <button
      v-if="type === 'password'"
      class="toggle-button"
      type="button"
      @click="togglePasswordVisibility"
    >
      <!-- Eye (password hidden) -->
      <svg
        v-if="isPasswordVisible"
        xmlns="http://www.w3.org/2000/svg"
        width="20"
        height="20"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
      >
        <path d="M1 12s4-7 11-7 11 7 11 7-4 7-11 7-11-7-11-7z" />
        <circle cx="12" cy="12" r="3" />
      </svg>

      <svg
        v-else
        xmlns="http://www.w3.org/2000/svg"
        width="20"
        height="20"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
      >
        <path d="M17.94 17.94A10.94 10.94 0 0112 19c-7 0-11-7-11-7a21.77 21.77 0 015.17-5.94" />
        <path d="M1 1l22 22" />
        <path d="M9.53 9.53A3 3 0 0012 15a3 3 0 002.47-5.47" />
      </svg>
    </button>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'

const props = defineProps({
  modelValue: String,
  placeholder: String,
  type: {
    type: String,
    default: 'text',
  },
})

const emit = defineEmits(['update:modelValue'])

const isPasswordVisible = ref(false)

const computedType = computed(() => {
  if (props.type === 'password') {
    return isPasswordVisible.value ? 'text' : 'password'
  }
  return props.type
})

function updateValue(event: Event) {
  const target = event.target as HTMLInputElement
  emit('update:modelValue', target.value)
}

function togglePasswordVisibility() {
  isPasswordVisible.value = !isPasswordVisible.value
}
</script>

<style scoped>
.input-wrapper {
  position: relative;
  width: 100%;
  display: flex;
  align-items: center;
}

.input-field {
  flex: 1;
  padding: 12px 40px 12px 14px;

  font-size: 14px;
  font-family: inherit;

  background: #ffffff;
  color: #111;

  border: 1px solid #e5e7eb;
  border-radius: 10px;

  outline: none;

  transition:
    border-color 0.2s ease,
    box-shadow 0.2s ease,
    background 0.2s ease;
}

.input-field::placeholder {
  color: #9ca3af;
}

.input-field:hover {
  border-color: #d1d5db;
}

.input-field:focus {
  border-color: #3b82f6;
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.15);
}

.toggle-button {
  position: absolute;
  right: 0.6rem;
  top: 50%;
  transform: translateY(-50%);

  background: none;
  border: none;
  cursor: pointer;

  color: #9ca3af;
  display: flex;
  align-items: center;
}

.toggle-button:hover {
  color: #374151;
}
</style>
