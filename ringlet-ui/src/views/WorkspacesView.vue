<script setup lang="ts">
import { onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useWorkspaceStore } from '@/stores/workspace'
import FolderPicker from '@/components/workspace/FolderPicker.vue'
import WorkspaceCard from '@/components/workspace/WorkspaceCard.vue'
import { ref } from 'vue'

const router = useRouter()
const workspaceStore = useWorkspaceStore()
const pickerValue = ref('')

onMounted(() => {
  workspaceStore.loadFromStorage()
  // Fetch git info for visible cards (first 12)
  const paths = [
    ...workspaceStore.sortedBookmarks.map(b => b.path),
    ...workspaceStore.sortedRecents.map(r => r.path),
  ]
  const unique = [...new Set(paths)].slice(0, 12)
  unique.forEach(path => workspaceStore.fetchGitInfoCached(path))
})

function openWorkspace(path: string) {
  router.push({ name: 'workspace', query: { path } })
}

function toggleBookmark(path: string) {
  if (workspaceStore.isBookmarked(path)) {
    workspaceStore.removeBookmark(path)
  } else {
    workspaceStore.addBookmark(path)
  }
}
</script>

<template>
  <div>
    <h1 class="text-2xl font-bold text-gray-900 dark:text-white mb-6">Workspaces</h1>

    <!-- Folder Picker -->
    <div class="mb-8">
      <FolderPicker
        v-model="pickerValue"
        placeholder="Type a path to open a workspace..."
        @select="openWorkspace"
      />
    </div>

    <!-- Bookmarks -->
    <div v-if="workspaceStore.sortedBookmarks.length > 0" class="mb-8">
      <h2 class="text-lg font-medium text-gray-900 dark:text-white mb-4">Bookmarks</h2>
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        <WorkspaceCard
          v-for="bm in workspaceStore.sortedBookmarks"
          :key="bm.path"
          :path="bm.path"
          :git-info="workspaceStore.gitInfoCache.get(bm.path)"
          :git-info-loading="workspaceStore.gitInfoCacheLoading.has(bm.path)"
          :is-bookmarked="true"
          @open="openWorkspace(bm.path)"
          @bookmark-toggle="toggleBookmark(bm.path)"
        />
      </div>
    </div>

    <!-- Recent -->
    <div v-if="workspaceStore.sortedRecents.length > 0">
      <h2 class="text-lg font-medium text-gray-900 dark:text-white mb-4">Recent</h2>
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        <WorkspaceCard
          v-for="ws in workspaceStore.sortedRecents"
          :key="ws.path"
          :path="ws.path"
          :git-info="workspaceStore.gitInfoCache.get(ws.path)"
          :git-info-loading="workspaceStore.gitInfoCacheLoading.has(ws.path)"
          :is-bookmarked="workspaceStore.isBookmarked(ws.path)"
          :last-opened="ws.last_opened"
          @open="openWorkspace(ws.path)"
          @bookmark-toggle="toggleBookmark(ws.path)"
        />
      </div>
    </div>

    <!-- Empty state -->
    <div
      v-if="workspaceStore.sortedBookmarks.length === 0 && workspaceStore.sortedRecents.length === 0"
      class="text-center py-16"
    >
      <svg class="w-16 h-16 mx-auto text-gray-300 dark:text-gray-600 mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
      </svg>
      <h3 class="text-lg font-medium text-gray-900 dark:text-white mb-1">No recent workspaces</h3>
      <p class="text-sm text-gray-500">Open a folder to get started.</p>
    </div>
  </div>
</template>
