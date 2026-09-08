import * as Sentry from '@sentry/vue'

// GlitchTip (Sentry-compatible) browser error capture. Inactive when
// NUXT_PUBLIC_SENTRY_DSN is unset, so dev/CI builds ship no client.
export default defineNuxtPlugin((nuxtApp) => {
  const dsn = useRuntimeConfig().public.sentryDsn
  if (!dsn) {
    return
  }
  Sentry.init({
    app: nuxtApp.vueApp,
    dsn,
  })
})
