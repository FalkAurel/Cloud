<template>
  <div class="layout">
    <SideBar />

    <BaseNotification :show="showError" type="error" :duration="5000" @update:show="showError = $event">
      {{ error }}
    </BaseNotification>

    <div class="main">
      <header class="topbar">
        <div class="topbar-left">
          <h1 class="page-title">Dashboard</h1>
          <span class="greeting">{{ timeOfDay() }}, {{ profile.name || '…' }}</span>
        </div>
        <div class="topbar-right">
          <span class="status-dot online" />
          <span class="status-label">Verbunden</span>
        </div>
      </header>

      <div v-if="loading" class="loading-state">
        <BaseSpinner />
        <span>Profil wird geladen…</span>
      </div>

      <NotAuthorized v-else-if="error" :message="error" @retry="fetchProfile" />

      <div v-else class="dashboard">

        <!-- Row 1: Profile card + Stats -->
        <div class="row">
          <div class="card profile-card">
            <div class="avatar">{{ initials }}</div>
            <div class="profile-info">
              <h2 class="profile-name">{{ profile.name }}</h2>
              <p class="profile-email">{{ profile.email }}</p>
              <BaseBadge :variant="profile.is_admin ? 'admin' : 'user'">
                {{ profile.is_admin ? 'Administrator' : 'Benutzer' }}
              </BaseBadge>
            </div>
          </div>

          <div class="card stat-card">
            <div class="stat-label">Genutzt</div>
            <div class="stat-value">{{ returnBytesFormated(usedBytes) }}</div>
            <div class="stat-sub">von {{ returnBytesFormated(totalBytes) }}</div>
          </div>

          <div class="card stat-card">
            <div class="stat-label">Verfügbar</div>
            <div class="stat-value">{{ returnBytesFormated(totalBytes - usedBytes) }}</div>
            <div class="stat-sub">{{ freePercent }}% frei</div>
          </div>

          <div class="card stat-card">
            <div class="stat-label">Dateien</div>
            <div class="stat-value">{{ fileCount }}</div>
            <div class="stat-sub">gespeichert</div>
          </div>
        </div>

        <!-- Row 2: Storage gauge + Activity -->
        <div class="row row-mid">
          <div class="card gauge-card">
            <h3 class="card-title">Speichernutzung</h3>
            <CircularGauge :percentage="usedPercent" :size="180" :strokeWidth="10">
              Speicher
            </CircularGauge>
            <div class="gauge-legend">
              <div class="legend-row">
                <span class="legend-dot dot-used" />
                <span>Genutzt — {{ returnBytesFormated(usedBytes) }}</span>
              </div>
              <div class="legend-row">
                <span class="legend-dot dot-free" />
                <span>Frei — {{ returnBytesFormated(totalBytes - usedBytes) }}</span>
              </div>
            </div>
          </div>

          <div class="card account-card">
            <h3 class="card-title">Kontoinformationen</h3>
            <div class="info-list">
              <div class="info-row">
                <span class="info-label">Benutzername</span>
                <span class="info-value">{{ profile.name }}</span>
              </div>
              <div class="info-row">
                <span class="info-label">E-Mail</span>
                <span class="info-value">{{ profile.email }}</span>
              </div>
              <div class="info-row">
                <span class="info-label">Rolle</span>
                <span class="info-value">{{ profile.is_admin ? 'Administrator' : 'Benutzer' }}</span>
              </div>
              <div class="info-row">
                <span class="info-label">Mitglied seit</span>
                <span class="info-value">{{ memberSince }}</span>
              </div>

            </div>
          </div>
        </div>

        <!-- Row 3: Storage bar + Quick actions -->
        <div class="row row-bottom">
          <div class="card storage-bar-card">
            <h3 class="card-title">Speicherkapazität</h3>
            <div class="storage-segments">
              <div
                v-for="seg in storageSegments"
                :key="seg.label"
                class="segment"
                :style="{ width: seg.percent + '%', background: seg.color }"
                :title="`${seg.label}: ${returnBytesFormated(seg.bytes)}`"
              />
            </div>
            <div class="segment-legend">
              <div v-for="seg in storageSegments" :key="seg.label" class="seg-legend-item">
                <span class="seg-dot" :style="{ background: seg.color }" />
                <span class="seg-text">{{ seg.label }}</span>
                <span class="seg-size">{{ returnBytesFormated(seg.bytes) }}</span>
              </div>
            </div>
          </div>

          <div class="card actions-card">
            <h3 class="card-title">Schnellzugriff</h3>
            <div class="actions-grid">
              <button class="action-btn" @click="navigate('/home')">
                <span class="action-icon">⌂</span>
                <span>Dateien</span>
              </button>
              <button class="action-btn" @click="navigate('/groups')">
                <span class="action-icon">◎</span>
                <span>Gruppen</span>
              </button>
              <button class="action-btn" @click="navigate('/settings')">
                <span class="action-icon">⚙</span>
                <span>Einstellungen</span>
              </button>
              <button class="action-btn action-btn-danger" @click="logout">
                <span class="action-icon">→</span>
                <span>Abmelden</span>
              </button>
              <button class="action-btn action-btn-danger" @click="showDeleteModal = true">
                <span class="action-icon">✕</span>
                <span>Konto löschen</span>
              </button>
            </div>
          </div>
        </div>

      </div>
    </div>
  </div>

  <div v-if="showDeleteModal" class="modal-overlay" @click.self="showDeleteModal = false">
    <div class="modal">
      <h3 class="modal-title">Konto löschen</h3>
      <p class="modal-body">Möchtest du dein Konto wirklich dauerhaft löschen? Diese Aktion kann nicht rückgängig gemacht werden.</p>
      <div class="modal-actions">
        <button class="modal-btn modal-btn-cancel" @click="showDeleteModal = false">Abbrechen</button>
        <button class="modal-btn modal-btn-confirm" :disabled="deleting" @click="deleteAccount">
          {{ deleting ? 'Wird gelöscht…' : 'Endgültig löschen' }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import SideBar from '@/components/nav/SideBar.vue'
import CircularGauge from '@/components/ui/CircularGauge.vue'
import BaseNotification from '@/components/ui/BaseNotification.vue'
import BaseBadge from '@/components/ui/BaseBadge.vue'
import BaseSpinner from '@/components/ui/BaseSpinner.vue'
import NotAuthorized from '@/components/ui/NotAuthorized.vue'
import { returnBytesFormated } from '@/utils/format'
import { useAuthStore } from '@/stores/auth'
import type { StandardUserView } from '@/types/bindings/StandardUserView'
import { timeOfDay } from '@/utils/timeOfDay'

defineOptions({ name: 'UserProfile' })

const router = useRouter()
const authStore = useAuthStore()

const profile = ref<StandardUserView>({
  id: 0,
  name: '',
  email: '',
  is_admin: false,
  created_at: '',
  modified_at: ''
})
const loading = ref(true)
const error = ref<string | null>(null)
const showError = ref(false)

const meEndpoint: string = `${import.meta.env.VITE_API_BASE}/me`;
const logOutEndpoint: string = `${import.meta.env.VITE_API_BASE}/logout`;

const showDeleteModal = ref(false)
const deleting = ref(false)

async function deleteAccount() {
  deleting.value = true
  const deleteEndpoint = `${import.meta.env.VITE_API_BASE}/users/${profile.value.id}`
  try {
    const res = await fetch(deleteEndpoint, {
      method: 'DELETE',
      credentials: 'include',
    })
    if (!res.ok) {
      error.value = 'Konto konnte nicht gelöscht werden. Bitte erneut versuchen.'
      showError.value = true
      showDeleteModal.value = false
      return
    }
    authStore.logout()
    router.push('/login')
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Unbekannter Fehler'
    showError.value = true
    showDeleteModal.value = false
  } finally {
    deleting.value = false
  }
}

async function fetchProfile() {
  loading.value = true
  error.value = null
  showError.value = false
  try {
    const res = await fetch(meEndpoint, {
      method: 'GET',
      credentials: 'include',
    })

    if (!res.ok) {
      const messages: Record<number, string> = {
        401: 'Nicht autorisiert. Bitte erneut anmelden.',
        403: 'Zugriff verweigert.',
        404: 'Benutzer nicht gefunden.',
        500: 'Serverfehler. Bitte später erneut versuchen.',
      }
      error.value = messages[res.status] ?? `Fehler ${res.status}`
      showError.value = true
      return
    }
    profile.value = await res.json()
    memberSince.value = new Date(profile.value.created_at).toLocaleDateString('de-DE', {
      day: '2-digit',
      month: 'long',
      year: 'numeric',
    })

  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Unbekannter Fehler'
    showError.value = true
  } finally {
    loading.value = false
  }
}

onMounted(fetchProfile)

// Placeholder storage values
const usedBytes = ref(1_400_000_000)
const totalBytes = ref(2_000_000_000)
const fileCount = ref(42)

const usedPercent = computed(() => (usedBytes.value / totalBytes.value) * 100)
const freePercent = computed(() => (100 - usedPercent.value).toFixed(0))

const initials = computed(() =>
  profile.value.name
    .split(' ')
    .map((w) => w[0])
    .join('')
    .toUpperCase()
    .slice(0, 2) || '?'
)

let memberSince = ref("")

const storageSegments = computed(() => [
  { label: 'Dokumente', bytes: 600_000_000, percent: 30, color: '#003580' },
  { label: 'Bilder', bytes: 500_000_000, percent: 25, color: '#0070d2' },
  { label: 'Videos', bytes: 200_000_000, percent: 10, color: '#a0b4d6' },
  { label: 'Sonstiges', bytes: 100_000_000, percent: 5, color: '#dde3ed' },
  { label: 'Frei', bytes: totalBytes.value - usedBytes.value, percent: 30, color: '#f0f4f9' },
])

function navigate(route: string) {
  router.push(route)
}

function logout() {
  authStore.logout()
  fetch(
    logOutEndpoint,
    {
      method: 'POST',
      credentials: 'include',
    }
  )
  router.push('/login')
}
</script>

<style scoped>
/* ── Layout ─────────────────────────────────────────────────── */
.layout {
  display: flex;
  height: 100vh;
  background: #f0f4f9;
}

.main {
  display: flex;
  flex-direction: column;
  flex: 1;
  overflow: hidden;
}

/* ── Topbar ─────────────────────────────────────────────────── */
.topbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 28px;
  height: 56px;
  background: white;
  border-bottom: 1px solid #dde3ed;
  flex-shrink: 0;
}

