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
        {{ clinic ? clinic.name : 'Edit clinic' }}
      </h1>
      <p v-if="clinic" class="mt-2 text-sm text-gray-500 dark:text-gray-400">
        Status: <span class="font-medium">{{ clinic.status }}</span>
      </p>
    </header>

    <div v-if="pending" class="py-12 text-center text-gray-500 dark:text-gray-400">Loading...</div>
    <div v-else-if="loadError" class="py-12 text-center text-red-600 dark:text-red-400">
      {{ loadError }}
    </div>

    <template v-else>
      <section
        class="rounded-2xl border bg-white p-6 md:p-8 dark:border-gray-800 dark:bg-gray-900"
        aria-labelledby="clinic-details-heading"
      >
        <h2
          id="clinic-details-heading"
          class="mb-5 text-xl font-bold text-gray-900 dark:text-gray-100"
        >
          Clinic details
        </h2>
        <ClinicForm :initial="clinic" :on-submit="saveClinic" />
      </section>

      <section
        class="mt-8 rounded-2xl border bg-white p-6 md:p-8 dark:border-gray-800 dark:bg-gray-900"
        aria-labelledby="packages-heading"
      >
        <div class="mb-5 flex items-center justify-between gap-4">
          <h2 id="packages-heading" class="text-xl font-bold text-gray-900 dark:text-gray-100">
            Packages
          </h2>
          <button
            v-if="!showNewPackage"
            type="button"
            class="inline-flex rounded-xl bg-blue-600 px-4 py-2 text-sm font-semibold text-white transition-colors hover:bg-blue-700 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-500 focus-visible:ring-offset-2 dark:focus-visible:ring-offset-gray-900"
            @click="showNewPackage = true"
          >
            Add package
          </button>
        </div>

        <div
          v-if="showNewPackage"
          class="mb-6 rounded-xl border border-dashed p-5 dark:border-gray-700"
        >
          <h3 class="mb-4 text-base font-semibold text-gray-900 dark:text-gray-100">New package</h3>
          <PackageForm
            :treatments="treatments"
            submit-label="Create package"
            :on-submit="createNewPackage"
          >
            <template #cancel>
              <button
                type="button"
                class="text-sm font-medium text-gray-500 hover:text-gray-700 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-500 rounded-md dark:text-gray-400 dark:hover:text-gray-200"
                @click="showNewPackage = false"
              >
                Cancel
              </button>
            </template>
          </PackageForm>
        </div>

        <div v-if="packages.length === 0 && !showNewPackage" class="py-6 text-center">
          <p class="text-sm text-gray-500 dark:text-gray-400">No packages yet.</p>
        </div>

        <ul class="space-y-4">
          <li
            v-for="pkg in packages"
            :key="pkg.id"
            class="rounded-xl border p-5 dark:border-gray-700"
            data-testid="package-card"
          >
            <template v-if="editingPackageId !== pkg.id">
              <div class="flex flex-wrap items-start justify-between gap-3">
                <div>
                  <h3 class="text-base font-semibold text-gray-900 dark:text-gray-100">
                    {{ pkg.name }}
                  </h3>
                  <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
                    {{ treatmentName(pkg.treatment_id) }}
                    <span v-if="pkg.price_min != null">
                      · ${{ pkg.price_min
                      }}<template v-if="pkg.price_max != null">–${{ pkg.price_max }}</template>
                    </span>
                    <span v-if="pkg.duration_days != null"> · {{ pkg.duration_days }} days</span>
                  </p>
                </div>
                <span
                  class="inline-block rounded-full px-2.5 py-0.5 text-[10px] font-bold uppercase tracking-wide"
                  :class="
                    pkg.is_published
                      ? 'bg-green-50 text-green-700 dark:bg-green-900/30 dark:text-green-300'
                      : 'bg-gray-100 text-gray-600 dark:bg-gray-800 dark:text-gray-400'
                  "
                >
                  {{ pkg.is_published ? 'Published' : 'Draft' }}
                </span>
              </div>
              <div class="mt-4 flex items-center gap-4 text-sm">
                <button
                  type="button"
                  class="font-medium text-blue-600 hover:text-blue-700 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-500 rounded-md dark:text-blue-400 dark:hover:text-blue-300"
                  @click="editingPackageId = pkg.id"
                >
                  Edit
                </button>
                <button
                  type="button"
                  class="font-medium text-red-600 hover:text-red-700 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-red-500 rounded-md dark:text-red-400 dark:hover:text-red-300"
                  @click="removePackage(pkg)"
                >
                  Delete
                </button>
              </div>
            </template>
            <PackageForm
              v-else
              :treatments="treatments"
              :initial="pkg"
              :on-submit="(payload) => savePackage(pkg, payload)"
            >
              <template #cancel>
                <button
                  type="button"
                  class="text-sm font-medium text-gray-500 hover:text-gray-700 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-500 rounded-md dark:text-gray-400 dark:hover:text-gray-200"
                  @click="editingPackageId = null"
                >
                  Cancel
                </button>
              </template>
            </PackageForm>
          </li>
        </ul>
      </section>
    </template>
  </div>
</template>

<script setup>
useSeo({
  title: 'Edit clinic — Health Travel',
  description: 'Edit your clinic details and treatment packages.',
})

definePageMeta({ middleware: ['provider'] })

const route = useRoute()
const clinicId = computed(() => String(route.params.id))

const { getMyClinics, updateClinic, getMyPackages, createPackage, updatePackage, deletePackage } =
  useProvider()
const { getTreatments } = useTreatments()

const pending = ref(true)
const loadError = ref('')
const clinic = ref(null)
const packages = ref([])
const treatments = ref([])
const showNewPackage = ref(false)
const editingPackageId = ref(null)

onMounted(async () => {
  try {
    const [clinicList, packageList, treatmentsResp] = await Promise.all([
      getMyClinics(),
      getMyPackages(clinicId.value),
      getTreatments().catch(() => ({ treatments: [] })),
    ])
    const found = clinicList.find((c) => c.id === clinicId.value)
    if (!found) {
      loadError.value = 'Clinic not found.'
      return
    }
    clinic.value = found
    packages.value = packageList
    treatments.value = treatmentsResp?.treatments || []
  } catch {
    loadError.value = 'Unable to load this clinic. Please try again later.'
  } finally {
    pending.value = false
  }
})

function treatmentName(id) {
  return treatments.value.find((t) => t.id === id)?.name || ''
}

async function saveClinic(payload) {
  const updated = await updateClinic(clinicId.value, payload)
  clinic.value = updated
}

async function createNewPackage(payload) {
  const created = await createPackage(clinicId.value, payload)
  packages.value = [created, ...packages.value]
  showNewPackage.value = false
}

async function savePackage(pkg, payload) {
  const updated = await updatePackage(clinicId.value, pkg.id, payload)
  packages.value = packages.value.map((p) => (p.id === pkg.id ? updated : p))
  editingPackageId.value = null
}

async function removePackage(pkg) {
  if (!window.confirm(`Delete package "${pkg.name}"? This cannot be undone.`)) return
  await deletePackage(clinicId.value, pkg.id)
  packages.value = packages.value.filter((p) => p.id !== pkg.id)
}
</script>
