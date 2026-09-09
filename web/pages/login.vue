<template>
  <div class="mx-auto max-w-md">
    <header class="mb-8 text-center">
      <h1 class="text-3xl font-extrabold tracking-tight text-gray-900 dark:text-gray-100">
        Sign in
      </h1>
      <p class="mt-2 text-sm text-gray-500 dark:text-gray-400">
        Access your Health Travel account.
      </p>
    </header>

    <div
      v-if="magicLinkSent"
      class="rounded-2xl border bg-white p-8 text-center dark:border-gray-800 dark:bg-gray-900"
      data-testid="check-your-email"
    >
      <h2 class="text-xl font-semibold text-gray-900 dark:text-gray-100">Check your email</h2>
      <p class="mt-2 text-sm text-gray-600 dark:text-gray-300">
        If an account exists for <span class="font-medium">{{ magicLinkEmail }}</span
        >, we sent a sign-in link. It expires in one hour.
      </p>
      <button
        type="button"
        class="mt-6 text-sm font-medium text-blue-600 hover:text-blue-700 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-500 rounded-md dark:text-blue-400 dark:hover:text-blue-300"
        @click="magicLinkSent = false"
      >
        Use a different email
      </button>
    </div>

    <div
      v-else
      class="rounded-2xl border bg-white p-6 md:p-8 dark:border-gray-800 dark:bg-gray-900"
    >
      <div
        class="mb-6 grid grid-cols-2 gap-1 rounded-xl bg-gray-100 p-1 text-sm font-medium dark:bg-gray-800"
        role="tablist"
      >
        <button
          type="button"
          role="tab"
          :aria-selected="mode === 'password'"
          class="rounded-lg px-3 py-2 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-500"
          :class="
            mode === 'password'
              ? 'bg-white text-gray-900 shadow-sm dark:bg-gray-900 dark:text-gray-100'
              : 'text-gray-500 hover:text-gray-900 dark:text-gray-400 dark:hover:text-gray-100'
          "
          @click="mode = 'password'"
        >
          Password
        </button>
        <button
          type="button"
          role="tab"
          :aria-selected="mode === 'magic'"
          class="rounded-lg px-3 py-2 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-500"
          :class="
            mode === 'magic'
              ? 'bg-white text-gray-900 shadow-sm dark:bg-gray-900 dark:text-gray-100'
              : 'text-gray-500 hover:text-gray-900 dark:text-gray-400 dark:hover:text-gray-100'
          "
          @click="mode = 'magic'"
        >
          Email link
        </button>
      </div>

      <form v-if="mode === 'password'" class="space-y-5" novalidate @submit.prevent="submitLogin">
        <div>
          <label
            for="login-email"
            class="mb-1 block text-sm font-medium text-gray-700 dark:text-gray-300"
          >
            Email <span aria-label="required">*</span>
          </label>
          <input
            id="login-email"
            v-model="loginForm.email"
            type="email"
            name="email"
            required
            autocomplete="email"
            class="w-full rounded-xl border-gray-300 px-4 py-2.5 text-sm focus:border-blue-500 focus:ring-blue-500 dark:border-gray-700 dark:bg-gray-900 dark:text-gray-100"
          />
        </div>
        <div>
          <label
            for="login-password"
            class="mb-1 block text-sm font-medium text-gray-700 dark:text-gray-300"
          >
            Password <span aria-label="required">*</span>
          </label>
          <input
            id="login-password"
            v-model="loginForm.password"
            type="password"
            name="password"
            required
            autocomplete="current-password"
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

        <button
          type="submit"
          :disabled="submitting"
          class="inline-flex w-full items-center justify-center rounded-xl bg-blue-600 px-5 py-2.5 text-sm font-semibold text-white transition-colors hover:bg-blue-700 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-500 focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-60 dark:focus-visible:ring-offset-gray-900"
        >
          {{ submitting ? 'Signing in...' : 'Sign in' }}
        </button>
      </form>

      <form v-else class="space-y-5" novalidate @submit.prevent="submitMagicLink">
        <p class="text-sm text-gray-500 dark:text-gray-400">
          We email you a one-time sign-in link — no password needed. New here? This creates your
          patient account automatically.
        </p>
        <div>
          <label
            for="magic-email"
            class="mb-1 block text-sm font-medium text-gray-700 dark:text-gray-300"
          >
            Email <span aria-label="required">*</span>
          </label>
          <input
            id="magic-email"
            v-model="magicForm.email"
            type="email"
            name="email"
            required
            autocomplete="email"
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

        <button
          type="submit"
          :disabled="submitting"
          class="inline-flex w-full items-center justify-center rounded-xl bg-blue-600 px-5 py-2.5 text-sm font-semibold text-white transition-colors hover:bg-blue-700 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-500 focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-60 dark:focus-visible:ring-offset-gray-900"
        >
          {{ submitting ? 'Sending...' : 'Email me a sign-in link' }}
        </button>
      </form>

      <p class="mt-6 text-center text-sm text-gray-500 dark:text-gray-400">
        Are you a clinic?
        <NuxtLink
          to="/register"
          class="font-medium text-blue-600 hover:text-blue-700 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-500 rounded-md dark:text-blue-400 dark:hover:text-blue-300"
        >
          Create a provider account
        </NuxtLink>
      </p>
    </div>
  </div>
</template>

<script setup>
useSeo({
  title: 'Sign in — Health Travel',
  description: 'Sign in to your Health Travel account.',
})

const route = useRoute()
const { login, requestMagicLink } = useUser()

const mode = ref('password')
const loginForm = reactive({ email: '', password: '' })
const magicForm = reactive({ email: '' })
const submitting = ref(false)
const errorMessage = ref('')
const magicLinkSent = ref(false)
const magicLinkEmail = ref('')

const redirectTarget = computed(() => {
  const target = route.query.redirect
  return typeof target === 'string' && target.startsWith('/') ? target : ''
})

async function submitLogin() {
  errorMessage.value = ''
  submitting.value = true
  try {
    const user = await login(loginForm.email.trim(), loginForm.password)
    await navigateTo(redirectTarget.value || homeForRole(user.role))
  } catch (err) {
    errorMessage.value =
      err?.response?.status === 401
        ? 'Invalid email or password.'
        : 'Something went wrong. Please try again.'
  } finally {
    submitting.value = false
  }
}

async function submitMagicLink() {
  errorMessage.value = ''
  submitting.value = true
  try {
    const email = magicForm.email.trim()
    await requestMagicLink(email)
    magicLinkEmail.value = email
    magicLinkSent.value = true
  } catch {
    errorMessage.value = 'Something went wrong. Please try again.'
  } finally {
    submitting.value = false
  }
}
</script>
