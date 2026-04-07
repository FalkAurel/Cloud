import { describe, it, expect } from 'vitest'
import { mount } from '@vue/test-utils'
import BaseSpinner from '../BaseSpinner.vue'

describe('BaseSpinner', () => {
  it('renders a div with the spinner class', () => {
    const wrapper = mount(BaseSpinner)
    expect(wrapper.find('.spinner').exists()).toBe(true)
  })
})
