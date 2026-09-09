import { beforeEach, describe, expect, it, vi } from 'vitest'

import { homeForRole, useUser } from '../../composables/useUser'

const fetchMock = vi.hoisted(() => vi.fn())

// The composable imports $fetch from 'ofetch' explicitly; mock it there.
vi.mock('ofetch', () => ({
  $fetch: (...args: unknown[]) => fetchMock(...args),
}))

// Under @nuxt/test-utils the Nuxt auto-imports in the composable resolve to
// real nuxt modules via the '#app' alias; mock them there (mockNuxtImport
// itself is not loadable under the vite-plus runner).
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

const PROVIDER = {
  id: 'u1',
  email: 'provider@example.com',
  role: 'provider_admin',
  display_name: null,
  email_verified_at: '2026-01-01T00:00:00Z',
  created_at: '2026-01-01T00:00:00Z',
}

beforeEach(() => {
  fetchMock.mockReset()
})

describe('homeForRole', () => {
  it('routes each role to its home area', () => {
    expect(homeForRole('provider_admin')).toBe('/dashboard')
    expect(homeForRole('platform_admin')).toBe('/admin')
    expect(homeForRole('patient')).toBe('/account')
    expect(homeForRole(undefined)).toBe('/account')
  })
})

describe('useUser', () => {
  it('starts logged out and not loaded', () => {
    const { user, loaded, isLoggedIn, isProvider } = useUser()
    expect(user.value).toBeNull()
    expect(loaded.value).toBe(false)
    expect(isLoggedIn.value).toBe(false)
    expect(isProvider.value).toBe(false)
  })

  it('fetchUser stores the user on success', async () => {
    fetchMock.mockResolvedValue({ ...PROVIDER })
    const { user, loaded, isProvider, isAdmin, fetchUser } = useUser()
    await fetchUser()
    expect(fetchMock).toHaveBeenCalledWith('http://api.test/auth/me', { credentials: 'include' })
    expect(user.value?.email).toBe('provider@example.com')
    expect(isProvider.value).toBe(true)
    expect(isAdmin.value).toBe(false)
    expect(loaded.value).toBe(true)
  })

  it('fetchUser clears the user on 401 but still marks loaded', async () => {
    fetchMock.mockRejectedValue(new Error('401'))
    const { user, loaded, fetchUser } = useUser()
    await fetchUser()
    expect(user.value).toBeNull()
    expect(loaded.value).toBe(true)
  })

  it('login posts credentials and stores the returned user', async () => {
    fetchMock.mockResolvedValue({ token: 't', user: { ...PROVIDER } })
    const { user, isLoggedIn, login } = useUser()
    const result = await login('provider@example.com', 'Password123!')
    expect(fetchMock).toHaveBeenCalledWith('http://api.test/auth/provider/login', {
      method: 'POST',
      body: { email: 'provider@example.com', password: 'Password123!' },
      credentials: 'include',
    })
    expect(result.role).toBe('provider_admin')
    expect(isLoggedIn.value).toBe(true)
    expect(user.value?.id).toBe('u1')
  })

  it('register posts to the provider register endpoint', async () => {
    fetchMock.mockResolvedValue({ token: 't', user: { ...PROVIDER } })
    const { user, register } = useUser()
    await register('new@clinic.example', 'Password123!')
    expect(fetchMock).toHaveBeenCalledWith('http://api.test/auth/provider/register', {
      method: 'POST',
      body: { email: 'new@clinic.example', password: 'Password123!' },
      credentials: 'include',
    })
    expect(user.value?.email).toBe('provider@example.com')
  })

  it('requestMagicLink posts the email without touching user state', async () => {
    fetchMock.mockResolvedValue({ ok: true })
    const { user, requestMagicLink } = useUser()
    await requestMagicLink('patient@example.com')
    expect(fetchMock).toHaveBeenCalledWith('http://api.test/auth/magic-link', {
      method: 'POST',
      body: { email: 'patient@example.com' },
      credentials: 'include',
    })
    expect(user.value).toBeNull()
  })

  it('verifyMagicLink stores the patient user on success', async () => {
    const patient = { ...PROVIDER, role: 'patient', email: 'patient@example.com' }
    fetchMock.mockResolvedValue({ token: 't', user: patient })
    const { user, isPatient, verifyMagicLink } = useUser()
    await verifyMagicLink('token-123')
    expect(fetchMock).toHaveBeenCalledWith('http://api.test/auth/magic-link/verify', {
      method: 'POST',
      body: { token: 'token-123' },
      credentials: 'include',
    })
    expect(user.value?.email).toBe('patient@example.com')
    expect(isPatient.value).toBe(true)
  })

  it('verifyMagicLink propagates API errors', async () => {
    fetchMock.mockRejectedValue(new Error('401'))
    const { verifyMagicLink } = useUser()
    await expect(verifyMagicLink('bad-token')).rejects.toThrow('401')
  })

  it('logout clears the user even when the request fails', async () => {
    fetchMock.mockResolvedValueOnce({ token: 't', user: { ...PROVIDER } })
    const { user, isLoggedIn, login, logout } = useUser()
    await login('provider@example.com', 'Password123!')
    expect(isLoggedIn.value).toBe(true)

    fetchMock.mockRejectedValueOnce(new Error('network'))
    await logout()
    expect(user.value).toBeNull()
    expect(isLoggedIn.value).toBe(false)
  })
})
