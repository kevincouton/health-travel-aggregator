import { buildManifest, buildWorkboxConfig, THEME_COLOR } from './pwa/config'

const apiUrl = process.env.NUXT_PUBLIC_API_URL || 'http://localhost:8080'

export default defineNuxtConfig({
  app: {
    head: {
      titleTemplate: '%s — health-travel-aggregator',
      htmlAttrs: { lang: 'en' },
      meta: [{ name: 'theme-color', content: THEME_COLOR }],
      link: [
        // Nitro prerendering bypasses @vite-pwa/nuxt's <head> injection, so
        // the manifest and icons are linked explicitly here.
        { rel: 'manifest', href: '/manifest.webmanifest' },
        { rel: 'icon', type: 'image/png', href: '/icons/icon-192.png' },
        { rel: 'apple-touch-icon', href: '/icons/apple-touch-icon.png' },
      ],
    },
  },
  devtools: { enabled: false },
  modules: ['@nuxtjs/tailwindcss', '@vite-pwa/nuxt'],
  pwa: {
    registerType: 'autoUpdate',
    manifest: buildManifest(),
    workbox: buildWorkboxConfig(apiUrl),
  },
  tailwindcss: {
    cssPath: '~/assets/css/tailwind.css',
  },
  runtimeConfig: {
    public: {
      apiUrl,
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
