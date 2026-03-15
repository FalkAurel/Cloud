<template>
  <div class="SignUp-Page-Wrapper">
    <div class="SignUp-View">
      <h2>Create your account</h2>

      <div class="SignUp-Input-Fields">
        <BaseInput v-model.trim="username" placeholder="Username" />
        <BaseInput v-model.trim="email" placeholder="Email" />
        <BaseInput v-model.trim="password" placeholder="Password" type="password" />
        <BaseInput v-model.trim="confirmPassword" placeholder="Confirm Password" type="password" />
      </div>

      <BaseButton
        :disabled="!username || !email || !password || password !== confirmPassword"
        @click="signup"
        class="signup-btn"
      >
        Sign Up
      </BaseButton>

      <CallToAction text="Already have an account? Login" route="/login" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import BaseInput from '../components/ui/BaseInput.vue'
import BaseButton from '../components/ui/BaseButton.vue'
import CallToAction from '../components/ui/CallToAction.vue'

const username = ref('')
const email = ref('')
const password = ref('')
const confirmPassword = ref('')

function signup() {
  if (password.value !== confirmPassword.value) {
    alert('Passwords do not match!')
    return
  }
  alert(`Username: ${username.value}, Email: ${email.value}`)
}
</script>

<style scoped>
/* Wrapper centers card vertically and horizontally */
.SignUp-Page-Wrapper {
  display: flex;
  justify-content: center;
  align-items: center;
  height: 100vh;

  /* Slightly softer gradient to differentiate from login page */
  background: linear-gradient(135deg, #f0fdfa, #d1fae5);
}

/* Card */
.SignUp-View {
  display: flex;
  flex-direction: column;
  gap: 16px;

  width: 95%;
  max-width: 450px;
  padding: 32px;

  background-color: #ffffff;
  border-radius: 16px;
  box-shadow: 0 6px 24px rgba(0, 0, 0, 0.08);
  text-align: center;
}

/* Header */
.SignUp-View h2 {
  margin: 0;
  font-size: 20px;
  color: #111827;
}

/* Input fields */
.SignUp-Input-Fields {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

/* Optional: make sign-up button match Vue green */
.signup-btn {
  background: linear-gradient(180deg, #42b983, #2c974b);
}
</style>
