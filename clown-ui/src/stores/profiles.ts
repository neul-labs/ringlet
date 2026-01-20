import { defineStore } from 'pinia'
import { ref } from 'vue'
import { api } from '@/api/client'
import type { ProfileInfo, ProfileCreateRequest } from '@/api/types'

export const useProfilesStore = defineStore('profiles', () => {
  const profiles = ref<ProfileInfo[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function fetchProfiles(agentId?: string) {
    loading.value = true
    error.value = null
    try {
      profiles.value = await api.profiles.list(agentId)
    } catch (e) {
      error.value = (e as Error).message
    } finally {
      loading.value = false
    }
  }

  async function getProfile(alias: string): Promise<ProfileInfo | null> {
    try {
      return await api.profiles.get(alias)
    } catch {
      return null
    }
  }

  async function createProfile(data: ProfileCreateRequest) {
    await api.profiles.create(data)
    await fetchProfiles()
  }

  async function deleteProfile(alias: string) {
    await api.profiles.delete(alias)
    profiles.value = profiles.value.filter((p) => p.alias !== alias)
  }

  function getProfileByAlias(alias: string): ProfileInfo | undefined {
    return profiles.value.find((p) => p.alias === alias)
  }

  // Called by WebSocket events
  function handleProfileCreated(_alias: string) {
    fetchProfiles() // Refresh list
  }

  function handleProfileDeleted(alias: string) {
    profiles.value = profiles.value.filter((p) => p.alias !== alias)
  }

  return {
    profiles,
    loading,
    error,
    fetchProfiles,
    getProfile,
    createProfile,
    deleteProfile,
    getProfileByAlias,
    handleProfileCreated,
    handleProfileDeleted,
  }
})
