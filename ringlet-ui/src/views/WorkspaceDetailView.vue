<script setup lang="ts">
import { onMounted, computed, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useWorkspaceStore } from '@/stores/workspace'
import { useTerminalStore } from '@/stores/terminal'
import GitInfoPanel from '@/components/workspace/GitInfoPanel.vue'
import RunPanel from '@/components/workspace/RunPanel.vue'

const route = useRoute()
const router = useRouter()
const workspaceStore = useWorkspaceStore()
const terminalStore = useTerminalStore()
const openingShell = ref(false)

const workspacePath = computed(() => (route.query.path as string) || '')

const folderName = computed(() => {
  const parts = workspacePath.value.split('/').filter(Boolean)
  return parts[parts.length - 1] || workspacePath.value
})

const pathSegments = computed(() => {
  const parts = workspacePath.value.split('/').filter(Boolean)
  return parts.map((part: string, i: number) => ({
    name: part,
    path: '/' + parts.slice(0, i + 1).join('/'),
  }))
})

const workspaceSessions = computed(() =>
  terminalStore.activeSessions.filter(() => true)
)

onMounted(async () => {
  if (!workspacePath.value) {
    router.push('/')
    return
  }
  workspaceStore.setWorkspace(workspacePath.value)
  terminalStore.fetchSessions()
})

function toggleBookmark() {
  if (workspaceStore.isBookmarked(workspacePath.value)) {
    workspaceStore.removeBookmark(workspacePath.value)
  } else {
    workspaceStore.addBookmark(workspacePath.value)
  }
}

async function openShell() {
  openingShell.value = true
  try {
    const sessionId = await terminalStore.createShellSession(
      undefined,
      workspacePath.value
    )
    if (sessionId) {
      router.push(`/terminal/${sessionId}`)
    }
  } finally {
    openingShell.value = false
  }
}
</script>

<template>
  <div v-if="workspacePath">
    <!-- Breadcrumb -->
    <nav class="flex items-center gap-1 text-sm text-gray-500 dark:text-gray-400 mb-4">
      <router-link to="/" class="hover:text-blue-600 dark:hover:text-blue-400">Workspaces</router-link>
      <template v-for="(seg, i) in pathSegments" :key="seg.path">
        <span class="mx-1">/</span>
        <span
          :class="Number(i) === pathSegments.length - 1
            ? 'text-gray-900 dark:text-white font-medium'
            : 'hover:text-blue-600 dark:hover:text-blue-400 cursor-pointer'"
          @click="Number(i) < pathSegments.length - 1 && router.push({ name: 'workspace', query: { path: seg.path } })"
        >
          {{ seg.name }}
        </span>
      </template>
    </nav>

    <!-- Header -->
    <div class="flex items-center justify-between mb-6">
      <div>
        <h1 class="text-2xl font-bold text-gray-900 dark:text-white">{{ folderName }}</h1>
        <p class="text-sm text-gray-500 font-mono">{{ workspacePath }}</p>
      </div>
      <div class="flex items-center gap-3">
        <button
          type="button"
          class="p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700"
          @click="toggleBookmark"
        >
          <svg
            class="w-6 h-6"
            :class="workspaceStore.isBookmarked(workspacePath) ? 'text-yellow-400' : 'text-gray-300 dark:text-gray-600'"
            :fill="workspaceStore.isBookmarked(workspacePath) ? 'currentColor' : 'none'"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11.049 2.927c.3-.921 1.603-.921 1.902 0l1.519 4.674a1 1 0 00.95.69h4.915c.969 0 1.371 1.24.588 1.81l-3.976 2.888a1 1 0 00-.363 1.118l1.518 4.674c.3.922-.755 1.688-1.538 1.118l-3.976-2.888a1 1 0 00-1.176 0l-3.976 2.888c-.783.57-1.838-.197-1.538-1.118l1.518-4.674a1 1 0 00-.363-1.118l-3.976-2.888c-.784-.57-.38-1.81.588-1.81h4.914a1 1 0 00.951-.69l1.519-4.674z" />
          </svg>
        </button>
        <button
          type="button"
          :disabled="openingShell"
          class="px-4 py-2 text-sm bg-gray-200 dark:bg-gray-700 rounded-lg hover:bg-gray-300 dark:hover:bg-gray-600 flex items-center gap-2 disabled:opacity-50"
          @click="openShell"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
          </svg>
          Open Shell
        </button>
      </div>
    </div>

    <!-- Two-column layout -->
    <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
      <div class="lg:col-span-1">
        <GitInfoPanel
          :git-info="workspaceStore.currentGitInfo"
          :loading="workspaceStore.gitInfoLoading"
          :error="workspaceStore.gitInfoError"
        />
      </div>
      <div class="lg:col-span-2">
        <RunPanel :workspace-path="workspacePath" />
      </div>
    </div>

    <!-- Active sessions -->
    <div v-if="workspaceSessions.length > 0" class="mt-6">
      <h2 class="text-lg font-medium text-gray-900 dark:text-white mb-3">Active Sessions</h2>
      <div class="space-y-2">
        <div
          v-for="session in workspaceSessions"
          :key="session.id"
          class="flex items-center justify-between p-3 bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700"
        >
          <div>
            <span class="text-sm font-medium text-gray-900 dark:text-white">{{ session.profile_alias }}</span>
            <span class="text-xs text-gray-400 ml-2">{{ session.id.slice(0, 8) }}</span>
          </div>
          <router-link
            :to="`/terminal/${session.id}`"
            class="px-3 py-1 text-xs bg-blue-600 text-white rounded hover:bg-blue-700"
          >
            Open
          </router-link>
        </div>
      </div>
    </div>
  </div>
</template>
