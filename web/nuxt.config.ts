export default defineNuxtConfig({
  app: {
    head: {
      titleTemplate: '%s — health-travel-aggregator',
      htmlAttrs: { lang: 'en' },
    },
  },
  devtools: { enabled: false },
  modules: ['@nuxtjs/tailwindcss'],
  tailwindcss: {
    cssPath: '~/assets/css/tailwind.css',
  },
  runtimeConfig: {
    public: {
      apiUrl: process.env.NUXT_PUBLIC_API_URL || 'http://localhost:8080',
      siteUrl: process.env.NUXT_PUBLIC_SITE_URL || 'https://health-travel.lucanian.app',
      siteName: 'health-travel-aggregator',
      sentryDsn: process.env.NUXT_PUBLIC_SENTRY_DSN || '',
    },
  },
  nitro: {
    prerender: {
      routes: ['/', '/about', '/sitemap.xml'],
    },
  },
})
