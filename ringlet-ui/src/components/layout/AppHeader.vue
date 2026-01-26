<script setup lang="ts">
import { useWebSocketStore } from '@/stores/websocket'

const wsStore = useWebSocketStore()
</script>

<template>
  <header class="bg-white dark:bg-gray-800 shadow-sm border-b border-gray-200 dark:border-gray-700">
    <div class="px-6 py-4 flex items-center justify-between">
      <div class="flex items-center space-x-3">
        <h1 class="text-xl font-bold text-gray-900 dark:text-white">Clown</h1>
        <span class="text-sm text-gray-500 dark:text-gray-400">Agent Orchestrator</span>
      </div>
      <div class="flex items-center space-x-4">
        <div class="flex items-center space-x-2">
          <span
            :class="[
              'w-2 h-2 rounded-full',
              wsStore.connected ? 'bg-green-500' : 'bg-red-500'
            ]"
          ></span>
          <span class="text-sm text-gray-600 dark:text-gray-300">
            {{ wsStore.connected ? 'Connected' : 'Disconnected' }}
          </span>
        </div>
        <span v-if="wsStore.version" class="text-xs text-gray-400">
          v{{ wsStore.version }}
        </span>
      </div>
    </div>
  </header>
</template>
