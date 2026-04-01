<script setup lang="ts">
import type { GitInfo } from '@/api/types'
import { relativeDate } from '@/utils/date'
import { computed } from 'vue'

const props = defineProps<{
  path: string
  gitInfo?: GitInfo | null
  gitInfoLoading?: boolean
  isBookmarked: boolean
  lastOpened?: string
}>()

const emit = defineEmits<{
  open: []
  'bookmark-toggle': []
}>()

const folderName = computed(() => {
  const parts = props.path.split('/').filter(Boolean)
  return parts[parts.length - 1] || props.path
})

const lastCommit = computed(() => {
  if (!props.gitInfo?.commits?.length) return null
  return props.gitInfo.commits[0]
})
</script>

<template>
  <div
    class="bg-white dark:bg-gray-800 rounded-lg shadow hover:shadow-md transition-shadow cursor-pointer border border-gray-200 dark:border-gray-700"
    @click="emit('open')"
  >
    <div class="p-5">
      <div class="flex items-start justify-between">
        <div class="min-w-0 flex-1">
          <h3 class="text-lg font-semibold text-gray-900 dark:text-white truncate">
            {{ folderName }}
          </h3>
          <p class="text-xs text-gray-400 dark:text-gray-500 font-mono truncate mt-0.5">
            {{ path }}
          </p>
        </div>
        <button
          type="button"
          class="ml-2 p-1 rounded hover:bg-gray-100 dark:hover:bg-gray-700 flex-shrink-0"
          @click.stop="emit('bookmark-toggle')"
        >
          <svg
            class="w-5 h-5"
            :class="isBookmarked ? 'text-yellow-400' : 'text-gray-300 dark:text-gray-600'"
            :fill="isBookmarked ? 'currentColor' : 'none'"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11.049 2.927c.3-.921 1.603-.921 1.902 0l1.519 4.674a1 1 0 00.95.69h4.915c.969 0 1.371 1.24.588 1.81l-3.976 2.888a1 1 0 00-.363 1.118l1.518 4.674c.3.922-.755 1.688-1.538 1.118l-3.976-2.888a1 1 0 00-1.176 0l-3.976 2.888c-.783.57-1.838-.197-1.538-1.118l1.518-4.674a1 1 0 00-.363-1.118l-3.976-2.888c-.784-.57-.38-1.81.588-1.81h4.914a1 1 0 00.951-.69l1.519-4.674z" />
          </svg>
        </button>
      </div>

      <div class="mt-3">
        <!-- Loading state -->
        <div v-if="gitInfoLoading" class="flex items-center gap-2 text-sm text-gray-400">
          <div class="w-3 h-3 border-2 border-gray-300 border-t-blue-500 rounded-full animate-spin"></div>
          Loading...
        </div>
        <!-- Git info -->
        <template v-else-if="gitInfo?.is_repo">
          <div class="flex items-center gap-2 mb-2">
            <span class="inline-flex items-center gap-1 px-2 py-0.5 text-xs font-mono rounded-full bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300">
              <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7h8m0 0v8m0-8l-8 8-4-4-6 6" />
              </svg>
              {{ gitInfo.branch }}
            </span>
            <span
              v-if="gitInfo.dirty"
              class="px-2 py-0.5 text-xs rounded-full bg-yellow-100 dark:bg-yellow-900/30 text-yellow-700 dark:text-yellow-300"
            >
              modified
            </span>
            <span
              v-else
              class="px-2 py-0.5 text-xs rounded-full bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-300"
            >
              clean
            </span>
          </div>
          <div v-if="lastCommit" class="text-sm text-gray-600 dark:text-gray-400 truncate">
            <span class="font-mono text-blue-600 dark:text-blue-400">{{ lastCommit.hash }}</span>
            {{ lastCommit.message }}
          </div>
          <div v-if="lastCommit" class="text-xs text-gray-400 dark:text-gray-500 mt-1">
            {{ relativeDate(lastCommit.date) }}
          </div>
        </template>
        <!-- Not a git repo -->
        <div v-else class="text-sm text-gray-400 dark:text-gray-500 italic">
          Not a git repository
        </div>
      </div>

      <div v-if="lastOpened" class="mt-3 pt-3 border-t border-gray-100 dark:border-gray-700">
        <span class="text-xs text-gray-400">Opened {{ relativeDate(lastOpened) }}</span>
      </div>
    </div>
  </div>
</template>
