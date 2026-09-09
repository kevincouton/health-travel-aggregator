<template>
  <div class="mx-auto max-w-md">
    <div class="rounded-2xl border bg-white p-8 text-center dark:border-gray-800 dark:bg-gray-900">
      <template v-if="state === 'verifying'">
        <h1 class="text-xl font-semibold text-gray-900 dark:text-gray-100">Signing you in...</h1>
        <p class="mt-2 text-sm text-gray-500 dark:text-gray-400">
          Hold on while we verify your sign-in link.
        </p>
      </template>

      <template v-else-if="state === 'error'">
        <h1 class="text-xl font-semibold text-gray-900 dark:text-gray-100">
          This link didn't work
        </h1>
        <p class="mt-2 text-sm text-gray-600 dark:text-gray-300">
          The sign-in link is invalid or has expired. Request a new one to continue.
        </p>
        <NuxtLink
          to="/login"
          class="mt-6 inline-flex rounded-xl bg-blue-600 px-5 py-2.5 text-sm font-semibold text-white transition-colors hover:bg-blue-700 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-500 focus-visible:ring-offset-2 dark:focus-visible:ring-offset-gray-900"
        >
          Back to sign in
        </NuxtLink>
      </template>
    </div>
  </div>
</template>

<script setup>
useSeo({
  title: 'Verifying sign-in link — Health Travel',
  description: 'Verifying your Health Travel sign-in link.',
})

const route = useRoute()
const { verifyMagicLink } = useUser()

const state = ref('verifying')

onMounted(async () => {
  const token = typeof route.query.token === 'string' ? route.query.token : ''
  if (!token) {
    state.value = 'error'
    return
  }
  try {
    const user = await verifyMagicLink(token)
    const target = route.query.redirect
    await navigateTo(
      typeof target === 'string' && target.startsWith('/') ? target : homeForRole(user.role)
    )
  } catch {
    state.value = 'error'
  }
})
</script>
