<template>
  <nav :class="['sidebar', { expanded }]">
    <ul>
      <li v-for="item in items" v-bind:key="item.name" @click="navigate(item.route)">
        <span class="icon">
          <img :src="item.icon" alt="icon" />
        </span>
        <span class="label" v-if="expanded">{{ item.label }}</span>
      </li>
    </ul>

    <footer class="sidebar-footer" v-if="expanded">
      <p class="about-label">About Us</p>
      <p class="company-info">© 2023 Your Company</p>
    </footer>

    <button class="toggle-btn" @click="expanded = !expanded">
      <span v-if="expanded">◀</span>
      <span v-else>▶</span>
    </button>
  </nav>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import HomeIcon from '@/assets/navigation/home.svg'
import UserIcon from '@/assets/navigation/user.svg'
import GroupIcon from '@/assets/navigation/group.svg'
import SettingsIcon from '@/assets/navigation/settings.svg'
import AdminIcon from '@/assets/navigation/admin.svg'

const expanded = ref(false)
const router = useRouter()
const authStore = useAuthStore()

interface SidebarItem {
  name: string
  label: string
  route: string
  icon: string
}

const items = computed<SidebarItem[]>(() => {
  const baseItems: SidebarItem[] = [
    { name: 'home', label: 'Home', route: '/home', icon: HomeIcon },
    { name: 'profile', label: 'Profile', route: '/profile', icon: UserIcon },
    { name: 'groups', label: 'Groups', route: '/groups', icon: GroupIcon },
    { name: 'settings', label: 'Settings', route: '/settings', icon: SettingsIcon },
  ]

  if (!authStore.user?.is_admin) {
    baseItems.push({ name: 'admin', label: 'Admin', route: '/admin', icon: AdminIcon })
  }

  return baseItems
})

function navigate(route: string) {
  router.push(route)
}
</script>

<style scoped>
.sidebar {
  position: relative;
  top: 0;
  left: 0;
  height: 100vh;
  width: 18%;
  max-width: 50px;
  background-color: #1f2937;
  color: white;
  display: flex;
  flex-direction: column;
  justify-content: flex-start;
  transition: width 0.3s ease;
  overflow: hidden;
  box-shadow: 2px 0 8px rgba(0, 0, 0, 0.15);
  padding-top: 12px;
}

.sidebar.expanded {
  width: 200px;
  max-width: 200px;
}

.sidebar ul {
  list-style: none;
  padding: 0;
  margin: 0;
  flex: 1; /* make list take available space */
}

.sidebar li {
  display: flex;
  align-items: center;
  padding: 12px;
  cursor: pointer;
  transition: background 0.2s ease;
}

.sidebar li:hover {
  background-color: #374151;
}

.icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
}

.icon img {
  max-width: 24px;
  max-height: 24px;
}

.label {
  margin-left: 12px;
  font-weight: 500;
}

.sidebar-footer {
  margin-top: auto;
  padding: 12px;
  font-size: 12px;
  text-align: center;
  background-color: #111827;
  color: #9ca3af;
}

.about-label {
  font-weight: 600;
  margin-bottom: 4px;
  font-size: 13px;
}

.company-info {
  font-size: 12px;
}

.toggle-btn {
  position: absolute;
  bottom: 12px;
  right: 12px;
  background: none;
  border: none;
  color: white;
  font-size: 18px;
  cursor: pointer;
  padding: 6px 12px;
  border-radius: 6px;
  transition: right 0.3s ease;
}

.toggle-btn:hover {
  background-color: rgba(255, 255, 255, 0.1);
}
</style>
