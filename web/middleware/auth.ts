// Requires a signed-in user (any role). Client-only: the site is statically
// generated, so the session cookie is only readable from the browser.
export default defineNuxtRouteMiddleware(async (to) => {
  if (import.meta.server) return

  const { user, loaded, fetchUser } = useUser()
  if (!loaded.value) {
    await fetchUser()
  }
  if (!user.value) {
    return navigateTo({ path: '/login', query: { redirect: to.fullPath } })
  }
})
