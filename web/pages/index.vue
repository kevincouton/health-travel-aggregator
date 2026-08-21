<template>
  <div>
    <div class="mb-8 md:mb-10">
      <h1 class="text-3xl md:text-4xl font-extrabold mb-3 tracking-tight">Entities</h1>
      <p class="text-gray-500 text-lg dark:text-gray-400">
        Scored and ranked with the health-travel-aggregator methodology.
      </p>
    </div>

    <ClientOnly>
      <div v-if="pending" class="text-gray-500 py-12 text-center dark:text-gray-400">
        Loading...
      </div>
      <div v-else-if="error" class="text-red-600 py-12 text-center dark:text-red-400">
        Error: {{ error }}
      </div>
      <div v-else class="grid gap-5 grid-cols-1 md:grid-cols-2 lg:grid-cols-3">
        <template v-for="(entity, index) in entities" :key="entity.id">
          <EntityCard :entity="entity" />
          <AdPlaceholder
            v-if="index === 2"
            slot-id="health-travel-aggregator-infeed-native"
            format="native"
            label="Sponsored"
            class-name="col-span-1 md:col-span-2 lg:col-span-3"
          />
        </template>
      </div>
      <template #fallback>
        <div class="text-gray-500 py-12 text-center dark:text-gray-400">Loading...</div>
      </template>
    </ClientOnly>

    <JsonLd :data="jsonLd" />
  </div>
</template>

<script setup>
useSeo({
  title: 'Entities — health-travel-aggregator',
  description:
    'Discover and compare top entities scored and ranked with the health-travel-aggregator methodology. Crowdsourced intelligence platform.',
  keywords: ['entities', 'ranking', 'scoring', 'crowdsourced', 'health-travel-aggregator'],
})

const { getEntities } = useApi()

const { data, pending, error } = useAsyncData('entities', () => getEntities({ limit: '50' }))

const entities = computed(() => data.value?.entities || [])

const jsonLd = computed(() => ({
  '@context': 'https://schema.org',
  '@type': 'WebPage',
  name: 'Entities — health-travel-aggregator',
  description:
    'Discover and compare top entities scored and ranked with the health-travel-aggregator methodology.',
  url: 'https://health-travel.lucanian.app/',
  mainEntity: {
    '@type': 'ItemList',
    itemListElement: entities.value.map((e, i) => ({
      '@type': 'ListItem',
      position: i + 1,
      name: e.full_name,
      url: `https://health-travel.lucanian.app/${e.full_name}`,
    })),
  },
}))
</script>
