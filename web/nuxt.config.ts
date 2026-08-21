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
      adsEnabled: process.env.NUXT_PUBLIC_ADS_ENABLED || 'false',
      adsProvider: process.env.NUXT_PUBLIC_ADS_PROVIDER || 'none',
      adsenseClientId: process.env.NUXT_PUBLIC_ADSENSE_CLIENT_ID || '',
    },
  },
  nitro: {
    prerender: {
      routes: ['/', '/about', '/sitemap.xml'],
    },
  },
})
