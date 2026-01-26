import { defineStore } from 'pinia'
import { ref } from 'vue'
import { api } from '@/api/client'
import type { ProviderInfo } from '@/api/types'

export const useProvidersStore = defineStore('providers', () => {
  const providers = ref<ProviderInfo[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function fetchProviders() {
    loading.value = true
    error.value = null
    try {
      providers.value = await api.providers.list()
    } catch (e) {
      error.value = (e as Error).message
    } finally {
      loading.value = false
    }
  }

  async function getProvider(id: string): Promise<ProviderInfo | null> {
    try {
      return await api.providers.get(id)
    } catch {
      return null
    }
  }

  function getProviderById(id: string): ProviderInfo | undefined {
    return providers.value.find((p) => p.id === id)
  }

  return {
    providers,
    loading,
    error,
    fetchProviders,
    getProvider,
    getProviderById,
  }
})
