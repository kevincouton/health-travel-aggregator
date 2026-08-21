<template>
  <div>
    <header class="mb-8 md:mb-10">
      <h1
        class="text-3xl font-extrabold tracking-tight text-gray-900 md:text-4xl dark:text-gray-100"
      >
        Treatments
      </h1>
      <p class="mt-3 text-lg text-gray-500 dark:text-gray-400">
        Browse medical treatments available at clinics around the world.
      </p>
    </header>

    <div v-if="pending" class="py-12 text-center text-gray-500 dark:text-gray-400">Loading...</div>
    <div v-else-if="error" class="py-12 text-center text-red-600 dark:text-red-400">
      Error loading treatments.
    </div>
    <div v-else class="grid gap-5 grid-cols-1 sm:grid-cols-2 lg:grid-cols-3">
      <article
        v-for="treatment in treatments"
        :key="treatment.id"
        class="rounded-2xl border bg-white p-5 dark:border-gray-800 dark:bg-gray-900"
      >
        <span
          class="inline-block rounded-full bg-blue-50 px-2.5 py-0.5 text-[10px] font-bold uppercase tracking-wide text-blue-700 dark:bg-blue-900/30 dark:text-blue-300"
        >
          {{ treatment.category }}
        </span>
        <h2 class="mt-3 text-xl font-semibold text-gray-900 dark:text-gray-100">
          {{ treatment.name }}
        </h2>
        <p
          v-if="treatment.description"
          class="mt-2 text-sm leading-relaxed text-gray-500 dark:text-gray-400"
        >
          {{ treatment.description }}
        </p>
      </article>
    </div>

    <JsonLd :data="jsonLd" />
  </div>
</template>

<script setup>
useSeo({
  title: 'Treatments — Health Travel',
  description: 'Browse medical treatments available at accredited clinics worldwide.',
  keywords: ['treatments', 'medical tourism', 'dental', 'ivf', 'surgery', 'health travel'],
})

const config = useRuntimeConfig()

const { data, pending, error } = await useFetch(() => `${config.public.apiUrl}/treatments`, {
  key: 'treatments',
  default: () => ({ treatments: [] }),
})

const treatments = computed(() => data.value?.treatments || [])

const jsonLd = computed(() => ({
  '@context': 'https://schema.org',
  '@type': 'ItemList',
  itemListElement: treatments.value.map((t, i) => ({
    '@type': 'ListItem',
    position: i + 1,
    name: t.name,
    description: t.description,
  })),
}))
</script>
