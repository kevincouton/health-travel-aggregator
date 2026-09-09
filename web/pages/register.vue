<template>
  <div class="mx-auto max-w-md">
    <header class="mb-8 text-center">
      <h1 class="text-3xl font-extrabold tracking-tight text-gray-900 dark:text-gray-100">
        List your clinic
      </h1>
      <p class="mt-2 text-sm text-gray-500 dark:text-gray-400">
        Create a provider account to manage your clinics, packages, and patient inquiries.
      </p>
    </header>

    <div class="rounded-2xl border bg-white p-6 md:p-8 dark:border-gray-800 dark:bg-gray-900">
      <form class="space-y-5" novalidate @submit.prevent="submitRegister">
        <div>
          <label
            for="register-email"
            class="mb-1 block text-sm font-medium text-gray-700 dark:text-gray-300"
          >
            Work email <span aria-label="required">*</span>
          </label>
          <input
            id="register-email"
            v-model="form.email"
            type="email"
            name="email"
            required
            autocomplete="email"
            class="w-full rounded-xl border-gray-300 px-4 py-2.5 text-sm focus:border-blue-500 focus:ring-blue-500 dark:border-gray-700 dark:bg-gray-900 dark:text-gray-100"
            :class="{ 'border-red-500 focus:border-red-500 focus:ring-red-500': errors.email }"
            :aria-invalid="errors.email ? 'true' : 'false'"
            :aria-describedby="errors.email ? 'register-email-error' : undefined"
          />
          <p
            v-if="errors.email"
            id="register-email-error"
            class="mt-1 text-sm text-red-600 dark:text-red-400"
          >
            {{ errors.email }}
          </p>
        </div>

        <div>
          <label
            for="register-password"
            class="mb-1 block text-sm font-medium text-gray-700 dark:text-gray-300"
          >
            Password <span aria-label="required">*</span>
          </label>
          <input
            id="register-password"
            v-model="form.password"
            type="password"
            name="password"
            required
            autocomplete="new-password"
            minlength="8"
            class="w-full rounded-xl border-gray-300 px-4 py-2.5 text-sm focus:border-blue-500 focus:ring-blue-500 dark:border-gray-700 dark:bg-gray-900 dark:text-gray-100"
            :class="{ 'border-red-500 focus:border-red-500 focus:ring-red-500': errors.password }"
            :aria-invalid="errors.password ? 'true' : 'false'"
            :aria-describedby="errors.password ? 'register-password-error' : undefined"
          />
          <p
            v-if="errors.password"
            id="register-password-error"
            class="mt-1 text-sm text-red-600 dark:text-red-400"
          >
            {{ errors.password }}
          </p>
          <p v-else class="mt-1 text-xs text-gray-500 dark:text-gray-400">At least 8 characters.</p>
        </div>

        <div>
          <label
            for="register-confirm"
            class="mb-1 block text-sm font-medium text-gray-700 dark:text-gray-300"
          >
            Confirm password <span aria-label="required">*</span>
          </label>
          <input
            id="register-confirm"
            v-model="form.confirm"
            type="password"
            name="confirm"
            required
            autocomplete="new-password"
            class="w-full rounded-xl border-gray-300 px-4 py-2.5 text-sm focus:border-blue-500 focus:ring-blue-500 dark:border-gray-700 dark:bg-gray-900 dark:text-gray-100"
            :class="{ 'border-red-500 focus:border-red-500 focus:ring-red-500': errors.confirm }"
            :aria-invalid="errors.confirm ? 'true' : 'false'"
            :aria-describedby="errors.confirm ? 'register-confirm-error' : undefined"
          />
          <p
            v-if="errors.confirm"
            id="register-confirm-error"
            class="mt-1 text-sm text-red-600 dark:text-red-400"
          >
            {{ errors.confirm }}
          </p>
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
          {{ submitting ? 'Creating account...' : 'Create provider account' }}
        </button>
      </form>

      <p class="mt-6 text-center text-sm text-gray-500 dark:text-gray-400">
        Already have an account?
        <NuxtLink
          to="/login"
          class="font-medium text-blue-600 hover:text-blue-700 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-500 rounded-md dark:text-blue-400 dark:hover:text-blue-300"
        >
          Sign in
        </NuxtLink>
      </p>
      <p class="mt-2 text-center text-xs text-gray-400 dark:text-gray-500">
        Patients don't need a password — use the email-link tab on the sign-in page.
      </p>
    </div>
  </div>
</template>

<script setup>
useSeo({
  title: 'Create a provider account — Health Travel',
  description: 'Register your clinic on Health Travel to reach international patients.',
})

const { register } = useUser()

const form = reactive({ email: '', password: '', confirm: '' })
const errors = reactive({ email: '', password: '', confirm: '' })
const submitting = ref(false)
const errorMessage = ref('')

function validate() {
  errors.email = ''
  errors.password = ''
  errors.confirm = ''

  const email = form.email.trim()
  if (!email || !email.includes('@')) {
    errors.email = 'Please enter a valid email address.'
  }
  if (form.password.length < 8) {
    errors.password = 'Password must be at least 8 characters.'
  }
  if (form.confirm !== form.password) {
    errors.confirm = 'Passwords do not match.'
  }
  return !errors.email && !errors.password && !errors.confirm
}

async function submitRegister() {
  errorMessage.value = ''
  if (!validate()) {
    return
  }
  submitting.value = true
  try {
    await register(form.email.trim(), form.password)
    await navigateTo('/dashboard')
  } catch (err) {
    errorMessage.value =
      err?.response?.status === 409
        ? 'An account with this email already exists. Try signing in instead.'
        : 'Something went wrong. Please try again.'
  } finally {
    submitting.value = false
  }
}
</script>
