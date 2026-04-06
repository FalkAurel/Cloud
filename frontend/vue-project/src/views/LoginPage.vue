<template>
  <div class="Login-Page-Wrapper">
    <div class="Login-View">
    <BaseNotification
      v-model:show="showNotification"
      :type="notificationType"
      :duration="3000"
    >
      {{ notificationMessage }}
    </BaseNotification>


      <div class="Login-Input-Fields">
        <BaseInput v-model.trim="email" placeholder="Email" />
        <BaseInput v-model.trim="password" placeholder="Password" type="password" />
      </div>

      <BaseButton :disabled="!email || !password || isLoading" @click="login"> Login </BaseButton>
      <CallToAction text="Don't have an account? Sign up" route="/signup" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import BaseInput from '../components/ui/BaseInput.vue'
import BaseButton from '../components/ui/BaseButton.vue'
import CallToAction from '../components/ui/CallToAction.vue'
import router from '@/router'
import type { UserLoginRequest } from '@/types/bindings/UserLoginRequest'
import BaseNotification from '@/components/ui/BaseNotification.vue'


const showNotification = ref(false)
const notificationMessage = ref('')
const notificationType = ref<'success' | 'error' | 'info'>('info')
const email = ref('')
const password = ref('')
const isLoading = ref(false)
const login_url: string = "http://localhost:8000/login"

async function login() {
  if (isLoading.value) return
  isLoading.value = true
  const login_request: UserLoginRequest = {
    email: email.value,
    password: password.value
  }

  try {
    const response: Response = await fetch(login_url, {
      method: "POST",
      headers: {
        'Content-Type': 'application/json',
        'Accept': 'text/plain'
      },
      body: JSON.stringify(login_request),
      credentials: 'include',
    })

    const message: string = await response.text()
    if (response.status === 200) {
      notificationMessage.value = message || "Login Successful"
      notificationType.value = 'success'
      showNotification.value = true
      router.push('/home')
    } else {
      notificationMessage.value = message
      notificationType.value = 'info'
      showNotification.value = true
    }
  } catch (e: unknown) {
      notificationMessage.value = e instanceof Error ? e.message : String(e)
      notificationType.value = 'error'
      showNotification.value = true
  } finally {
    isLoading.value = false
  }
}
</script>

<style scoped>
.Login-Page-Wrapper {
  display: flex;
  justify-content: center; /* horizontal centering */
  align-items: center; /* vertical centering */
  height: 100vh; /* full viewport height */
  background-color: #f9fafb; /* optional light background */
}

/* Actual login card */
.Login-View {
  display: flex;
  flex-direction: column;
  gap: 12px;
  width: 90%;
  max-width: 400px;
  padding: 24px;
  background-color: #ffffff;
  border-radius: 12px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.08);
}

.Login-Input-Fields {
  display: flex;
  flex-direction: column;
  gap: 12px;
}
</style>
