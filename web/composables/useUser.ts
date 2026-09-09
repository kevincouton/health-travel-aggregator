import { computed } from 'vue'
// Imported explicitly (not Nuxt auto-import) so the composable always uses
// plain ofetch with absolute API URLs — and so unit tests can mock 'ofetch'
// (the Nuxt $fetch wrapper requires a running Nuxt app).
import { $fetch } from 'ofetch'

export type UserRole = 'patient' | 'provider_admin' | 'platform_admin'

export interface AuthUser {
  id: string
  email: string
  role: UserRole
  display_name: string | null
  email_verified_at: string | null
  created_at: string
}

interface AuthResponse {
  token: string
  user: AuthUser
}

/** Where a user lands after signing in, based on their role. */
export const homeForRole = (role: UserRole | undefined): string => {
  if (role === 'provider_admin') return '/dashboard'
  if (role === 'platform_admin') return '/admin'
  return '/account'
}

export const useUser = () => {
  const config = useRuntimeConfig()
  const baseURL = config.public.apiUrl

  const user = useState<AuthUser | null>('auth-user', () => null)
  // Whether /auth/me has been attempted this session, so route guards and the
  // header do not re-fetch on every navigation.
  const loaded = useState<boolean>('auth-user-loaded', () => false)

  const isLoggedIn = computed(() => !!user.value)
  const isPatient = computed(() => user.value?.role === 'patient')
  const isProvider = computed(() => user.value?.role === 'provider_admin')
  const isAdmin = computed(() => user.value?.role === 'platform_admin')

  const applyAuth = (resp: AuthResponse): AuthUser => {
    user.value = resp.user
    loaded.value = true
    return resp.user
  }

  const fetchUser = async () => {
    try {
      user.value = await $fetch<AuthUser>(`${baseURL}/auth/me`, {
        credentials: 'include',
      })
    } catch {
      user.value = null
    } finally {
      loaded.value = true
    }
  }

  const login = async (email: string, password: string) => {
    const resp = await $fetch<AuthResponse>(`${baseURL}/auth/provider/login`, {
      method: 'POST',
      body: { email, password },
      credentials: 'include',
    })
    return applyAuth(resp)
  }

  const register = async (email: string, password: string) => {
    const resp = await $fetch<AuthResponse>(`${baseURL}/auth/provider/register`, {
      method: 'POST',
      body: { email, password },
      credentials: 'include',
    })
    return applyAuth(resp)
  }

  const requestMagicLink = async (email: string) => {
    return $fetch<{ ok: boolean }>(`${baseURL}/auth/magic-link`, {
      method: 'POST',
      body: { email },
      credentials: 'include',
    })
  }

  const verifyMagicLink = async (token: string) => {
    const resp = await $fetch<AuthResponse>(`${baseURL}/auth/magic-link/verify`, {
      method: 'POST',
      body: { token },
      credentials: 'include',
    })
    return applyAuth(resp)
  }

  const logout = async () => {
    try {
      await $fetch(`${baseURL}/auth/logout`, {
        method: 'POST',
        credentials: 'include',
      })
    } catch {
      // Clear local state even if the API is unreachable.
    } finally {
      user.value = null
    }
  }

  return {
    user,
    loaded,
    isLoggedIn,
    isPatient,
    isProvider,
    isAdmin,
    fetchUser,
    login,
    register,
    requestMagicLink,
    verifyMagicLink,
    logout,
  }
}
