<template>
  <div class="mx-auto max-w-2xl">
    <header class="mb-8">
      <NuxtLink
        to="/dashboard"
        class="text-sm font-medium text-blue-600 hover:text-blue-700 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-500 rounded-md dark:text-blue-400 dark:hover:text-blue-300"
      >
        ← Back to dashboard
      </NuxtLink>
      <h1 class="mt-3 text-3xl font-extrabold tracking-tight text-gray-900 dark:text-gray-100">
        Add a clinic
      </h1>
      <p class="mt-2 text-sm text-gray-500 dark:text-gray-400">
        New clinics are reviewed by our team before appearing in public search.
      </p>
    </header>

    <div class="rounded-2xl border bg-white p-6 md:p-8 dark:border-gray-800 dark:bg-gray-900">
      <ClinicForm submit-label="Create clinic" :on-submit="create" />
    </div>
  </div>
</template>

<script setup>
useSeo({
  title: 'Add a clinic — Health Travel',
  description: 'Add a new clinic to your Health Travel provider account.',
})

definePageMeta({ middleware: ['provider'] })

const { createClinic } = useProvider()

async function create(payload) {
  await createClinic(payload)
  await navigateTo('/dashboard')
}
</script>
