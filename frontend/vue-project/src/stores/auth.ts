import { defineStore } from 'pinia'

type User = {
  email: string
  jwt: string
  name: string
  is_admin: boolean
}

export const useAuthStore = defineStore('auth', {
  state: (): { user: User | null } => ({
    user: null, // Pinia automatically makes this reactive
  }),
  actions: {
    login(user: User) {
      this.user = user
    },
    logout() {
      this.user = null
    },
  },
})
