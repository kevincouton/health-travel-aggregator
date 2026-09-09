<template>
  <span
    class="inline-block rounded-full px-2.5 py-0.5 text-[10px] font-bold uppercase tracking-wide"
    :class="badgeClass"
  >
    {{ label }}
  </span>
</template>

<script setup>
const props = defineProps({
  status: { type: String, required: true },
})

const LABELS = {
  new: 'New',
  contacted: 'Contacted',
  converted: 'Converted',
  closed: 'Closed',
}

const CLASSES = {
  new: 'bg-blue-50 text-blue-700 dark:bg-blue-900/30 dark:text-blue-300',
  contacted: 'bg-amber-50 text-amber-700 dark:bg-amber-900/30 dark:text-amber-300',
  converted: 'bg-green-50 text-green-700 dark:bg-green-900/30 dark:text-green-300',
  closed: 'bg-gray-100 text-gray-600 dark:bg-gray-800 dark:text-gray-400',
}

const label = computed(() => LABELS[props.status] || props.status)
const badgeClass = computed(() => CLASSES[props.status] || CLASSES.closed)
</script>
