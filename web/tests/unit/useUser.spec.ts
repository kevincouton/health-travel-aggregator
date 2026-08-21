import { beforeEach, describe, expect, it, vi } from 'vitest'

import { useUser } from '../../composables/useUser'

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

beforeEach(() => {
  fetchMock.mockReset()
})

describe('useUser', () => {
  it('starts logged out and is not premium', () => {
    const { user, isPremium } = useUser()
    expect(user.value).toBeNull()
    expect(isPremium.value).toBe(false)
  })

  it('fetchUser stores the user on success', async () => {
    fetchMock.mockResolvedValue({
      id: 'u1',
      email: 'a@b.c',
      display_name: 'Alice',
      groups: ['users'],
      premium: true,
    })
    const { user, isPremium, fetchUser } = useUser()
    await fetchUser()
    expect(fetchMock).toHaveBeenCalledWith('http://api.test/auth/me', { credentials: 'include' })
    expect(user.value?.email).toBe('a@b.c')
    expect(isPremium.value).toBe(true)
  })

  it('fetchUser clears the user on 401', async () => {
    fetchMock.mockRejectedValue(new Error('401'))
    const { user, fetchUser } = useUser()
    await fetchUser()
    expect(user.value).toBeNull()
  })

  it('group membership counts as premium', async () => {
    fetchMock.mockResolvedValue({
      id: 'u1',
      email: 'a@b.c',
      display_name: 'Alice',
      groups: ['premium'],
      premium: false,
    })
    const { isPremium, fetchUser } = useUser()
    await fetchUser()
    expect(isPremium.value).toBe(true)
  })

  it('login redirects to the API login endpoint', () => {
    // jsdom does not implement navigation; swap location for a plain object.
    const original = window.location
    // @ts-expect-error -- test-only location swap
    delete window.location
    // @ts-expect-error -- test-only location swap
    window.location = { href: '' }
    try {
      const { login } = useUser()
      login()
      expect(window.location.href).toBe('http://api.test/auth/login')
    } finally {
      window.location = original
    }
  })
})
