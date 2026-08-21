import { toValue } from 'vue'

interface SeoOptions {
  title: string
  description: string
  image?: string
  type?: 'website' | 'article'
  canonical?: string
  path?: string
  keywords?: string[]
  author?: string
  publishedAt?: string
  modifiedAt?: string
  jsonLd?: MaybeRefOrGetter<Record<string, unknown> | Record<string, unknown>[] | undefined>
}

interface PageSeoOptions {
  title: string
  description: string
  image?: string
  path?: string
  jsonLd?: MaybeRefOrGetter<Record<string, unknown> | Record<string, unknown>[] | undefined>
}

const buildJsonLdScripts = (
  jsonLd?: Record<string, unknown> | Record<string, unknown>[] | undefined
) => {
  if (!jsonLd) return []
  const items = Array.isArray(jsonLd) ? jsonLd : [jsonLd]
  return items.map((item) => ({
    type: 'application/ld+json' as const,
    innerHTML: JSON.stringify({ '@context': 'https://schema.org', ...item }),
  }))
}

export const usePageSeo = (opts: PageSeoOptions) => {
  const route = useRoute()
  const config = useRuntimeConfig()

  const siteUrl = config.public.siteUrl || 'https://example.com'
  const path = opts.path ?? route.path
  const canonical = `${siteUrl}${path}`
  const image = opts.image || `${siteUrl}/og-default.png`

  const scripts = computed(() => buildJsonLdScripts(toValue(opts.jsonLd)))

  useHead({
    title: opts.title,
    meta: [
      { name: 'description', content: opts.description },
      { property: 'og:title', content: opts.title },
      { property: 'og:description', content: opts.description },
      { property: 'og:image', content: image },
      { property: 'og:url', content: canonical },
      { property: 'og:type', content: 'website' },
      { property: 'og:site_name', content: config.public.siteName || 'Health Travel' },
      { name: 'twitter:card', content: 'summary_large_image' },
      { name: 'twitter:title', content: opts.title },
      { name: 'twitter:description', content: opts.description },
      { name: 'twitter:image', content: image },
    ],
    link: [{ rel: 'canonical', href: canonical }],
    script: scripts,
  })
}

export const useSeo = (opts: SeoOptions) => {
  const route = useRoute()
  const config = useRuntimeConfig()

  const siteUrl = config.public.siteUrl || 'https://example.com'
  const canonical = opts.canonical || `${siteUrl}${opts.path || route.path}`
  const image = opts.image || `${siteUrl}/og-default.png`

  useHead({
    title: opts.title,
    meta: [
      { name: 'description', content: opts.description },
      { name: 'keywords', content: (opts.keywords || []).join(', ') },
      { name: 'author', content: opts.author || 'Lucanian' },
      { name: 'robots', content: 'index, follow' },
      { property: 'og:title', content: opts.title },
      { property: 'og:description', content: opts.description },
      { property: 'og:image', content: image },
      { property: 'og:url', content: canonical },
      { property: 'og:type', content: opts.type || 'website' },
      { property: 'og:site_name', content: config.public.siteName || 'Platform' },
      { name: 'twitter:card', content: 'summary_large_image' },
      { name: 'twitter:title', content: opts.title },
      { name: 'twitter:description', content: opts.description },
      { name: 'twitter:image', content: image },
      ...(opts.publishedAt ? [{ name: 'article:published_time', content: opts.publishedAt }] : []),
      ...(opts.modifiedAt ? [{ name: 'article:modified_time', content: opts.modifiedAt }] : []),
    ],
    link: [{ rel: 'canonical', href: canonical }],
    script: computed(() => buildJsonLdScripts(toValue(opts.jsonLd))),
  })
}
