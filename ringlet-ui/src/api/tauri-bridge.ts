/**
 * Tauri IPC bridge for API and WebSocket calls.
 *
 * Bridges Vue frontend calls to Tauri Rust backend commands,
 * which then proxy requests to the ringletd daemon.
 */

import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

/**
 * Proxy an HTTP API request through Tauri IPC to the daemon.
 *
 * Replaces direct fetch() calls when running in Tauri mode.
 * The Rust backend adds auth tokens and routes to the configured daemon.
 */
export async function tauriApiRequest<T>(
  method: string,
  endpoint: string,
  body?: unknown
): Promise<T> {
  const response = await invoke<T>('api_request', {
    method,
    endpoint,
    body: body ?? null,
  })
  return response
}

/** Handle for controlling a Tauri-proxied WebSocket connection. */
export interface TauriWsHandle {
  /** Connection ID. */
  id: string
  /** Send a text message. */
  send: (message: string) => Promise<void>
  /** Send binary data (for terminal I/O). */
  sendBinary: (data: Uint8Array) => Promise<void>
  /** Close the connection. */
  close: () => Promise<void>
}

/**
 * Connect to a WebSocket endpoint through Tauri IPC.
 *
 * Server messages are received via Tauri events:
 * - Text messages → onMessage callback
 * - Binary messages → onBinary callback
 * - Connection close → onClose callback
 */
export async function tauriWsConnect(
  path: string,
  onMessage: (data: string) => void,
  onBinary: ((data: Uint8Array) => void) | null,
  onClose: () => void
): Promise<TauriWsHandle> {
  const id = await invoke<string>('ws_connect', { path })

  const unlisteners: UnlistenFn[] = []

  // Listen for text messages
  const unlistenMessage = await listen<string>(`ws-message-${id}`, (event) => {
    onMessage(event.payload)
  })
  unlisteners.push(unlistenMessage)

  // Listen for binary messages
  if (onBinary) {
    const unlistenBinary = await listen<number[]>(`ws-binary-${id}`, (event) => {
      onBinary(new Uint8Array(event.payload))
    })
    unlisteners.push(unlistenBinary)
  }

  // Listen for connection close
  const unlistenClose = await listen(`ws-close-${id}`, () => {
    // Clean up all listeners
    unlisteners.forEach((unlisten) => unlisten())
    onClose()
  })
  unlisteners.push(unlistenClose)

  return {
    id,
    send: async (message: string) => {
      await invoke('ws_send', { id, message })
    },
    sendBinary: async (data: Uint8Array) => {
      await invoke('ws_send_binary', { id, data: Array.from(data) })
    },
    close: async () => {
      unlisteners.forEach((unlisten) => unlisten())
      await invoke('ws_close', { id })
    },
  }
}
