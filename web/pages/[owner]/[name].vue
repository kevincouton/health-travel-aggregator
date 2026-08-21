<template>
  <div>
    <div v-if="pending" class="text-gray-500 py-12 text-center dark:text-gray-400">Loading...</div>
    <div v-else-if="error" class="text-red-600 py-12 text-center dark:text-red-400">
      Error loading entity
    </div>
    <div v-else-if="entity">
      <div class="mb-6">
        <NuxtLink
          to="/"
          class="text-sm text-gray-500 hover:text-gray-900 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-500 rounded-md dark:text-gray-400 dark:hover:text-gray-100"
        >
          ← Back
        </NuxtLink>
      </div>
      <div
        class="bg-white rounded-2xl border p-6 md:p-8 mb-8 dark:border-gray-800 dark:bg-gray-900"
      >
        <div class="flex items-start justify-between mb-6">
          <div class="flex-1">
            <h1 class="text-2xl md:text-3xl font-bold tracking-tight">{{ entity.full_name }}</h1>
            <p class="text-gray-500 text-lg leading-relaxed mt-2 dark:text-gray-400">
              {{ entity.description }}
            </p>
          </div>
        </div>
        <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
          <div class="bg-gray-50 px-5 py-4 rounded-xl text-center dark:bg-gray-800">
            <div
              class="text-gray-400 text-xs font-medium uppercase tracking-wider mb-1 dark:text-gray-500"
            >
              Score
            </div>
            <div class="text-2xl font-extrabold">{{ entity.composite_score }}/100</div>
          </div>
          <div class="bg-gray-50 px-5 py-4 rounded-xl text-center dark:bg-gray-800">
            <div
              class="text-gray-400 text-xs font-medium uppercase tracking-wider mb-1 dark:text-gray-500"
            >
              Value
            </div>
            <div class="text-2xl font-extrabold">{{ entity.score_value }}</div>
          </div>
        </div>
      </div>

      <JsonLd :data="jsonLd" />
    </div>
  </div>
</template>

<script setup>
const { getEntity } = useApi()
const route = useRoute()
const owner = route.params.owner
const name = route.params.name

const {
  data: entity,
  pending,
  error,
} = await useAsyncData(`entity-${owner}-${name}`, () => getEntity(owner, name))

useSeo({
  title: entity.value?.full_name
    ? `${entity.value.full_name} — health-travel-aggregator`
    : 'Entity — health-travel-aggregator',
  description: entity.value?.description
    ? `${entity.value.description}. Scored ${entity.value.composite_score}/100 on health-travel-aggregator.`
    : 'Entity detail page on health-travel-aggregator.',
  keywords: [entity.value?.full_name, 'health-travel-aggregator', 'scoring', 'ranking'].filter(Boolean),
})

const jsonLd = computed(() =>
  entity.value
    ? {
        '@context': 'https://schema.org',
        '@type': 'WebPage',
        name: entity.value.full_name,
        description: entity.value.description,
        url: `https://health-travel.lucanian.app/${entity.value.full_name}`,
        mainEntity: {
          '@type': 'Thing',
          name: entity.value.full_name,
          description: entity.value.description,
          aggregateRating: {
            '@type': 'AggregateRating',
            ratingValue: entity.value.composite_score,
            bestRating: 100,
          },
        },
      }
    : {}
)
</script>
