<template>
  <div>
    <div class="mb-6">
      <NuxtLink
        to="/clinics"
        class="text-sm text-gray-500 hover:text-gray-900 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-500 rounded-md dark:text-gray-400 dark:hover:text-gray-100"
      >
        ← Back to clinics
      </NuxtLink>
    </div>

    <div v-if="pending" class="py-12 text-center text-gray-500 dark:text-gray-400">Loading...</div>
    <div v-else-if="error" class="py-12 text-center text-red-600 dark:text-red-400">
      Package not found.
    </div>
    <article v-else-if="pkg">
      <div class="rounded-2xl border bg-white p-6 md:p-8 dark:border-gray-800 dark:bg-gray-900">
        <div class="flex flex-col gap-4 md:flex-row md:items-start md:justify-between">
          <div>
            <h1
              class="text-2xl font-bold tracking-tight text-gray-900 md:text-3xl dark:text-gray-100"
            >
              {{ pkg.name }}
            </h1>
            <p class="mt-2 flex flex-wrap items-center gap-2 text-gray-500 dark:text-gray-400">
              <span v-if="pkg.treatment_name">{{ pkg.treatment_name }}</span>
              <span v-if="pkg.treatment_name && pkg.clinic_name">•</span>
              <NuxtLink
                v-if="pkg.clinic_name && pkg.clinic_slug"
                :to="`/clinics/${pkg.clinic_slug}`"
                class="hover:text-blue-700 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-500 rounded-md dark:hover:text-blue-300"
              >
                {{ pkg.clinic_name }}
              </NuxtLink>
            </p>
          </div>
          <div v-if="formattedPrice || pkg.duration_days" class="flex flex-wrap items-center gap-2">
            <span
              v-if="formattedPrice"
              class="rounded-full bg-green-50 px-3 py-1 text-xs font-bold uppercase tracking-wide text-green-700 dark:bg-green-900/30 dark:text-green-300"
            >
              {{ formattedPrice }}
            </span>
            <span
              v-if="pkg.duration_days"
              class="rounded-full bg-blue-50 px-3 py-1 text-xs font-bold uppercase tracking-wide text-blue-700 dark:bg-blue-900/30 dark:text-blue-300"
            >
              {{ pkg.duration_days }} days
            </span>
          </div>
        </div>

        <div class="mt-8 grid gap-8 md:grid-cols-2">
          <div v-if="pkg.inclusions && pkg.inclusions.length">
            <h2 class="mb-3 text-lg font-semibold text-gray-900 dark:text-gray-100">Inclusions</h2>
            <ul class="list-inside list-disc space-y-1 text-gray-600 dark:text-gray-300">
              <li v-for="item in pkg.inclusions" :key="item">{{ item }}</li>
            </ul>
          </div>
          <div v-if="pkg.exclusions && pkg.exclusions.length">
            <h2 class="mb-3 text-lg font-semibold text-gray-900 dark:text-gray-100">Exclusions</h2>
            <ul class="list-inside list-disc space-y-1 text-gray-600 dark:text-gray-300">
              <li v-for="item in pkg.exclusions" :key="item">{{ item }}</li>
            </ul>
          </div>
        </div>

        <div class="mt-8">
          <NuxtLink
            :to="`/quote?package=${pkg.id}`"
            class="inline-flex rounded-xl bg-blue-600 px-5 py-2.5 text-sm font-semibold text-white transition-colors hover:bg-blue-700 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-500 focus-visible:ring-offset-2 dark:focus-visible:ring-offset-gray-900"
          >
            Request a quote
          </NuxtLink>
        </div>
      </div>

      <JsonLd :data="jsonLd" />
    </article>
  </div>
</template>

<script setup>
const route = useRoute()
const config = useRuntimeConfig()
const id = route.params.id

const {
  data: pkg,
  pending,
  error,
} = await useFetch(() => `${config.public.apiUrl}/packages/${encodeURIComponent(id)}`, {
  key: `package-${id}`,
})

const formattedPrice = computed(() => {
  if (!pkg.value) return null
  const min = pkg.value.price_min
  const max = pkg.value.price_max
  if (min != null && max != null) return `$${min.toLocaleString()} – $${max.toLocaleString()}`
  if (min != null) return `From $${min.toLocaleString()}`
  if (max != null) return `Up to $${max.toLocaleString()}`
  return null
})

useSeo({
  title: pkg.value?.name ? `${pkg.value.name} — Health Travel` : 'Package — Health Travel',
  description: pkg.value?.treatment_name
    ? `${pkg.value.name} for ${pkg.value.treatment_name}. Request a quote on Health Travel.`
    : 'Medical tourism package details on Health Travel.',
  keywords: [
    pkg.value?.name,
    pkg.value?.treatment_name,
    pkg.value?.clinic_name,
    'medical tourism',
    'health travel',
    'package',
  ].filter(Boolean),
})

const jsonLd = computed(() =>
  pkg.value
    ? {
        '@context': 'https://schema.org',
        '@type': 'Product',
        name: pkg.value.name,
        description: `Medical tourism package for ${pkg.value.treatment_name || 'treatment'}.`,
        url: `https://health-travel.lucanian.app/packages/${pkg.value.id}`,
        brand: pkg.value.clinic_name
          ? {
              '@type': 'MedicalBusiness',
              name: pkg.value.clinic_name,
              url: `https://health-travel.lucanian.app/clinics/${pkg.value.clinic_slug}`,
            }
          : undefined,
        offers: {
          '@type': 'Offer',
          priceCurrency: 'USD',
          price: pkg.value.price_min != null ? String(pkg.value.price_min) : undefined,
          url: `https://health-travel.lucanian.app/quote?package=${pkg.value.id}`,
        },
      }
    : {}
)
</script>
