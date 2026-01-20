<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useAgentsStore } from '@/stores/agents'
import type { AgentInfo } from '@/api/types'
import LoadingSpinner from '@/components/common/LoadingSpinner.vue'
import Modal from '@/components/common/Modal.vue'
import StatusBadge from '@/components/common/StatusBadge.vue'

const agentsStore = useAgentsStore()

const selectedAgent = ref<AgentInfo | null>(null)
const showDetailModal = ref(false)

onMounted(() => {
  agentsStore.fetchAgents()
})

function viewAgent(agent: AgentInfo) {
  selectedAgent.value = agent
  showDetailModal.value = true
}
</script>

<template>
  <div>
    <div class="flex items-center justify-between mb-6">
      <h1 class="text-2xl font-bold text-gray-900 dark:text-white">Agents</h1>
    </div>

    <div v-if="agentsStore.loading" class="flex justify-center py-12">
      <LoadingSpinner size="lg" />
    </div>

    <div v-else-if="agentsStore.error" class="bg-red-50 dark:bg-red-900/50 text-red-700 dark:text-red-300 p-4 rounded-lg">
      {{ agentsStore.error }}
    </div>

    <div v-else class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
      <div
        v-for="agent in agentsStore.agents"
        :key="agent.id"
        class="bg-white dark:bg-gray-800 rounded-lg shadow hover:shadow-md transition-shadow cursor-pointer"
        @click="viewAgent(agent)"
      >
        <div class="p-6">
          <div class="flex items-start justify-between">
            <div>
              <h3 class="text-lg font-semibold text-gray-900 dark:text-white">
                {{ agent.name }}
              </h3>
              <p class="text-sm text-gray-500 dark:text-gray-400 mt-1">
                {{ agent.id }}
              </p>
            </div>
            <StatusBadge :status="agent.installed ? 'enabled' : 'disabled'" />
          </div>
          <div class="mt-4 space-y-2">
            <div v-if="agent.version" class="text-sm text-gray-600 dark:text-gray-300">
              Version: {{ agent.version }}
            </div>
            <div class="text-sm text-gray-500 dark:text-gray-400">
              {{ agent.profile_count }} profiles
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Agent Detail Modal -->
    <Modal
      :open="showDetailModal"
      :title="selectedAgent?.name || 'Agent Details'"
      @close="showDetailModal = false"
    >
      <div v-if="selectedAgent" class="space-y-4">
        <div>
          <label class="text-sm font-medium text-gray-500 dark:text-gray-400">ID</label>
          <p class="text-gray-900 dark:text-white">{{ selectedAgent.id }}</p>
        </div>
        <div>
          <label class="text-sm font-medium text-gray-500 dark:text-gray-400">Installed</label>
          <p>
            <StatusBadge :status="selectedAgent.installed ? 'enabled' : 'disabled'" />
          </p>
        </div>
        <div v-if="selectedAgent.version">
          <label class="text-sm font-medium text-gray-500 dark:text-gray-400">Version</label>
          <p class="text-gray-900 dark:text-white">{{ selectedAgent.version }}</p>
        </div>
        <div v-if="selectedAgent.binary_path">
          <label class="text-sm font-medium text-gray-500 dark:text-gray-400">Binary Path</label>
          <p class="text-gray-900 dark:text-white font-mono text-sm">{{ selectedAgent.binary_path }}</p>
        </div>
        <div>
          <label class="text-sm font-medium text-gray-500 dark:text-gray-400">Profile Count</label>
          <p class="text-gray-900 dark:text-white">{{ selectedAgent.profile_count }}</p>
        </div>
        <div v-if="selectedAgent.default_model">
          <label class="text-sm font-medium text-gray-500 dark:text-gray-400">Default Model</label>
          <p class="text-gray-900 dark:text-white">{{ selectedAgent.default_model }}</p>
        </div>
        <div v-if="selectedAgent.default_provider">
          <label class="text-sm font-medium text-gray-500 dark:text-gray-400">Default Provider</label>
          <p class="text-gray-900 dark:text-white">{{ selectedAgent.default_provider }}</p>
        </div>
        <div>
          <label class="text-sm font-medium text-gray-500 dark:text-gray-400">Supports Hooks</label>
          <p>
            <StatusBadge :status="selectedAgent.supports_hooks ? 'enabled' : 'disabled'" />
          </p>
        </div>
      </div>
    </Modal>
  </div>
</template>
