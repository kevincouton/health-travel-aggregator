<template>
  <div>
    <header class="mb-8 md:mb-10">
      <h1
        class="text-3xl font-extrabold tracking-tight text-gray-900 md:text-4xl dark:text-gray-100"
      >
        Find a clinic
      </h1>
      <p class="mt-3 text-lg text-gray-500 dark:text-gray-400">
        Search accredited clinics by location or name.
      </p>
    </header>

    <div class="grid gap-8 lg:grid-cols-4">
      <aside class="lg:col-span-1">
        <FilterPanel
          v-model:q="filters.q"
          v-model:country-code="filters.country_code"
          v-model:city="filters.city"
          v-model:treatment="filters.treatment"
          v-model:accreditation="filters.accreditation"
          v-model:min-price="filters.min_price"
          v-model:max-price="filters.max_price"
          @submit="applyFilters"
        />
      </aside>

      <section class="lg:col-span-3">
        <div v-if="pending" class="py-12 text-center text-gray-500 dark:text-gray-400">
          Loading...
        </div>
        <div v-else-if="error" class="py-12 text-center text-red-600 dark:text-red-400">
          Error loading clinics.
        </div>
        <div
          v-else-if="clinics.length === 0"
          class="py-12 text-center text-gray-500 dark:text-gray-400"
        >
          No clinics match your search.
        </div>
        <ul v-else class="grid gap-5 grid-cols-1 md:grid-cols-2 xl:grid-cols-3">
          <li v-for="clinic in clinics" :key="clinic.id">
            <ClinicCard :clinic="clinic" />
          </li>
        </ul>
        <div
          v-if="totalPages > 1"
          class="mt-8 flex items-center justify-between rounded-2xl border bg-white p-4 dark:border-gray-800 dark:bg-gray-900"
        >
          <button
            :disabled="page <= 1"
            class="rounded-xl bg-blue-600 px-4 py-2 text-sm font-semibold text-white transition-colors hover:bg-blue-700 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-500 focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50 dark:focus-visible:ring-offset-gray-900"
            @click="setPage(page - 1)"
          >
            Previous
          </button>
          <span class="text-sm text-gray-600 dark:text-gray-400">
            Page {{ page }} of {{ totalPages }}
          </span>
          <button
            :disabled="page >= totalPages"
            class="rounded-xl bg-blue-600 px-4 py-2 text-sm font-semibold text-white transition-colors hover:bg-blue-700 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-500 focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50 dark:focus-visible:ring-offset-gray-900"
            @click="setPage(page + 1)"
          >
            Next
          </button>
        </div>
      </section>
    </div>

    <JsonLd :data="jsonLd" />
  </div>
</template>

<script setup>
useSeo({
  title: 'Clinics — Health Travel',
  description: 'Search and compare accredited medical tourism clinics by location and specialty.',
  keywords: ['clinics', 'medical tourism', 'accredited clinics', 'search', 'health travel'],
})

const route = useRoute()
const router = useRouter()
const config = useRuntimeConfig()

const filters = reactive({
  q: String(route.query.q || ''),
  country_code: String(route.query.country_code || ''),
  city: String(route.query.city || ''),
  treatment: String(route.query.treatment || ''),
  accreditation: String(route.query.accreditation || ''),
  min_price: String(route.query.min_price || ''),
  max_price: String(route.query.max_price || ''),
})

const page = computed(() => {
  const n = Number.parseInt(route.query.page, 10)
  return Number.isFinite(n) && n > 0 ? n : 1
})

const serverFilters = computed(() => {
  const out = {}
  if (filters.q.trim()) out.q = filters.q.trim()
  if (filters.country_code.trim()) out.country = filters.country_code.trim()
  if (filters.city.trim()) out.city = filters.city.trim()
  if (filters.treatment.trim()) out.treatment = filters.treatment.trim()
  if (filters.accreditation.trim()) out.accreditation = filters.accreditation.trim()
  if (filters.min_price.trim()) out.min_price = filters.min_price.trim()
  if (filters.max_price.trim()) out.max_price = filters.max_price.trim()
  out.page = page.value
  out.per_page = 20
  return out
})

const { data, pending, error } = await useFetch(() => `${config.public.apiUrl}/clinics`, {
  key: () => `clinics-${JSON.stringify(serverFilters.value)}`,
  query: serverFilters,
  default: () => ({ clinics: [], total: 0, page: 1, per_page: 20 }),
})

const clinics = computed(() => data.value?.clinics || [])
const total = computed(() => data.value?.total || 0)
const perPage = computed(() => data.value?.per_page || 20)
const totalPages = computed(() => Math.ceil(total.value / perPage.value))

function applyFilters() {
  const query = {}
  if (filters.q.trim()) query.q = filters.q.trim()
  if (filters.country_code.trim()) query.country_code = filters.country_code.trim()
  if (filters.city.trim()) query.city = filters.city.trim()
  if (filters.treatment.trim()) query.treatment = filters.treatment.trim()
  if (filters.accreditation.trim()) query.accreditation = filters.accreditation.trim()
  if (filters.min_price.trim()) query.min_price = filters.min_price.trim()
  if (filters.max_price.trim()) query.max_price = filters.max_price.trim()
  router.replace({ query })
}

function setPage(next) {
  router.replace({ query: { ...route.query, page: next } })
}

watch(
  () => route.query,
  (query) => {
    filters.q = String(query.q || '')
    filters.country_code = String(query.country_code || '')
    filters.city = String(query.city || '')
    filters.treatment = String(query.treatment || '')
    filters.accreditation = String(query.accreditation || '')
    filters.min_price = String(query.min_price || '')
    filters.max_price = String(query.max_price || '')
  },
  { deep: true }
)

const jsonLd = computed(() => ({
  '@context': 'https://schema.org',
  '@type': 'ItemList',
  itemListElement: clinics.value.map((c, i) => ({
    '@type': 'ListItem',
    position: i + 1,
    name: c.name,
    url: `https://health-travel.lucanian.app/clinics/${c.slug}`,
  })),
}))
</script>
