<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { api } from '@/api/client'
import type { StatsResponse } from '@/api/types'
import LoadingSpinner from '@/components/common/LoadingSpinner.vue'

const stats = ref<StatsResponse | null>(null)
const loading = ref(true)
const error = ref<string | null>(null)

onMounted(async () => {
  await fetchStats()
})

async function fetchStats() {
  loading.value = true
  error.value = null
  try {
    stats.value = await api.stats.get()
  } catch (e) {
    error.value = (e as Error).message
  } finally {
    loading.value = false
  }
}

function formatNumber(n: number): string {
  return new Intl.NumberFormat().format(n)
}

function formatDuration(seconds: number): string {
  if (seconds < 60) return `${seconds}s`
  if (seconds < 3600) return `${Math.floor(seconds / 60)}m ${seconds % 60}s`
  const hours = Math.floor(seconds / 3600)
  const mins = Math.floor((seconds % 3600) / 60)
  return `${hours}h ${mins}m`
}
</script>

<template>
  <div>
    <div class="flex items-center justify-between mb-6">
      <h1 class="text-2xl font-bold text-gray-900 dark:text-white">Statistics</h1>
      <button
        class="px-4 py-2 text-gray-600 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors"
        @click="fetchStats"
      >
        Refresh
      </button>
    </div>

    <div v-if="loading" class="flex justify-center py-12">
      <LoadingSpinner size="lg" />
    </div>

    <div v-else-if="error" class="bg-red-50 dark:bg-red-900/50 text-red-700 dark:text-red-300 p-4 rounded-lg">
      {{ error }}
    </div>

    <div v-else-if="stats" class="space-y-6">
      <!-- Overview Cards -->
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
          <div class="text-sm font-medium text-gray-500 dark:text-gray-400">Total Sessions</div>
          <div class="text-3xl font-bold text-blue-600 mt-2">
            {{ formatNumber(stats.total_sessions) }}
          </div>
        </div>
        <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
          <div class="text-sm font-medium text-gray-500 dark:text-gray-400">Total Runtime</div>
          <div class="text-3xl font-bold text-green-600 mt-2">
            {{ formatDuration(stats.total_runtime_secs) }}
          </div>
        </div>
        <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
          <div class="text-sm font-medium text-gray-500 dark:text-gray-400">Active Agents</div>
          <div class="text-3xl font-bold text-purple-600 mt-2">
            {{ Object.keys(stats.by_agent).length }}
          </div>
        </div>
      </div>

      <!-- Per-Agent Stats -->
      <div class="bg-white dark:bg-gray-800 rounded-lg shadow">
        <div class="px-6 py-4 border-b border-gray-200 dark:border-gray-700">
          <h2 class="text-lg font-semibold text-gray-900 dark:text-white">Usage by Agent</h2>
        </div>

        <div v-if="Object.keys(stats.by_agent).length === 0" class="p-6 text-center text-gray-500 dark:text-gray-400">
          No agent usage data available
        </div>

        <div v-else class="overflow-x-auto">
          <table class="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
            <thead class="bg-gray-50 dark:bg-gray-900">
              <tr>
                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">Agent</th>
                <th class="px-6 py-3 text-right text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">Sessions</th>
                <th class="px-6 py-3 text-right text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">Runtime</th>
                <th class="px-6 py-3 text-right text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">Profiles</th>
              </tr>
            </thead>
            <tbody class="divide-y divide-gray-200 dark:divide-gray-700">
              <tr v-for="(agentStats, agentId) in stats.by_agent" :key="agentId">
                <td class="px-6 py-4 whitespace-nowrap font-medium text-gray-900 dark:text-white">
                  {{ agentId }}
                </td>
                <td class="px-6 py-4 whitespace-nowrap text-right text-gray-500 dark:text-gray-400">
                  {{ formatNumber(agentStats.sessions) }}
                </td>
                <td class="px-6 py-4 whitespace-nowrap text-right text-gray-500 dark:text-gray-400">
                  {{ formatDuration(agentStats.runtime_secs) }}
                </td>
                <td class="px-6 py-4 whitespace-nowrap text-right font-medium text-gray-900 dark:text-white">
                  {{ agentStats.profiles }}
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

      <!-- Per-Provider Stats -->
      <div class="bg-white dark:bg-gray-800 rounded-lg shadow">
        <div class="px-6 py-4 border-b border-gray-200 dark:border-gray-700">
          <h2 class="text-lg font-semibold text-gray-900 dark:text-white">Usage by Provider</h2>
        </div>

        <div v-if="Object.keys(stats.by_provider).length === 0" class="p-6 text-center text-gray-500 dark:text-gray-400">
          No provider usage data available
        </div>

        <div v-else class="overflow-x-auto">
          <table class="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
            <thead class="bg-gray-50 dark:bg-gray-900">
              <tr>
                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">Provider</th>
                <th class="px-6 py-3 text-right text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">Sessions</th>
                <th class="px-6 py-3 text-right text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">Runtime</th>
              </tr>
            </thead>
            <tbody class="divide-y divide-gray-200 dark:divide-gray-700">
              <tr v-for="(providerStats, providerId) in stats.by_provider" :key="providerId">
                <td class="px-6 py-4 whitespace-nowrap font-medium text-gray-900 dark:text-white">
                  {{ providerId }}
                </td>
                <td class="px-6 py-4 whitespace-nowrap text-right text-gray-500 dark:text-gray-400">
                  {{ formatNumber(providerStats.sessions) }}
                </td>
                <td class="px-6 py-4 whitespace-nowrap text-right text-gray-500 dark:text-gray-400">
                  {{ formatDuration(providerStats.runtime_secs) }}
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

      <!-- Per-Profile Stats -->
      <div class="bg-white dark:bg-gray-800 rounded-lg shadow">
        <div class="px-6 py-4 border-b border-gray-200 dark:border-gray-700">
          <h2 class="text-lg font-semibold text-gray-900 dark:text-white">Usage by Profile</h2>
        </div>

        <div v-if="Object.keys(stats.by_profile).length === 0" class="p-6 text-center text-gray-500 dark:text-gray-400">
          No profile usage data available
        </div>

        <div v-else class="overflow-x-auto">
          <table class="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
            <thead class="bg-gray-50 dark:bg-gray-900">
              <tr>
                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">Profile</th>
                <th class="px-6 py-3 text-right text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">Sessions</th>
                <th class="px-6 py-3 text-right text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">Runtime</th>
                <th class="px-6 py-3 text-right text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">Last Used</th>
              </tr>
            </thead>
            <tbody class="divide-y divide-gray-200 dark:divide-gray-700">
              <tr v-for="(profileStats, profileId) in stats.by_profile" :key="profileId">
                <td class="px-6 py-4 whitespace-nowrap">
                  <RouterLink
                    :to="`/profiles/${profileId}`"
                    class="font-medium text-blue-600 hover:text-blue-700"
                  >
                    {{ profileId }}
                  </RouterLink>
                </td>
                <td class="px-6 py-4 whitespace-nowrap text-right text-gray-500 dark:text-gray-400">
                  {{ formatNumber(profileStats.sessions) }}
                </td>
                <td class="px-6 py-4 whitespace-nowrap text-right text-gray-500 dark:text-gray-400">
                  {{ formatDuration(profileStats.runtime_secs) }}
                </td>
                <td class="px-6 py-4 whitespace-nowrap text-right text-gray-500 dark:text-gray-400">
                  {{ profileStats.last_used ? new Date(profileStats.last_used).toLocaleDateString() : '-' }}
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </div>
  </div>
</template>
