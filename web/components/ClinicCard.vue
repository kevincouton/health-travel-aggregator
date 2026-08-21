<template>
  <article
    class="group flex h-full flex-col rounded-2xl border bg-white p-5 transition-shadow hover:shadow-lg dark:border-gray-800 dark:bg-gray-900"
  >
    <NuxtLink :to="`/clinics/${clinic.slug}`" class="flex h-full flex-col">
      <div class="mb-3 flex items-start justify-between gap-3">
        <h3
          class="text-lg font-semibold leading-tight text-gray-900 transition-colors group-hover:text-blue-700 dark:text-gray-100"
        >
          {{ clinic.name }}
        </h3>
        <span
          v-if="clinic.accreditations && clinic.accreditations.length"
          class="shrink-0 rounded-full bg-blue-50 px-2 py-0.5 text-[10px] font-bold uppercase tracking-wide text-blue-700 dark:bg-blue-900/30 dark:text-blue-300"
        >
          {{ clinic.accreditations[0] }}
        </span>
      </div>
      <p
        v-if="clinic.description"
        class="line-clamp-2 text-sm leading-relaxed text-gray-500 dark:text-gray-400"
      >
        {{ clinic.description }}
      </p>
      <div
        class="mt-auto flex flex-wrap items-center gap-2 pt-4 text-xs text-gray-500 dark:text-gray-400"
      >
        <span
          v-if="clinic.city"
          class="rounded-lg bg-gray-100 px-2.5 py-1 font-medium text-gray-600 dark:bg-gray-800 dark:text-gray-300"
        >
          {{ clinic.city }}
        </span>
        <span
          v-if="clinic.country_code"
          class="rounded-lg bg-gray-100 px-2.5 py-1 font-medium text-gray-600 dark:bg-gray-800 dark:text-gray-300"
        >
          {{ clinic.country_code }}
        </span>
      </div>
    </NuxtLink>
  </article>
</template>

<script setup>
defineProps({
  clinic: { type: Object, required: true },
})
</script>
