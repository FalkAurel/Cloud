<template>
  <div class="Login-Page-Wrapper">
    <div class="Login-View">
      <div class="Login-Input-Fields">
        <BaseInput v-model.trim="email" placeholder="Email" />
        <BaseInput v-model.trim="password" placeholder="Password" type="password" />
      </div>

      <BaseButton v-bind:disabled="!email || !password" @click="login"> Login </BaseButton>
      <CallToAction text="Don't have an account? Sign up" route="/signup" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import BaseInput from '../components/ui/BaseInput.vue'
import BaseButton from '../components/ui/BaseButton.vue'
import CallToAction from '../components/ui/CallToAction.vue'
import { useAuthStore } from '@/stores/auth'
import axios, { type AxiosResponse } from 'axios'
import router from '@/router'

const email = ref('')
const password = ref('')

async function login(): Promise<void> {
  console.log(`Attempting login with email: ${email.value}`)

  {
    let response: AxiosResponse<{ jwt: string; name: string; is_admin: boolean }>
    try {
      await axios.post('/api/login', {
        email: email.value,
        password: password.value,
      })
    } catch (error) {
      console.error(`Login error: ${error}`)
      alert('Login failed: network or server error')
      //return
    }

    //if (response.status !== 200) {
    //  alert(`Login failed: ${response.statusText}`)
    //  return
    //}

    //const authStore = useAuthStore()
    //authStore.login({
    //  email: email.value,
    //  jwt: response.data.jwt,
    //  name: response.data.name,
    //  is_admin: response.data.is_admin,
    //})

    router.push('/home')
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
