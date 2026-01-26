<script setup lang="ts">
import { onMounted, ref, computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useProfilesStore } from '@/stores/profiles'
import { useProxyStore } from '@/stores/proxy'
import { api } from '@/api/client'
import type { ProfileInfo, HooksConfig, ProfileProxyConfig, RoutingRule } from '@/api/types'
import LoadingSpinner from '@/components/common/LoadingSpinner.vue'
import StatusBadge from '@/components/common/StatusBadge.vue'
import Modal from '@/components/common/Modal.vue'

const route = useRoute()
const router = useRouter()
const profilesStore = useProfilesStore()
const proxyStore = useProxyStore()

const alias = computed(() => route.params.alias as string)
const profile = ref<ProfileInfo | null>(null)
const hooks = ref<HooksConfig | null>(null)
const proxyConfig = ref<ProfileProxyConfig | null>(null)
const routes = ref<RoutingRule[]>([])
const modelAliases = ref<Record<string, string>>({})
const envVars = ref<Record<string, string>>({})
const loading = ref(true)
const activeTab = ref<'overview' | 'hooks' | 'proxy' | 'env'>('overview')

// Proxy running state
const proxyRunning = computed(() => {
  return proxyStore.instances.some(i => i.alias === alias.value)
})

const proxyInstance = computed(() => {
  return proxyStore.getInstanceByAlias(alias.value)
})

const proxyEnabled = computed(() => {
  return proxyConfig.value?.enabled ?? false
})

onMounted(async () => {
  await loadProfile()
})

async function loadProfile() {
  loading.value = true
  try {
    const [profileData, hooksData, proxyConfigData, routesData, aliasesData, envData] = await Promise.all([
      api.profiles.get(alias.value),
      api.hooks.list(alias.value).catch(() => null),
      api.proxy.config(alias.value).catch(() => null),
      api.proxy.routes.list(alias.value).catch(() => []),
      api.proxy.aliases.list(alias.value).catch(() => ({})),
      api.profiles.env(alias.value).catch(() => ({})),
    ])

    profile.value = profileData
    hooks.value = hooksData
    proxyConfig.value = proxyConfigData
    routes.value = routesData
    modelAliases.value = aliasesData
    envVars.value = envData

    await proxyStore.fetchStatus(alias.value)
  } catch (e) {
    console.error('Failed to load profile:', e)
  } finally {
    loading.value = false
  }
}

async function deleteProfile() {
  if (confirm(`Are you sure you want to delete profile "${alias.value}"?`)) {
    try {
      await profilesStore.deleteProfile(alias.value)
      router.push('/profiles')
    } catch (e) {
      alert((e as Error).message)
    }
  }
}

async function toggleProxy() {
  try {
    if (proxyEnabled.value) {
      await api.proxy.disable(alias.value)
    } else {
      await api.proxy.enable(alias.value)
    }
    await loadProfile()
  } catch (e) {
    alert((e as Error).message)
  }
}

async function startProxy() {
  try {
    await proxyStore.startProxy(alias.value)
  } catch (e) {
    alert((e as Error).message)
  }
}

async function stopProxy() {
  try {
    await proxyStore.stopProxy(alias.value)
  } catch (e) {
    alert((e as Error).message)
  }
}

// Hook management
const showAddHookModal = ref(false)
const newHook = ref({ event: 'PreToolUse', matcher: '*', command: '' })

async function addHook() {
  try {
    await api.hooks.add(alias.value, newHook.value.event, {
      matcher: newHook.value.matcher,
      command: newHook.value.command,
    })
    showAddHookModal.value = false
    newHook.value = { event: 'PreToolUse', matcher: '*', command: '' }
    await loadProfile()
  } catch (e) {
    alert((e as Error).message)
  }
}

async function removeHook(event: string, index: number) {
  if (confirm('Remove this hook?')) {
    try {
      await api.hooks.remove(alias.value, event, index)
      await loadProfile()
    } catch (e) {
      alert((e as Error).message)
    }
  }
}

