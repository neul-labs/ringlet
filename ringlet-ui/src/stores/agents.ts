import { defineStore } from 'pinia'
import { ref } from 'vue'
import { api } from '@/api/client'
import type { AgentInfo } from '@/api/types'

export const useAgentsStore = defineStore('agents', () => {
  const agents = ref<AgentInfo[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function fetchAgents() {
    loading.value = true
    error.value = null
    try {
      agents.value = await api.agents.list()
    } catch (e) {
      error.value = (e as Error).message
    } finally {
      loading.value = false
    }
  }

  async function getAgent(id: string): Promise<AgentInfo | null> {
    try {
      return await api.agents.get(id)
    } catch {
      return null
    }
  }

  function getAgentById(id: string): AgentInfo | undefined {
    return agents.value.find((a) => a.id === id)
  }

  return {
    agents,
    loading,
    error,
    fetchAgents,
    getAgent,
    getAgentById,
  }
})
