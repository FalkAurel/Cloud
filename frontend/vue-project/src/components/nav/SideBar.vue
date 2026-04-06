<template>
  <nav :class="['sidebar', { expanded }]">
    <div class="sidebar-brand">
      <div class="brand-icon">CL</div>
      <span class="label" v-if="expanded">Cloud</span>
    </div>

    <ul>
      <li v-for="item in items" v-bind:key="item.name" @click="navigate(item.route)">
        <span class="icon">
          <img :src="item.icon" alt="icon" />
        </span>
        <span class="label" v-if="expanded">{{ item.label }}</span>
      </li>
    </ul>

    <footer class="sidebar-footer" v-if="expanded">
      © 2024 Cloud
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
import HomeIcon from '@/assets/navigation/home.svg?url'
import UserIcon from '@/assets/navigation/user.svg?url'
import GroupIcon from '@/assets/navigation/group.svg?url'
import SettingsIcon from '@/assets/navigation/settings.svg?url'
import AdminIcon from '@/assets/navigation/admin.svg?url'

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
  width: 56px;
  background-color: #003580;
  color: white;
  display: flex;
  flex-direction: column;
  transition: width 0.25s ease;
  overflow: hidden;
  flex-shrink: 0;
}

.sidebar.expanded {
  width: 220px;
}

.sidebar-brand {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 16px 10px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  min-height: 56px;
}

.brand-icon {
  width: 36px;
  height: 36px;
  background: white;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  font-weight: 700;
  font-size: 14px;
  color: #003580;
  letter-spacing: -0.5px;
}

.brand-name {
  font-weight: 600;
  font-size: 15px;
  white-space: nowrap;
  letter-spacing: 0.01em;
}

.sidebar ul {
  list-style: none;
  padding: 8px 0;
  margin: 0;
  flex: 1;
}

.sidebar li {
  display: flex;
  align-items: center;
  padding: 0 10px;
  height: 44px;
  cursor: pointer;
  transition: background 0.15s ease;
  border-left: 3px solid transparent;
  gap: 12px;
}

.sidebar li:hover {
  background-color: rgba(255, 255, 255, 0.1);
  border-left-color: rgba(255, 255, 255, 0.4);
}

.icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 33px;
  height: 33px;
  flex-shrink: 0;
}

.icon img {
  width: 20px;
  height: 20px;
  filter: brightness(0) invert(1);
  opacity: 0.85;
}

.sidebar li:hover .icon img {
  opacity: 1;
}

.label {
  font-size: 14px;
  font-weight: 500;
  white-space: nowrap;
  opacity: 0.9;
}

.sidebar-footer {
  padding: 12px 10px;
  font-size: 11px;
  color: rgba(255, 255, 255, 0.4);
  border-top: 1px solid rgba(255, 255, 255, 0.1);
  white-space: nowrap;
}

.toggle-btn {
  position: absolute;
  bottom: 16px;
  right: 10px;
  background: rgba(255, 255, 255, 0.1);
  border: none;
  color: white;
  font-size: 12px;
  cursor: pointer;
  padding: 6px 8px;
  border-radius: 4px;
  transition: background 0.15s ease;
}

.toggle-btn:hover {
  background: rgba(255, 255, 255, 0.2);
}
</style>
