import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Event, ServerMessage, ClientMessage } from '@/api/types'
import { useProfilesStore } from './profiles'
import { useProxyStore } from './proxy'

export const useWebSocketStore = defineStore('websocket', () => {
  const connected = ref(false)
  const events = ref<Event[]>([])
  const version = ref<string | null>(null)

  let socket: WebSocket | null = null
  let reconnectTimeout: ReturnType<typeof setTimeout> | null = null

  const recentEvents = computed(() => events.value.slice(-50))

  function connect() {
    if (socket?.readyState === WebSocket.OPEN) return

    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:'
    const wsUrl = `${protocol}//${window.location.host}/ws`

    socket = new WebSocket(wsUrl)

    socket.onopen = () => {
      connected.value = true
      // Subscribe to all events
      send({ type: 'subscribe', topics: ['*'] })
    }

    socket.onmessage = (e) => {
      try {
        const msg: ServerMessage = JSON.parse(e.data)
        handleMessage(msg)
      } catch {
        console.error('Failed to parse WebSocket message:', e.data)
      }
    }

    socket.onclose = () => {
      connected.value = false
      socket = null
      // Reconnect after 3 seconds
      reconnectTimeout = setTimeout(connect, 3000)
    }

    socket.onerror = () => {
      socket?.close()
    }
  }

  function disconnect() {
    if (reconnectTimeout) {
      clearTimeout(reconnectTimeout)
      reconnectTimeout = null
    }
    socket?.close()
    socket = null
    connected.value = false
  }

  function send(msg: ClientMessage) {
    if (socket?.readyState === WebSocket.OPEN) {
      socket.send(JSON.stringify(msg))
    }
  }

  function handleMessage(msg: ServerMessage) {
    if (msg.type === 'event' && msg.event) {
      const event = msg.event
      events.value.push(event)

      // Keep only last 100 events
      if (events.value.length > 100) {
        events.value = events.value.slice(-100)
      }

      // Handle specific events
      handleEvent(event)
    }
  }

  function handleEvent(event: Event) {
    const profilesStore = useProfilesStore()
    const proxyStore = useProxyStore()

    switch (event.type) {
      case 'connected':
        version.value = event.data.version
        break

      case 'profile_created':
        profilesStore.handleProfileCreated(event.data.alias)
        break

      case 'profile_deleted':
        profilesStore.handleProfileDeleted(event.data.alias)
        break

      case 'proxy_started':
        proxyStore.handleProxyStarted(event.data.alias, event.data.port)
        break

      case 'proxy_stopped':
        proxyStore.handleProxyStopped(event.data.alias)
        break
    }
  }

  return {
    connected,
    events,
    recentEvents,
    version,
    connect,
    disconnect,
    send,
  }
})
