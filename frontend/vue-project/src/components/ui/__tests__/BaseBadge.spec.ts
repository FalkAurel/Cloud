import { describe, it, expect } from 'vitest'
import { mount } from '@vue/test-utils'
import BaseBadge from '../BaseBadge.vue'

describe('BaseBadge', () => {
  it('renders slot content', () => {
    const wrapper = mount(BaseBadge, {
      props: { variant: 'user' },
      slots: { default: 'Benutzer' },
    })
    expect(wrapper.text()).toBe('Benutzer')
  })

  it('applies badge-user class for user variant', () => {
    const wrapper = mount(BaseBadge, { props: { variant: 'user' } })
    expect(wrapper.classes()).toContain('badge-user')
  })

  it('applies badge-admin class for admin variant', () => {
    const wrapper = mount(BaseBadge, { props: { variant: 'admin' } })
    expect(wrapper.classes()).toContain('badge-admin')
  })
})
