<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useProvidersStore } from '@/stores/providers'
import type { ProviderInfo } from '@/api/types'
import LoadingSpinner from '@/components/common/LoadingSpinner.vue'
import Modal from '@/components/common/Modal.vue'

const providersStore = useProvidersStore()

const selectedProvider = ref<ProviderInfo | null>(null)
const showDetailModal = ref(false)

onMounted(() => {
  providersStore.fetchProviders()
})

function viewProvider(provider: ProviderInfo) {
  selectedProvider.value = provider
  showDetailModal.value = true
}
</script>

<template>
  <div>
    <div class="flex items-center justify-between mb-6">
      <h1 class="text-2xl font-bold text-gray-900 dark:text-white">Providers</h1>
    </div>

    <div v-if="providersStore.loading" class="flex justify-center py-12">
      <LoadingSpinner size="lg" />
    </div>

    <div v-else-if="providersStore.error" class="bg-red-50 dark:bg-red-900/50 text-red-700 dark:text-red-300 p-4 rounded-lg">
      {{ providersStore.error }}
    </div>

    <div v-else class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
      <div
        v-for="provider in providersStore.providers"
        :key="provider.id"
        class="bg-white dark:bg-gray-800 rounded-lg shadow hover:shadow-md transition-shadow cursor-pointer"
        @click="viewProvider(provider)"
      >
        <div class="p-6">
          <div class="flex items-start justify-between">
            <div>
              <h3 class="text-lg font-semibold text-gray-900 dark:text-white">
                {{ provider.name }}
              </h3>
              <p class="text-sm text-gray-500 dark:text-gray-400 mt-1">
                {{ provider.id }}
              </p>
            </div>
          </div>
          <div class="mt-4 space-y-2">
            <div class="text-sm text-gray-600 dark:text-gray-300">
              Type: {{ provider.provider_type }}
            </div>
            <div class="text-sm text-gray-500 dark:text-gray-400">
              {{ provider.endpoints.length }} endpoints
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Provider Detail Modal -->
    <Modal
      :open="showDetailModal"
      :title="selectedProvider?.name || 'Provider Details'"
      @close="showDetailModal = false"
    >
      <div v-if="selectedProvider" class="space-y-4">
        <div>
          <label class="text-sm font-medium text-gray-500 dark:text-gray-400">ID</label>
          <p class="text-gray-900 dark:text-white">{{ selectedProvider.id }}</p>
        </div>
        <div>
          <label class="text-sm font-medium text-gray-500 dark:text-gray-400">Type</label>
          <p class="text-gray-900 dark:text-white">{{ selectedProvider.provider_type }}</p>
        </div>
        <div v-if="selectedProvider.default_model">
          <label class="text-sm font-medium text-gray-500 dark:text-gray-400">Default Model</label>
          <p class="text-gray-900 dark:text-white">{{ selectedProvider.default_model }}</p>
        </div>
        <div>
          <label class="text-sm font-medium text-gray-500 dark:text-gray-400">Auth Required</label>
          <p class="text-gray-900 dark:text-white">{{ selectedProvider.auth_required ? 'Yes' : 'No' }}</p>
        </div>
        <div v-if="selectedProvider.auth_prompt">
          <label class="text-sm font-medium text-gray-500 dark:text-gray-400">Auth Prompt</label>
          <p class="text-gray-900 dark:text-white">{{ selectedProvider.auth_prompt }}</p>
        </div>
        <div>
          <label class="text-sm font-medium text-gray-500 dark:text-gray-400">Endpoints</label>
          <div class="mt-2 space-y-2">
            <div
              v-for="endpoint in selectedProvider.endpoints"
              :key="endpoint.id"
              class="bg-gray-50 dark:bg-gray-700 p-3 rounded"
            >
              <div class="flex items-center justify-between">
                <span class="font-medium text-gray-900 dark:text-white">{{ endpoint.id }}</span>
                <span v-if="endpoint.is_default" class="text-xs bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-300 px-2 py-0.5 rounded">
                  Default
                </span>
              </div>
              <p class="text-sm text-gray-600 dark:text-gray-400 font-mono mt-1 break-all">
                {{ endpoint.url }}
              </p>
            </div>
          </div>
        </div>
      </div>
    </Modal>
  </div>
</template>
