<template>
  <form class="space-y-5" novalidate @submit.prevent="handleSubmit">
    <div>
      <label
        for="clinic-name"
        class="mb-1 block text-sm font-medium text-gray-700 dark:text-gray-300"
      >
        Clinic name <span aria-label="required">*</span>
      </label>
      <input
        id="clinic-name"
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

    <div>
      <label
        for="clinic-slug"
        class="mb-1 block text-sm font-medium text-gray-700 dark:text-gray-300"
      >
        Slug <span aria-label="required">*</span>
      </label>
      <input
        id="clinic-slug"
        v-model="form.slug"
        type="text"
        name="slug"
        required
        placeholder="e.g. istanbul-smile-clinic"
        class="w-full rounded-xl border-gray-300 px-4 py-2.5 text-sm focus:border-blue-500 focus:ring-blue-500 dark:border-gray-700 dark:bg-gray-900 dark:text-gray-100"
        :class="{ 'border-red-500 focus:border-red-500 focus:ring-red-500': errors.slug }"
        :aria-invalid="errors.slug ? 'true' : 'false'"
      />
      <p v-if="errors.slug" class="mt-1 text-sm text-red-600 dark:text-red-400">
        {{ errors.slug }}
      </p>
      <p v-else class="mt-1 text-xs text-gray-500 dark:text-gray-400">
        Lowercase letters, numbers, and hyphens. Used in the public URL.
      </p>
    </div>

    <div class="grid gap-5 sm:grid-cols-2">
      <div>
        <label
          for="clinic-country"
          class="mb-1 block text-sm font-medium text-gray-700 dark:text-gray-300"
        >
          Country code <span aria-label="required">*</span>
        </label>
        <input
          id="clinic-country"
          v-model="form.country_code"
          type="text"
          name="country_code"
          required
          maxlength="2"
          placeholder="e.g. TR"
          class="w-full rounded-xl border-gray-300 px-4 py-2.5 text-sm uppercase focus:border-blue-500 focus:ring-blue-500 dark:border-gray-700 dark:bg-gray-900 dark:text-gray-100"
          :class="{
            'border-red-500 focus:border-red-500 focus:ring-red-500': errors.country_code,
          }"
          :aria-invalid="errors.country_code ? 'true' : 'false'"
        />
        <p v-if="errors.country_code" class="mt-1 text-sm text-red-600 dark:text-red-400">
          {{ errors.country_code }}
        </p>
      </div>
      <div>
        <label
          for="clinic-city"
          class="mb-1 block text-sm font-medium text-gray-700 dark:text-gray-300"
        >
          City <span aria-label="required">*</span>
        </label>
        <input
          id="clinic-city"
          v-model="form.city"
          type="text"
          name="city"
          required
          class="w-full rounded-xl border-gray-300 px-4 py-2.5 text-sm focus:border-blue-500 focus:ring-blue-500 dark:border-gray-700 dark:bg-gray-900 dark:text-gray-100"
          :class="{ 'border-red-500 focus:border-red-500 focus:ring-red-500': errors.city }"
          :aria-invalid="errors.city ? 'true' : 'false'"
        />
        <p v-if="errors.city" class="mt-1 text-sm text-red-600 dark:text-red-400">
          {{ errors.city }}
        </p>
      </div>
    </div>

    <div>
      <label
        for="clinic-accreditations"
        class="mb-1 block text-sm font-medium text-gray-700 dark:text-gray-300"
      >
        Accreditations
      </label>
      <input
        id="clinic-accreditations"
        v-model="form.accreditations"
        type="text"
        name="accreditations"
        placeholder="e.g. JCI, ISO 9001"
        class="w-full rounded-xl border-gray-300 px-4 py-2.5 text-sm focus:border-blue-500 focus:ring-blue-500 dark:border-gray-700 dark:bg-gray-900 dark:text-gray-100"
      />
      <p class="mt-1 text-xs text-gray-500 dark:text-gray-400">Comma-separated.</p>
    </div>

    <div>
      <label
        for="clinic-description"
        class="mb-1 block text-sm font-medium text-gray-700 dark:text-gray-300"
      >
        Description
      </label>
      <textarea
        id="clinic-description"
        v-model="form.description"
        name="description"
        rows="4"
        class="w-full rounded-xl border-gray-300 px-4 py-2.5 text-sm focus:border-blue-500 focus:ring-blue-500 dark:border-gray-700 dark:bg-gray-900 dark:text-gray-100"
      />
    </div>

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
  initial: { type: Object, default: null },
  submitLabel: { type: String, default: 'Save clinic' },
  // Async handler so the form can await the API call and surface errors.
  onSubmit: { type: Function, required: true },
})

const form = reactive({
  name: props.initial?.name || '',
  slug: props.initial?.slug || '',
  country_code: props.initial?.country_code || '',
  city: props.initial?.city || '',
  accreditations: (props.initial?.accreditations || []).join(', '),
  description: props.initial?.description || '',
})

const errors = reactive({ name: '', slug: '', country_code: '', city: '' })
const submitting = ref(false)
const errorMessage = ref('')

const SLUG_RE = /^[a-z0-9]+(?:-[a-z0-9]+)*$/

function validate() {
  errors.name = ''
  errors.slug = ''
  errors.country_code = ''
  errors.city = ''

  if (!form.name.trim()) errors.name = 'Please enter the clinic name.'
  if (!SLUG_RE.test(form.slug.trim())) {
    errors.slug = 'Use lowercase letters, numbers, and hyphens only.'
  }
  if (!/^[A-Za-z]{2}$/.test(form.country_code.trim())) {
    errors.country_code = 'Use the two-letter ISO code, e.g. TR.'
  }
  if (!form.city.trim()) errors.city = 'Please enter the city.'
  return !errors.name && !errors.slug && !errors.country_code && !errors.city
}

async function handleSubmit() {
  errorMessage.value = ''
  if (!validate()) return

  submitting.value = true
  try {
    await props.onSubmit({
      name: form.name.trim(),
      slug: form.slug.trim(),
      country_code: form.country_code.trim().toUpperCase(),
      city: form.city.trim(),
      accreditations: form.accreditations
        .split(',')
        .map((a) => a.trim())
        .filter(Boolean),
      description: form.description.trim() || null,
    })
  } catch (err) {
    errorMessage.value =
      err?.response?.status === 409
        ? 'That slug is already taken. Please choose another.'
        : err?.data?.error || 'Something went wrong. Please try again.'
  } finally {
    submitting.value = false
  }
}
</script>
