<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useAgentsStore } from '@/stores/agents'
import { useProvidersStore } from '@/stores/providers'
import { useProfilesStore } from '@/stores/profiles'
import { useTerminalStore } from '@/stores/terminal'

const props = defineProps<{
  workspacePath: string
}>()

const router = useRouter()
const agentsStore = useAgentsStore()
const providersStore = useProvidersStore()
const profilesStore = useProfilesStore()
const terminalStore = useTerminalStore()

const selectedProviders = ref<Record<string, string>>({})
const runningAgent = ref<string | null>(null)
const error = ref<string | null>(null)

const installedAgents = computed(() =>
  agentsStore.agents.filter(a => a.installed)
)

const workspaceProfiles = computed(() =>
  profilesStore.profiles.filter(p => {
    const agent = agentsStore.getAgentById(p.agent_id)
    return agent?.installed
  })
)

onMounted(async () => {
  await Promise.all([
    agentsStore.fetchAgents(),
    providersStore.fetchProviders(),
    profilesStore.fetchProfiles(),
  ])

  // Pre-select default providers
  for (const agent of installedAgents.value) {
    if (agent.default_provider) {
      selectedProviders.value[agent.id] = agent.default_provider
    } else if (providersStore.providers.length > 0) {
      selectedProviders.value[agent.id] = providersStore.providers[0].id
    }
  }
})

async function runAgent(agentId: string) {
  const providerId = selectedProviders.value[agentId]
  if (!providerId) {
    error.value = 'Please select a provider'
    return
  }

  error.value = null
  runningAgent.value = agentId

  try {
    // Check if a profile exists for this combo
    let profileAlias = profilesStore.profiles.find(
      p => p.agent_id === agentId && p.provider_id === providerId
    )?.alias

    // Create auto-named profile if needed
    if (!profileAlias) {
      profileAlias = `ws-${agentId}-${providerId}`
      await profilesStore.createProfile({
        agent_id: agentId,
        alias: profileAlias,
        provider_id: providerId,
        api_key: '',
        hooks: [],
        mcp_servers: [],
        args: [],
        working_dir: props.workspacePath,
        bare: false,
        proxy: false,
      })
    }

    // Create terminal session
    const sessionId = await terminalStore.createSession(
      profileAlias,
      [],
      80,
      24,
      props.workspacePath
    )

    if (sessionId) {
      router.push(`/terminal/${sessionId}`)
    }
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Failed to run agent'
  } finally {
    runningAgent.value = null
  }
}

async function runProfile(alias: string) {
  error.value = null
  runningAgent.value = alias

  try {
    const sessionId = await terminalStore.createSession(
      alias,
      [],
      80,
      24,
      props.workspacePath
    )

    if (sessionId) {
      router.push(`/terminal/${sessionId}`)
    }
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Failed to run profile'
  } finally {
    runningAgent.value = null
  }
}
</script>

<template>
  <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-5">
    <h3 class="text-sm font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider mb-4">Run</h3>

    <div v-if="error" class="mb-4 p-3 bg-red-50 dark:bg-red-900/30 text-red-700 dark:text-red-300 rounded text-sm">
      {{ error }}
    </div>

    <!-- Agents -->
    <div v-if="installedAgents.length === 0" class="text-sm text-gray-400 italic">
      No installed agents found
    </div>
    <div v-else class="space-y-3">
      <div
        v-for="agent in installedAgents"
        :key="agent.id"
        class="flex items-center gap-3 p-3 rounded-lg bg-gray-50 dark:bg-gray-700/50"
      >
        <div class="min-w-0 flex-1">
          <div class="font-medium text-sm text-gray-900 dark:text-white">{{ agent.name }}</div>
          <div v-if="agent.version" class="text-xs text-gray-400">v{{ agent.version }}</div>
        </div>
        <select
          v-model="selectedProviders[agent.id]"
          class="text-sm px-2 py-1.5 rounded border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-900 dark:text-white"
        >
          <option v-for="provider in providersStore.providers" :key="provider.id" :value="provider.id">
            {{ provider.name }}
          </option>
        </select>
        <button
          type="button"
          :disabled="runningAgent === agent.id"
          class="px-4 py-1.5 text-sm bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-1"
          @click="runAgent(agent.id)"
        >
          <div v-if="runningAgent === agent.id" class="w-3 h-3 border-2 border-white border-t-transparent rounded-full animate-spin"></div>
          <template v-else>
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" />
            </svg>
            Run
          </template>
        </button>
      </div>
    </div>

    <!-- Existing profiles -->
    <div v-if="workspaceProfiles.length > 0" class="mt-6">
      <h4 class="text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider mb-2">
        Existing Profiles
      </h4>
      <div class="space-y-2">
        <div
          v-for="profile in workspaceProfiles"
          :key="profile.alias"
          class="flex items-center justify-between p-2 rounded bg-gray-50 dark:bg-gray-700/50"
        >
          <div>
            <span class="text-sm font-medium text-gray-900 dark:text-white">{{ profile.alias }}</span>
            <span class="text-xs text-gray-400 ml-2">{{ profile.agent_id }} / {{ profile.provider_id }}</span>
          </div>
          <button
            type="button"
            :disabled="runningAgent === profile.alias"
            class="px-3 py-1 text-xs bg-gray-200 dark:bg-gray-600 rounded hover:bg-gray-300 dark:hover:bg-gray-500 disabled:opacity-50"
            @click="runProfile(profile.alias)"
          >
            Run
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
