// PWA options for @vite-pwa/nuxt.
//
// The security-critical rule: authenticated and mutation surfaces are
// network-only and must never be served from a cache. NETWORK_ONLY_SEGMENTS
// is the single source of truth — it drives the precache ignore list and the
// navigation-fallback denylist. The runtime navigation matcher below repeats
// the same list inline because workbox-build serializes route functions with
// Function.prototype.toString, so they cannot close over module scope; the
// unit tests assert both copies agree.

export const PWA_NAME = 'Health Travel Aggregator'
export const PWA_SHORT_NAME = 'Health Travel'
export const PWA_DESCRIPTION =
  'Compare accredited clinics, treatments, and packages for medical travel.'

// Tailwind blue-600, the accent color used across the layout.
export const THEME_COLOR = '#2563eb'
export const BACKGROUND_COLOR = '#ffffff'

export const NETWORK_ONLY_SEGMENTS = ['auth', 'me', 'admin', 'dashboard', 'account'] as const

export function isNetworkOnlyPath(pathname: string): boolean {
  const segment = pathname.split('/').filter(Boolean)[0] ?? ''
  return (NETWORK_ONLY_SEGMENTS as readonly string[]).includes(segment)
}

export function buildManifest() {
  return {
    name: PWA_NAME,
    short_name: PWA_SHORT_NAME,
    description: PWA_DESCRIPTION,
    lang: 'en',
    start_url: '/',
    scope: '/',
    display: 'standalone',
    theme_color: THEME_COLOR,
    background_color: BACKGROUND_COLOR,
    categories: ['health', 'travel'],
    icons: [
      { src: '/icons/icon-192.png', sizes: '192x192', type: 'image/png' },
      { src: '/icons/icon-512.png', sizes: '512x512', type: 'image/png' },
      { src: '/icons/maskable-512.png', sizes: '512x512', type: 'image/png', purpose: 'maskable' },
    ],
  }
}

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
}

export function buildWorkboxConfig(apiUrl: string) {
  const apiOrigin = new URL(apiUrl).origin
  // Read-only API GETs that the public listing/detail pages depend on.
  const publicApiPattern = new RegExp(
    `^${escapeRegExp(apiOrigin)}/api/v1/(clinics|treatments|packages)`
  )
  return {
    // The experimental vite-plus (rolldown) toolchain does not give
    // vite-plugin-pwa a usable default outDir — without this the precache
    // manifest only picks up payload JSONs and misses the app shell.
    globDirectory: '.output/public',
    globPatterns: ['**/*.{js,css,html,png,svg,webmanifest}'],
    globIgnores: [
      'sw.js',
      'workbox-*.js',
      '200.html',
      '404.html',
      ...NETWORK_ONLY_SEGMENTS.map((segment) => `${segment}/**`),
    ],
    // Uncached public routes fall back to a static offline page; authenticated
    // and API routes fall through to the network and simply fail offline.
    navigateFallback: '/offline.html',
    navigateFallbackDenylist: [
      ...NETWORK_ONLY_SEGMENTS.map((segment) => new RegExp(`^/${segment}(/|$)`)),
      /^\/api(\/|$)/,
    ],
    // Replicates @vite-pwa/nuxt's default html -> route URL mapping and, in
    // addition, precaches /offline.html under its plain URL (revision null) so
    // the runtime handlerDidError hook below can find it with caches.match.
    manifestTransforms: [
      async (entries: { url: string; revision: string | null }[]) => {
        const manifest = entries.map((entry) => {
          if (entry.url.endsWith('.html')) {
            const url = entry.url.startsWith('/') ? entry.url.slice(1) : entry.url
            if (url === 'index.html') {
              entry.url = '/'
            } else if (url.endsWith('/index.html')) {
              entry.url = url.slice(0, -'/index.html'.length) || '/'
            }
          }
          return entry
        })
        const offline = manifest.find((entry) => entry.url === 'offline.html')
        if (offline) offline.revision = null
        return { manifest, warnings: [] }
      },
    ],
    runtimeCaching: [
      {
        // Public pages already visited stay viewable offline; uncached public
        // routes degrade to the offline page instead of a browser error. Keep
        // the inline segment list in sync with NETWORK_ONLY_SEGMENTS plus the
        // API prefix (asserted by tests).
        urlPattern: ({ request }: { request: Request }) =>
          request.mode === 'navigate' &&
          !/^\/(auth|me|admin|dashboard|account|api)(\/|$)/.test(new URL(request.url).pathname),
        handler: 'StaleWhileRevalidate' as const,
        options: {
          cacheName: 'public-pages',
          expiration: { maxEntries: 64, maxAgeSeconds: 7 * 24 * 60 * 60 },
          plugins: [
            {
              handlerDidError: async () =>
                (await caches.match('/offline.html')) ?? Response.error(),
            },
          ],
        },
      },
      {
        // Public listing/detail data: fresh when online, stale when offline.
        // Non-GET requests are never matched by workbox runtime routes.
        urlPattern: publicApiPattern,
        handler: 'NetworkFirst' as const,
        method: 'GET' as const,
        options: {
          cacheName: 'api-public',
          networkTimeoutSeconds: 4,
          expiration: { maxEntries: 128, maxAgeSeconds: 24 * 60 * 60 },
        },
      },
    ],
  }
}
