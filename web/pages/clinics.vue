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
          v-else-if="filteredClinics.length === 0"
          class="py-12 text-center text-gray-500 dark:text-gray-400"
        >
          No clinics match your search.
        </div>
        <div v-else class="grid gap-5 grid-cols-1 md:grid-cols-2 xl:grid-cols-3">
          <ClinicCard v-for="clinic in filteredClinics" :key="clinic.id" :clinic="clinic" />
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
})

const serverFilters = computed(() => {
  const out = {}
  if (filters.country_code.trim()) out.country_code = filters.country_code.trim()
  if (filters.city.trim()) out.city = filters.city.trim()
  return out
})

const { data, pending, error } = await useFetch(() => `${config.public.apiUrl}/clinics`, {
  key: 'clinics',
  query: serverFilters,
  default: () => [],
})

const filteredClinics = computed(() => {
  const q = filters.q.trim().toLowerCase()
  if (!q) return data.value || []
  return (data.value || []).filter((clinic) => {
    const haystack = [
      clinic.name,
      clinic.city,
      clinic.country_code,
      clinic.description,
      ...(clinic.accreditations || []),
    ]
      .filter(Boolean)
      .join(' ')
      .toLowerCase()
    return haystack.includes(q)
  })
})

function applyFilters() {
  const query = {}
  if (filters.q.trim()) query.q = filters.q.trim()
  if (filters.country_code.trim()) query.country_code = filters.country_code.trim()
  if (filters.city.trim()) query.city = filters.city.trim()
  router.replace({ query })
}

watch(
  () => route.query,
  (query) => {
    filters.q = String(query.q || '')
    filters.country_code = String(query.country_code || '')
    filters.city = String(query.city || '')
  },
  { deep: true }
)

const jsonLd = computed(() => ({
  '@context': 'https://schema.org',
  '@type': 'ItemList',
  itemListElement: filteredClinics.value.map((c, i) => ({
    '@type': 'ListItem',
    position: i + 1,
    name: c.name,
    url: `https://health-travel.lucanian.app/clinics/${c.slug}`,
  })),
}))
</script>
