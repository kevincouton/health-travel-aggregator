// Provider area: signed-in provider_admin only. Client-only because the
// statically generated site can only resolve the session in the browser.
export default defineNuxtRouteMiddleware(async (to) => {
  if (import.meta.server) return

  const { user, loaded, fetchUser } = useUser()
  if (!loaded.value) {
    await fetchUser()
  }
  if (!user.value) {
    return navigateTo({ path: '/login', query: { redirect: to.fullPath } })
  }
  if (user.value.role !== 'provider_admin') {
    return navigateTo('/')
  }
})
