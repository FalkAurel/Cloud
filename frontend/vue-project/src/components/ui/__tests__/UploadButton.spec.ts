import { describe, it, expect } from 'vitest'
import { mount } from '@vue/test-utils'
import UploadButton from '../UploadButton.vue'

describe('UploadButton', () => {
  it('renders the upload button', () => {
    const wrapper = mount(UploadButton)
    expect(wrapper.find('.upload-fab').exists()).toBe(true)
  })

  it('contains a hidden file input (single file)', () => {
    const wrapper = mount(UploadButton)
    const input = wrapper.find('input[type="file"]')
    expect(input.exists()).toBe(true)
    expect(input.attributes('multiple')).toBeUndefined()
  })

  it('emits upload with the selected File', async () => {
    const wrapper = mount(UploadButton)
    const file = new File(['hello'], 'test.txt', { type: 'text/plain' })

    const input = wrapper.find('input[type="file"]')
    Object.defineProperty(input.element, 'files', { value: [file] })
    await input.trigger('change')

    expect(wrapper.emitted('upload')).toHaveLength(1)
    expect((wrapper.emitted('upload')![0] as [File])[0]).toBe(file)
  })

  it('does not emit upload when no file is selected', async () => {
    const wrapper = mount(UploadButton)
    const input = wrapper.find('input[type="file"]')
    Object.defineProperty(input.element, 'files', { value: [] })
    await input.trigger('change')

    expect(wrapper.emitted('upload')).toBeUndefined()
  })
})
