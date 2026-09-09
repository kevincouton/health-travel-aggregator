<template>
  <div>
    <header class="mb-8 md:mb-10">
      <h1
        class="text-3xl font-extrabold tracking-tight text-gray-900 md:text-4xl dark:text-gray-100"
      >
        Admin moderation
      </h1>
      <p class="mt-3 text-lg text-gray-500 dark:text-gray-400">
        Review clinic listings and ownership claims.
      </p>
    </header>

    <div v-if="pending" class="py-12 text-center text-gray-500 dark:text-gray-400">Loading...</div>
    <div v-else-if="loadError" class="py-12 text-center text-red-600 dark:text-red-400">
      {{ loadError }}
    </div>

    <template v-else>
      <section aria-labelledby="clinic-queue-heading">
        <div class="mb-4 flex flex-wrap items-center justify-between gap-3">
          <h2 id="clinic-queue-heading" class="text-xl font-bold text-gray-900 dark:text-gray-100">
            Clinic review queue
          </h2>
          <div class="flex items-center gap-2 text-sm">
            <label for="clinic-status-filter" class="text-gray-500 dark:text-gray-400">
              Status
            </label>
            <select
              id="clinic-status-filter"
              v-model="clinicStatusFilter"
              class="rounded-lg border-gray-300 px-3 py-1.5 text-sm focus:border-blue-500 focus:ring-blue-500 dark:border-gray-700 dark:bg-gray-900 dark:text-gray-100"
              @change="loadClinics"
            >
              <option v-for="s in CLINIC_STATUSES" :key="s" :value="s">{{ s }}</option>
            </select>
          </div>
        </div>

        <div
          v-if="clinics.length === 0"
          class="rounded-2xl border bg-white p-8 text-center dark:border-gray-800 dark:bg-gray-900"
        >
          <p class="text-gray-600 dark:text-gray-300">
            No clinics with status "{{ clinicStatusFilter }}".
          </p>
        </div>
        <ul v-else class="space-y-4">
          <li
            v-for="clinic in clinics"
            :key="clinic.id"
            class="rounded-2xl border bg-white p-5 dark:border-gray-800 dark:bg-gray-900"
            data-testid="admin-clinic-card"
          >
            <div class="flex flex-wrap items-start justify-between gap-3">
              <div>
                <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100">
                  {{ clinic.name }}
                </h3>
                <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
                  {{ clinic.city }}, {{ clinic.country_code }} ·
                  {{ (clinic.accreditations || []).join(', ') || 'No accreditations' }}
                  <span v-if="clinic.source"> · source: {{ clinic.source }}</span>
                </p>
                <p v-if="clinic.flag_reason" class="mt-1 text-sm text-red-600 dark:text-red-400">
                  Flagged: {{ clinic.flag_reason }}
                </p>
              </div>
            </div>
            <p v-if="clinic.description" class="mt-3 text-sm text-gray-600 dark:text-gray-300">
              {{ clinic.description }}
            </p>
            <div class="mt-4 flex flex-wrap items-center gap-2">
              <button
                v-if="clinic.status !== 'approved'"
                type="button"
                class="rounded-lg bg-green-600 px-3 py-1.5 text-sm font-semibold text-white transition-colors hover:bg-green-700 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-green-500"
                @click="setStatus(clinic, 'approved')"
              >
                Approve
              </button>
              <button
                v-if="clinic.status !== 'suspended'"
                type="button"
                class="rounded-lg bg-amber-600 px-3 py-1.5 text-sm font-semibold text-white transition-colors hover:bg-amber-700 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-amber-500"
                @click="setStatus(clinic, 'suspended')"
              >
                Suspend
              </button>
              <button
                type="button"
                class="rounded-lg bg-red-600 px-3 py-1.5 text-sm font-semibold text-white transition-colors hover:bg-red-700 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-red-500"
                @click="flag(clinic)"
              >
                Flag with reason
              </button>
            </div>
          </li>
        </ul>
      </section>

      <section class="mt-12" aria-labelledby="claims-heading">
        <h2 id="claims-heading" class="mb-4 text-xl font-bold text-gray-900 dark:text-gray-100">
          Ownership claims
        </h2>
        <div
          v-if="claims.length === 0"
          class="rounded-2xl border bg-white p-8 text-center dark:border-gray-800 dark:bg-gray-900"
        >
          <p class="text-gray-600 dark:text-gray-300">No pending claims.</p>
        </div>
        <ul v-else class="space-y-4">
          <li
            v-for="claim in claims"
            :key="claim.id"
            class="rounded-2xl border bg-white p-5 dark:border-gray-800 dark:bg-gray-900"
            data-testid="admin-claim-card"
          >
            <p class="text-sm text-gray-600 dark:text-gray-300">
              Claim on clinic <span class="font-mono text-xs">{{ claim.clinic_id }}</span> by user
              <span class="font-mono text-xs">{{ claim.user_id }}</span>
            </p>
            <p v-if="claim.message" class="mt-2 text-sm text-gray-600 dark:text-gray-300">
              "{{ claim.message }}"
            </p>
            <p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
              Submitted {{ formatDate(claim.created_at) }}
            </p>
            <div class="mt-4 flex items-center gap-2">
              <button
                type="button"
                class="rounded-lg bg-green-600 px-3 py-1.5 text-sm font-semibold text-white transition-colors hover:bg-green-700 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-green-500"
                @click="resolve(claim, 'approved')"
              >
                Approve (transfer ownership)
              </button>
              <button
                type="button"
                class="rounded-lg bg-red-600 px-3 py-1.5 text-sm font-semibold text-white transition-colors hover:bg-red-700 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-red-500"
                @click="resolve(claim, 'rejected')"
              >
                Reject
              </button>
            </div>
          </li>
        </ul>
      </section>
    </template>
  </div>
</template>

<script setup>
useSeo({
  title: 'Admin moderation — Health Travel',
  description: 'Moderate clinic listings and ownership claims.',
})

definePageMeta({ middleware: ['admin'] })

const CLINIC_STATUSES = ['pending', 'approved', 'suspended', 'draft']

const { listClinics, updateClinicStatus, flagClinic, listClaims, resolveClaim } = useAdmin()

const pending = ref(true)
const loadError = ref('')
const clinics = ref([])
const claims = ref([])
const clinicStatusFilter = ref('pending')

onMounted(async () => {
  try {
    const [clinicList, claimList] = await Promise.all([listClinics('pending'), listClaims()])
    clinics.value = clinicList
    claims.value = claimList
  } catch {
    loadError.value = 'Unable to load the moderation queue. Please try again later.'
  } finally {
    pending.value = false
  }
})

async function loadClinics() {
  try {
    clinics.value = await listClinics(clinicStatusFilter.value)
  } catch {
    loadError.value = 'Unable to load clinics. Please try again later.'
  }
}

async function setStatus(clinic, status) {
  const updated = await updateClinicStatus(clinic.id, status)
  clinics.value = clinics.value.map((c) => (c.id === clinic.id ? updated : c))
}

async function flag(clinic) {
  const reason = window.prompt(`Reason for flagging "${clinic.name}":`)
  if (!reason || !reason.trim()) return
  const updated = await flagClinic(clinic.id, reason.trim())
  clinics.value = clinics.value.map((c) => (c.id === clinic.id ? updated : c))
}

async function resolve(claim, status) {
  await resolveClaim(claim.id, status)
  claims.value = claims.value.filter((c) => c.id !== claim.id)
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
