import { describe, it, expect } from 'vitest'
import { mount } from '@vue/test-utils'
import NotAuthorized from '../NotAuthorized.vue'

describe('NotAuthorized', () => {
  it('renders the message prop', () => {
    const wrapper = mount(NotAuthorized, { props: { message: 'Nicht autorisiert.' } })
    expect(wrapper.text()).toContain('Nicht autorisiert.')
  })

  it('emits retry when the button is clicked', async () => {
    const wrapper = mount(NotAuthorized, { props: { message: 'Error' } })
    await wrapper.find('.retry-btn').trigger('click')
    expect(wrapper.emitted('retry')).toHaveLength(1)
  })
})