// Get hook rules by event type
function getHookRules(event: string) {
  if (!hooks.value) return []
  switch (event) {
    case 'PreToolUse': return hooks.value.PreToolUse || []
    case 'PostToolUse': return hooks.value.PostToolUse || []
    case 'Notification': return hooks.value.Notification || []
    case 'Stop': return hooks.value.Stop || []
    default: return []
  }
}

const hookEvents = ['PreToolUse', 'PostToolUse', 'Notification', 'Stop']
</script>

<template>
  <div>
    <div v-if="loading" class="flex justify-center py-12">
      <LoadingSpinner size="lg" />
    </div>

    <div v-else-if="!profile" class="text-center py-12">
      <p class="text-gray-500 dark:text-gray-400">Profile not found</p>
      <RouterLink to="/profiles" class="text-blue-600 hover:text-blue-700 mt-4 inline-block">
        Back to Profiles
      </RouterLink>
    </div>

    <div v-else>
      <!-- Header -->
      <div class="flex items-center justify-between mb-6">
        <div>
          <div class="flex items-center space-x-3">
            <h1 class="text-2xl font-bold text-gray-900 dark:text-white">{{ profile.alias }}</h1>
            <StatusBadge :status="proxyEnabled ? 'enabled' : 'disabled'" />
          </div>
          <p class="text-gray-500 dark:text-gray-400 mt-1">{{ profile.agent_id }} / {{ profile.provider_id }}</p>
        </div>
        <div class="flex items-center space-x-3">
          <button
            class="px-4 py-2 text-red-600 hover:bg-red-50 dark:hover:bg-red-900/50 rounded-lg transition-colors"
            @click="deleteProfile"
          >
            Delete
          </button>
        </div>
      </div>

      <!-- Tabs -->
      <div class="border-b border-gray-200 dark:border-gray-700 mb-6">
        <nav class="-mb-px flex space-x-8">
          <button
            v-for="tab in ['overview', 'hooks', 'proxy', 'env']"
            :key="tab"
            :class="[
              'py-2 px-1 border-b-2 font-medium text-sm capitalize',
              activeTab === tab
                ? 'border-blue-500 text-blue-600'
                : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
            ]"
            @click="activeTab = tab as typeof activeTab"
          >
            {{ tab }}
          </button>
        </nav>
      </div>

      <!-- Overview Tab -->
      <div v-if="activeTab === 'overview'" class="space-y-6">
        <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
          <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">Profile Details</h2>
          <dl class="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div>
              <dt class="text-sm font-medium text-gray-500 dark:text-gray-400">Agent</dt>
              <dd class="text-gray-900 dark:text-white">{{ profile.agent_id }}</dd>
            </div>
            <div>
              <dt class="text-sm font-medium text-gray-500 dark:text-gray-400">Provider</dt>
              <dd class="text-gray-900 dark:text-white">{{ profile.provider_id }}</dd>
            </div>
            <div>
              <dt class="text-sm font-medium text-gray-500 dark:text-gray-400">Model</dt>
              <dd class="text-gray-900 dark:text-white">{{ profile.model }}</dd>
            </div>
            <div>
              <dt class="text-sm font-medium text-gray-500 dark:text-gray-400">Endpoint</dt>
              <dd class="text-gray-900 dark:text-white">{{ profile.endpoint_id }}</dd>
            </div>
            <div>
              <dt class="text-sm font-medium text-gray-500 dark:text-gray-400">Total Runs</dt>
              <dd class="text-gray-900 dark:text-white">{{ profile.total_runs }}</dd>
            </div>
            <div v-if="proxyRunning">
              <dt class="text-sm font-medium text-gray-500 dark:text-gray-400">Proxy Port</dt>
              <dd class="text-gray-900 dark:text-white">{{ proxyInstance?.port }}</dd>
            </div>
          </dl>
        </div>
      </div>

      <!-- Hooks Tab -->
      <div v-else-if="activeTab === 'hooks'" class="space-y-6">
        <div class="flex justify-end">
          <button
            class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
            @click="showAddHookModal = true"
          >
            Add Hook
          </button>
        </div>

        <div v-if="!hooks || (getHookRules('PreToolUse').length === 0 && getHookRules('PostToolUse').length === 0 && getHookRules('Notification').length === 0 && getHookRules('Stop').length === 0)" class="text-center py-12 bg-white dark:bg-gray-800 rounded-lg shadow">
          <p class="text-gray-500 dark:text-gray-400">No hooks configured</p>
        </div>

        <div v-else class="space-y-4">
          <div
            v-for="event in hookEvents"
            :key="event"
            v-show="getHookRules(event).length > 0"
            class="bg-white dark:bg-gray-800 rounded-lg shadow"
          >
            <div class="px-6 py-4 border-b border-gray-200 dark:border-gray-700">
              <h3 class="font-medium text-gray-900 dark:text-white">{{ event }}</h3>
            </div>
            <ul class="divide-y divide-gray-200 dark:divide-gray-700">
              <li
                v-for="(rule, index) in getHookRules(event)"
                :key="index"
                class="px-6 py-4 flex items-center justify-between"
              >
                <div>
                  <p class="text-sm text-gray-500 dark:text-gray-400">
                    Matcher: <code class="font-mono">{{ rule.matcher }}</code>
                  </p>
                  <div v-for="(action, actionIdx) in rule.hooks" :key="actionIdx" class="text-sm text-gray-900 dark:text-white font-mono mt-1">
                    <template v-if="action.type === 'command'">
                      Command: {{ action.command }}
                    </template>
                    <template v-else-if="action.type === 'url'">
                      URL: {{ action.url }}
                    </template>
                  </div>
                </div>
                <button
                  class="text-red-600 hover:text-red-700 text-sm"
                  @click="removeHook(event, index)"
                >
                  Remove
                </button>
              </li>
            </ul>
          </div>
        </div>
      </div>

      <!-- Proxy Tab -->
      <div v-else-if="activeTab === 'proxy'" class="space-y-6">
        <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
          <div class="flex items-center justify-between mb-4">
            <h2 class="text-lg font-semibold text-gray-900 dark:text-white">Proxy Status</h2>
            <div class="flex items-center space-x-3">
              <button
                class="px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors"
                @click="toggleProxy"
              >
                {{ proxyEnabled ? 'Disable' : 'Enable' }} Proxy
              </button>
              <button
                v-if="proxyEnabled && !proxyRunning"
                class="px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors"
                @click="startProxy"
              >
                Start
              </button>
              <button
                v-if="proxyRunning"
                class="px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 transition-colors"
                @click="stopProxy"
              >
                Stop
              </button>
            </div>
          </div>

          <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div>
              <dt class="text-sm font-medium text-gray-500 dark:text-gray-400">Status</dt>
              <dd>
                <StatusBadge :status="proxyRunning ? 'running' : 'stopped'" />
              </dd>
            </div>
            <div v-if="proxyRunning">
              <dt class="text-sm font-medium text-gray-500 dark:text-gray-400">Port</dt>
              <dd class="text-gray-900 dark:text-white">{{ proxyInstance?.port }}</dd>
            </div>
            <div v-if="proxyConfig?.routing">
              <dt class="text-sm font-medium text-gray-500 dark:text-gray-400">Strategy</dt>
              <dd class="text-gray-900 dark:text-white">{{ proxyConfig.routing.strategy }}</dd>
            </div>
          </div>
        </div>

        <!-- Routing Rules -->
        <div class="bg-white dark:bg-gray-800 rounded-lg shadow">
          <div class="px-6 py-4 border-b border-gray-200 dark:border-gray-700">
            <h3 class="font-medium text-gray-900 dark:text-white">Routing Rules</h3>
          </div>
          <div v-if="routes.length === 0" class="p-6 text-center text-gray-500 dark:text-gray-400">
            No routing rules configured
          </div>
          <ul v-else class="divide-y divide-gray-200 dark:divide-gray-700">
            <li v-for="(rule, index) in routes" :key="index" class="px-6 py-4">
              <div class="flex items-center justify-between">
                <div>
                  <p class="text-sm font-medium text-gray-900 dark:text-white">
                    {{ rule.name }} → {{ rule.target }}
                  </p>
                  <p class="text-xs text-gray-500 dark:text-gray-400">
                    Priority: {{ rule.priority }} | Condition: {{ rule.condition.type }}
                  </p>
                </div>
              </div>
            </li>
          </ul>
        </div>

        <!-- Model Aliases -->
        <div class="bg-white dark:bg-gray-800 rounded-lg shadow">
          <div class="px-6 py-4 border-b border-gray-200 dark:border-gray-700">
            <h3 class="font-medium text-gray-900 dark:text-white">Model Aliases</h3>
          </div>
          <div v-if="Object.keys(modelAliases).length === 0" class="p-6 text-center text-gray-500 dark:text-gray-400">
            No model aliases configured
          </div>
          <ul v-else class="divide-y divide-gray-200 dark:divide-gray-700">
            <li v-for="(target, aliasName) in modelAliases" :key="aliasName" class="px-6 py-4">
              <p class="text-sm font-medium text-gray-900 dark:text-white">
                {{ aliasName }} → {{ target }}
              </p>
            </li>
          </ul>
        </div>
      </div>

      <!-- Env Tab -->
      <div v-else-if="activeTab === 'env'" class="bg-white dark:bg-gray-800 rounded-lg shadow">
        <div class="px-6 py-4 border-b border-gray-200 dark:border-gray-700">
          <h2 class="text-lg font-semibold text-gray-900 dark:text-white">Environment Variables</h2>
        </div>
        <div v-if="Object.keys(envVars).length === 0" class="p-6 text-center text-gray-500 dark:text-gray-400">
          No environment variables set
        </div>
        <ul v-else class="divide-y divide-gray-200 dark:divide-gray-700">
          <li v-for="(value, key) in envVars" :key="key" class="px-6 py-4">
            <span class="font-mono text-sm text-gray-900 dark:text-white">{{ key }}</span>
            <span class="text-gray-500 dark:text-gray-400">=</span>
            <span class="font-mono text-sm text-gray-600 dark:text-gray-300">{{ value }}</span>
          </li>
        </ul>
      </div>
    </div>

    <!-- Add Hook Modal -->
    <Modal
      :open="showAddHookModal"
      title="Add Hook"
      @close="showAddHookModal = false"
    >
      <form @submit.prevent="addHook" class="space-y-4">
        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Event</label>
          <select
            v-model="newHook.event"
            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
          >
            <option value="PreToolUse">Pre Tool Use</option>
            <option value="PostToolUse">Post Tool Use</option>
            <option value="Notification">Notification</option>
            <option value="Stop">Stop</option>
          </select>
        </div>
        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Matcher (tool pattern)</label>
          <input
            v-model="newHook.matcher"
            type="text"
            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
            placeholder="Bash|Write|Edit or *"
          />
        </div>
        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Command</label>
          <input
            v-model="newHook.command"
            type="text"
            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
            placeholder="echo $EVENT"
            required
          />
        </div>
      </form>
      <template #footer>
        <button
          type="button"
          class="px-4 py-2 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg"
          @click="showAddHookModal = false"
        >
          Cancel
        </button>
        <button
          class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700"
          @click="addHook"
        >
          Add
        </button>
      </template>
    </Modal>
  </div>
</template>
