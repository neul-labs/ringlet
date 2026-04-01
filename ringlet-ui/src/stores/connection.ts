import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { isTauri } from '@/api/config'

export interface ConnectionConfig {
  mode: 'local' | 'remote' | 'standalone'
  host: string
  port: number
  tls: boolean
}

export interface SavedConnection {
  name: string
  config: ConnectionConfig
  token?: string
}

export const useConnectionStore = defineStore('connection', () => {
  const config = ref<ConnectionConfig>({
    mode: 'local',
    host: '127.0.0.1',
    port: 8765,
    tls: false,
  })

  const status = ref<'disconnected' | 'connecting' | 'connected'>('disconnected')
  const error = ref<string | null>(null)
  const savedConnections = ref<SavedConnection[]>([])

  const isConnected = computed(() => status.value === 'connected')
  const isConnecting = computed(() => status.value === 'connecting')

  async function connect(cfg: ConnectionConfig, token: string): Promise<boolean> {
    if (!isTauri()) return true

    status.value = 'connecting'
    error.value = null

    try {
      // Test the connection first
      await invoke('test_connection', { config: cfg, token })

      // Set the connection
      await invoke('set_connection', { config: cfg, token })

      config.value = cfg
      status.value = 'connected'
      return true
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      status.value = 'disconnected'
      return false
    }
  }

  async function connectLocal(): Promise<boolean> {
    if (!isTauri()) return true

    status.value = 'connecting'
    error.value = null

    try {
      const token = await invoke<string>('load_local_token')
      const cfg: ConnectionConfig = {
        mode: 'local',
        host: '127.0.0.1',
        port: 8765,
        tls: false,
      }

      return await connect(cfg, token)
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      status.value = 'disconnected'
      return false
    }
  }

  async function connectStandalone(): Promise<boolean> {
    if (!isTauri()) return true

    status.value = 'connecting'
    error.value = null

    try {
      const cfg: ConnectionConfig = {
        mode: 'standalone',
        host: '127.0.0.1',
        port: 8765,
        tls: false,
      }

      // Set mode first so daemon commands know the mode
      await invoke('set_connection', { config: cfg, token: '' })

      // Start the daemon
      const token = await invoke<string>('start_daemon')

      config.value = cfg
      status.value = 'connected'

      // Save the token
      await invoke('set_connection', { config: cfg, token })

      return true
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      status.value = 'disconnected'
      return false
    }
  }

  async function disconnect(): Promise<void> {
    if (!isTauri()) return

    try {
      if (config.value.mode === 'standalone') {
        await invoke('stop_daemon')
      }
    } catch {
      // Ignore errors during disconnect
    }

    status.value = 'disconnected'
    error.value = null
  }

  function addSavedConnection(conn: SavedConnection) {
    const existing = savedConnections.value.findIndex((c) => c.name === conn.name)
    if (existing >= 0) {
      savedConnections.value[existing] = conn
    } else {
      savedConnections.value.push(conn)
    }
  }

  function removeSavedConnection(name: string) {
    savedConnections.value = savedConnections.value.filter((c) => c.name !== name)
  }

  return {
    config,
    status,
    error,
    savedConnections,
    isConnected,
    isConnecting,
    connect,
    connectLocal,
    connectStandalone,
    disconnect,
    addSavedConnection,
    removeSavedConnection,
  }
})
