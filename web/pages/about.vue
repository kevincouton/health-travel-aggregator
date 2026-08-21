<template>
  <div class="max-w-3xl">
    <header class="mb-8 md:mb-10">
      <h1
        class="text-3xl font-extrabold tracking-tight text-gray-900 md:text-4xl dark:text-gray-100"
      >
        About health-travel-aggregator
      </h1>
      <p class="mt-3 text-lg text-gray-500 dark:text-gray-400">
        Learn more about the platform and how it helps patients find medical travel options.
      </p>
    </header>
    <section>
      <p class="text-lg leading-relaxed text-gray-600 dark:text-gray-300">
        health-travel-aggregator is a crowdsourced intelligence platform. It empowers users to build,
        share, and vote on strategies across multiple domains.
      </p>
    </section>

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
