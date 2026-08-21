import { describe, expect, it } from 'vitest'

import { resolveAdConfig } from '../../composables/useAds'

describe('resolveAdConfig', () => {
  it('is disabled when adsEnabled is not "true"', () => {
    expect(resolveAdConfig({}).enabled).toBe(false)
    expect(resolveAdConfig({ adsEnabled: 'false' }).enabled).toBe(false)
    expect(resolveAdConfig({ adsEnabled: 'true' }).enabled).toBe(true)
  })

  it('reports no provider by default', () => {
    const cfg = resolveAdConfig({ adsEnabled: 'true' })
    expect(cfg.provider).toBeNull()
    expect(cfg.clientId).toBeNull()
  })

  it('resolves the adsense provider and client id', () => {
    const cfg = resolveAdConfig({
      adsEnabled: 'true',
      adsProvider: 'adsense',
      adsenseClientId: 'ca-pub-123',
    })
    expect(cfg.provider).toBe('adsense')
    expect(cfg.clientId).toBe('ca-pub-123')
  })

  it('treats provider "none" as no provider', () => {
    expect(resolveAdConfig({ adsEnabled: 'true', adsProvider: 'none' }).provider).toBeNull()
  })
})
