<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted } from 'vue'
import { useWorkspaceStore } from '@/stores/workspace'
import DirectoryPicker from '@/components/common/DirectoryPicker.vue'

const props = defineProps<{
  modelValue: string
  placeholder?: string
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string]
  select: [path: string]
}>()

const workspaceStore = useWorkspaceStore()

const inputValue = ref(props.modelValue)
const showDropdown = ref(false)
const showBrowse = ref(false)
const selectedIndex = ref(-1)
const containerRef = ref<HTMLElement | null>(null)

let debounceTimer: ReturnType<typeof setTimeout> | null = null

watch(() => props.modelValue, (v) => {
  inputValue.value = v
})

function onInput(e: Event) {
  const val = (e.target as HTMLInputElement).value
  inputValue.value = val
  emit('update:modelValue', val)
  selectedIndex.value = -1

  if (debounceTimer) clearTimeout(debounceTimer)
  debounceTimer = setTimeout(() => {
    if (val.length > 0) {
      workspaceStore.fetchCompletions(val)
      showDropdown.value = true
    } else {
      workspaceStore.clearCompletions()
      showDropdown.value = false
    }
  }, 250)
}

function selectCompletion(path: string) {
  inputValue.value = path + '/'
  emit('update:modelValue', path)
  showDropdown.value = false
  workspaceStore.clearCompletions()
  // Trigger new completions for the selected directory
  workspaceStore.fetchCompletions(path + '/')
  showDropdown.value = true
}

function confirmSelection() {
  const val = inputValue.value.replace(/\/+$/, '')
  if (val) {
    emit('select', val)
    showDropdown.value = false
    workspaceStore.clearCompletions()
  }
}

function onKeydown(e: KeyboardEvent) {
  const completions = workspaceStore.completions
  if (!showDropdown.value || completions.length === 0) {
    if (e.key === 'Enter') {
      e.preventDefault()
      confirmSelection()
    }
    return
  }

  switch (e.key) {
    case 'ArrowDown':
      e.preventDefault()
      selectedIndex.value = Math.min(selectedIndex.value + 1, completions.length - 1)
      break
    case 'ArrowUp':
      e.preventDefault()
      selectedIndex.value = Math.max(selectedIndex.value - 1, -1)
      break
    case 'Enter':
      e.preventDefault()
      if (selectedIndex.value >= 0 && selectedIndex.value < completions.length) {
        selectCompletion(completions[selectedIndex.value].path)
      } else {
        confirmSelection()
      }
      break
    case 'Tab':
      e.preventDefault()
      if (completions.length === 1) {
        selectCompletion(completions[0].path)
      } else if (selectedIndex.value >= 0) {
        selectCompletion(completions[selectedIndex.value].path)
      }
      break
    case 'Escape':
      showDropdown.value = false
      workspaceStore.clearCompletions()
      break
  }
}

function onClickOutside(e: MouseEvent) {
  if (containerRef.value && !containerRef.value.contains(e.target as Node)) {
    showDropdown.value = false
  }
}

function onBrowseSelect(path: string) {
  inputValue.value = path
  emit('update:modelValue', path)
  emit('select', path)
  showBrowse.value = false
}

function selectBookmark(path: string) {
  inputValue.value = path
  emit('update:modelValue', path)
  emit('select', path)
}

onMounted(() => {
  document.addEventListener('click', onClickOutside)
})

onUnmounted(() => {
  document.removeEventListener('click', onClickOutside)
  if (debounceTimer) clearTimeout(debounceTimer)
})
</script>

