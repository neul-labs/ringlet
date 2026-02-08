<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import Modal from './Modal.vue'
import { api } from '@/api/client'
import type { DirEntry } from '@/api/types'

const props = defineProps<{
  open: boolean
  initialPath?: string
}>()

const emit = defineEmits<{
  close: []
  select: [path: string]
}>()

const currentPath = ref('')
const parentPath = ref<string | null>(null)
const entries = ref<DirEntry[]>([])
const loading = ref(false)
const error = ref<string | null>(null)

const directories = computed(() => entries.value.filter(e => e.is_dir))

async function loadDirectory(path?: string) {
  loading.value = true
  error.value = null

  try {
    const response = await api.fs.list(path)
    currentPath.value = response.path
    parentPath.value = response.parent
    entries.value = response.entries
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Failed to load directory'
  } finally {
    loading.value = false
  }
}

function navigateTo(path: string) {
  loadDirectory(path)
}

function goUp() {
  if (parentPath.value) {
    loadDirectory(parentPath.value)
  }
}

function selectCurrent() {
  emit('select', currentPath.value)
  emit('close')
}

onMounted(() => {
  loadDirectory(props.initialPath)
})
</script>

<template>
  <Modal :open="open" title="Select Directory" @close="emit('close')">
    <div class="space-y-4">
      <!-- Current path -->
      <div class="flex items-center gap-2 p-2 bg-gray-100 dark:bg-gray-700 rounded text-sm font-mono">
        <span class="truncate flex-1">{{ currentPath }}</span>
      </div>

      <!-- Navigation -->
      <div class="flex items-center gap-2">
        <button
          type="button"
          :disabled="!parentPath || loading"
          class="px-3 py-1.5 text-sm bg-gray-200 dark:bg-gray-600 rounded hover:bg-gray-300 dark:hover:bg-gray-500 disabled:opacity-50 disabled:cursor-not-allowed"
          @click="goUp"
        >
          <span class="flex items-center gap-1">
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 16l-4-4m0 0l4-4m-4 4h18" />
            </svg>
            Up
          </span>
        </button>
        <button
          type="button"
          :disabled="loading"
          class="px-3 py-1.5 text-sm bg-gray-200 dark:bg-gray-600 rounded hover:bg-gray-300 dark:hover:bg-gray-500 disabled:opacity-50"
          @click="loadDirectory(currentPath)"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
          </svg>
        </button>
      </div>

      <!-- Error message -->
      <div v-if="error" class="p-3 bg-red-100 dark:bg-red-900/30 text-red-700 dark:text-red-300 rounded text-sm">
        {{ error }}
      </div>

      <!-- Directory listing -->
      <div class="border border-gray-200 dark:border-gray-600 rounded max-h-64 overflow-y-auto">
        <div v-if="loading" class="p-4 text-center text-gray-500">
          Loading...
        </div>
        <div v-else-if="directories.length === 0" class="p-4 text-center text-gray-500">
          No subdirectories
        </div>
        <div v-else class="divide-y divide-gray-200 dark:divide-gray-600">
          <button
            v-for="entry in directories"
            :key="entry.path"
            type="button"
            class="w-full px-3 py-2 text-left hover:bg-gray-100 dark:hover:bg-gray-700 flex items-center gap-2"
            @click="navigateTo(entry.path)"
          >
            <svg class="w-5 h-5 text-yellow-500" fill="currentColor" viewBox="0 0 20 20">
              <path d="M2 6a2 2 0 012-2h5l2 2h5a2 2 0 012 2v6a2 2 0 01-2 2H4a2 2 0 01-2-2V6z" />
            </svg>
            <span class="truncate">{{ entry.name }}</span>
          </button>
        </div>
      </div>
    </div>

    <template #footer>
      <button
        type="button"
        class="px-4 py-2 text-sm bg-gray-200 dark:bg-gray-600 rounded hover:bg-gray-300 dark:hover:bg-gray-500"
        @click="emit('close')"
      >
        Cancel
      </button>
      <button
        type="button"
        class="px-4 py-2 text-sm bg-blue-600 text-white rounded hover:bg-blue-700"
        @click="selectCurrent"
      >
        Select This Folder
      </button>
    </template>
  </Modal>
</template>
