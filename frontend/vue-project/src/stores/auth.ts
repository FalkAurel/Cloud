import { defineStore } from 'pinia'

type User = {
  email: string
  name: string
  is_admin: boolean
}

export const useAuthStore = defineStore('auth', {
  state: (): { user: User | null; isAuthenticated: boolean } => ({
    user: null,
    isAuthenticated: false,
  }),
  actions: {
    login(user: User) {
      this.user = user
      this.isAuthenticated = true
    },
    logout() {
      this.user = null
      this.isAuthenticated = false
    },
  },
})
