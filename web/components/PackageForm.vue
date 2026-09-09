<template>
  <form class="space-y-5" novalidate @submit.prevent="handleSubmit">
    <div v-if="!isEdit">
      <label
        for="package-treatment"
        class="mb-1 block text-sm font-medium text-gray-700 dark:text-gray-300"
      >
        Treatment <span aria-label="required">*</span>
      </label>
      <select
        id="package-treatment"
        v-model="form.treatment_id"
        name="treatment_id"
        required
        class="w-full rounded-xl border-gray-300 px-4 py-2.5 text-sm focus:border-blue-500 focus:ring-blue-500 dark:border-gray-700 dark:bg-gray-900 dark:text-gray-100"
        :class="{
          'border-red-500 focus:border-red-500 focus:ring-red-500': errors.treatment_id,
        }"
        :aria-invalid="errors.treatment_id ? 'true' : 'false'"
      >
        <option value="" disabled>Select a treatment</option>
        <option v-for="t in treatments" :key="t.id" :value="t.id">{{ t.name }}</option>
      </select>
      <p v-if="errors.treatment_id" class="mt-1 text-sm text-red-600 dark:text-red-400">
        {{ errors.treatment_id }}
      </p>
    </div>

    <div>
      <label
        for="package-name"
        class="mb-1 block text-sm font-medium text-gray-700 dark:text-gray-300"
      >
        Package name <span aria-label="required">*</span>
      </label>
      <input
        id="package-name"
        v-model="form.name"
        type="text"
        name="name"
        required
        class="w-full rounded-xl border-gray-300 px-4 py-2.5 text-sm focus:border-blue-500 focus:ring-blue-500 dark:border-gray-700 dark:bg-gray-900 dark:text-gray-100"
        :class="{ 'border-red-500 focus:border-red-500 focus:ring-red-500': errors.name }"
        :aria-invalid="errors.name ? 'true' : 'false'"
      />
      <p v-if="errors.name" class="mt-1 text-sm text-red-600 dark:text-red-400">
        {{ errors.name }}
      </p>
    </div>

    <div class="grid gap-5 sm:grid-cols-3">
      <div>
        <label
          for="package-price-min"
          class="mb-1 block text-sm font-medium text-gray-700 dark:text-gray-300"
        >
          Price from (USD)
        </label>
        <input
          id="package-price-min"
          v-model="form.price_min"
          type="number"
          name="price_min"
          min="0"
          class="w-full rounded-xl border-gray-300 px-4 py-2.5 text-sm focus:border-blue-500 focus:ring-blue-500 dark:border-gray-700 dark:bg-gray-900 dark:text-gray-100"
        />
      </div>
      <div>
        <label
          for="package-price-max"
          class="mb-1 block text-sm font-medium text-gray-700 dark:text-gray-300"
        >
          Price to (USD)
        </label>
        <input
          id="package-price-max"
          v-model="form.price_max"
          type="number"
          name="price_max"
          min="0"
          class="w-full rounded-xl border-gray-300 px-4 py-2.5 text-sm focus:border-blue-500 focus:ring-blue-500 dark:border-gray-700 dark:bg-gray-900 dark:text-gray-100"
        />
      </div>
      <div>
        <label
          for="package-duration"
          class="mb-1 block text-sm font-medium text-gray-700 dark:text-gray-300"
        >
          Duration (days)
        </label>
        <input
          id="package-duration"
          v-model="form.duration_days"
          type="number"
          name="duration_days"
          min="1"
          class="w-full rounded-xl border-gray-300 px-4 py-2.5 text-sm focus:border-blue-500 focus:ring-blue-500 dark:border-gray-700 dark:bg-gray-900 dark:text-gray-100"
        />
      </div>
    </div>

    <div>
      <label
        for="package-inclusions"
        class="mb-1 block text-sm font-medium text-gray-700 dark:text-gray-300"
      >
        Inclusions
      </label>
      <textarea
        id="package-inclusions"
        v-model="form.inclusions"
        name="inclusions"
        rows="2"
        placeholder="One per line, e.g. Consultation"
        class="w-full rounded-xl border-gray-300 px-4 py-2.5 text-sm focus:border-blue-500 focus:ring-blue-500 dark:border-gray-700 dark:bg-gray-900 dark:text-gray-100"
      />
    </div>

    <div>
      <label
        for="package-exclusions"
        class="mb-1 block text-sm font-medium text-gray-700 dark:text-gray-300"
      >
        Exclusions
      </label>
      <textarea
        id="package-exclusions"
        v-model="form.exclusions"
        name="exclusions"
        rows="2"
        placeholder="One per line, e.g. Flights"
        class="w-full rounded-xl border-gray-300 px-4 py-2.5 text-sm focus:border-blue-500 focus:ring-blue-500 dark:border-gray-700 dark:bg-gray-900 dark:text-gray-100"
      />
    </div>

    <label v-if="isEdit" class="flex items-center gap-2 text-sm text-gray-700 dark:text-gray-300">
      <input
        v-model="form.is_published"
        type="checkbox"
        name="is_published"
        class="rounded border-gray-300 text-blue-600 focus:ring-blue-500 dark:border-gray-700 dark:bg-gray-900"
      />
      Published (visible to patients)
    </label>

    <div
      v-if="errorMessage"
      role="alert"
      class="rounded-xl bg-red-50 p-4 text-sm text-red-800 dark:bg-red-900/30 dark:text-red-200"
    >
      {{ errorMessage }}
    </div>

    <div class="flex items-center gap-3">
      <button
        type="submit"
        :disabled="submitting"
        class="inline-flex items-center justify-center rounded-xl bg-blue-600 px-5 py-2.5 text-sm font-semibold text-white transition-colors hover:bg-blue-700 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-500 focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-60 dark:focus-visible:ring-offset-gray-900"
      >
        {{ submitting ? 'Saving...' : submitLabel }}
      </button>
      <slot name="cancel" />
    </div>
  </form>
