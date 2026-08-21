interface Clinic {
  slug: string
  status?: string
}

interface ClinicsResponse {
  clinics?: Clinic[]
}

const escapeXml = (value: string): string =>
  value
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&apos;')

export default defineEventHandler(async (event) => {
  const config = useRuntimeConfig()
  const siteUrl = config.public.siteUrl || 'https://example.com'
  const apiUrl = config.public.apiUrl || 'http://localhost:8080'

  const staticRoutes = [
    { path: '', priority: 1.0, changefreq: 'daily' },
    { path: '/treatments', priority: 0.8, changefreq: 'weekly' },
    { path: '/clinics', priority: 0.9, changefreq: 'daily' },
    { path: '/about', priority: 0.8, changefreq: 'monthly' },
  ]

  let clinicUrls: { path: string; priority: number; changefreq: string }[] = []
  try {
    const response = await $fetch<ClinicsResponse>(`${apiUrl}/clinics?per_page=1000`)
    const clinics = response.clinics || []
    clinicUrls = clinics
      .filter((c) => !c.status || c.status === 'approved')
      .map((c) => ({
        path: `/clinics/${c.slug}`,
        priority: 0.7,
        changefreq: 'weekly',
      }))
  } catch {
    // If the API is unreachable, fall back to static routes only.
  }

  const routes = [...staticRoutes, ...clinicUrls]

  const sitemap = `<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
${routes
  .map(
    (r) => `  <url>
    <loc>${escapeXml(`${siteUrl}${r.path}`)}</loc>
    <priority>${r.priority}</priority>
    <changefreq>${r.changefreq}</changefreq>
  </url>`
  )
  .join('\n')}
</urlset>`

  event.node.res.setHeader('content-type', 'application/xml')
  return sitemap
})
