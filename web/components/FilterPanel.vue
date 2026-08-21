<template>
  <search>
    <form
      class="rounded-2xl border bg-white p-5 dark:border-gray-800 dark:bg-gray-900"
      @submit.prevent="$emit('submit')"
    >
      <h2
        class="mb-4 text-sm font-semibold uppercase tracking-wide text-gray-500 dark:text-gray-400"
      >
        Filters
      </h2>
      <div class="space-y-4">
        <div>
          <label :for="qId" class="mb-1 block text-sm font-medium text-gray-700 dark:text-gray-300">
            Search
          </label>
          <SearchBar
            :id="qId"
            :model-value="q"
            placeholder="Clinic name, city, country..."
            label="Search clinics"
            @update:model-value="$emit('update:q', $event)"
            @submit="$emit('submit')"
          />
        </div>
        <div>
          <label
            :for="countryId"
            class="mb-1 block text-sm font-medium text-gray-700 dark:text-gray-300"
          >
            Country code
          </label>
          <input
            :id="countryId"
            :value="countryCode"
            type="text"
            placeholder="e.g. US"
            class="w-full rounded-xl border border-gray-300 bg-white px-4 py-2.5 text-gray-900 placeholder-gray-400 focus:border-blue-500 focus:outline-none focus:ring-2 focus:ring-blue-500 dark:border-gray-700 dark:bg-gray-900 dark:text-gray-100 dark:placeholder-gray-500"
            @input="$emit('update:countryCode', $event.target.value)"
          />
        </div>
        <div>
          <label
            :for="cityId"
            class="mb-1 block text-sm font-medium text-gray-700 dark:text-gray-300"
          >
            City
          </label>
          <input
            :id="cityId"
            :value="city"
            type="text"
            placeholder="e.g. Miami"
            class="w-full rounded-xl border border-gray-300 bg-white px-4 py-2.5 text-gray-900 placeholder-gray-400 focus:border-blue-500 focus:outline-none focus:ring-2 focus:ring-blue-500 dark:border-gray-700 dark:bg-gray-900 dark:text-gray-100 dark:placeholder-gray-500"
            @input="$emit('update:city', $event.target.value)"
          />
        </div>
        <button
          type="submit"
          class="w-full rounded-xl bg-blue-600 px-4 py-2.5 text-sm font-semibold text-white transition-colors hover:bg-blue-700 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-500 focus-visible:ring-offset-2 dark:focus-visible:ring-offset-gray-900"
        >
          Apply filters
        </button>
      </div>
    </form>
  </search>
</template>

<script setup>
defineProps({
  q: { type: String, default: '' },
  countryCode: { type: String, default: '' },
  city: { type: String, default: '' },
})

defineEmits(['update:q', 'update:countryCode', 'update:city', 'submit'])

const qId = `filter-q-${useId()}`
const countryId = `filter-country-${useId()}`
const cityId = `filter-city-${useId()}`
</script>
