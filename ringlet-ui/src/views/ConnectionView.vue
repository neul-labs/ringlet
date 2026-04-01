<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useConnectionStore, type ConnectionConfig } from '@/stores/connection'
import { useWebSocketStore } from '@/stores/websocket'

const router = useRouter()
const connectionStore = useConnectionStore()
const wsStore = useWebSocketStore()

const selectedMode = ref<'local' | 'remote' | 'standalone' | null>(null)
const remoteHost = ref('127.0.0.1')
const remotePort = ref(8765)
const remoteToken = ref('')
const connecting = ref(false)
const testResult = ref<string | null>(null)
const testError = ref<string | null>(null)

async function connectLocal() {
  connecting.value = true
  testError.value = null
  try {
    const success = await connectionStore.connectLocal()
    if (success) {
      wsStore.connect()
      router.push('/')
    } else {
      testError.value = connectionStore.error || 'Failed to connect'
    }
  } finally {
    connecting.value = false
  }
}

async function connectRemote() {
  connecting.value = true
  testError.value = null
  try {
    const config: ConnectionConfig = {
      mode: 'remote',
      host: remoteHost.value,
      port: remotePort.value,
      tls: false,
    }
    const success = await connectionStore.connect(config, remoteToken.value)
    if (success) {
      wsStore.connect()
      router.push('/')
    } else {
      testError.value = connectionStore.error || 'Failed to connect'
    }
  } finally {
    connecting.value = false
  }
}

async function connectStandalone() {
  connecting.value = true
  testError.value = null
  try {
    const success = await connectionStore.connectStandalone()
    if (success) {
      wsStore.connect()
      router.push('/')
    } else {
      testError.value = connectionStore.error || 'Failed to start daemon'
    }
  } finally {
    connecting.value = false
  }
}

async function testConnection() {
  testResult.value = null
  testError.value = null
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    const config: ConnectionConfig = {
      mode: 'remote',
      host: remoteHost.value,
      port: remotePort.value,
      tls: false,
    }
    const result = await invoke<Record<string, unknown>>('test_connection', {
      config,
      token: remoteToken.value,
    })
    testResult.value = `Connected! Version: ${(result as { data?: { version?: string } })?.data?.version || 'unknown'}`
  } catch (e) {
    testError.value = e instanceof Error ? e.message : String(e)
  }
}
</script>

