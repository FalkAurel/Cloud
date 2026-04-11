import { createRouter, createWebHistory } from 'vue-router'
import LoginPage from '@/views/LoginPage.vue'
import Home from '@/views/HomePage.vue'
import { useAuthStore } from '@/stores/auth'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      name: 'landingPage',
      redirect: '/login',
    },

    {
      path: '/login',
      name: 'login',
      component: LoginPage,
    },
    {
      path: '/signup',
      name: 'signup',
      component: () => import('@/views/SignUp.vue'),
    },
    {
      path: "/profile",
      name: "profile",
      component: () => import("@/views/Profile.vue"),
      meta: { requiresAuth: true },
    },

    {
      path: '/home',
      name: 'home',
      component: Home,
      meta: { requiresAuth: true },
    },
  ],
})

router.beforeEach(async (to) => {
  if (!to.meta.requiresAuth) return true

  const authStore = useAuthStore()
  if (authStore.isAuthenticated) return true

  try {
    const res = await fetch(`${import.meta.env.VITE_API_BASE}/me`, {
      method: 'GET',
      credentials: 'include',
    })
    if (res.ok) {
      authStore.isAuthenticated = true
      return true
    }
  } catch {
    // network error — fall through to redirect
  }

  return { name: 'login' }
})

export default router
