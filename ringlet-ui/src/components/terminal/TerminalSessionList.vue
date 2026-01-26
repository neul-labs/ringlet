<script setup lang="ts">
import type { TerminalSessionInfo } from '@/api/types'

defineProps<{
  sessions: TerminalSessionInfo[]
  selectedId?: string
}>()

const emit = defineEmits<{
  (e: 'select', sessionId: string): void
  (e: 'terminate', sessionId: string): void
}>()

function getStateLabel(session: TerminalSessionInfo): string {
  if (typeof session.state === 'string') {
    return session.state
  }
  return 'terminated'
}

function getStateColor(session: TerminalSessionInfo): string {
  const state = getStateLabel(session)
  switch (state) {
    case 'running':
      return 'text-green-500'
    case 'starting':
      return 'text-yellow-500'
    case 'terminated':
      return 'text-red-500'
    default:
      return 'text-gray-500'
  }
}
</script>

<template>
  <div class="session-list">
    <div v-if="sessions.length === 0" class="empty-state">
      No active terminal sessions
    </div>
    <div
      v-for="session in sessions"
      :key="session.id"
      class="session-item"
      :class="{ selected: session.id === selectedId }"
      @click="emit('select', session.id)"
    >
      <div class="session-main">
        <div class="session-profile">{{ session.profile_alias }}</div>
        <div class="session-meta">
          <span :class="getStateColor(session)">{{ getStateLabel(session) }}</span>
          <span class="separator">|</span>
          <span>{{ session.cols }}x{{ session.rows }}</span>
          <span class="separator">|</span>
          <span>{{ session.client_count }} client(s)</span>
        </div>
      </div>
      <div class="session-actions">
        <button
          v-if="getStateLabel(session) !== 'terminated'"
          class="btn-terminate"
          title="Terminate session"
          @click.stop="emit('terminate', session.id)"
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect>
          </svg>
        </button>
      </div>
      <div class="session-id">{{ session.id.slice(0, 8) }}...</div>
    </div>
  </div>
</template>

<style scoped>
.session-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.empty-state {
  text-align: center;
  padding: 24px;
  color: #666;
}

.session-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 12px;
  background: #fff;
  border: 1px solid #e5e7eb;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.session-item:hover {
  border-color: #3b82f6;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.05);
}

.session-item.selected {
  border-color: #3b82f6;
  background: #eff6ff;
}

.dark .session-item {
  background: #1f2937;
  border-color: #374151;
}

.dark .session-item:hover {
  border-color: #3b82f6;
}

.dark .session-item.selected {
  background: #1e3a5f;
}

.session-main {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.session-profile {
  font-weight: 600;
  font-size: 14px;
}

.session-meta {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  color: #666;
}

.separator {
  color: #ccc;
}

.session-actions {
  position: absolute;
  right: 12px;
  top: 50%;
  transform: translateY(-50%);
}

.session-item {
  position: relative;
}

.btn-terminate {
  padding: 4px;
  color: #666;
  background: transparent;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  opacity: 0;
  transition: all 0.15s ease;
}

.session-item:hover .btn-terminate {
  opacity: 1;
}

.btn-terminate:hover {
  color: #ef4444;
  background: #fef2f2;
}

.session-id {
  font-size: 11px;
  font-family: monospace;
  color: #999;
}
</style>
