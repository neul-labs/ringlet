<script setup lang="ts">
import { onMounted, ref, computed, watch } from 'vue'
import { api } from '@/api/client'
import type { UsageStatsResponse, TokenUsage, AgentType } from '@/api/types'
import { useWebSocketStore } from '@/stores/websocket'
import LoadingSpinner from '@/components/common/LoadingSpinner.vue'

const usage = ref<UsageStatsResponse | null>(null)
const loading = ref(true)
const error = ref<string | null>(null)
const selectedPeriod = ref('today')
const selectedAgent = ref<AgentType | 'all'>('all')
const importing = ref(false)

// WebSocket store for real-time updates
const wsStore = useWebSocketStore()

const periods = [
  { value: 'today', label: 'Today' },
  { value: 'yesterday', label: 'Yesterday' },
  { value: 'week', label: 'This Week' },
  { value: 'month', label: 'This Month' },
  { value: '7d', label: 'Last 7 Days' },
  { value: '30d', label: 'Last 30 Days' },
  { value: 'all', label: 'All Time' },
]

const agents = [
  { value: 'all', label: 'All Agents' },
  { value: 'claude', label: 'Claude Code' },
  { value: 'codex', label: 'Codex CLI' },
  { value: 'opencode', label: 'OpenCode' },
]

// Watch for usage_updated events from WebSocket
watch(
  () => wsStore.recentEvents,
  (events) => {
    const lastEvent = events[events.length - 1]
    if (lastEvent?.type === 'usage_updated') {
      // Refresh usage data when new entries are detected
      fetchUsage()
    }
  },
  { deep: true }
)

onMounted(async () => {
  await fetchUsage()
})

async function fetchUsage() {
  loading.value = true
  error.value = null
  try {
    usage.value = await api.usage.get({ period: selectedPeriod.value })
  } catch (e) {
    error.value = (e as Error).message
  } finally {
    loading.value = false
  }
}

async function importClaude() {
  importing.value = true
  try {
    await api.usage.importClaude()
    await fetchUsage()
  } catch (e) {
    error.value = (e as Error).message
  } finally {
    importing.value = false
  }
}

function formatNumber(n: number): string {
  return new Intl.NumberFormat().format(n)
}

function formatCost(cost: number): string {
  if (cost < 0.01) return `$${cost.toFixed(4)}`
  return `$${cost.toFixed(2)}`
}

function formatDuration(seconds: number): string {
  if (seconds < 60) return `${seconds}s`
  if (seconds < 3600) return `${Math.floor(seconds / 60)}m ${seconds % 60}s`
  const hours = Math.floor(seconds / 3600)
  const mins = Math.floor((seconds % 3600) / 60)
  return `${hours}h ${mins}m`
}

function totalTokens(t: TokenUsage): number {
  return t.input_tokens + t.output_tokens + t.cache_creation_input_tokens + t.cache_read_input_tokens
}

const sortedProfiles = computed(() => {
  if (!usage.value) return []
  return Object.entries(usage.value.aggregates.by_profile)
    .sort(([, a], [, b]) => b.sessions - a.sessions)
})

const agentUsageList = computed(() => {
  if (!usage.value?.aggregates.by_agent) return []
  return Object.entries(usage.value.aggregates.by_agent)
    .map(([key, value]) => ({ key: key as AgentType, ...value }))
    .sort((a, b) => totalTokens(b.tokens) - totalTokens(a.tokens))
})

function getAgentDisplayName(agent: AgentType): string {
  const names: Record<AgentType, string> = {
    claude: 'Claude Code',
    codex: 'Codex CLI',
    opencode: 'OpenCode',
  }
  return names[agent] || agent
}

function getAgentColor(agent: AgentType): string {
  const colors: Record<AgentType, string> = {
    claude: 'text-amber-600',
    codex: 'text-emerald-600',
    opencode: 'text-sky-600',
  }
  return colors[agent] || 'text-gray-600'
}

function getAgentBgColor(agent: AgentType): string {
  const colors: Record<AgentType, string> = {
    claude: 'bg-amber-50 dark:bg-amber-900/20',
    codex: 'bg-emerald-50 dark:bg-emerald-900/20',
    opencode: 'bg-sky-50 dark:bg-sky-900/20',
  }
  return colors[agent] || 'bg-gray-50 dark:bg-gray-900/20'
}
</script>

