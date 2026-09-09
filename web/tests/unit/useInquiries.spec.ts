import { beforeEach, describe, expect, it, vi } from 'vitest'

import { useInquiries } from '../../composables/useInquiries'

const fetchMock = vi.hoisted(() => vi.fn())

vi.mock('ofetch', () => ({
  $fetch: (...args: unknown[]) => fetchMock(...args),
}))

vi.mock('#app', async (importOriginal) => {
  const { ref } = await import('vue')
  return {
    ...(await importOriginal<object>()),
    useRuntimeConfig: () => ({ public: { apiUrl: 'http://api.test' } }),
    useState: (_key: string, init: () => unknown) => ref(init()),
  }
})

vi.mock('#app/nuxt', async (importOriginal) => ({
  ...(await importOriginal<object>()),
  useRuntimeConfig: () => ({ public: { apiUrl: 'http://api.test' } }),
}))

vi.mock('#app/composables/state', async () => {
  const { ref } = await import('vue')
  return { useState: (_key: string, init: () => unknown) => ref(init()) }
})

beforeEach(() => {
  fetchMock.mockReset()
})

describe('useInquiries', () => {
  it('createInquiry posts with the session cookie', async () => {
    fetchMock.mockResolvedValue({ id: 'i1' })
    const payload = {
      clinic_id: 'c1',
      package_id: null,
      contact_email: 'patient@example.com',
      medical_notes: 'notes',
      preferred_dates: null,
    }
    const { createInquiry } = useInquiries()
    await createInquiry(payload)
    expect(fetchMock).toHaveBeenCalledWith('http://api.test/inquiries', {
      method: 'POST',
      body: payload,
      credentials: 'include',
    })
  })

  it('getMyInquiries fetches the role-aware list', async () => {
    fetchMock.mockResolvedValue([])
    const { getMyInquiries } = useInquiries()
    await getMyInquiries()
    expect(fetchMock).toHaveBeenCalledWith('http://api.test/me/inquiries', {
      credentials: 'include',
    })
  })

  it('updateInquiryStatus patches the status', async () => {
    fetchMock.mockResolvedValue({ id: 'i1', status: 'contacted' })
    const { updateInquiryStatus } = useInquiries()
    await updateInquiryStatus('i1', 'contacted')
    expect(fetchMock).toHaveBeenCalledWith('http://api.test/me/inquiries/i1/status', {
      method: 'PATCH',
      body: { status: 'contacted' },
      credentials: 'include',
    })
  })
})
