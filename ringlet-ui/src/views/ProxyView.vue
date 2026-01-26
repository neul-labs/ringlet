<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useProxyStore } from '@/stores/proxy'
import { useProfilesStore } from '@/stores/profiles'
import { api } from '@/api/client'
import type { ProfileProxyConfig } from '@/api/types'
import LoadingSpinner from '@/components/common/LoadingSpinner.vue'
import StatusBadge from '@/components/common/StatusBadge.vue'

const proxyStore = useProxyStore()
const profilesStore = useProfilesStore()

// Store proxy configs for each profile
const proxyConfigs = ref<Record<string, ProfileProxyConfig | null>>({})

onMounted(async () => {
  await Promise.all([
    proxyStore.fetchStatus(),
    profilesStore.fetchProfiles(),
  ])
  // Fetch proxy configs for each profile
  for (const profile of profilesStore.profiles) {
    proxyConfigs.value[profile.alias] = await api.proxy.config(profile.alias).catch(() => null)
  }
})

function isProxyEnabled(alias: string): boolean {
  return proxyConfigs.value[alias]?.enabled ?? false
}

async function startProxy(alias: string) {
  try {
    await proxyStore.startProxy(alias)
  } catch (e) {
    alert((e as Error).message)
  }
}

async function stopProxy(alias: string) {
  try {
    await proxyStore.stopProxy(alias)
  } catch (e) {
    alert((e as Error).message)
  }
}

async function stopAll() {
  if (confirm('Stop all running proxies?')) {
    try {
      await proxyStore.stopAll()
    } catch (e) {
      alert((e as Error).message)
    }
  }
}

function isRunning(alias: string): boolean {
  return proxyStore.instances.some(i => i.alias === alias)
}

function getPort(alias: string): number | undefined {
  return proxyStore.getInstanceByAlias(alias)?.port
}
</script>

<template>
  <div>
    <div class="flex items-center justify-between mb-6">
      <h1 class="text-2xl font-bold text-gray-900 dark:text-white">Proxy Management</h1>
      <button
        v-if="proxyStore.instances.length > 0"
        class="px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 transition-colors"
        @click="stopAll"
      >
        Stop All
      </button>
    </div>

    <div v-if="proxyStore.loading" class="flex justify-center py-12">
      <LoadingSpinner size="lg" />
    </div>

    <div v-else class="space-y-6">
      <!-- Running Proxies -->
      <div class="bg-white dark:bg-gray-800 rounded-lg shadow">
        <div class="px-6 py-4 border-b border-gray-200 dark:border-gray-700">
          <h2 class="text-lg font-semibold text-gray-900 dark:text-white">
            Running Proxies ({{ proxyStore.instances.length }})
          </h2>
        </div>

        <div v-if="proxyStore.instances.length === 0" class="p-6 text-center text-gray-500 dark:text-gray-400">
          No proxies currently running
        </div>

        <ul v-else class="divide-y divide-gray-200 dark:divide-gray-700">
          <li
            v-for="instance in proxyStore.instances"
            :key="instance.alias"
            class="px-6 py-4 flex items-center justify-between"
          >
            <div class="flex items-center space-x-4">
              <div>
                <RouterLink
                  :to="`/profiles/${instance.alias}`"
                  class="font-medium text-blue-600 hover:text-blue-700"
                >
                  {{ instance.alias }}
                </RouterLink>
                <div class="text-sm text-gray-500 dark:text-gray-400">
                  Port: {{ instance.port }}
                </div>
              </div>
            </div>
            <div class="flex items-center space-x-3">
              <StatusBadge status="running" />
              <button
                class="px-3 py-1.5 text-sm text-red-600 hover:bg-red-50 dark:hover:bg-red-900/50 rounded transition-colors"
                @click="stopProxy(instance.alias)"
              >
                Stop
              </button>
            </div>
          </li>
        </ul>
      </div>

      <!-- All Profiles -->
      <div class="bg-white dark:bg-gray-800 rounded-lg shadow">
        <div class="px-6 py-4 border-b border-gray-200 dark:border-gray-700">
          <h2 class="text-lg font-semibold text-gray-900 dark:text-white">
            Profiles
          </h2>
        </div>

        <div v-if="profilesStore.profiles.length === 0" class="p-6 text-center text-gray-500 dark:text-gray-400">
          No profiles found
        </div>

        <ul v-else class="divide-y divide-gray-200 dark:divide-gray-700">
          <li
            v-for="profile in profilesStore.profiles"
            :key="profile.alias"
            class="px-6 py-4 flex items-center justify-between"
          >
            <div>
              <RouterLink
                :to="`/profiles/${profile.alias}`"
                class="font-medium text-blue-600 hover:text-blue-700"
              >
                {{ profile.alias }}
              </RouterLink>
              <div class="text-sm text-gray-500 dark:text-gray-400">
                {{ profile.agent_id }}
                <span v-if="isRunning(profile.alias)">
                  · Port {{ getPort(profile.alias) }}
                </span>
              </div>
            </div>
            <div class="flex items-center space-x-3">
              <StatusBadge :status="isProxyEnabled(profile.alias) ? (isRunning(profile.alias) ? 'running' : 'stopped') : 'disabled'" />
              <button
                v-if="isProxyEnabled(profile.alias) && !isRunning(profile.alias)"
                class="px-3 py-1.5 text-sm bg-green-600 text-white rounded hover:bg-green-700 transition-colors"
                @click="startProxy(profile.alias)"
              >
                Start
              </button>
              <button
                v-else-if="isRunning(profile.alias)"
                class="px-3 py-1.5 text-sm text-red-600 hover:bg-red-50 dark:hover:bg-red-900/50 rounded transition-colors"
                @click="stopProxy(profile.alias)"
              >
                Stop
              </button>
            </div>
          </li>
        </ul>
      </div>

      <!-- Quick Guide -->
      <div class="bg-blue-50 dark:bg-blue-900/30 rounded-lg p-6">
        <h3 class="text-lg font-semibold text-blue-900 dark:text-blue-100 mb-2">Using the Proxy</h3>
        <p class="text-blue-800 dark:text-blue-200 text-sm mb-4">
          Once a proxy is running, configure your AI agent to use it as an API endpoint:
        </p>
        <div v-if="proxyStore.instances.length > 0" class="space-y-2">
          <div
            v-for="instance in proxyStore.instances"
            :key="instance.alias"
            class="bg-white dark:bg-gray-800 rounded p-3"
          >
            <p class="text-sm font-medium text-gray-900 dark:text-white mb-1">{{ instance.alias }}</p>
            <code class="text-sm text-gray-600 dark:text-gray-300 block">
              http://127.0.0.1:{{ instance.port }}/v1
            </code>
          </div>
        </div>
        <p v-else class="text-sm text-blue-700 dark:text-blue-300 italic">
          Start a proxy to see the endpoint URL
        </p>
      </div>
    </div>
  </div>
</template>
