<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { useProfilesStore } from '@/stores/profiles'
import { useAgentsStore } from '@/stores/agents'
import { useProvidersStore } from '@/stores/providers'
import LoadingSpinner from '@/components/common/LoadingSpinner.vue'
import Modal from '@/components/common/Modal.vue'

const router = useRouter()
const profilesStore = useProfilesStore()
const agentsStore = useAgentsStore()
const providersStore = useProvidersStore()

const showCreateModal = ref(false)
const creating = ref(false)
const createError = ref<string | null>(null)

const newProfile = ref({
  alias: '',
  agent_id: '',
  provider_id: '',
  api_key: '',
})

onMounted(async () => {
  await Promise.all([
    profilesStore.fetchProfiles(),
    agentsStore.fetchAgents(),
    providersStore.fetchProviders(),
  ])
})

function openCreateModal() {
  newProfile.value = { alias: '', agent_id: '', provider_id: '', api_key: '' }
  createError.value = null
  showCreateModal.value = true
}

async function createProfile() {
  if (!newProfile.value.alias || !newProfile.value.agent_id || !newProfile.value.provider_id) {
    createError.value = 'Alias, Agent, and Provider are required'
    return
  }

  creating.value = true
  createError.value = null
  try {
    await profilesStore.createProfile({
      alias: newProfile.value.alias,
      agent_id: newProfile.value.agent_id,
      provider_id: newProfile.value.provider_id,
      api_key: newProfile.value.api_key,
      hooks: [],
      mcp_servers: [],
      args: [],
      bare: false,
      proxy: false,
    })
    showCreateModal.value = false
    router.push(`/profiles/${newProfile.value.alias}`)
  } catch (e) {
    createError.value = (e as Error).message
  } finally {
    creating.value = false
  }
}

async function deleteProfile(alias: string) {
  if (confirm(`Are you sure you want to delete profile "${alias}"?`)) {
    try {
      await profilesStore.deleteProfile(alias)
    } catch (e) {
      alert((e as Error).message)
    }
  }
}
</script>

<template>
  <div>
    <div class="flex items-center justify-between mb-6">
      <h1 class="text-2xl font-bold text-gray-900 dark:text-white">Profiles</h1>
      <button
        class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
        @click="openCreateModal"
      >
        Create Profile
      </button>
    </div>

    <div v-if="profilesStore.loading" class="flex justify-center py-12">
      <LoadingSpinner size="lg" />
    </div>

    <div v-else-if="profilesStore.error" class="bg-red-50 dark:bg-red-900/50 text-red-700 dark:text-red-300 p-4 rounded-lg">
      {{ profilesStore.error }}
    </div>

    <div v-else-if="profilesStore.profiles.length === 0" class="text-center py-12">
      <p class="text-gray-500 dark:text-gray-400">No profiles created yet.</p>
      <button
        class="mt-4 px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
        @click="openCreateModal"
      >
        Create your first profile
      </button>
    </div>

    <div v-else class="bg-white dark:bg-gray-800 rounded-lg shadow overflow-hidden">
      <table class="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
        <thead class="bg-gray-50 dark:bg-gray-900">
          <tr>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
              Alias
            </th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
              Agent
            </th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
              Provider
            </th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
              Model
            </th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
              Runs
            </th>
            <th class="px-6 py-3 text-right text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
              Actions
            </th>
          </tr>
        </thead>
        <tbody class="divide-y divide-gray-200 dark:divide-gray-700">
          <tr
            v-for="profile in profilesStore.profiles"
            :key="profile.alias"
            class="hover:bg-gray-50 dark:hover:bg-gray-800/50"
          >
            <td class="px-6 py-4 whitespace-nowrap">
              <RouterLink
                :to="`/profiles/${profile.alias}`"
                class="text-blue-600 hover:text-blue-700 font-medium"
              >
                {{ profile.alias }}
              </RouterLink>
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900 dark:text-white">
              {{ profile.agent_id }}
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400">
              {{ profile.provider_id || '-' }}
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400">
              {{ profile.model || '-' }}
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400">
              {{ profile.total_runs }}
            </td>
            <td class="px-6 py-4 whitespace-nowrap text-right text-sm">
              <button
                class="text-red-600 hover:text-red-700"
                @click.stop="deleteProfile(profile.alias)"
              >
                Delete
              </button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- Create Profile Modal -->
    <Modal
      :open="showCreateModal"
      title="Create Profile"
      @close="showCreateModal = false"
    >
      <form @submit.prevent="createProfile" class="space-y-4">
        <div v-if="createError" class="bg-red-50 dark:bg-red-900/50 text-red-700 dark:text-red-300 p-3 rounded-lg text-sm">
          {{ createError }}
        </div>

        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
            Alias
          </label>
          <input
            v-model="newProfile.alias"
            type="text"
            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            placeholder="my-profile"
            required
          />
        </div>

        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
            Agent
          </label>
          <select
            v-model="newProfile.agent_id"
            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            required
          >
            <option value="">Select an agent</option>
            <option
              v-for="agent in agentsStore.agents"
              :key="agent.id"
              :value="agent.id"
            >
              {{ agent.name }}
            </option>
          </select>
        </div>

        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
            Provider
          </label>
          <select
            v-model="newProfile.provider_id"
            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            required
          >
            <option value="">Select a provider</option>
            <option
              v-for="provider in providersStore.providers"
              :key="provider.id"
              :value="provider.id"
            >
              {{ provider.name }}
            </option>
          </select>
        </div>

        <div>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
            API Key
          </label>
          <input
            v-model="newProfile.api_key"
            type="password"
            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            placeholder="sk-..."
            required
          />
        </div>
      </form>

      <template #footer>
        <button
          type="button"
          class="px-4 py-2 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors"
          @click="showCreateModal = false"
        >
          Cancel
        </button>
        <button
          type="submit"
          class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors disabled:opacity-50"
          :disabled="creating"
          @click="createProfile"
        >
          {{ creating ? 'Creating...' : 'Create' }}
        </button>
      </template>
    </Modal>
  </div>
</template>