<template>
  <div class="min-h-screen flex items-center justify-center bg-gray-100 dark:bg-gray-900">
    <div class="max-w-lg w-full mx-4">
      <div class="text-center mb-8">
        <h1 class="text-3xl font-bold text-gray-900 dark:text-white">Ringlet</h1>
        <p class="mt-2 text-gray-600 dark:text-gray-400">Connect to a ringlet daemon</p>
      </div>

      <!-- Mode selection -->
      <div v-if="!selectedMode" class="space-y-3">
        <button
          class="w-full p-4 bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 hover:border-blue-500 dark:hover:border-blue-400 transition-colors text-left"
          @click="selectedMode = 'local'"
        >
          <div class="font-medium text-gray-900 dark:text-white">Local</div>
          <div class="text-sm text-gray-500 dark:text-gray-400 mt-1">
            Connect to a daemon already running on this machine
          </div>
        </button>

        <button
          class="w-full p-4 bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 hover:border-blue-500 dark:hover:border-blue-400 transition-colors text-left"
          @click="selectedMode = 'remote'"
        >
          <div class="font-medium text-gray-900 dark:text-white">Remote</div>
          <div class="text-sm text-gray-500 dark:text-gray-400 mt-1">
            Connect to a daemon running on another machine
          </div>
        </button>

        <button
          class="w-full p-4 bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 hover:border-blue-500 dark:hover:border-blue-400 transition-colors text-left"
          @click="selectedMode = 'standalone'"
        >
          <div class="font-medium text-gray-900 dark:text-white">Standalone</div>
          <div class="text-sm text-gray-500 dark:text-gray-400 mt-1">
            Start and manage the daemon automatically with this app
          </div>
        </button>
      </div>

      <!-- Local mode -->
      <div v-else-if="selectedMode === 'local'" class="bg-white dark:bg-gray-800 rounded-lg p-6 border border-gray-200 dark:border-gray-700">
        <h2 class="text-lg font-medium text-gray-900 dark:text-white mb-4">Local Connection</h2>
        <p class="text-sm text-gray-600 dark:text-gray-400 mb-6">
          Connects to the daemon on localhost:8765 using the local auth token.
        </p>

        <div v-if="testError" class="mb-4 p-3 bg-red-100 dark:bg-red-900/30 text-red-700 dark:text-red-300 rounded text-sm">
          {{ testError }}
        </div>

        <div class="flex gap-3">
          <button
            class="px-4 py-2 text-sm bg-gray-200 dark:bg-gray-600 rounded hover:bg-gray-300 dark:hover:bg-gray-500"
            @click="selectedMode = null"
          >
            Back
          </button>
          <button
            class="flex-1 px-4 py-2 text-sm bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50"
            :disabled="connecting"
            @click="connectLocal"
          >
            {{ connecting ? 'Connecting...' : 'Connect' }}
          </button>
        </div>
      </div>

      <!-- Remote mode -->
      <div v-else-if="selectedMode === 'remote'" class="bg-white dark:bg-gray-800 rounded-lg p-6 border border-gray-200 dark:border-gray-700">
        <h2 class="text-lg font-medium text-gray-900 dark:text-white mb-4">Remote Connection</h2>

        <div class="space-y-4 mb-6">
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Host</label>
            <input
              v-model="remoteHost"
              type="text"
              class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded bg-white dark:bg-gray-700 text-gray-900 dark:text-white text-sm"
              placeholder="192.168.1.100"
            />
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Port</label>
            <input
              v-model.number="remotePort"
              type="number"
              class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded bg-white dark:bg-gray-700 text-gray-900 dark:text-white text-sm"
            />
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Auth Token</label>
            <input
              v-model="remoteToken"
              type="password"
              class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded bg-white dark:bg-gray-700 text-gray-900 dark:text-white text-sm"
              placeholder="Paste daemon auth token"
            />
          </div>
        </div>

        <div v-if="testResult" class="mb-4 p-3 bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-300 rounded text-sm">
          {{ testResult }}
        </div>
        <div v-if="testError" class="mb-4 p-3 bg-red-100 dark:bg-red-900/30 text-red-700 dark:text-red-300 rounded text-sm">
          {{ testError }}
        </div>

        <div class="flex gap-3">
          <button
            class="px-4 py-2 text-sm bg-gray-200 dark:bg-gray-600 rounded hover:bg-gray-300 dark:hover:bg-gray-500"
            @click="selectedMode = null"
          >
            Back
          </button>
          <button
            class="px-4 py-2 text-sm bg-gray-200 dark:bg-gray-600 rounded hover:bg-gray-300 dark:hover:bg-gray-500"
            @click="testConnection"
          >
            Test
          </button>
          <button
            class="flex-1 px-4 py-2 text-sm bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50"
            :disabled="connecting || !remoteToken"
            @click="connectRemote"
          >
            {{ connecting ? 'Connecting...' : 'Connect' }}
          </button>
        </div>
      </div>

      <!-- Standalone mode -->
      <div v-else-if="selectedMode === 'standalone'" class="bg-white dark:bg-gray-800 rounded-lg p-6 border border-gray-200 dark:border-gray-700">
        <h2 class="text-lg font-medium text-gray-900 dark:text-white mb-4">Standalone Mode</h2>
        <p class="text-sm text-gray-600 dark:text-gray-400 mb-6">
          The app will start the ringletd daemon automatically and stop it when the app closes.
          Requires <code class="px-1 py-0.5 bg-gray-100 dark:bg-gray-700 rounded text-xs">ringletd</code> to be installed and in PATH.
        </p>

        <div v-if="testError" class="mb-4 p-3 bg-red-100 dark:bg-red-900/30 text-red-700 dark:text-red-300 rounded text-sm">
          {{ testError }}
        </div>

        <div class="flex gap-3">
          <button
            class="px-4 py-2 text-sm bg-gray-200 dark:bg-gray-600 rounded hover:bg-gray-300 dark:hover:bg-gray-500"
            @click="selectedMode = null"
          >
            Back
          </button>
          <button
            class="flex-1 px-4 py-2 text-sm bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50"
            :disabled="connecting"
            @click="connectStandalone"
          >
            {{ connecting ? 'Starting Daemon...' : 'Start Daemon' }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
