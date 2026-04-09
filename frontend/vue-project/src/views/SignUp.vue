<template>
  <div class="SignUp-Page-Wrapper">
    
    <BaseNotification
      v-model:show="showNotification"
      :type="notificationType"
      :duration="3000"
    >
      {{ notificationMessage }}
    </BaseNotification>

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
import BaseNotification from '../components/ui/BaseNotification.vue'
import type { UserSignupRequest } from '../types/bindings/UserSignupRequest'
import router from '@/router'

const showNotification = ref(false)
const notificationMessage = ref('')
const notificationType = ref<'success' | 'error' | 'info'>('info')

const username = ref('')
const email = ref('')
const password = ref('')
const confirmPassword = ref('')
const signup_url = `${import.meta.env.VITE_API_BASE}/signup`

async function signup() {
  if (password.value !== confirmPassword.value) {
    notificationMessage.value = 'Passwords do not match!'
    notificationType.value = 'error'
    showNotification.value = true
    return
  }

  const sign_up_request: UserSignupRequest = {
    email: email.value,
    password: password.value,
    name: username.value
  }

  const response: Response = await fetch(signup_url, {
    method: "POST",
    headers: {
      'Content-Type': 'application/json',
      'Accept': 'text/plain'
    },
    body: JSON.stringify(sign_up_request)
  })

  const message = await response.text()

  if (response.ok) {
    notificationMessage.value = message || 'Signup successful!'
    notificationType.value = 'success'
    router.push("/login")
  } else {
    notificationMessage.value = message || 'Signup failed'
    notificationType.value = 'error'
  }

  showNotification.value = true
}
</script>

<style scoped>
.SignUp-Page-Wrapper {
  display: flex;
  justify-content: center;
  align-items: center;
  height: 100vh;
  background-color: #f0f4f9;
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
  border-radius: 4px;
  box-shadow: 0 2px 12px rgba(0, 53, 128, 0.1);
  border-top: 3px solid #003580;
  text-align: center;
}

/* Header */
.SignUp-View h2 {
  margin: 0;
  font-size: 20px;
  color: #003580;
  font-weight: 600;
}

/* Input fields */
.SignUp-Input-Fields {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.signup-btn {
  background: #003580;
}
</style>
