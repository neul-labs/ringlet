<script setup lang="ts">
defineProps<{
  status: 'running' | 'stopped' | 'enabled' | 'disabled' | 'error' | 'pending'
}>()

const statusColors = {
  running: 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-300',
  stopped: 'bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-300',
  enabled: 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-300',
  disabled: 'bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-300',
  error: 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-300',
  pending: 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-300',
}
</script>

<template>
  <span
    :class="[
      'inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium capitalize',
      statusColors[status]
    ]"
  >
    {{ status }}
  </span>
</template>