</template>

<script setup>
const props = defineProps({
  treatments: { type: Array, default: () => [] },
  initial: { type: Object, default: null },
  submitLabel: { type: String, default: 'Save package' },
  // Async handler so the form can await the API call and surface errors.
  onSubmit: { type: Function, required: true },
})

const isEdit = computed(() => !!props.initial)

const form = reactive({
  treatment_id: props.initial?.treatment_id || '',
  name: props.initial?.name || '',
  price_min: props.initial?.price_min ?? '',
  price_max: props.initial?.price_max ?? '',
  duration_days: props.initial?.duration_days ?? '',
  inclusions: (props.initial?.inclusions || []).join('\n'),
  exclusions: (props.initial?.exclusions || []).join('\n'),
  is_published: props.initial?.is_published ?? false,
})

const errors = reactive({ treatment_id: '', name: '' })
const submitting = ref(false)
const errorMessage = ref('')

function lines(text) {
  return text
    .split('\n')
    .map((l) => l.trim())
    .filter(Boolean)
}

function toInt(value) {
  if (value === '' || value === null || value === undefined) return null
  const n = Number.parseInt(value, 10)
  return Number.isFinite(n) ? n : null
}

function validate() {
  errors.treatment_id = ''
  errors.name = ''
  if (!isEdit.value && !form.treatment_id) {
    errors.treatment_id = 'Please select a treatment.'
  }
  if (!form.name.trim()) errors.name = 'Please enter the package name.'
  return !errors.treatment_id && !errors.name
}

async function handleSubmit() {
  errorMessage.value = ''
  if (!validate()) return

  submitting.value = true
  try {
    const base = {
      name: form.name.trim(),
      price_min: toInt(form.price_min),
      price_max: toInt(form.price_max),
      duration_days: toInt(form.duration_days),
      inclusions: lines(form.inclusions),
      exclusions: lines(form.exclusions),
    }
    await props.onSubmit(
      isEdit.value
        ? { ...base, is_published: form.is_published }
        : { ...base, treatment_id: form.treatment_id }
    )
  } catch (err) {
    errorMessage.value = err?.data?.error || 'Something went wrong. Please try again.'
  } finally {
    submitting.value = false
  }
}
</script>
