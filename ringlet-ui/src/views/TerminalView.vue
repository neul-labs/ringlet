<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRoute } from 'vue-router'
import { useTerminalStore } from '@/stores/terminal'
import { useProfilesStore } from '@/stores/profiles'
import TerminalView from '@/components/terminal/TerminalView.vue'
import TerminalSessionList from '@/components/terminal/TerminalSessionList.vue'
import LoadingSpinner from '@/components/common/LoadingSpinner.vue'
import Modal from '@/components/common/Modal.vue'

const route = useRoute()
const terminalStore = useTerminalStore()
const profilesStore = useProfilesStore()

const selectedSessionId = ref<string | null>(null)
const showNewSessionModal = ref(false)
const newSessionProfile = ref('')
const newSessionArgs = ref('')
const newSessionWorkingDir = ref('')
const creating = ref(false)

// Get session ID from route if provided
const routeSessionId = computed(() => route.params.sessionId as string | undefined)

onMounted(async () => {
  await terminalStore.fetchSessions()
  await profilesStore.fetchProfiles()

  // If a session ID is in the route, select it
  if (routeSessionId.value) {
    selectedSessionId.value = routeSessionId.value
  }
})

function selectSession(sessionId: string) {
  selectedSessionId.value = sessionId
}

async function terminateSession(sessionId: string) {
  if (confirm('Are you sure you want to terminate this session?')) {
    await terminalStore.terminateSession(sessionId)
    if (selectedSessionId.value === sessionId) {
      selectedSessionId.value = null
    }
  }
}

function openNewSessionModal() {
  newSessionProfile.value = ''
  newSessionArgs.value = ''
  newSessionWorkingDir.value = ''
  showNewSessionModal.value = true
}

async function createSession() {
  if (!newSessionProfile.value) return

  creating.value = true
  const args = newSessionArgs.value ? newSessionArgs.value.split(' ').filter(Boolean) : []
  const workingDir = newSessionWorkingDir.value.trim() || undefined
  const sessionId = await terminalStore.createSession(newSessionProfile.value, args, 80, 24, workingDir)

  if (sessionId) {
    selectedSessionId.value = sessionId
    showNewSessionModal.value = false
  }

  creating.value = false
}

function handleStateChange(state: string, _exitCode: number | null) {
  if (state === 'terminated') {
    terminalStore.fetchSessions()
  }
}

function handleError(message: string) {
  console.error('Terminal error:', message)
}
</script>

<template>
  <div class="terminal-page">
    <div class="page-header">
      <h1 class="page-title">Terminal Sessions</h1>
      <button class="btn-primary" @click="openNewSessionModal">
        New Session
      </button>
    </div>

    <div class="terminal-layout">
      <!-- Sidebar with session list -->
      <div class="sessions-panel">
        <div class="panel-header">
          <h2>Sessions</h2>
          <button class="btn-icon" title="Refresh" @click="terminalStore.fetchSessions()">
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M21.5 2v6h-6M2.5 22v-6h6M2 11.5a10 10 0 0 1 18.8-4.3M22 12.5a10 10 0 0 1-18.8 4.2"/>
            </svg>
          </button>
        </div>

        <LoadingSpinner v-if="terminalStore.loading" />
        <TerminalSessionList
          v-else
          :sessions="terminalStore.sessions"
          :selected-id="selectedSessionId ?? undefined"
          @select="selectSession"
          @terminate="terminateSession"
        />
      </div>

      <!-- Main terminal area -->
      <div class="terminal-panel">
        <div v-if="!selectedSessionId" class="empty-terminal">
          <div class="empty-content">
            <svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
              <polyline points="4 17 10 11 4 5"></polyline>
              <line x1="12" y1="19" x2="20" y2="19"></line>
            </svg>
            <h3>No Session Selected</h3>
            <p>Select a session from the list or create a new one</p>
            <button class="btn-primary" @click="openNewSessionModal">
              Create New Session
            </button>
          </div>
        </div>
        <TerminalView
          v-else
          :key="selectedSessionId"
          :session-id="selectedSessionId"
          @state-change="handleStateChange"
          @error="handleError"
        />
      </div>
    </div>

    <!-- New Session Modal -->
    <Modal
      :open="showNewSessionModal"
      title="New Terminal Session"
      @close="showNewSessionModal = false"
    >
      <div class="form-group">
        <label for="profile">Profile</label>
        <select id="profile" v-model="newSessionProfile" class="form-select">
          <option value="">Select a profile...</option>
          <option
            v-for="profile in profilesStore.profiles"
            :key="profile.alias"
            :value="profile.alias"
          >
            {{ profile.alias }} ({{ profile.agent_id }})
          </option>
        </select>
      </div>

      <div class="form-group">
        <label for="args">Arguments (optional)</label>
        <input
          id="args"
          v-model="newSessionArgs"
          type="text"
          class="form-input"
          placeholder="e.g., --dangerously-skip-permissions"
        />
      </div>

      <div class="form-group">
        <label for="workingDir">Working Directory (optional)</label>
        <input
          id="workingDir"
          v-model="newSessionWorkingDir"
          type="text"
          class="form-input"
          placeholder="e.g., /home/user/project"
        />
      </div>

      <template #footer>
        <button class="btn-secondary" @click="showNewSessionModal = false">
          Cancel
        </button>
        <button
          class="btn-primary"
          :disabled="!newSessionProfile || creating"
          @click="createSession"
        >
          {{ creating ? 'Creating...' : 'Create Session' }}
        </button>
      </template>
    </Modal>
  </div>
