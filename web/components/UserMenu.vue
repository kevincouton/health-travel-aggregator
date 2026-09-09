<template>
  <div class="relative">
    <NuxtLink
      v-if="!user"
      to="/login"
      class="rounded-md text-sm text-gray-600 hover:text-gray-900 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-500 dark:text-gray-400 dark:hover:text-gray-100"
    >
      Sign in
    </NuxtLink>

    <div v-else>
      <button
        ref="buttonRef"
        type="button"
        :aria-expanded="isOpen"
        aria-haspopup="menu"
        aria-label="Account menu"
        class="inline-flex items-center gap-1.5 rounded-lg px-2.5 py-2 text-sm font-medium text-gray-700 hover:bg-gray-100 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-500 dark:text-gray-300 dark:hover:bg-gray-800"
        @click="isOpen = !isOpen"
      >
        <span class="max-w-36 truncate">{{ user.display_name || user.email }}</span>
        <svg
          xmlns="http://www.w3.org/2000/svg"
          class="h-4 w-4"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
          stroke-width="2"
          aria-hidden="true"
        >
          <path stroke-linecap="round" stroke-linejoin="round" d="M19 9l-7 7-7-7" />
        </svg>
      </button>

      <div
        v-show="isOpen"
        ref="menuRef"
        role="menu"
        class="absolute right-0 top-full z-50 mt-1 w-52 rounded-xl border bg-white py-1 shadow-lg dark:border-gray-800 dark:bg-gray-900"
      >
        <p class="truncate px-4 py-2 text-xs text-gray-500 dark:text-gray-400" role="none">
          {{ user.email }}
        </p>
        <NuxtLink
          v-if="user.role === 'provider_admin'"
          to="/dashboard"
          role="menuitem"
          class="block px-4 py-2 text-sm text-gray-700 hover:bg-gray-50 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-blue-500 dark:text-gray-300 dark:hover:bg-gray-800"
          @click="isOpen = false"
        >
          Provider dashboard
        </NuxtLink>
        <NuxtLink
          v-if="user.role === 'platform_admin'"
          to="/admin"
          role="menuitem"
          class="block px-4 py-2 text-sm text-gray-700 hover:bg-gray-50 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-blue-500 dark:text-gray-300 dark:hover:bg-gray-800"
          @click="isOpen = false"
        >
          Admin moderation
        </NuxtLink>
        <NuxtLink
          to="/account"
          role="menuitem"
          class="block px-4 py-2 text-sm text-gray-700 hover:bg-gray-50 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-blue-500 dark:text-gray-300 dark:hover:bg-gray-800"
          @click="isOpen = false"
        >
          My inquiries
        </NuxtLink>
        <button
          type="button"
          role="menuitem"
          class="block w-full px-4 py-2 text-left text-sm text-gray-700 hover:bg-gray-50 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-blue-500 dark:text-gray-300 dark:hover:bg-gray-800"
          @click="onLogout"
        >
          Sign out
        </button>
      </div>
    </div>
  </div>
</template>

<script setup>
const { user, fetchUser, logout } = useUser()

const isOpen = ref(false)
const menuRef = ref(null)
const buttonRef = ref(null)

onMounted(() => {
  fetchUser()
  document.addEventListener('click', onClickOutside)
  document.addEventListener('keydown', onKeydown)
})

onBeforeUnmount(() => {
  document.removeEventListener('click', onClickOutside)
  document.removeEventListener('keydown', onKeydown)
})

function onClickOutside(e) {
  if (
    isOpen.value &&
    menuRef.value &&
    !menuRef.value.contains(e.target) &&
    !buttonRef.value?.contains(e.target)
  ) {
    isOpen.value = false
  }
}

function onKeydown(e) {
  if (e.key === 'Escape' && isOpen.value) {
    isOpen.value = false
    nextTick(() => buttonRef.value?.focus())
  }
}

async function onLogout() {
  isOpen.value = false
  await logout()
  await navigateTo('/')
}
</script>
