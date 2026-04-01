import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { api } from '@/api/client'
import type { TerminalSessionInfo, TerminalServerMessage } from '@/api/types'
import { isTauri } from '@/api/config'
import { tauriWsConnect, type TauriWsHandle } from '@/api/tauri-bridge'
import { notifySessionComplete } from '@/utils/notifications'

export const useTerminalStore = defineStore('terminal', () => {
  const sessions = ref<TerminalSessionInfo[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  // Active terminal connections (session_id -> WebSocket or TauriWsHandle)
  const connections = ref<Map<string, WebSocket>>(new Map())
  const tauriConnections = new Map<string, TauriWsHandle>()

  const activeSessions = computed(() =>
    sessions.value.filter(s => {
      if (typeof s.state === 'string') {
        return s.state !== 'terminated'
      }
      return false
    })
  )

  async function fetchSessions() {
    loading.value = true
    error.value = null
    try {
      sessions.value = await api.terminal.list()
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to fetch sessions'
    } finally {
      loading.value = false
    }
  }

  async function createSession(
    profileAlias: string,
    args: string[] = [],
    cols = 80,
    rows = 24,
    workingDir?: string,
    noSandbox = false
  ): Promise<string | null> {
    try {
      const response = await api.terminal.create({
        profile_alias: profileAlias,
        args,
        cols,
        rows,
        working_dir: workingDir,
        no_sandbox: noSandbox,
      })
      // Refresh the sessions list
      await fetchSessions()
      return response.session_id
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to create session'
      return null
    }
  }

  async function createShellSession(
    shell = 'bash',
    workingDir?: string,
    cols = 80,
    rows = 24,
    noSandbox = false
  ): Promise<string | null> {
    try {
      const response = await api.terminal.createShell({
        shell,
        cols,
        rows,
        working_dir: workingDir,
        no_sandbox: noSandbox,
      })
      // Refresh the sessions list
      await fetchSessions()
      return response.session_id
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to create shell session'
      return null
    }
  }

  async function terminateSession(sessionId: string) {
    try {
      await api.terminal.terminate(sessionId)
      // Disconnect WebSocket if connected
      disconnectSession(sessionId)
      // Refresh the sessions list
      await fetchSessions()
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to terminate session'
    }
  }

  function connectSession(
    sessionId: string,
    onData: (data: Uint8Array) => void,
    onStateChange: (state: string, exitCode: number | null) => void,
    onError: (message: string) => void
  ): WebSocket | null {
    // Close existing connection if any
    disconnectSession(sessionId)

    if (isTauri()) {
      connectSessionTauri(sessionId, onData, onStateChange, onError)
      return null // Tauri mode doesn't return a WebSocket
    }

    return connectSessionBrowser(sessionId, onData, onStateChange, onError)
  }

  async function connectSessionTauri(
    sessionId: string,
    onData: (data: Uint8Array) => void,
    onStateChange: (state: string, exitCode: number | null) => void,
    onError: (message: string) => void
  ) {
    try {
      const handle = await tauriWsConnect(
        `/ws/terminal/${sessionId}`,
        (data: string) => {
          // JSON control message
          try {
            const msg: TerminalServerMessage = JSON.parse(data)
            switch (msg.type) {
              case 'state_changed':
                onStateChange(msg.state, msg.exit_code)
                if (msg.state === 'terminated') {
                  const session = sessions.value.find(s => s.id === sessionId)
                  notifySessionComplete(session?.profile_alias || sessionId, msg.exit_code)
                  fetchSessions()
                }
                break
              case 'error':
                onError(msg.message)
                break
            }
          } catch {
            console.error('Failed to parse terminal message:', data)
          }
        },
        (data: Uint8Array) => {
          // Binary data from terminal
          onData(data)
        },
        () => {
          tauriConnections.delete(sessionId)
        }
      )

      tauriConnections.set(sessionId, handle)
    } catch (e) {
      onError(e instanceof Error ? e.message : 'Failed to connect')
    }
  }

  function connectSessionBrowser(
    sessionId: string,
    onData: (data: Uint8Array) => void,
    onStateChange: (state: string, exitCode: number | null) => void,
    onError: (message: string) => void
  ): WebSocket | null {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:'
    const wsUrl = `${protocol}//${window.location.host}/ws/terminal/${sessionId}`

    try {
      const socket = new WebSocket(wsUrl)
      socket.binaryType = 'arraybuffer'

      socket.onopen = () => {
        connections.value.set(sessionId, socket)
      }

      socket.onmessage = (event) => {
        if (event.data instanceof ArrayBuffer) {
          // Binary data from terminal
          onData(new Uint8Array(event.data))
        } else {
          // JSON control message
          try {
            const msg: TerminalServerMessage = JSON.parse(event.data)
            switch (msg.type) {
              case 'state_changed':
                onStateChange(msg.state, msg.exit_code)
                if (msg.state === 'terminated') {
                  const session = sessions.value.find(s => s.id === sessionId)
                  notifySessionComplete(session?.profile_alias || sessionId, msg.exit_code)
                  fetchSessions()
                }
                break
              case 'error':
                onError(msg.message)
                break
            }
          } catch {
            console.error('Failed to parse terminal message:', event.data)
          }
        }
      }

      socket.onclose = () => {
        connections.value.delete(sessionId)
      }

      socket.onerror = () => {
        onError('WebSocket connection error')
        connections.value.delete(sessionId)
      }

      return socket
    } catch (e) {
      onError(e instanceof Error ? e.message : 'Failed to connect')
      return null
    }
  }

  function disconnectSession(sessionId: string) {
    // Disconnect Tauri handle if present
    const tauriHandle = tauriConnections.get(sessionId)
    if (tauriHandle) {
      tauriHandle.close()
      tauriConnections.delete(sessionId)
    }
    // Disconnect browser WebSocket if present
    const socket = connections.value.get(sessionId)
    if (socket) {
      socket.close()
      connections.value.delete(sessionId)
    }
  }

  function sendInput(sessionId: string, data: Uint8Array) {
    const tauriHandle = tauriConnections.get(sessionId)
    if (tauriHandle) {
      tauriHandle.sendBinary(data)
      return
    }
    const socket = connections.value.get(sessionId)
    if (socket?.readyState === WebSocket.OPEN) {
      socket.send(data)
    }
  }

  function sendResize(sessionId: string, cols: number, rows: number) {
    const msg = JSON.stringify({ type: 'resize', cols, rows })
    const tauriHandle = tauriConnections.get(sessionId)
    if (tauriHandle) {
      tauriHandle.send(msg)
      return
    }
    const socket = connections.value.get(sessionId)
    if (socket?.readyState === WebSocket.OPEN) {
      socket.send(msg)
    }
  }

  function sendSignal(sessionId: string, signal: number) {
    const msg = JSON.stringify({ type: 'signal', signal })
    const tauriHandle = tauriConnections.get(sessionId)
    if (tauriHandle) {
      tauriHandle.send(msg)
      return
    }
    const socket = connections.value.get(sessionId)
    if (socket?.readyState === WebSocket.OPEN) {
      socket.send(msg)
    }
  }

  function isConnected(sessionId: string): boolean {
    if (tauriConnections.has(sessionId)) {
      return true
    }
    const socket = connections.value.get(sessionId)
    return socket?.readyState === WebSocket.OPEN
  }

  return {
    sessions,
    activeSessions,
    loading,
    error,
    fetchSessions,
    createSession,
    createShellSession,
    terminateSession,
    connectSession,
    disconnectSession,
    sendInput,
    sendResize,
    sendSignal,
    isConnected,
  }
})