.topbar-left {
  display: flex;
  align-items: baseline;
  gap: 12px;
}

.page-title {
  font-size: 16px;
  font-weight: 600;
  color: #003580;
  margin: 0;
  letter-spacing: 0.01em;
}

.greeting {
  font-size: 13px;
  color: #6b7a90;
}

.topbar-right {
  display: flex;
  align-items: center;
  gap: 6px;
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #a0b4d6;
}

.status-dot.online {
  background: #22c55e;
}

.status-label {
  font-size: 12px;
  color: #6b7a90;
}

/* ── Loading / Error ────────────────────────────────────────── */
.loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  flex: 1;
  gap: 12px;
  color: #6b7a90;
  font-size: 14px;
}

/* ── Dashboard grid ─────────────────────────────────────────── */
.dashboard {
  flex: 1;
  overflow-y: auto;
  padding: 24px 28px;
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.row {
  display: flex;
  gap: 20px;
  flex-wrap: wrap;
}

/* ── Shared card ────────────────────────────────────────────── */
.card {
  background: white;
  border-radius: 10px;
  padding: 20px;
  box-shadow: 0 2px 12px rgba(0, 53, 128, 0.08);
  border: 1px solid #e6ecf5;
}

.card-title {
  font-size: 13px;
  font-weight: 600;
  color: #003580;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin: 0 0 16px;
}

/* ── Profile card ───────────────────────────────────────────── */
.profile-card{
  display: flex;
  align-items: center;
  gap: 16px;
  flex: 2;
  min-width: 220px;
}

.avatar {
  width: 56px;
  height: 56px;
  border-radius: 10px;
  background: #003580;
  color: white;
  font-size: 20px;
  font-weight: 700;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  letter-spacing: -0.5px;
}

.profile-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.profile-name {
  font-size: 18px;
  font-weight: 600;
  color: #1a2b4b;
  margin: 0;
}

.profile-email {
  font-size: 13px;
  color: #6b7a90;
  margin: 0;
}

/* ── Stat cards ─────────────────────────────────────────────── */
.stat-card {
  flex: 1;
  min-width: 120px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.stat-label {
  font-size: 12px;
  font-weight: 600;
  color: #6b7a90;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.stat-value {
  font-size: 22px;
  font-weight: 700;
  color: #1a2b4b;
  line-height: 1.1;
}

.stat-sub {
  font-size: 12px;
  color: #a0b4d6;
}

/* ── Gauge card ─────────────────────────────────────────────── */
.gauge-card {
  flex: 1;
  min-width: 220px;
  display: flex;
  flex-direction: column;
  align-items: center;
}

.gauge-legend {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-top: 16px;
  width: 100%;
}

.legend-row {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  color: #6b7a90;
}

.legend-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  flex-shrink: 0;
}

.dot-used { background: #003580; }
.dot-free { background: #e6ecf5; }

/* ── Account card ───────────────────────────────────────────── */
.account-card {
  flex: 1.5;
  min-width: 220px;
}

.info-list {
  display: flex;
  flex-direction: column;
}

.info-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 9px 0;
  border-bottom: 1px solid #eef2f7;
}

.info-row:last-child {
  border-bottom: none;
}

.info-label {
  font-size: 12px;
  color: #6b7a90;
}

.info-value {
  font-size: 13px;
  font-weight: 600;
  color: #1a2b4b;
}

/* ── Storage bar card ───────────────────────────────────────── */
.storage-bar-card {
  flex: 2;
  min-width: 260px;
}

.storage-segments {
  display: flex;
  height: 12px;
  border-radius: 6px;
  overflow: hidden;
  gap: 2px;
  margin-bottom: 16px;
}

.segment {
  border-radius: 3px;
  transition: opacity 0.15s ease;
}

.segment:hover {
  opacity: 0.75;
}

.segment-legend {
  display: flex;
  flex-wrap: wrap;
  gap: 10px 20px;
}

.seg-legend-item {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: #6b7a90;
}

.seg-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  flex-shrink: 0;
}

.seg-text {
  color: #1a2b4b;
}

.seg-size {
  color: #a0b4d6;
}

/* ── Actions card ───────────────────────────────────────────── */
.actions-card {
  flex: 1;
  min-width: 180px;
}

.actions-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 10px;
}

