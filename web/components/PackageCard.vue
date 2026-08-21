<template>
  <article
    class="group flex h-full flex-col rounded-2xl border bg-white p-5 transition-shadow hover:shadow-lg dark:border-gray-800 dark:bg-gray-900"
  >
    <div class="mb-3 flex items-start justify-between gap-3">
      <h3
        class="text-lg font-semibold leading-tight text-gray-900 transition-colors group-hover:text-blue-700 dark:text-gray-100"
      >
        <NuxtLink
          :to="`/packages/${pkg.id}`"
          class="focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-500 rounded-md"
        >
          {{ pkg.name }}
        </NuxtLink>
      </h3>
      <span
        v-if="pkg.treatment_name"
        class="shrink-0 rounded-full bg-blue-50 px-2 py-0.5 text-[10px] font-bold uppercase tracking-wide text-blue-700 dark:bg-blue-900/30 dark:text-blue-300"
      >
        {{ pkg.treatment_name }}
      </span>
    </div>

    <div class="mb-3 flex flex-wrap items-center gap-2 text-sm text-gray-600 dark:text-gray-300">
      <span v-if="formattedPrice" class="font-medium">{{ formattedPrice }}</span>
      <span v-if="formattedPrice && pkg.duration_days">•</span>
      <span v-if="pkg.duration_days">{{ pkg.duration_days }} days</span>
    </div>

    <ul
      v-if="shortInclusions.length"
      class="mb-4 list-inside list-disc text-sm text-gray-500 dark:text-gray-400"
    >
      <li v-for="item in shortInclusions" :key="item">{{ item }}</li>
    </ul>

    <div class="mt-auto flex flex-wrap items-center gap-2 pt-4">
      <NuxtLink
        :to="`/packages/${pkg.id}`"
        class="inline-flex rounded-xl bg-blue-600 px-4 py-2 text-sm font-semibold text-white transition-colors hover:bg-blue-700 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-500 focus-visible:ring-offset-2 dark:focus-visible:ring-offset-gray-900"
      >
        View package
      </NuxtLink>
      <NuxtLink
        :to="`/quote?package=${pkg.id}`"
        class="inline-flex rounded-xl border border-gray-300 px-4 py-2 text-sm font-semibold text-gray-700 transition-colors hover:bg-gray-50 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-500 focus-visible:ring-offset-2 dark:border-gray-700 dark:text-gray-200 dark:hover:bg-gray-800 dark:focus-visible:ring-offset-gray-900"
      >
        Request quote
      </NuxtLink>
    </div>
  </article>
</template>

<script setup>
const props = defineProps({
  pkg: { type: Object, required: true },
})

const formattedPrice = computed(() => {
  const min = props.pkg.price_min
  const max = props.pkg.price_max
  if (min != null && max != null) return `$${min.toLocaleString()} – $${max.toLocaleString()}`
  if (min != null) return `From $${min.toLocaleString()}`
  if (max != null) return `Up to $${max.toLocaleString()}`
  return null
})

const shortInclusions = computed(() => (props.pkg.inclusions || []).slice(0, 3))
</script>
