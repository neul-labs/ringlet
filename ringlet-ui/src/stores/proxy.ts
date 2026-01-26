import { defineStore } from 'pinia'
import { ref } from 'vue'
import { api } from '@/api/client'
import type { ProxyInstanceInfo } from '@/api/types'

export const useProxyStore = defineStore('proxy', () => {
  const instances = ref<ProxyInstanceInfo[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function fetchStatus(alias?: string) {
    loading.value = true
    error.value = null
    try {
      instances.value = await api.proxy.status(alias)
    } catch (e) {
      error.value = (e as Error).message
    } finally {
      loading.value = false
    }
  }

  async function startProxy(alias: string) {
    await api.proxy.start(alias)
    await fetchStatus()
  }

  async function stopProxy(alias: string) {
    await api.proxy.stop(alias)
    await fetchStatus()
  }

  async function stopAll() {
    await api.proxy.stopAll()
    instances.value = []
  }

  function getInstanceByAlias(alias: string): ProxyInstanceInfo | undefined {
    return instances.value.find((i) => i.alias === alias)
  }

  // Called by WebSocket events
  function handleProxyStarted(_alias: string, _port: number) {
    fetchStatus() // Refresh
  }

  function handleProxyStopped(alias: string) {
    instances.value = instances.value.filter((i) => i.alias !== alias)
  }

  return {
    instances,
    loading,
    error,
    fetchStatus,
    startProxy,
    stopProxy,
    stopAll,
    getInstanceByAlias,
    handleProxyStarted,
    handleProxyStopped,
  }
})
