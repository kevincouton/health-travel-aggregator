<template>
  <form class="space-y-5" novalidate @submit.prevent="handleSubmit">
    <input type="hidden" name="clinic_id" :value="clinicId" />
    <input v-if="packageId" type="hidden" name="package_id" :value="packageId" />

    <div>
      <label
        for="quote-email"
        class="mb-1 block text-sm font-medium text-gray-700 dark:text-gray-300"
      >
        Contact email <span aria-label="required">*</span>
      </label>
      <input
        id="quote-email"
        v-model="form.contact_email"
        type="email"
        name="contact_email"
        required
        autocomplete="email"
        class="w-full rounded-xl border-gray-300 px-4 py-2.5 text-sm focus:border-blue-500 focus:ring-blue-500 dark:border-gray-700 dark:bg-gray-900 dark:text-gray-100"
        :class="{ 'border-red-500 focus:border-red-500 focus:ring-red-500': errors.contact_email }"
        :aria-invalid="errors.contact_email ? 'true' : 'false'"
        :aria-describedby="errors.contact_email ? 'quote-email-error' : undefined"
      />
      <p
        v-if="errors.contact_email"
        id="quote-email-error"
        class="mt-1 text-sm text-red-600 dark:text-red-400"
      >
        {{ errors.contact_email }}
      </p>
    </div>

    <div>
      <label
        for="quote-dates"
        class="mb-1 block text-sm font-medium text-gray-700 dark:text-gray-300"
      >
        Preferred dates
      </label>
      <input
        id="quote-dates"
        v-model="form.preferred_dates"
        type="text"
        name="preferred_dates"
        placeholder="e.g. March 2026"
        class="w-full rounded-xl border-gray-300 px-4 py-2.5 text-sm focus:border-blue-500 focus:ring-blue-500 dark:border-gray-700 dark:bg-gray-900 dark:text-gray-100"
      />
    </div>

    <div>
      <label
        for="quote-notes"
        class="mb-1 block text-sm font-medium text-gray-700 dark:text-gray-300"
      >
        Medical notes / questions
      </label>
      <textarea
        id="quote-notes"
        v-model="form.medical_notes"
        name="medical_notes"
        rows="4"
        placeholder="Describe the treatment, travel party, or any special requirements."
        class="w-full rounded-xl border-gray-300 px-4 py-2.5 text-sm focus:border-blue-500 focus:ring-blue-500 dark:border-gray-700 dark:bg-gray-900 dark:text-gray-100"
      />
    </div>

    <div
      v-if="successMessage"
      role="status"
      class="rounded-xl bg-green-50 p-4 text-sm text-green-800 dark:bg-green-900/30 dark:text-green-200"
    >
      {{ successMessage }}
    </div>

    <div
      v-if="errorMessage"
      role="alert"
      class="rounded-xl bg-red-50 p-4 text-sm text-red-800 dark:bg-red-900/30 dark:text-red-200"
    >
      {{ errorMessage }}
    </div>

    <button
      type="submit"
      :disabled="submitting"
      class="inline-flex w-full items-center justify-center rounded-xl bg-blue-600 px-5 py-2.5 text-sm font-semibold text-white transition-colors hover:bg-blue-700 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-500 focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-60 dark:focus-visible:ring-offset-gray-900"
    >
      {{ submitting ? 'Sending...' : 'Request quote' }}
    </button>
  </form>
</template>

<script setup>
const props = defineProps({
  clinicId: { type: String, required: true },
  packageId: { type: String, default: null },
  clinicName: { type: String, default: '' },
  packageName: { type: String, default: '' },
  initialEmail: { type: String, default: '' },
})

const emit = defineEmits(['success'])

const { createInquiry } = useInquiries()

const form = reactive({
  contact_email: props.initialEmail,
  preferred_dates: '',
  medical_notes: '',
})

const errors = reactive({
  contact_email: '',
})

const submitting = ref(false)
const successMessage = ref('')
const errorMessage = ref('')

function validate() {
  errors.contact_email = ''

  const email = form.contact_email.trim()
  if (!email) {
    errors.contact_email = 'Please enter your contact email.'
    return false
  }
  if (!email.includes('@')) {
    errors.contact_email = 'Please enter a valid email address.'
    return false
  }
  return true
}

async function handleSubmit() {
  successMessage.value = ''
  errorMessage.value = ''

  if (!validate()) {
    return
  }

  submitting.value = true
  try {
    const inquiry = await createInquiry({
      clinic_id: props.clinicId,
      package_id: props.packageId || null,
      contact_email: form.contact_email.trim(),
      preferred_dates: form.preferred_dates.trim() || null,
      medical_notes: form.medical_notes.trim() || null,
    })
    successMessage.value = 'Your quote request has been sent. We will be in touch soon.'
    emit('success', inquiry)
  } catch (err) {
    errorMessage.value =
      err?.data?.error || err?.message || 'Something went wrong. Please try again.'
  } finally {
    submitting.value = false
  }
}
</script>
