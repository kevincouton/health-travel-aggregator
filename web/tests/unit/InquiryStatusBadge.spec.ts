import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'

import InquiryStatusBadge from '../../components/InquiryStatusBadge.vue'

describe('InquiryStatusBadge', () => {
  it('renders the human label for each status', () => {
    const cases = [
      ['new', 'New'],
      ['contacted', 'Contacted'],
      ['converted', 'Converted'],
      ['closed', 'Closed'],
    ] as const
    for (const [status, label] of cases) {
      const wrapper = mount(InquiryStatusBadge, { props: { status } })
      expect(wrapper.text()).toBe(label)
    }
  })

  it('falls back to the raw status for unknown values', () => {
    const wrapper = mount(InquiryStatusBadge, { props: { status: 'weird' } })
    expect(wrapper.text()).toBe('weird')
  })
})
