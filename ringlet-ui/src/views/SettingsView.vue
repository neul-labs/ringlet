<script setup lang="ts">
import { computed, ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useConnectionStore } from '@/stores/connection'
import { useWebSocketStore } from '@/stores/websocket'
import {
  getNotifyEnabled,
  setNotifyEnabled,
  getSoundEnabled,
  setSoundEnabled,
  requestNotificationPermission,
  playNotificationSound,
} from '@/utils/notifications'

const router = useRouter()
const connectionStore = useConnectionStore()
const wsStore = useWebSocketStore()

const notifyEnabled = ref(true)
const soundEnabled = ref(true)

onMounted(() => {
  notifyEnabled.value = getNotifyEnabled()
  soundEnabled.value = getSoundEnabled()
})

function toggleNotify() {
  notifyEnabled.value = !notifyEnabled.value
  setNotifyEnabled(notifyEnabled.value)
  if (notifyEnabled.value) {
    requestNotificationPermission()
  }
}

function toggleSound() {
  soundEnabled.value = !soundEnabled.value
  setSoundEnabled(soundEnabled.value)
}

function testNotification() {
  playNotificationSound()
  if ('Notification' in window && Notification.permission === 'granted') {
    new Notification('Test Notification', {
      body: 'Session notifications are working!',
      silent: false,
    })
  }
}

const modeLabel = computed(() => {
  switch (connectionStore.config.mode) {
    case 'local':
      return 'Local'
    case 'remote':
      return 'Remote'
    case 'standalone':
      return 'Standalone'
    default:
      return 'Unknown'
  }
})

async function switchConnection() {
  wsStore.disconnect()
  await connectionStore.disconnect()
  router.push('/connect')
}

async function stopDaemon() {
  wsStore.disconnect()
  await connectionStore.disconnect()
  router.push('/connect')
}
</script>

<template>
  <div class="max-w-2xl">
    <h1 class="text-2xl font-bold text-gray-900 dark:text-white mb-6">Settings</h1>

    <!-- Connection Info -->
    <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6 mb-6">
      <h2 class="text-lg font-medium text-gray-900 dark:text-white mb-4">Connection</h2>

      <div class="space-y-3">
        <div class="flex justify-between">
          <span class="text-sm text-gray-600 dark:text-gray-400">Mode</span>
          <span class="text-sm font-medium text-gray-900 dark:text-white">{{ modeLabel }}</span>
        </div>
        <div class="flex justify-between">
          <span class="text-sm text-gray-600 dark:text-gray-400">Host</span>
          <span class="text-sm font-mono text-gray-900 dark:text-white">{{ connectionStore.config.host }}</span>
        </div>
        <div class="flex justify-between">
          <span class="text-sm text-gray-600 dark:text-gray-400">Port</span>
          <span class="text-sm font-mono text-gray-900 dark:text-white">{{ connectionStore.config.port }}</span>
        </div>
        <div class="flex justify-between">
          <span class="text-sm text-gray-600 dark:text-gray-400">Status</span>
          <span
            :class="[
              'text-sm font-medium',
              connectionStore.isConnected
                ? 'text-green-600 dark:text-green-400'
                : 'text-red-600 dark:text-red-400'
            ]"
          >
            {{ connectionStore.isConnected ? 'Connected' : 'Disconnected' }}
          </span>
        </div>
        <div class="flex justify-between">
          <span class="text-sm text-gray-600 dark:text-gray-400">WebSocket</span>
          <span
            :class="[
              'text-sm font-medium',
              wsStore.connected
                ? 'text-green-600 dark:text-green-400'
                : 'text-yellow-600 dark:text-yellow-400'
            ]"
          >
            {{ wsStore.connected ? 'Connected' : 'Reconnecting...' }}
          </span>
        </div>
      </div>

      <div class="mt-6 flex gap-3">
        <button
          class="px-4 py-2 text-sm bg-gray-200 dark:bg-gray-600 rounded hover:bg-gray-300 dark:hover:bg-gray-500"
          @click="switchConnection"
        >
          Switch Connection
        </button>
        <button
          v-if="connectionStore.config.mode === 'standalone'"
          class="px-4 py-2 text-sm bg-red-600 text-white rounded hover:bg-red-700"
          @click="stopDaemon"
        >
          Stop Daemon
        </button>
      </div>
    </div>

    <!-- Notifications -->
    <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6 mb-6">
      <h2 class="text-lg font-medium text-gray-900 dark:text-white mb-4">Notifications</h2>

      <div class="space-y-4">
        <div class="flex items-center justify-between">
          <div>
            <span class="text-sm text-gray-900 dark:text-white">Notify when sessions complete</span>
            <p class="text-xs text-gray-500 dark:text-gray-400">Show a desktop notification when an agent finishes running</p>
          </div>
          <button
            type="button"
            :class="[
              'relative inline-flex h-6 w-11 items-center rounded-full transition-colors',
              notifyEnabled ? 'bg-blue-600' : 'bg-gray-300 dark:bg-gray-600'
            ]"
            @click="toggleNotify"
          >
            <span
              :class="[
                'inline-block h-4 w-4 transform rounded-full bg-white transition-transform',
                notifyEnabled ? 'translate-x-6' : 'translate-x-1'
              ]"
            />
          </button>
        </div>

        <div class="flex items-center justify-between">
          <div>
            <span class="text-sm text-gray-900 dark:text-white">Play notification sound</span>
            <p class="text-xs text-gray-500 dark:text-gray-400">Play a chime when a session finishes</p>
          </div>
          <button
            type="button"
            :class="[
              'relative inline-flex h-6 w-11 items-center rounded-full transition-colors',
              soundEnabled ? 'bg-blue-600' : 'bg-gray-300 dark:bg-gray-600'
            ]"
            @click="toggleSound"
          >
            <span
              :class="[
                'inline-block h-4 w-4 transform rounded-full bg-white transition-transform',
                soundEnabled ? 'translate-x-6' : 'translate-x-1'
              ]"
            />
          </button>
        </div>

        <div>
          <button
            type="button"
            class="px-4 py-2 text-sm bg-gray-200 dark:bg-gray-600 rounded hover:bg-gray-300 dark:hover:bg-gray-500"
            @click="testNotification"
          >
            Test Notification
          </button>
        </div>
      </div>
    </div>

    <!-- About -->
    <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-6">
      <h2 class="text-lg font-medium text-gray-900 dark:text-white mb-4">About</h2>
      <div class="space-y-2">
        <div class="flex justify-between">
          <span class="text-sm text-gray-600 dark:text-gray-400">Daemon Version</span>
          <span class="text-sm font-mono text-gray-900 dark:text-white">{{ wsStore.version || 'N/A' }}</span>
        </div>
      </div>
    </div>
  </div>
</template>
