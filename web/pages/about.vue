<template>
  <div class="max-w-3xl">
    <h1 class="text-3xl md:text-4xl font-extrabold mb-6 tracking-tight">
      About health-travel-aggregator
    </h1>
    <p class="text-lg leading-relaxed text-gray-600 dark:text-gray-300">
      health-travel-aggregator is a crowdsourced intelligence platform. It empowers users to build,
      share, and vote on strategies across multiple domains.
    </p>

    <JsonLd :data="jsonLd" />
  </div>
</template>

<script setup>
useSeo({
  title: 'About — health-travel-aggregator',
  description:
    'Learn about health-travel-aggregator, a crowdsourced intelligence platform for scoring, ranking, and discovering entities.',
  keywords: ['about', 'health-travel-aggregator', 'methodology', 'scoring', 'ranking'],
})

const jsonLd = {
  '@context': 'https://schema.org',
  '@type': 'AboutPage',
  name: 'About health-travel-aggregator',
  description: 'Learn about health-travel-aggregator, a crowdsourced intelligence platform.',
  url: 'https://health-travel.lucanian.app/about',
  mainEntity: {
    '@type': 'Organization',
    name: 'health-travel-aggregator',
    url: 'https://health-travel.lucanian.app',
    sameAs: [],
  },
}
</script>
