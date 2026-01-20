<script setup lang="ts">
import { onMounted, computed } from 'vue'
import { useAgentsStore } from '@/stores/agents'
import { useProvidersStore } from '@/stores/providers'
import { useProfilesStore } from '@/stores/profiles'
import { useProxyStore } from '@/stores/proxy'
import LoadingSpinner from '@/components/common/LoadingSpinner.vue'

const agentsStore = useAgentsStore()
const providersStore = useProvidersStore()
const profilesStore = useProfilesStore()
const proxyStore = useProxyStore()

const loading = computed(() =>
  agentsStore.loading || providersStore.loading || profilesStore.loading || proxyStore.loading
)

const runningProxies = computed(() => proxyStore.instances.length)

onMounted(async () => {
  await Promise.all([
    agentsStore.fetchAgents(),
    providersStore.fetchProviders(),
    profilesStore.fetchProfiles(),
    proxyStore.fetchStatus(),
  ])
})

const stats = computed(() => [
  { label: 'Agents', value: agentsStore.agents.length, color: 'blue' },
  { label: 'Providers', value: providersStore.providers.length, color: 'green' },
  { label: 'Profiles', value: profilesStore.profiles.length, color: 'purple' },
  { label: 'Running Proxies', value: runningProxies.value, color: 'orange' },
])
</script>

<template>
  <div>
    <h1 class="text-2xl font-bold text-gray-900 dark:text-white mb-6">Dashboard</h1>

    <div v-if="loading" class="flex justify-center py-12">
      <LoadingSpinner size="lg" />
    </div>

    <div v-else class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
      <div
        v-for="stat in stats"
        :key="stat.label"
        class="bg-white dark:bg-gray-800 rounded-lg shadow p-6"
      >
        <div class="text-sm font-medium text-gray-500 dark:text-gray-400">
          {{ stat.label }}
        </div>
        <div
          :class="[
            'text-3xl font-bold mt-2',
            stat.color === 'blue' ? 'text-blue-600' :
            stat.color === 'green' ? 'text-green-600' :
            stat.color === 'purple' ? 'text-purple-600' :
            'text-orange-600'
          ]"
        >
          {{ stat.value }}
        </div>
      </div>
    </div>

    <div class="mt-8 grid grid-cols-1 lg:grid-cols-2 gap-6">
      <!-- Recent Profiles -->
      <div class="bg-white dark:bg-gray-800 rounded-lg shadow">
        <div class="px-6 py-4 border-b border-gray-200 dark:border-gray-700">
          <h2 class="text-lg font-semibold text-gray-900 dark:text-white">Recent Profiles</h2>
        </div>
        <div class="p-6">
          <div v-if="profilesStore.profiles.length === 0" class="text-gray-500 dark:text-gray-400 text-center py-4">
            No profiles created yet
          </div>
          <ul v-else class="divide-y divide-gray-200 dark:divide-gray-700">
            <li
              v-for="profile in profilesStore.profiles.slice(0, 5)"
              :key="profile.alias"
              class="py-3 flex items-center justify-between"
            >
              <div>
                <div class="font-medium text-gray-900 dark:text-white">{{ profile.alias }}</div>
                <div class="text-sm text-gray-500 dark:text-gray-400">{{ profile.agent_id }}</div>
              </div>
              <RouterLink
                :to="`/profiles/${profile.alias}`"
                class="text-blue-600 hover:text-blue-700 text-sm"
              >
                View
              </RouterLink>
            </li>
          </ul>
        </div>
      </div>

      <!-- Running Proxies -->
      <div class="bg-white dark:bg-gray-800 rounded-lg shadow">
        <div class="px-6 py-4 border-b border-gray-200 dark:border-gray-700">
          <h2 class="text-lg font-semibold text-gray-900 dark:text-white">Running Proxies</h2>
        </div>
        <div class="p-6">
          <div v-if="proxyStore.instances.length === 0" class="text-gray-500 dark:text-gray-400 text-center py-4">
            No proxies running
          </div>
          <ul v-else class="divide-y divide-gray-200 dark:divide-gray-700">
            <li
              v-for="instance in proxyStore.instances"
              :key="instance.alias"
              class="py-3 flex items-center justify-between"
            >
              <div>
                <div class="font-medium text-gray-900 dark:text-white">{{ instance.alias }}</div>
                <div class="text-sm text-gray-500 dark:text-gray-400">
                  Port: {{ instance.port }}
                </div>
              </div>
              <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-300">
                Running
              </span>
            </li>
          </ul>
        </div>
      </div>
    </div>
  </div>
</template>
