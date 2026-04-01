import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { api } from '@/api/client'
import type { GitInfo, PathCompletion, RecentWorkspace, WorkspaceBookmark } from '@/api/types'

const RECENTS_KEY = 'ringlet_recent_workspaces'
const BOOKMARKS_KEY = 'ringlet_workspace_bookmarks'
const MAX_RECENTS = 20

export const useWorkspaceStore = defineStore('workspace', () => {
  const currentPath = ref<string | null>(null)
  const currentGitInfo = ref<GitInfo | null>(null)
  const gitInfoLoading = ref(false)
  const gitInfoError = ref<string | null>(null)
  const recentWorkspaces = ref<RecentWorkspace[]>([])
  const bookmarks = ref<WorkspaceBookmark[]>([])
  const completions = ref<PathCompletion[]>([])

  // Git info cache for workspace cards
  const gitInfoCache = ref<Map<string, GitInfo>>(new Map())
  const gitInfoCacheLoading = ref<Set<string>>(new Set())

  const sortedRecents = computed(() =>
    [...recentWorkspaces.value].sort(
      (a, b) => new Date(b.last_opened).getTime() - new Date(a.last_opened).getTime()
    )
  )

  const sortedBookmarks = computed(() =>
    [...bookmarks.value].sort((a, b) => {
      if (a.pinned !== b.pinned) return a.pinned ? -1 : 1
      return new Date(b.added_at).getTime() - new Date(a.added_at).getTime()
    })
  )

  function loadFromStorage() {
    try {
      const recentsRaw = localStorage.getItem(RECENTS_KEY)
      if (recentsRaw) recentWorkspaces.value = JSON.parse(recentsRaw)
    } catch { /* ignore */ }
    try {
      const bookmarksRaw = localStorage.getItem(BOOKMARKS_KEY)
      if (bookmarksRaw) bookmarks.value = JSON.parse(bookmarksRaw)
    } catch { /* ignore */ }
  }

  function saveToStorage() {
    localStorage.setItem(RECENTS_KEY, JSON.stringify(recentWorkspaces.value))
    localStorage.setItem(BOOKMARKS_KEY, JSON.stringify(bookmarks.value))
  }

  async function fetchGitInfo(path: string): Promise<GitInfo | null> {
    try {
      return await api.git.info(path)
    } catch {
      return null
    }
  }

  async function fetchGitInfoCached(path: string) {
    if (gitInfoCache.value.has(path) || gitInfoCacheLoading.value.has(path)) return
    gitInfoCacheLoading.value.add(path)
    try {
      const info = await api.git.info(path)
      gitInfoCache.value.set(path, info)
    } catch { /* ignore */ } finally {
      gitInfoCacheLoading.value.delete(path)
    }
  }

  async function setWorkspace(path: string) {
    currentPath.value = path
    gitInfoLoading.value = true
    gitInfoError.value = null
    try {
      currentGitInfo.value = await api.git.info(path)
    } catch (e) {
      gitInfoError.value = e instanceof Error ? e.message : 'Failed to fetch git info'
      currentGitInfo.value = null
    } finally {
      gitInfoLoading.value = false
    }
    recordVisit(path)
  }

  function recordVisit(path: string) {
    const existing = recentWorkspaces.value.find(w => w.path === path)
    if (existing) {
      existing.last_opened = new Date().toISOString()
      existing.open_count++
    } else {
      recentWorkspaces.value.push({
        path,
        last_opened: new Date().toISOString(),
        open_count: 1,
      })
    }
    // Cap at MAX_RECENTS
    if (recentWorkspaces.value.length > MAX_RECENTS) {
      recentWorkspaces.value.sort(
        (a, b) => new Date(b.last_opened).getTime() - new Date(a.last_opened).getTime()
      )
      recentWorkspaces.value = recentWorkspaces.value.slice(0, MAX_RECENTS)
    }
    saveToStorage()
  }

  function addBookmark(path: string, name?: string) {
    if (bookmarks.value.some(b => b.path === path)) return
    const basename = path.split('/').filter(Boolean).pop() || path
    bookmarks.value.push({
      path,
      name: name || basename,
      pinned: false,
      added_at: new Date().toISOString(),
    })
    saveToStorage()
  }

  function removeBookmark(path: string) {
    bookmarks.value = bookmarks.value.filter(b => b.path !== path)
    saveToStorage()
  }

  function togglePin(path: string) {
    const bm = bookmarks.value.find(b => b.path === path)
    if (bm) {
      bm.pinned = !bm.pinned
      saveToStorage()
    }
  }

  function isBookmarked(path: string): boolean {
    return bookmarks.value.some(b => b.path === path)
  }

  async function fetchCompletions(prefix: string) {
    if (!prefix) {
      completions.value = []
      return
    }
    try {
      const response = await api.fs.complete(prefix)
      completions.value = response.completions
    } catch {
      completions.value = []
    }
  }

  function clearCompletions() {
    completions.value = []
  }

  return {
    currentPath,
    currentGitInfo,
    gitInfoLoading,
    gitInfoError,
    recentWorkspaces,
    bookmarks,
    completions,
    gitInfoCache,
    gitInfoCacheLoading,
    sortedRecents,
    sortedBookmarks,
    loadFromStorage,
    saveToStorage,
    fetchGitInfo,
    fetchGitInfoCached,
    setWorkspace,
    recordVisit,
    addBookmark,
    removeBookmark,
    togglePin,
    isBookmarked,
    fetchCompletions,
    clearCompletions,
  }
})
