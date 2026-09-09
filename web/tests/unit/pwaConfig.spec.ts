import { describe, expect, it } from 'vitest'

import {
  BACKGROUND_COLOR,
  buildManifest,
  buildWorkboxConfig,
  isNetworkOnlyPath,
  NETWORK_ONLY_SEGMENTS,
  THEME_COLOR,
} from '../../pwa/config'

const API_URL = 'https://api.example.test'

describe('buildManifest', () => {
  it('includes every field required for installability', () => {
    const manifest = buildManifest()
    expect(manifest.name).toBeTruthy()
    expect(manifest.short_name).toBeTruthy()
    expect(manifest.description).toBeTruthy()
    expect(manifest.start_url).toBe('/')
    expect(manifest.scope).toBe('/')
    expect(manifest.display).toBe('standalone')
    expect(manifest.theme_color).toBe(THEME_COLOR)
    expect(manifest.background_color).toBe(BACKGROUND_COLOR)
  })

  it('ships 192/512 icons plus a maskable icon', () => {
    const icons = buildManifest().icons
    const sizes = icons.map((icon) => icon.sizes)
    expect(sizes).toContain('192x192')
    expect(sizes).toContain('512x512')
    expect(icons.some((icon) => icon.purpose === 'maskable')).toBe(true)
    for (const icon of icons) {
      expect(icon.type).toBe('image/png')
      expect(icon.src).toMatch(/^\//)
    }
  })
})

describe('isNetworkOnlyPath', () => {
  it.each([
    ['/auth/verify', true],
    ['/me/clinics', true],
    ['/admin', true],
    ['/dashboard/clinics/new', true],
    ['/account', true],
    ['/', false],
    ['/clinics', false],
    ['/clinics/smile-dental', false],
    ['/treatments', false],
    ['/about', false],
    ['/authentication-help', false],
  ])('%s -> %s', (path, expected) => {
    expect(isNetworkOnlyPath(path)).toBe(expected)
  })

  it('covers exactly the declared segments', () => {
    for (const segment of NETWORK_ONLY_SEGMENTS) {
      expect(isNetworkOnlyPath(`/${segment}`)).toBe(true)
    }
  })
})

describe('buildWorkboxConfig', () => {
  const config = buildWorkboxConfig(API_URL)

  it('points generateSW at the prerendered output and falls back to the offline page', () => {
    expect(config.globDirectory).toBe('.output/public')
    expect(config.navigateFallback).toBe('/offline.html')
  })

  it('excludes authenticated surfaces from the precache', () => {
    for (const segment of NETWORK_ONLY_SEGMENTS) {
      expect(config.globIgnores).toContain(`${segment}/**`)
    }
    expect(config.globIgnores).toEqual(expect.arrayContaining(['sw.js', 'workbox-*.js']))
  })

  it('denies the offline fallback for authenticated and API navigations', () => {
    const denied = (path: string) =>
      config.navigateFallbackDenylist.some((pattern) => pattern.test(path))
    for (const path of ['/auth/verify', '/me/inquiries', '/admin', '/dashboard', '/account']) {
      expect(denied(path)).toBe(true)
    }
    expect(denied('/api/v1/clinics')).toBe(true)
    expect(denied('/clinics')).toBe(false)
    expect(denied('/treatments')).toBe(false)
  })

  it('caches visited public pages offline but never authenticated ones', () => {
    const route = config.runtimeCaching[0]
    expect(route.handler).toBe('StaleWhileRevalidate')
    // workbox-build serializes the matcher with Function.prototype.toString;
    // rebuild it the same way to test exactly what lands in the service worker.
    const matcher = new Function(`return (${route.urlPattern.toString()})`)() as (arg: {
      request: { mode: string; url: string }
    }) => boolean
    // Request.mode 'navigate' cannot be constructed in jsdom; a structural
    // stub matches what the service worker passes to the matcher.
    const at = (path: string, mode = 'navigate') =>
      matcher({ request: { mode, url: `https://health-travel.lucanian.app${path}` } })
    expect(at('/clinics/smile-dental')).toBe(true)
    expect(at('/treatments')).toBe(true)
    for (const segment of NETWORK_ONLY_SEGMENTS) {
      expect(at(`/${segment}`)).toBe(false)
    }
    expect(at('/api/v1/clinics')).toBe(false)
    // Non-navigation requests are not handled by this route.
    expect(at('/clinics', 'cors')).toBe(false)
    // Offline cache misses degrade to the precached offline page.
    const plugin = route.options.plugins[0]
    expect(plugin.handlerDidError.toString()).toMatch(/caches\.match\((['"])\/offline\.html\1\)/)
  })

  it('precaches the offline page under its plain URL for caches.match', async () => {
    const transform = config.manifestTransforms[0]
    const { manifest } = await transform([
      { url: 'index.html', revision: 'abc' },
      { url: 'clinics/index.html', revision: 'def' },
      { url: 'offline.html', revision: 'ghi' },
    ])
    const byUrl = new Map(manifest.map((entry) => [entry.url, entry.revision]))
    expect(byUrl.get('/')).toBe('abc')
    expect(byUrl.get('clinics')).toBe('def')
    expect(byUrl.get('offline.html')).toBeNull()
  })

  it('keeps the inline matcher in sync with NETWORK_ONLY_SEGMENTS and the API prefix', () => {
    const source = config.runtimeCaching[0].urlPattern.toString()
    for (const segment of [...NETWORK_ONLY_SEGMENTS, 'api']) {
      expect(source).toContain(segment)
    }
  })

  it('serves public API reads network-first and nothing authenticated', () => {
    const route = config.runtimeCaching[1]
    expect(route.handler).toBe('NetworkFirst')
    expect(route.method).toBe('GET')
    const pattern = route.urlPattern as RegExp
    expect(pattern.test(`${API_URL}/api/v1/clinics`)).toBe(true)
    expect(pattern.test(`${API_URL}/api/v1/clinics/smile-dental`)).toBe(true)
    expect(pattern.test(`${API_URL}/api/v1/treatments`)).toBe(true)
    expect(pattern.test(`${API_URL}/api/v1/packages/abc`)).toBe(true)
    expect(pattern.test(`${API_URL}/api/v1/auth/me`)).toBe(false)
    expect(pattern.test(`${API_URL}/api/v1/me/inquiries`)).toBe(false)
    expect(pattern.test(`${API_URL}/api/v1/admin/clinics`)).toBe(false)
    expect(pattern.test('https://unrelated.test/api/v1/clinics')).toBe(false)
  })
})
