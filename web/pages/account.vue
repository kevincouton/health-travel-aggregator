<template>
  <div>
    <header class="mb-8 md:mb-10">
      <h1
        class="text-3xl font-extrabold tracking-tight text-gray-900 md:text-4xl dark:text-gray-100"
      >
        My inquiries
      </h1>
      <p class="mt-3 text-lg text-gray-500 dark:text-gray-400">
        Track the quote requests you've sent to clinics.
      </p>
    </header>

    <div v-if="pending" class="py-12 text-center text-gray-500 dark:text-gray-400">Loading...</div>
    <div v-else-if="loadError" class="py-12 text-center text-red-600 dark:text-red-400">
      {{ loadError }}
    </div>
    <div
      v-else-if="inquiries.length === 0"
      class="rounded-2xl border bg-white p-8 text-center dark:border-gray-800 dark:bg-gray-900"
    >
      <p class="text-gray-600 dark:text-gray-300">You haven't sent any inquiries yet.</p>
      <NuxtLink
        to="/clinics"
        class="mt-6 inline-flex rounded-xl bg-blue-600 px-5 py-2.5 text-sm font-semibold text-white transition-colors hover:bg-blue-700 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-500 focus-visible:ring-offset-2 dark:focus-visible:ring-offset-gray-900"
      >
        Find a clinic
      </NuxtLink>
    </div>

    <ul v-else class="space-y-4">
      <li
        v-for="inquiry in inquiries"
        :key="inquiry.id"
        class="rounded-2xl border bg-white p-5 dark:border-gray-800 dark:bg-gray-900"
        data-testid="inquiry-card"
      >
        <div class="flex flex-wrap items-start justify-between gap-3">
          <div>
            <h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100">
              <NuxtLink
                v-if="clinicSlug(inquiry.clinic_id)"
                :to="`/clinics/${clinicSlug(inquiry.clinic_id)}`"
                class="hover:text-blue-600 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-500 rounded-md dark:hover:text-blue-400"
              >
                {{ clinicName(inquiry.clinic_id) }}
              </NuxtLink>
              <template v-else>{{ clinicName(inquiry.clinic_id) }}</template>
            </h2>
            <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
              Sent {{ formatDate(inquiry.created_at) }}
              <span v-if="inquiry.preferred_dates">
                · Preferred: {{ inquiry.preferred_dates }}</span
              >
            </p>
          </div>
          <InquiryStatusBadge :status="inquiry.status" />
        </div>
        <p v-if="inquiry.medical_notes" class="mt-3 text-sm text-gray-600 dark:text-gray-300">
          {{ inquiry.medical_notes }}
        </p>
      </li>
    </ul>
  </div>
</template>

<script setup>
useSeo({
  title: 'My inquiries — Health Travel',
  description: 'Track the quote requests you have sent to clinics.',
})

definePageMeta({ middleware: ['auth'] })

const { getMyInquiries } = useInquiries()
const { getClinics } = useClinics()

const pending = ref(true)
const loadError = ref('')
const inquiries = ref([])
const clinicsById = ref({})

onMounted(async () => {
  try {
    const [list, clinicsResp] = await Promise.all([getMyInquiries(), getClinics().catch(() => [])])
    inquiries.value = list
    const map = {}
    for (const clinic of clinicsResp?.clinics || clinicsResp || []) {
      map[clinic.id] = clinic
    }
    clinicsById.value = map
  } catch {
    loadError.value = 'Unable to load your inquiries. Please try again later.'
  } finally {
    pending.value = false
  }
})

function clinicName(id) {
  return clinicsById.value[id]?.name || 'Clinic'
}

function clinicSlug(id) {
  return clinicsById.value[id]?.slug || ''
}

function formatDate(iso) {
  try {
    return new Date(iso).toLocaleDateString(undefined, {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
    })
  } catch {
    return iso
  }
}
</script>
