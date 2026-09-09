<template>
  <div>
    <header class="mb-8 flex flex-wrap items-start justify-between gap-4 md:mb-10">
      <div>
        <h1
          class="text-3xl font-extrabold tracking-tight text-gray-900 md:text-4xl dark:text-gray-100"
        >
          Provider dashboard
        </h1>
        <p class="mt-3 text-lg text-gray-500 dark:text-gray-400">
          Manage your clinics, packages, and patient inquiries.
        </p>
      </div>
      <NuxtLink
        to="/dashboard/clinics/new"
        class="inline-flex rounded-xl bg-blue-600 px-5 py-2.5 text-sm font-semibold text-white transition-colors hover:bg-blue-700 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-500 focus-visible:ring-offset-2 dark:focus-visible:ring-offset-gray-900"
      >
        Add clinic
      </NuxtLink>
    </header>

    <div v-if="pending" class="py-12 text-center text-gray-500 dark:text-gray-400">Loading...</div>
    <div v-else-if="loadError" class="py-12 text-center text-red-600 dark:text-red-400">
      {{ loadError }}
    </div>

    <template v-else>
      <section aria-labelledby="my-clinics-heading">
        <h2 id="my-clinics-heading" class="mb-4 text-xl font-bold text-gray-900 dark:text-gray-100">
          My clinics
        </h2>
        <div
          v-if="clinics.length === 0"
          class="rounded-2xl border bg-white p-8 text-center dark:border-gray-800 dark:bg-gray-900"
        >
          <p class="text-gray-600 dark:text-gray-300">
            No clinics yet. Add your first clinic to start receiving patient inquiries.
          </p>
        </div>
        <ul v-else class="grid gap-5 grid-cols-1 md:grid-cols-2">
          <li
            v-for="clinic in clinics"
            :key="clinic.id"
            class="rounded-2xl border bg-white p-5 dark:border-gray-800 dark:bg-gray-900"
            data-testid="clinic-card"
          >
            <div class="flex items-start justify-between gap-3">
              <div>
                <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100">
                  {{ clinic.name }}
                </h3>
                <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
                  {{ clinic.city }}, {{ clinic.country_code }}
                </p>
              </div>
              <span
                class="inline-block rounded-full px-2.5 py-0.5 text-[10px] font-bold uppercase tracking-wide"
                :class="statusClass(clinic.status)"
              >
                {{ clinic.status }}
              </span>
            </div>
            <p v-if="clinic.flag_reason" class="mt-2 text-sm text-red-600 dark:text-red-400">
              Flagged: {{ clinic.flag_reason }}
            </p>
            <div class="mt-4 flex items-center gap-4 text-sm">
              <NuxtLink
                :to="`/dashboard/clinics/${clinic.id}`"
                class="font-medium text-blue-600 hover:text-blue-700 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-500 rounded-md dark:text-blue-400 dark:hover:text-blue-300"
              >
                Edit &amp; packages
              </NuxtLink>
              <NuxtLink
                v-if="clinic.status === 'approved'"
                :to="`/clinics/${clinic.slug}`"
                class="font-medium text-gray-500 hover:text-gray-700 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-500 rounded-md dark:text-gray-400 dark:hover:text-gray-200"
              >
                View public page
              </NuxtLink>
            </div>
          </li>
        </ul>
      </section>

      <section class="mt-12" aria-labelledby="inbox-heading">
        <h2 id="inbox-heading" class="mb-4 text-xl font-bold text-gray-900 dark:text-gray-100">
          Inquiry inbox
        </h2>
        <div
          v-if="inquiries.length === 0"
          class="rounded-2xl border bg-white p-8 text-center dark:border-gray-800 dark:bg-gray-900"
        >
          <p class="text-gray-600 dark:text-gray-300">No inquiries yet.</p>
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
                <h3 class="text-base font-semibold text-gray-900 dark:text-gray-100">
                  {{ clinicName(inquiry.clinic_id) }}
                </h3>
                <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
                  {{ inquiry.contact_email }} · {{ formatDate(inquiry.created_at) }}
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
            <div class="mt-4 flex flex-wrap items-center gap-2">
              <label :for="`status-${inquiry.id}`" class="sr-only">Update status</label>
              <select
                :id="`status-${inquiry.id}`"
                :value="inquiry.status"
                class="rounded-lg border-gray-300 px-3 py-1.5 text-sm focus:border-blue-500 focus:ring-blue-500 dark:border-gray-700 dark:bg-gray-900 dark:text-gray-100"
                @change="onStatusChange(inquiry, $event)"
              >
                <option v-for="s in STATUSES" :key="s" :value="s">{{ s }}</option>
              </select>
            </div>
          </li>
        </ul>
      </section>
    </template>
  </div>
</template>

<script setup>
useSeo({
  title: 'Provider dashboard — Health Travel',
  description: 'Manage your clinics, packages, and patient inquiries.',
})

definePageMeta({ middleware: ['provider'] })

const STATUSES = ['new', 'contacted', 'converted', 'closed']

const { getMyClinics } = useProvider()
const { getMyInquiries, updateInquiryStatus } = useInquiries()

const pending = ref(true)
const loadError = ref('')
const clinics = ref([])
const inquiries = ref([])

onMounted(async () => {
  try {
    const [clinicList, inquiryList] = await Promise.all([getMyClinics(), getMyInquiries()])
    clinics.value = clinicList
    inquiries.value = inquiryList
  } catch {
    loadError.value = 'Unable to load your dashboard. Please try again later.'
  } finally {
    pending.value = false
  }
})

function clinicName(id) {
  return clinics.value.find((c) => c.id === id)?.name || 'Clinic'
}

function statusClass(status) {
  switch (status) {
    case 'approved':
      return 'bg-green-50 text-green-700 dark:bg-green-900/30 dark:text-green-300'
    case 'pending':
      return 'bg-amber-50 text-amber-700 dark:bg-amber-900/30 dark:text-amber-300'
    case 'suspended':
      return 'bg-red-50 text-red-700 dark:bg-red-900/30 dark:text-red-300'
    default:
      return 'bg-gray-100 text-gray-600 dark:bg-gray-800 dark:text-gray-400'
  }
}

async function onStatusChange(inquiry, event) {
  const status = event.target.value
  try {
    const updated = await updateInquiryStatus(inquiry.id, status)
    Object.assign(inquiry, updated)
  } catch {
    event.target.value = inquiry.status
  }
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