<template>
  <div ref="containerRef" class="relative">
    <div class="flex gap-2">
      <div class="relative flex-1">
        <input
          :value="inputValue"
          type="text"
          :placeholder="placeholder || 'Type a path... (e.g. /home/user/project)'"
          class="w-full px-4 py-3 text-sm font-mono bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
          @input="onInput"
          @keydown="onKeydown"
          @focus="inputValue.length > 0 && (showDropdown = true)"
        />
        <!-- Dropdown -->
        <div
          v-if="showDropdown && workspaceStore.completions.length > 0"
          class="absolute z-50 w-full mt-1 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-600 rounded-lg shadow-lg max-h-60 overflow-y-auto"
        >
          <button
            v-for="(completion, index) in workspaceStore.completions"
            :key="completion.path"
            type="button"
            :class="[
              'w-full px-3 py-2 text-left flex items-center gap-2 text-sm',
              index === selectedIndex
                ? 'bg-blue-50 dark:bg-blue-900/50'
                : 'hover:bg-gray-50 dark:hover:bg-gray-700'
            ]"
            @click="selectCompletion(completion.path)"
          >
            <svg class="w-4 h-4 text-yellow-500 flex-shrink-0" fill="currentColor" viewBox="0 0 20 20">
              <path d="M2 6a2 2 0 012-2h5l2 2h5a2 2 0 012 2v6a2 2 0 01-2 2H4a2 2 0 01-2-2V6z" />
            </svg>
            <span class="font-medium truncate">{{ completion.name }}</span>
            <span class="text-xs text-gray-400 truncate ml-auto">{{ completion.path }}</span>
          </button>
        </div>
      </div>
      <button
        type="button"
        class="px-4 py-3 text-sm bg-gray-200 dark:bg-gray-700 rounded-lg hover:bg-gray-300 dark:hover:bg-gray-600 flex items-center gap-1"
        @click="showBrowse = true"
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 19a2 2 0 01-2-2V7a2 2 0 012-2h4l2 2h4a2 2 0 012 2v1M5 19h14a2 2 0 002-2v-5a2 2 0 00-2-2H9a2 2 0 00-2 2v5a2 2 0 01-2 2z" />
        </svg>
        Browse
      </button>
      <button
        type="button"
        class="px-4 py-3 text-sm bg-blue-600 text-white rounded-lg hover:bg-blue-700"
        @click="confirmSelection"
      >
        Open
      </button>
    </div>

    <!-- Bookmark chips -->
    <div v-if="workspaceStore.sortedBookmarks.length > 0" class="flex flex-wrap gap-2 mt-2">
      <button
        v-for="bm in workspaceStore.sortedBookmarks"
        :key="bm.path"
        type="button"
        class="inline-flex items-center gap-1 px-3 py-1 text-xs rounded-full bg-blue-50 dark:bg-blue-900/30 text-blue-700 dark:text-blue-300 hover:bg-blue-100 dark:hover:bg-blue-900/50"
        @click="selectBookmark(bm.path)"
      >
        <svg v-if="bm.pinned" class="w-3 h-3" fill="currentColor" viewBox="0 0 20 20">
          <path d="M9.049 2.927c.3-.921 1.603-.921 1.902 0l1.07 3.292a1 1 0 00.95.69h3.462c.969 0 1.371 1.24.588 1.81l-2.8 2.034a1 1 0 00-.364 1.118l1.07 3.292c.3.921-.755 1.688-1.54 1.118l-2.8-2.034a1 1 0 00-1.175 0l-2.8 2.034c-.784.57-1.838-.197-1.539-1.118l1.07-3.292a1 1 0 00-.364-1.118L2.98 8.72c-.783-.57-.38-1.81.588-1.81h3.461a1 1 0 00.951-.69l1.07-3.292z" />
        </svg>
        <svg v-else class="w-3 h-3" fill="currentColor" viewBox="0 0 20 20">
          <path d="M2 6a2 2 0 012-2h5l2 2h5a2 2 0 012 2v6a2 2 0 01-2 2H4a2 2 0 01-2-2V6z" />
        </svg>
        {{ bm.name }}
      </button>
    </div>

    <!-- Browse modal -->
    <DirectoryPicker
      v-if="showBrowse"
      :open="showBrowse"
      :initial-path="inputValue || undefined"
      @close="showBrowse = false"
      @select="onBrowseSelect"
    />
  </div>
</template>
