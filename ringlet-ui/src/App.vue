<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue'
import { RouterView, useRouter } from 'vue-router'
import AppHeader from '@/components/layout/AppHeader.vue'
import AppSidebar from '@/components/layout/AppSidebar.vue'
import { useWebSocketStore } from '@/stores/websocket'
import { useConnectionStore } from '@/stores/connection'
import { isTauri } from '@/api/config'
import { requestNotificationPermission } from '@/utils/notifications'

const wsStore = useWebSocketStore()
const connectionStore = useConnectionStore()
const router = useRouter()

onMounted(async () => {
  requestNotificationPermission()

  if (isTauri()) {
    // In Tauri mode, try auto-connecting with local token
    const success = await connectionStore.connectLocal()
    if (success) {
      wsStore.connect()
    } else {
      // Navigate to connection dialog if auto-connect fails
      router.push('/connect')
    }
  } else {
    // Browser mode: connect immediately (same-origin, no auth needed)
    wsStore.connect()
  }
})

onUnmounted(() => {
  wsStore.disconnect()
})
</script>

<template>
  <div class="min-h-screen bg-gray-100 dark:bg-gray-900">
    <!-- Show full-screen connection view when on /connect route -->
    <template v-if="$route.path === '/connect'">
      <RouterView />
    </template>
    <template v-else>
      <AppHeader />
      <div class="flex">
        <AppSidebar />
        <main class="flex-1 p-6">
          <RouterView />
        </main>
      </div>
    </template>
  </div>
</template>
