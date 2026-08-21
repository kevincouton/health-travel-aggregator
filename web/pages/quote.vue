<template>
  <div>
    <header class="mb-8 md:mb-10">
      <h1
        class="text-3xl font-extrabold tracking-tight text-gray-900 md:text-4xl dark:text-gray-100"
      >
        Request a quote
      </h1>
      <p class="mt-3 text-lg text-gray-500 dark:text-gray-400">
        Tell us a little about your trip and the clinic will reply with a personalised quote.
      </p>
    </header>

    <div v-if="pending" class="py-12 text-center text-gray-500 dark:text-gray-400">Loading...</div>
    <div v-else-if="loadError" class="py-12 text-center text-red-600 dark:text-red-400">
      {{ loadError }}
    </div>
    <div
      v-else-if="submitted"
      class="rounded-2xl border bg-white p-8 text-center dark:border-gray-800 dark:bg-gray-900"
    >
      <h2 class="text-xl font-semibold text-gray-900 dark:text-gray-100">Thank you!</h2>
      <p class="mt-2 text-gray-600 dark:text-gray-300">
        Your quote request has been sent to {{ displayClinicName || 'the clinic' }}. You can track
        it from your account dashboard.
      </p>
      <NuxtLink
        to="/"
        class="mt-6 inline-flex rounded-xl bg-blue-600 px-5 py-2.5 text-sm font-semibold text-white transition-colors hover:bg-blue-700 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-500 focus-visible:ring-offset-2 dark:focus-visible:ring-offset-gray-900"
      >
        Back to home
      </NuxtLink>
    </div>
    <div v-else class="grid gap-8 lg:grid-cols-3">
      <div class="lg:col-span-2">
        <div class="rounded-2xl border bg-white p-6 md:p-8 dark:border-gray-800 dark:bg-gray-900">
          <QuoteForm
            :clinic-id="clinicId"
            :package-id="packageId"
            :clinic-name="displayClinicName"
            :package-name="displayPackageName"
            @success="onSuccess"
          />
        </div>
      </div>
      <aside class="lg:col-span-1">
        <div class="rounded-2xl border bg-white p-6 dark:border-gray-800 dark:bg-gray-900">
          <h2 class="mb-4 text-lg font-semibold text-gray-900 dark:text-gray-100">Quote summary</h2>
          <dl class="space-y-3 text-sm">
            <div>
              <dt class="text-gray-500 dark:text-gray-400">Clinic</dt>
              <dd class="font-medium text-gray-900 dark:text-gray-100">
                {{ displayClinicName || 'Not selected' }}
              </dd>
            </div>
            <div v-if="displayPackageName">
              <dt class="text-gray-500 dark:text-gray-400">Package</dt>
              <dd class="font-medium text-gray-900 dark:text-gray-100">
                {{ displayPackageName }}
              </dd>
            </div>
          </dl>
        </div>
      </aside>
    </div>
  </div>
</template>

<script setup>
useSeo({
  title: 'Request a quote — Health Travel',
  description: 'Request a personalised medical tourism quote from accredited clinics.',
  keywords: ['quote', 'medical tourism', 'request quote', 'health travel'],
})

const route = useRoute()
const { getPackage } = usePackages()
const { getClinic } = useClinics()

const clinicSlug = computed(() => route.query.clinic || '')
const packageId = computed(() => route.query.package || '')

const pending = ref(true)
const loadError = ref('')
const submitted = ref(false)
const clinicId = ref('')
const displayClinicName = ref('')
const displayPackageName = ref('')

onMounted(async () => {
  try {
    if (packageId.value) {
      const pkg = await getPackage(packageId.value)
      clinicId.value = pkg.clinic_id || ''
      displayClinicName.value = pkg.clinic_name || ''
      displayPackageName.value = pkg.name || ''
    }

    if (clinicSlug.value) {
      const clinic = await getClinic(clinicSlug.value)
      clinicId.value = clinic.id || ''
      displayClinicName.value = clinic.name || ''
    }

    if (!clinicId.value) {
      loadError.value = 'Please select a clinic or package before requesting a quote.'
    }
  } catch (err) {
    loadError.value = 'Unable to load clinic details. Please try again later.'
  } finally {
    pending.value = false
  }
})

function onSuccess() {
  submitted.value = true
}
</script>
