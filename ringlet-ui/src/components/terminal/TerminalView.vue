<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch, nextTick } from 'vue'
import { Terminal } from '@xterm/xterm'
import { FitAddon } from '@xterm/addon-fit'
import { WebLinksAddon } from '@xterm/addon-web-links'
import '@xterm/xterm/css/xterm.css'
import { useTerminalStore } from '@/stores/terminal'

const props = defineProps<{
  sessionId: string
}>()

const emit = defineEmits<{
  (e: 'state-change', state: string, exitCode: number | null): void
  (e: 'error', message: string): void
}>()

const terminalStore = useTerminalStore()
const terminalRef = ref<HTMLDivElement | null>(null)
const connected = ref(false)
const sessionState = ref<string>('connecting')

let terminal: Terminal | null = null
let fitAddon: FitAddon | null = null
let resizeObserver: ResizeObserver | null = null

function initTerminal() {
  if (!terminalRef.value) return

  terminal = new Terminal({
    cursorBlink: true,
    fontSize: 14,
    fontFamily: 'Menlo, Monaco, "Courier New", monospace',
    theme: {
      background: '#1e1e1e',
      foreground: '#d4d4d4',
      cursor: '#d4d4d4',
      cursorAccent: '#1e1e1e',
      selectionBackground: '#264f78',
      black: '#000000',
      red: '#cd3131',
      green: '#0dbc79',
      yellow: '#e5e510',
      blue: '#2472c8',
      magenta: '#bc3fbc',
      cyan: '#11a8cd',
      white: '#e5e5e5',
      brightBlack: '#666666',
      brightRed: '#f14c4c',
      brightGreen: '#23d18b',
      brightYellow: '#f5f543',
      brightBlue: '#3b8eea',
      brightMagenta: '#d670d6',
      brightCyan: '#29b8db',
      brightWhite: '#e5e5e5',
    },
  })

  fitAddon = new FitAddon()
  terminal.loadAddon(fitAddon)
  terminal.loadAddon(new WebLinksAddon())

  terminal.open(terminalRef.value)
  fitAddon.fit()

  // Handle user input
  terminal.onData((data) => {
    const encoder = new TextEncoder()
    terminalStore.sendInput(props.sessionId, encoder.encode(data))
  })

  // Handle resize
  terminal.onResize(({ cols, rows }) => {
    terminalStore.sendResize(props.sessionId, cols, rows)
  })

  // Observe container resize
  resizeObserver = new ResizeObserver(() => {
    fitAddon?.fit()
  })
  resizeObserver.observe(terminalRef.value)

  // Connect to WebSocket
  connectToSession()
}

function connectToSession() {
  terminalStore.connectSession(
    props.sessionId,
    // onData
    (data) => {
      const decoder = new TextDecoder()
      terminal?.write(decoder.decode(data))
    },
    // onStateChange
    (state, exitCode) => {
      sessionState.value = state
      connected.value = state === 'running'
      emit('state-change', state, exitCode)
      if (state === 'terminated') {
        terminal?.write(`\r\n\x1b[33m[Session terminated${exitCode !== null ? ` with exit code ${exitCode}` : ''}]\x1b[0m\r\n`)
      }
      // Auto-focus terminal when connected
      if (state === 'running') {
        terminal?.focus()
      }
    },
    // onError
    (message) => {
      emit('error', message)
      terminal?.write(`\r\n\x1b[31m[Error: ${message}]\x1b[0m\r\n`)
    }
  )
  connected.value = true
  sessionState.value = 'running'
  // Focus the terminal
  terminal?.focus()
}

function handleFit() {
  fitAddon?.fit()
}

onMounted(async () => {
  await nextTick()
  initTerminal()
})

onUnmounted(() => {
  terminalStore.disconnectSession(props.sessionId)
  resizeObserver?.disconnect()
  terminal?.dispose()
})

// Reconnect if sessionId changes
watch(() => props.sessionId, () => {
  terminalStore.disconnectSession(props.sessionId)
  terminal?.clear()
  connectToSession()
})

defineExpose({
  fit: handleFit,
  focus: () => terminal?.focus(),
})
</script>

<template>
  <div class="terminal-container" @click="terminal?.focus()">
    <div class="terminal-header">
      <span class="status-indicator" :class="{ connected }"></span>
      <span class="session-info">Session: {{ sessionId.slice(0, 8) }}...</span>
      <span class="session-state">{{ sessionState }}</span>
    </div>
    <div ref="terminalRef" class="terminal-content"></div>
  </div>
</template>

<style scoped>
.terminal-container {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: #1e1e1e;
  border-radius: 8px;
  overflow: hidden;
}

.terminal-header {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 12px;
  background: #252526;
  border-bottom: 1px solid #333;
  font-size: 12px;
  color: #999;
}

.status-indicator {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #666;
}

.status-indicator.connected {
  background: #0dbc79;
}

.session-info {
  flex: 1;
  font-family: monospace;
}

.session-state {
  text-transform: capitalize;
  padding: 2px 8px;
  border-radius: 4px;
  background: #333;
}

.terminal-content {
  flex: 1;
  padding: 4px;
}

.terminal-content :deep(.xterm) {
  height: 100%;
}

.terminal-content :deep(.xterm-viewport) {
  overflow-y: auto !important;
}
</style>