.action-btn {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 14px 8px;
  border-radius: 8px;
  border: 1px solid #e6ecf5;
  background: #f8fafd;
  color: #1a2b4b;
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: background 0.15s ease, border-color 0.15s ease, transform 0.1s ease;
}

.action-btn:hover {
  background: #e8effa;
  border-color: #a0b4d6;
  transform: translateY(-1px);
}

.action-icon {
  font-size: 18px;
  color: #003580;
}

.action-btn-danger {
  color: #c0392b;
}

.action-btn-danger .action-icon {
  color: #c0392b;
}

.action-btn-danger:hover {
  background: #fef2f2;
  border-color: #f5b7b1;
}

/* ── Delete modal ───────────────────────────────────────────── */
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.4);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
}

.modal {
  background: white;
  border-radius: 12px;
  padding: 28px;
  width: 100%;
  max-width: 400px;
  box-shadow: 0 8px 32px rgba(0, 53, 128, 0.18);
}

.modal-title {
  font-size: 17px;
  font-weight: 700;
  color: #1a2b4b;
  margin: 0 0 12px;
}

.modal-body {
  font-size: 14px;
  color: #6b7a90;
  line-height: 1.5;
  margin: 0 0 24px;
}

.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}

.modal-btn {
  padding: 9px 18px;
  border-radius: 7px;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  border: 1px solid transparent;
  transition: background 0.15s ease, opacity 0.15s ease;
}

.modal-btn-cancel {
  background: #f0f4f9;
  border-color: #dde3ed;
  color: #1a2b4b;
}

.modal-btn-cancel:hover {
  background: #e6ecf5;
}

.modal-btn-confirm {
  background: #c0392b;
  color: white;
}

.modal-btn-confirm:hover:not(:disabled) {
  background: #a93226;
}

.modal-btn-confirm:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}
</style>
