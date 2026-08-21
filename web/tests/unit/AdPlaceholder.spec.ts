import { mount } from '@vue/test-utils'
import { beforeEach, describe, expect, it, vi } from 'vitest'

// The composable and component reference Nuxt auto-imports; under
// @nuxt/test-utils these are rewritten at transform time to real module
// imports, so vi.stubGlobal never takes effect (same finding as W1-2).
// Mock the resolved import ids instead, per-test configurable via `state`.
const state = vi.hoisted(() => ({
  publicCfg: {} as Record<string, unknown>,
  headCalls: [] as unknown[],
}))

vi.mock('#app', async (importOriginal) => ({
  ...(await importOriginal<object>()),
  useRuntimeConfig: () => ({ public: state.publicCfg }),
  useHead: (input: unknown) => state.headCalls.push(input),
}))

vi.mock('#app/nuxt', async (importOriginal) => ({
  ...(await importOriginal<object>()),
  useRuntimeConfig: () => ({ public: state.publicCfg }),
}))

vi.mock('#app/composables/head', async (importOriginal) => ({
  ...(await importOriginal<object>()),
  useHead: (input: unknown) => state.headCalls.push(input),
}))

function stubConfig(publicCfg: Record<string, unknown>) {
  state.publicCfg = publicCfg
}

beforeEach(() => {
  state.headCalls.length = 0
  delete (window as unknown as Record<string, unknown>).adsbygoogle
})

async function mountComponent() {
  const { default: AdPlaceholder } = await import('../../components/AdPlaceholder.vue')
  return mount(AdPlaceholder, { props: { slotId: 'test-slot' } })
}

describe('AdPlaceholder', () => {
  it('renders nothing when ads are disabled', async () => {
    stubConfig({ adsEnabled: 'false' })
    const wrapper = await mountComponent()
    expect(wrapper.find('.ad-slot').exists()).toBe(false)
    expect(wrapper.find('ins.adsbygoogle').exists()).toBe(false)
  })

  it('renders the placeholder box when enabled without a provider', async () => {
    stubConfig({ adsEnabled: 'true', adsProvider: 'none' })
    const wrapper = await mountComponent()
    expect(wrapper.find('.ad-slot').exists()).toBe(true)
    expect(wrapper.find('ins.adsbygoogle').exists()).toBe(false)
  })

  it('renders an AdSense slot and injects the script when provider is adsense', async () => {
    stubConfig({ adsEnabled: 'true', adsProvider: 'adsense', adsenseClientId: 'ca-pub-123' })
    const wrapper = await mountComponent()
    const ins = wrapper.find('ins.adsbygoogle')
    expect(ins.exists()).toBe(true)
    expect(ins.attributes('data-ad-client')).toBe('ca-pub-123')
    expect(ins.attributes('data-ad-slot')).toBe('test-slot')
    expect(state.headCalls.length).toBe(1)
    const head = state.headCalls[0] as { script: Array<{ src: string }> }
    expect(head.script[0].src).toContain('pagead2.googlesyndication.com/pagead/js/adsbygoogle.js')
    expect(head.script[0].src).toContain('ca-pub-123')
  })

  it('pushes the slot onto window.adsbygoogle when mounting the real ad', async () => {
    stubConfig({ adsEnabled: 'true', adsProvider: 'adsense', adsenseClientId: 'ca-pub-123' })
    const pushed: unknown[] = []
    ;(window as unknown as Record<string, unknown>).adsbygoogle = pushed
    const wrapper = await mountComponent()
    expect(wrapper.find('ins.adsbygoogle').exists()).toBe(true)
    expect(pushed).toHaveLength(1)
  })

  it('does not touch window.adsbygoogle when rendering the placeholder', async () => {
    stubConfig({ adsEnabled: 'true', adsProvider: 'none' })
    ;(window as unknown as Record<string, unknown>).adsbygoogle = []
    await mountComponent()
    expect(((window as unknown as Record<string, unknown>).adsbygoogle as unknown[]).length).toBe(0)
  })

  it('falls back to the placeholder when adsense has no client id', async () => {
    stubConfig({ adsEnabled: 'true', adsProvider: 'adsense' })
    const wrapper = await mountComponent()
    expect(wrapper.find('ins.adsbygoogle').exists()).toBe(false)
    expect(wrapper.find('.ad-slot').exists()).toBe(true)
  })
})