</template>

<style scoped>
.terminal-page {
  height: calc(100vh - 140px);
  display: flex;
  flex-direction: column;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
}

.page-title {
  font-size: 24px;
  font-weight: 600;
  color: #111827;
}

.dark .page-title {
  color: #f9fafb;
}

.terminal-layout {
  flex: 1;
  display: grid;
  grid-template-columns: 300px 1fr;
  gap: 24px;
  min-height: 0;
}

.sessions-panel {
  display: flex;
  flex-direction: column;
  background: #fff;
  border-radius: 12px;
  padding: 16px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
  overflow: hidden;
}

.dark .sessions-panel {
  background: #1f2937;
}

.panel-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.panel-header h2 {
  font-size: 14px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: #6b7280;
}

.btn-icon {
  padding: 6px;
  color: #6b7280;
  background: transparent;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.btn-icon:hover {
  background: #f3f4f6;
  color: #374151;
}

.dark .btn-icon:hover {
  background: #374151;
  color: #f9fafb;
}

.terminal-panel {
  background: #1e1e1e;
  border-radius: 12px;
  overflow: hidden;
  min-height: 400px;
}

.empty-terminal {
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #666;
}

.empty-content {
  text-align: center;
}

.empty-content svg {
  margin-bottom: 16px;
  color: #444;
}

.empty-content h3 {
  font-size: 18px;
  font-weight: 600;
  margin-bottom: 8px;
  color: #888;
}

.empty-content p {
  margin-bottom: 24px;
  color: #666;
}

.btn-primary {
  padding: 10px 20px;
  background: #3b82f6;
  color: white;
  border: none;
  border-radius: 8px;
  font-weight: 500;
  cursor: pointer;
  transition: background 0.15s ease;
}

.btn-primary:hover {
  background: #2563eb;
}

.btn-primary:disabled {
  background: #93c5fd;
  cursor: not-allowed;
}

.btn-secondary {
  padding: 10px 20px;
  background: #f3f4f6;
  color: #374151;
  border: none;
  border-radius: 8px;
  font-weight: 500;
  cursor: pointer;
  transition: background 0.15s ease;
}

.btn-secondary:hover {
  background: #e5e7eb;
}

.dark .btn-secondary {
  background: #374151;
  color: #f9fafb;
}

.dark .btn-secondary:hover {
  background: #4b5563;
}

.form-group {
  margin-bottom: 16px;
}

.form-group label {
  display: block;
  margin-bottom: 6px;
  font-size: 14px;
  font-weight: 500;
  color: #374151;
}

.dark .form-group label {
  color: #d1d5db;
}

.form-select,
.form-input {
  width: 100%;
  padding: 10px 12px;
  border: 1px solid #d1d5db;
  border-radius: 8px;
  font-size: 14px;
  background: #fff;
  color: #111827;
}

.dark .form-select,
.dark .form-input {
  background: #374151;
  border-color: #4b5563;
  color: #f9fafb;
}

.form-select:focus,
.form-input:focus {
  outline: none;
  border-color: #3b82f6;
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
}
</style>
