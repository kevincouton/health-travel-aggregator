import { beforeEach, describe, expect, it, vi } from 'vitest'

import { useProvider } from '../../composables/useProvider'

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

describe('useProvider', () => {
  it('getMyClinics fetches with credentials', async () => {
    fetchMock.mockResolvedValue([])
    const { getMyClinics } = useProvider()
    await getMyClinics()
    expect(fetchMock).toHaveBeenCalledWith('http://api.test/me/clinics', {
      credentials: 'include',
    })
  })

  it('createClinic posts the payload', async () => {
    fetchMock.mockResolvedValue({ id: 'c1' })
    const payload = {
      name: 'Clinic',
      slug: 'clinic',
      country_code: 'TR',
      city: 'Istanbul',
      accreditations: ['JCI'],
      description: null,
    }
    const { createClinic } = useProvider()
    await createClinic(payload)
    expect(fetchMock).toHaveBeenCalledWith('http://api.test/me/clinics', {
      method: 'POST',
      body: payload,
      credentials: 'include',
    })
  })

  it('updateClinic patches the clinic by id', async () => {
    fetchMock.mockResolvedValue({ id: 'c1' })
    const payload = {
      name: 'Clinic',
      slug: 'clinic',
      country_code: 'TR',
      city: 'Istanbul',
      accreditations: [],
      description: null,
    }
    const { updateClinic } = useProvider()
    await updateClinic('c1', payload)
    expect(fetchMock).toHaveBeenCalledWith('http://api.test/me/clinics/c1', {
      method: 'PATCH',
      body: payload,
      credentials: 'include',
    })
  })

  it('getMyPackages scopes to the clinic', async () => {
    fetchMock.mockResolvedValue([])
    const { getMyPackages } = useProvider()
    await getMyPackages('c1')
    expect(fetchMock).toHaveBeenCalledWith('http://api.test/me/clinics/c1/packages', {
      credentials: 'include',
    })
  })

  it('createPackage posts to the clinic packages collection', async () => {
    fetchMock.mockResolvedValue({ id: 'p1' })
    const payload = { treatment_id: 't1', name: 'Pkg', inclusions: [], exclusions: [] }
    const { createPackage } = useProvider()
    await createPackage('c1', payload)
    expect(fetchMock).toHaveBeenCalledWith('http://api.test/me/clinics/c1/packages', {
      method: 'POST',
      body: payload,
      credentials: 'include',
    })
  })

  it('updatePackage patches and deletePackage deletes within the clinic scope', async () => {
    fetchMock.mockResolvedValue({ id: 'p1' })
    const { updatePackage, deletePackage } = useProvider()
    const payload = {
      name: 'Pkg',
      price_min: 100,
      price_max: 200,
      duration_days: 5,
      inclusions: [],
      exclusions: [],
      is_published: true,
    }
    await updatePackage('c1', 'p1', payload)
    expect(fetchMock).toHaveBeenCalledWith('http://api.test/me/clinics/c1/packages/p1', {
      method: 'PATCH',
      body: payload,
      credentials: 'include',
    })

    fetchMock.mockResolvedValue({ ok: true })
    await deletePackage('c1', 'p1')
    expect(fetchMock).toHaveBeenCalledWith('http://api.test/me/clinics/c1/packages/p1', {
      method: 'DELETE',
      credentials: 'include',
    })
  })
})
