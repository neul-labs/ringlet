<script setup lang="ts">
import type { GitInfo } from '@/api/types'
import { relativeDate } from '@/utils/date'

defineProps<{
  gitInfo: GitInfo | null
  loading: boolean
  error: string | null
}>()
</script>

<template>
  <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-5">
    <h3 class="text-sm font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider mb-4">Git</h3>

    <!-- Loading -->
    <div v-if="loading" class="flex items-center gap-2 text-sm text-gray-400">
      <div class="w-4 h-4 border-2 border-gray-300 border-t-blue-500 rounded-full animate-spin"></div>
      Loading git info...
    </div>

    <!-- Error -->
    <div v-else-if="error" class="text-sm text-red-500">{{ error }}</div>

    <!-- Not a repo -->
    <div v-else-if="!gitInfo?.is_repo" class="text-sm text-gray-400 italic">
      Not a git repository
    </div>

    <!-- Git info -->
    <div v-else class="space-y-4">
      <!-- Branch + status -->
      <div class="flex items-center gap-3">
        <span class="text-lg font-mono font-semibold text-gray-900 dark:text-white">
          {{ gitInfo.branch }}
        </span>
        <span
          :class="[
            'px-2 py-0.5 text-xs rounded-full',
            gitInfo.dirty
              ? 'bg-yellow-100 dark:bg-yellow-900/30 text-yellow-700 dark:text-yellow-300'
              : 'bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-300'
          ]"
        >
          {{ gitInfo.dirty ? 'dirty' : 'clean' }}
        </span>
      </div>

      <!-- Remote URL -->
      <div v-if="gitInfo.remote_url" class="text-xs font-mono text-gray-400 truncate">
        {{ gitInfo.remote_url }}
      </div>

      <!-- Commits -->
      <div v-if="gitInfo.commits.length > 0">
        <h4 class="text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider mb-2">
          Recent Commits
        </h4>
        <div class="space-y-2">
          <div
            v-for="commit in gitInfo.commits"
            :key="commit.hash"
            class="flex items-start gap-2 text-sm"
          >
            <span class="font-mono text-blue-600 dark:text-blue-400 flex-shrink-0">{{ commit.hash }}</span>
            <span class="text-gray-700 dark:text-gray-300 truncate flex-1">{{ commit.message }}</span>
            <span class="text-xs text-gray-400 flex-shrink-0">{{ relativeDate(commit.date) }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
