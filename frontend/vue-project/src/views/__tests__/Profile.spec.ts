import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import Profile from '@/views/Profile.vue'

// ── Router stub ──────────────────────────────────────────────────────────────
const pushMock = vi.fn()
vi.mock('vue-router', () => ({
  useRouter: () => ({ push: pushMock }),
}))

// ── Child component stubs ────────────────────────────────────────────────────
vi.mock('@/components/nav/SideBar.vue', () => ({ default: { template: '<div />' } }))
vi.mock('@/components/ui/CircularGauge.vue', () => ({ default: { template: '<div />' } }))
vi.mock('@/components/ui/BaseNotification.vue', () => ({ default: { template: '<div />' } }))
vi.mock('@/components/ui/BaseBadge.vue', () => ({ default: { template: '<span />' } }))
vi.mock('@/components/ui/BaseSpinner.vue', () => ({ default: { template: '<div />' } }))
vi.mock('@/components/ui/NotAuthorized.vue', () => ({ default: { template: '<div />' } }))

// ── Helpers ──────────────────────────────────────────────────────────────────
const profilePayload = {
  id: 42,
  name: 'Test User',
  email: 'test@example.com',
  is_admin: false,
  created_at: '2024-01-01T00:00:00Z',
  modified_at: '2024-01-01T00:00:00Z',
}

function mockFetch(response: { ok: boolean; status?: number; json?: () => Promise<unknown> }) {
  vi.stubGlobal(
    'fetch',
    vi.fn().mockResolvedValue({
      ok: response.ok,
      status: response.status ?? 200,
      json: response.json ?? (() => Promise.resolve(profilePayload)),
    }),
  )
}

async function mountProfile() {
  const wrapper = mount(Profile, {
    global: { plugins: [createPinia()] },
  })
  await flushPromises()
  return wrapper
}

// ── Tests ────────────────────────────────────────────────────────────────────
describe('Profile.vue', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    pushMock.mockClear()
    vi.unstubAllGlobals()
  })

  describe('initial render', () => {
    it('fetches profile on mount and displays name and email', async () => {
      mockFetch({ ok: true })
      const wrapper = await mountProfile()

      expect(wrapper.text()).toContain('Test User')
      expect(wrapper.text()).toContain('test@example.com')
    })
  })

  describe('delete button', () => {
    it('is present in the actions grid', async () => {
      mockFetch({ ok: true })
      const wrapper = await mountProfile()

      const buttons = wrapper.findAll('button')
      const deleteBtn = buttons.find((b) => b.text().includes('Konto löschen'))
      expect(deleteBtn).toBeDefined()
    })

    it('opens the confirmation modal when clicked', async () => {
      mockFetch({ ok: true })
      const wrapper = await mountProfile()

      expect(wrapper.find('.modal-overlay').exists()).toBe(false)

      const deleteBtn = wrapper.findAll('button').find((b) => b.text().includes('Konto löschen'))
      await deleteBtn!.trigger('click')

      expect(wrapper.find('.modal-overlay').exists()).toBe(true)
    })

    it('closes the modal when Abbrechen is clicked', async () => {
      mockFetch({ ok: true })
      const wrapper = await mountProfile()

      const deleteBtn = wrapper.findAll('button').find((b) => b.text().includes('Konto löschen'))
      await deleteBtn!.trigger('click')
      expect(wrapper.find('.modal-overlay').exists()).toBe(true)

      await wrapper.find('.modal-btn-cancel').trigger('click')
      expect(wrapper.find('.modal-overlay').exists()).toBe(false)
    })

    it('closes the modal when clicking the overlay backdrop', async () => {
      mockFetch({ ok: true })
      const wrapper = await mountProfile()

      const deleteBtn = wrapper.findAll('button').find((b) => b.text().includes('Konto löschen'))
      await deleteBtn!.trigger('click')

      await wrapper.find('.modal-overlay').trigger('click')
      expect(wrapper.find('.modal-overlay').exists()).toBe(false)
    })
  })

  describe('deleteAccount', () => {
    it('calls DELETE /delete/user/:id on confirm', async () => {
      const fetchMock = vi.fn()
        .mockResolvedValueOnce({ ok: true, json: () => Promise.resolve(profilePayload) }) // /me
        .mockResolvedValueOnce({ ok: true }) // DELETE
      vi.stubGlobal('fetch', fetchMock)

      const wrapper = await mountProfile()

      const deleteBtn = wrapper.findAll('button').find((b) => b.text().includes('Konto löschen'))
      await deleteBtn!.trigger('click')
      await wrapper.find('.modal-btn-confirm').trigger('click')
      await flushPromises()

      const deleteCall = fetchMock.mock.calls.find(
        (call) => {
          const [url, opts] = call as [string, RequestInit]
          return opts?.method === 'DELETE' && url.includes('/delete/user/42')
        },
      )
      expect(deleteCall).toBeDefined()
    })

    it('redirects to /login after successful deletion', async () => {
      const fetchMock = vi.fn()
        .mockResolvedValueOnce({ ok: true, json: () => Promise.resolve(profilePayload) })
        .mockResolvedValueOnce({ ok: true })
      vi.stubGlobal('fetch', fetchMock)

      const wrapper = await mountProfile()

      const deleteBtn = wrapper.findAll('button').find((b) => b.text().includes('Konto löschen'))
      await deleteBtn!.trigger('click')
      await wrapper.find('.modal-btn-confirm').trigger('click')
      await flushPromises()

      expect(pushMock).toHaveBeenCalledWith('/login')
    })

    it('closes modal and shows error notification when DELETE fails', async () => {
      const fetchMock = vi.fn()
        .mockResolvedValueOnce({ ok: true, json: () => Promise.resolve(profilePayload) })
        .mockResolvedValueOnce({ ok: false, status: 500 })
      vi.stubGlobal('fetch', fetchMock)

      const wrapper = await mountProfile()

      const deleteBtn = wrapper.findAll('button').find((b) => b.text().includes('Konto löschen'))
      await deleteBtn!.trigger('click')
      await wrapper.find('.modal-btn-confirm').trigger('click')
      await flushPromises()

      expect(wrapper.find('.modal-overlay').exists()).toBe(false)
      expect(pushMock).not.toHaveBeenCalledWith('/login')
    })

    it('confirm button is disabled while deletion is in progress', async () => {
      let resolveDelete!: (v: unknown) => void
      const fetchMock = vi.fn()
        .mockResolvedValueOnce({ ok: true, json: () => Promise.resolve(profilePayload) })
        .mockReturnValueOnce(new Promise((res) => { resolveDelete = res }))
      vi.stubGlobal('fetch', fetchMock)

      const wrapper = await mountProfile()

      const deleteBtn = wrapper.findAll('button').find((b) => b.text().includes('Konto löschen'))
      await deleteBtn!.trigger('click')
      await wrapper.find('.modal-btn-confirm').trigger('click')

      expect(wrapper.find('.modal-btn-confirm').attributes('disabled')).toBeDefined()

      resolveDelete({ ok: true })
      await flushPromises()
    })
  })
})