<template>
  <div>
    <div class="flex items-center justify-between mb-6">
      <h1 class="text-2xl font-bold text-gray-900 dark:text-white">Usage</h1>
      <div class="flex items-center gap-4">
        <select
          v-model="selectedPeriod"
          class="px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-white"
          @change="fetchUsage"
        >
          <option v-for="p in periods" :key="p.value" :value="p.value">
            {{ p.label }}
          </option>
        </select>
        <select
          v-model="selectedAgent"
          class="px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-white"
        >
          <option v-for="a in agents" :key="a.value" :value="a.value">
            {{ a.label }}
          </option>
        </select>
        <button
          class="px-4 py-2 text-gray-600 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors"
          @click="fetchUsage"
        >
          Refresh
        </button>
        <button
          class="px-4 py-2 bg-blue-600 text-white hover:bg-blue-700 rounded-lg transition-colors disabled:opacity-50"
          :disabled="importing"
          @click="importClaude"
        >
          {{ importing ? 'Importing...' : 'Import Claude Data' }}
        </button>
      </div>
    </div>

    <div v-if="loading" class="flex justify-center py-12">
      <LoadingSpinner size="lg" />
    </div>

    <div v-else-if="error" class="bg-red-50 dark:bg-red-900/50 text-red-700 dark:text-red-300 p-4 rounded-lg">
      {{ error }}
    </div>

    <div v-else-if="usage" class="space-y-6">
      <!-- Period Banner -->
      <div class="text-sm text-gray-500 dark:text-gray-400">
        Showing data for: <span class="font-medium text-gray-700 dark:text-gray-200">{{ usage.period }}</span>
      </div>

      <!-- Overview Cards -->
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
          <div class="text-sm font-medium text-gray-500 dark:text-gray-400">Total Tokens</div>
          <div class="text-3xl font-bold text-blue-600 mt-2">
            {{ formatNumber(totalTokens(usage.total_tokens)) }}
          </div>
          <div class="text-xs text-gray-400 mt-1">
            {{ formatNumber(usage.total_tokens.input_tokens) }} in / {{ formatNumber(usage.total_tokens.output_tokens) }} out
          </div>
        </div>

        <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
          <div class="text-sm font-medium text-gray-500 dark:text-gray-400">Total Cost</div>
          <div class="text-3xl font-bold text-green-600 mt-2">
            {{ usage.total_cost ? formatCost(usage.total_cost.total_cost) : '-' }}
          </div>
          <div v-if="usage.total_cost" class="text-xs text-gray-400 mt-1">
            {{ formatCost(usage.total_cost.input_cost) }} in / {{ formatCost(usage.total_cost.output_cost) }} out
          </div>
          <div v-else class="text-xs text-gray-400 mt-1">
            Only tracked for direct API usage
          </div>
        </div>

        <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
          <div class="text-sm font-medium text-gray-500 dark:text-gray-400">Sessions</div>
          <div class="text-3xl font-bold text-purple-600 mt-2">
            {{ formatNumber(usage.total_sessions) }}
          </div>
        </div>

        <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
          <div class="text-sm font-medium text-gray-500 dark:text-gray-400">Runtime</div>
          <div class="text-3xl font-bold text-orange-600 mt-2">
            {{ formatDuration(usage.total_runtime_secs) }}
          </div>
        </div>
      </div>

      <!-- Token Breakdown -->
      <div class="bg-white dark:bg-gray-800 rounded-lg shadow">
        <div class="px-6 py-4 border-b border-gray-200 dark:border-gray-700">
          <h2 class="text-lg font-semibold text-gray-900 dark:text-white">Token Breakdown</h2>
        </div>
        <div class="p-6">
          <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
            <div>
              <div class="text-sm text-gray-500 dark:text-gray-400">Input</div>
              <div class="text-xl font-semibold text-gray-900 dark:text-white">
                {{ formatNumber(usage.total_tokens.input_tokens) }}
              </div>
            </div>
            <div>
              <div class="text-sm text-gray-500 dark:text-gray-400">Output</div>
              <div class="text-xl font-semibold text-gray-900 dark:text-white">
                {{ formatNumber(usage.total_tokens.output_tokens) }}
              </div>
            </div>
            <div>
              <div class="text-sm text-gray-500 dark:text-gray-400">Cache Creation</div>
              <div class="text-xl font-semibold text-gray-900 dark:text-white">
                {{ formatNumber(usage.total_tokens.cache_creation_input_tokens) }}
              </div>
            </div>
            <div>
              <div class="text-sm text-gray-500 dark:text-gray-400">Cache Read</div>
              <div class="text-xl font-semibold text-gray-900 dark:text-white">
                {{ formatNumber(usage.total_tokens.cache_read_input_tokens) }}
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Cost Breakdown (if available) -->
      <div v-if="usage.total_cost" class="bg-white dark:bg-gray-800 rounded-lg shadow">
        <div class="px-6 py-4 border-b border-gray-200 dark:border-gray-700">
          <h2 class="text-lg font-semibold text-gray-900 dark:text-white">Cost Breakdown</h2>
        </div>
        <div class="p-6">
          <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
            <div>
              <div class="text-sm text-gray-500 dark:text-gray-400">Input</div>
              <div class="text-xl font-semibold text-green-600">
                {{ formatCost(usage.total_cost.input_cost) }}
              </div>
            </div>
            <div>
              <div class="text-sm text-gray-500 dark:text-gray-400">Output</div>
              <div class="text-xl font-semibold text-green-600">
                {{ formatCost(usage.total_cost.output_cost) }}
              </div>
            </div>
            <div>
              <div class="text-sm text-gray-500 dark:text-gray-400">Cache Creation</div>
              <div class="text-xl font-semibold text-green-600">
                {{ formatCost(usage.total_cost.cache_creation_cost) }}
              </div>
            </div>
            <div>
              <div class="text-sm text-gray-500 dark:text-gray-400">Cache Read</div>
              <div class="text-xl font-semibold text-green-600">
                {{ formatCost(usage.total_cost.cache_read_cost) }}
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Usage by Agent -->
      <div v-if="agentUsageList.length > 0" class="bg-white dark:bg-gray-800 rounded-lg shadow">
        <div class="px-6 py-4 border-b border-gray-200 dark:border-gray-700">
          <h2 class="text-lg font-semibold text-gray-900 dark:text-white">Usage by Agent</h2>
        </div>
        <div class="p-6">
          <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div
              v-for="agent in agentUsageList"
              :key="agent.key"
              :class="[getAgentBgColor(agent.key), 'rounded-lg p-4']"
            >
              <div class="flex items-center justify-between">
                <span :class="['font-semibold', getAgentColor(agent.key)]">
                  {{ getAgentDisplayName(agent.key) }}
                </span>
                <span class="text-sm text-gray-500 dark:text-gray-400">
                  {{ formatNumber(agent.sessions) }} sessions
                </span>
              </div>
              <div class="mt-3 grid grid-cols-2 gap-2 text-sm">
                <div>
                  <div class="text-gray-500 dark:text-gray-400">Tokens</div>
                  <div class="font-medium text-gray-900 dark:text-white">
                    {{ formatNumber(totalTokens(agent.tokens)) }}
                  </div>
                </div>
                <div>
                  <div class="text-gray-500 dark:text-gray-400">Cost</div>
                  <div class="font-medium text-gray-900 dark:text-white">
                    {{ agent.cost ? formatCost(agent.cost.total_cost) : '-' }}
                  </div>
                </div>
              </div>
              <div class="mt-2 text-xs text-gray-400">
                {{ formatNumber(agent.tokens.input_tokens) }} in / {{ formatNumber(agent.tokens.output_tokens) }} out
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Per-Profile Usage -->
      <div class="bg-white dark:bg-gray-800 rounded-lg shadow">
        <div class="px-6 py-4 border-b border-gray-200 dark:border-gray-700">
          <h2 class="text-lg font-semibold text-gray-900 dark:text-white">Usage by Profile</h2>
        </div>

        <div v-if="sortedProfiles.length === 0" class="p-6 text-center text-gray-500 dark:text-gray-400">
          No profile usage data available
        </div>

        <div v-else class="overflow-x-auto">
          <table class="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
            <thead class="bg-gray-50 dark:bg-gray-900">
              <tr>
                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">Profile</th>
                <th class="px-6 py-3 text-right text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">Sessions</th>
                <th class="px-6 py-3 text-right text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">Tokens</th>
                <th class="px-6 py-3 text-right text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">Cost</th>
                <th class="px-6 py-3 text-right text-xs font-medium text-gray-500 dark:text-gray-400 uppercase">Last Used</th>
              </tr>
            </thead>
            <tbody class="divide-y divide-gray-200 dark:divide-gray-700">
              <tr v-for="[profileId, profileUsage] in sortedProfiles" :key="profileId">
                <td class="px-6 py-4 whitespace-nowrap">
                  <RouterLink
                    :to="`/profiles/${profileId}`"
                    class="font-medium text-blue-600 hover:text-blue-700"
                  >
                    {{ profileId }}
                  </RouterLink>
                </td>
                <td class="px-6 py-4 whitespace-nowrap text-right text-gray-500 dark:text-gray-400">
                  {{ formatNumber(profileUsage.sessions) }}
                </td>
                <td class="px-6 py-4 whitespace-nowrap text-right text-gray-500 dark:text-gray-400">
                  {{ formatNumber(totalTokens(profileUsage.tokens)) }}
                </td>
                <td class="px-6 py-4 whitespace-nowrap text-right text-gray-500 dark:text-gray-400">
                  {{ profileUsage.cost ? formatCost(profileUsage.cost.total_cost) : '-' }}
                </td>
                <td class="px-6 py-4 whitespace-nowrap text-right text-gray-500 dark:text-gray-400">
                  {{ profileUsage.last_used ? new Date(profileUsage.last_used).toLocaleDateString() : '-' }}
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </div>
  </div>
</template>
