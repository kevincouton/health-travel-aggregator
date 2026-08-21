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
      Clinic not found.
    </div>
    <article v-else-if="clinic">
      <div class="rounded-2xl border bg-white p-6 md:p-8 dark:border-gray-800 dark:bg-gray-900">
        <div class="flex flex-col gap-4 md:flex-row md:items-start md:justify-between">
          <div>
            <h1
              class="text-2xl font-bold tracking-tight text-gray-900 md:text-3xl dark:text-gray-100"
            >
              {{ clinic.name }}
            </h1>
            <p class="mt-2 flex flex-wrap items-center gap-2 text-gray-500 dark:text-gray-400">
              <span v-if="clinic.city">{{ clinic.city }}</span>
              <span v-if="clinic.city && clinic.country_code">•</span>
              <span v-if="clinic.country_code">{{ clinic.country_code }}</span>
            </p>
          </div>
          <div
            v-if="clinic.accreditations && clinic.accreditations.length"
            class="flex flex-wrap gap-2"
          >
            <span
              v-for="acc in clinic.accreditations"
              :key="acc"
              class="rounded-full bg-blue-50 px-3 py-1 text-xs font-bold uppercase tracking-wide text-blue-700 dark:bg-blue-900/30 dark:text-blue-300"
            >
              {{ acc }}
            </span>
          </div>
        </div>

        <p
          v-if="clinic.description"
          class="mt-6 text-lg leading-relaxed text-gray-600 dark:text-gray-300"
        >
          {{ clinic.description }}
        </p>

        <div class="mt-8">
          <NuxtLink
            to="/clinics"
            class="inline-flex rounded-xl bg-blue-600 px-5 py-2.5 text-sm font-semibold text-white transition-colors hover:bg-blue-700 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-500 focus-visible:ring-offset-2 dark:focus-visible:ring-offset-gray-900"
          >
            Contact clinic
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
const slug = route.params.slug

const {
  data: clinic,
  pending,
  error,
} = await useFetch(() => `${config.public.apiUrl}/clinics/${encodeURIComponent(slug)}`, {
  key: `clinic-${slug}`,
})

useSeo({
  title: clinic.value?.name ? `${clinic.value.name} — Health Travel` : 'Clinic — Health Travel',
  description: clinic.value?.description
    ? `${clinic.value.description}. Located in ${clinic.value.city || ''}, ${clinic.value.country_code || ''}.`
    : 'Clinic profile on Health Travel.',
  keywords: [
    clinic.value?.name,
    clinic.value?.city,
    'clinic',
    'medical tourism',
    'health travel',
  ].filter(Boolean),
})

const jsonLd = computed(() =>
  clinic.value
    ? {
        '@context': 'https://schema.org',
        '@type': 'MedicalBusiness',
        name: clinic.value.name,
        description: clinic.value.description,
        url: `https://health-travel.lucanian.app/clinics/${clinic.value.slug}`,
        address: {
          '@type': 'PostalAddress',
          addressLocality: clinic.value.city,
          addressCountry: clinic.value.country_code,
        },
      }
    : {}
)
</script>
