<template>
  <section v-if="reviews && reviews.length" class="mt-10">
    <h2 class="mb-5 text-xl font-bold text-gray-900 dark:text-gray-100">Patient reviews</h2>
    <ul class="space-y-4">
      <li
        v-for="review in reviews"
        :key="review.id"
        class="rounded-2xl border bg-white p-5 dark:border-gray-800 dark:bg-gray-900"
      >
        <div
          class="mb-2 flex items-center gap-1 text-yellow-500"
          role="img"
          :aria-label="`Rated ${review.rating} out of 5 stars`"
        >
          <span
            v-for="n in 5"
            :key="n"
            aria-hidden="true"
            :class="n <= review.rating ? 'opacity-100' : 'opacity-30'"
          >
            ★
          </span>
        </div>
        <p v-if="review.comment" class="text-gray-700 dark:text-gray-300">{{ review.comment }}</p>
        <p class="mt-2 text-xs text-gray-400 dark:text-gray-500">
          {{ formatDate(review.created_at) }}
        </p>
      </li>
    </ul>
  </section>
</template>

<script setup>
const props = defineProps({
  reviews: { type: Array, default: () => [] },
})

function formatDate(value) {
  if (!value) return ''
  return new Date(value).toLocaleDateString()
}
</script>
