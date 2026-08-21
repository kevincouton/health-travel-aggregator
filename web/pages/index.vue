<template>
  <div>
    <section
      class="rounded-3xl border bg-gradient-to-br from-blue-50 to-white px-6 py-16 text-center dark:border-gray-800 dark:from-gray-900 dark:to-gray-950"
    >
      <h1 class="text-4xl font-extrabold tracking-tight text-gray-900 md:text-5xl dark:text-gray-100">
        Discover world-class medical travel
      </h1>
      <p class="mx-auto mt-4 max-w-2xl text-lg text-gray-600 dark:text-gray-300">
        Compare accredited clinics, explore treatments, and find the right care abroad with Health
        Travel.
      </p>
      <div class="mx-auto mt-8 flex max-w-xl flex-col items-center gap-3 sm:flex-row">
        <SearchBar
          v-model="heroQuery"
          placeholder="Search treatments or clinics..."
          label="Search treatments or clinics"
          class="sm:flex-1"
          @submit="onHeroSearch"
        />
        <NuxtLink
          to="/clinics"
          class="w-full rounded-xl bg-blue-600 px-6 py-3 text-center text-sm font-semibold text-white transition-colors hover:bg-blue-700 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-500 focus-visible:ring-offset-2 dark:focus-visible:ring-offset-gray-900 sm:w-auto"
        >
          Find clinics
        </NuxtLink>
      </div>
    </section>

    <section v-if="featuredTreatments.length" class="mt-12">
      <div class="mb-6 flex items-center justify-between">
        <h2 class="text-2xl font-bold text-gray-900 dark:text-gray-100">Featured treatments</h2>
        <NuxtLink
          to="/treatments"
          class="text-sm font-medium text-blue-600 hover:text-blue-700 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-500 rounded-md dark:text-blue-400 dark:hover:text-blue-300"
        >
          View all →
        </NuxtLink>
      </div>
      <div class="grid gap-5 grid-cols-1 sm:grid-cols-2 lg:grid-cols-3">
        <article
          v-for="treatment in featuredTreatments"
          :key="treatment.id"
          class="rounded-2xl border bg-white p-5 dark:border-gray-800 dark:bg-gray-900"
        >
          <span
            class="inline-block rounded-full bg-blue-50 px-2.5 py-0.5 text-[10px] font-bold uppercase tracking-wide text-blue-700 dark:bg-blue-900/30 dark:text-blue-300"
          >
            {{ treatment.category }}
          </span>
          <h3 class="mt-3 text-lg font-semibold text-gray-900 dark:text-gray-100">
            {{ treatment.name }}
          </h3>
          <p
            v-if="treatment.description"
            class="mt-2 line-clamp-2 text-sm text-gray-500 dark:text-gray-400"
          >
            {{ treatment.description }}
          </p>
        </article>
      </div>
    </section>

    <section v-if="featuredClinics.length" class="mt-12">
      <div class="mb-6 flex items-center justify-between">
        <h2 class="text-2xl font-bold text-gray-900 dark:text-gray-100">Featured clinics</h2>
        <NuxtLink
          to="/clinics"
          class="text-sm font-medium text-blue-600 hover:text-blue-700 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-500 rounded-md dark:text-blue-400 dark:hover:text-blue-300"
        >
          Browse clinics →
        </NuxtLink>
      </div>
      <div class="grid gap-5 grid-cols-1 md:grid-cols-2 lg:grid-cols-3">
        <ClinicCard v-for="clinic in featuredClinics" :key="clinic.id" :clinic="clinic" />
      </div>
    </section>

    <JsonLd :data="jsonLd" />
  </div>
</template>

<script setup>
useSeo({
  title: 'Health Travel — Medical Tourism Aggregator',
  description:
    'Compare accredited clinics and treatments for medical tourism with Health Travel.',
  keywords: ['medical tourism', 'clinics', 'treatments', 'health travel', 'accredited clinics'],
})

const { getClinics } = useClinics()
const { getTreatments } = useTreatments()
const router = useRouter()

const heroQuery = ref('')

const [{ data: clinicsData, pending: clinicsPending }, { data: treatmentsData, pending: treatmentsPending }] =
  await Promise.all([
    useFetch(() => `${useRuntimeConfig().public.apiUrl}/clinics`, {
      key: 'home-clinics',
      default: () => [],
    }),
    useFetch(() => `${useRuntimeConfig().public.apiUrl}/treatments`, {
      key: 'home-treatments',
      default: () => ({ treatments: [] }),
    }),
  ])

const featuredClinics = computed(() => (clinicsData.value || []).slice(0, 3))
const featuredTreatments = computed(() => (treatmentsData.value?.treatments || []).slice(0, 6))

function onHeroSearch() {
  const q = heroQuery.value.trim()
  if (q) {
    router.push({ path: '/clinics', query: { q } })
  } else {
    router.push('/clinics')
  }
}

const jsonLd = computed(() => ({
  '@context': 'https://schema.org',
  '@type': 'WebSite',
  name: 'Health Travel',
  url: 'https://health-travel.lucanian.app/',
  potentialAction: {
    '@type': 'SearchAction',
    target: 'https://health-travel.lucanian.app/clinics?q={search_term_string}',
    'query-input': 'required name=search_term_string',
  },
}))
</script>
