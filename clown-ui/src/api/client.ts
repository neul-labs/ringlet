import type {
  ApiResponse,
  AgentInfo,
  ProviderInfo,
  ProfileInfo,
  ProfileCreateRequest,
  HooksConfig,
  ProxyInstanceInfo,
  ProfileProxyConfig,
  RoutingRule,
  StatsResponse,
  RegistryStatus,
  UsageStatsResponse,
} from './types'

class ApiError extends Error {
  constructor(
    public code: number,
    message: string
  ) {
    super(message)
    this.name = 'ApiError'
  }
}

const BASE_URL = '/api'

async function request<T>(
  endpoint: string,
  options: RequestInit = {}
): Promise<T> {
  const response = await fetch(`${BASE_URL}${endpoint}`, {
    headers: {
      'Content-Type': 'application/json',
      ...options.headers,
    },
    ...options,
  })

  const data: ApiResponse<T> = await response.json()

  if (!data.success || data.error) {
    throw new ApiError(data.error?.code ?? 0, data.error?.message ?? 'Unknown error')
  }

  return data.data as T
}

export const api = {
  // Agents
  agents: {
    list: () => request<AgentInfo[]>('/agents'),
    get: (id: string) => request<AgentInfo>(`/agents/${id}`),
  },

  // Providers
  providers: {
    list: () => request<ProviderInfo[]>('/providers'),
    get: (id: string) => request<ProviderInfo>(`/providers/${id}`),
  },

  // Profiles
  profiles: {
    list: (agentId?: string) =>
      request<ProfileInfo[]>(`/profiles${agentId ? `?agent=${agentId}` : ''}`),
    get: (alias: string) => request<ProfileInfo>(`/profiles/${alias}`),
    create: (data: ProfileCreateRequest) =>
      request<void>('/profiles', {
        method: 'POST',
        body: JSON.stringify(data),
      }),
    delete: (alias: string) =>
      request<void>(`/profiles/${alias}`, { method: 'DELETE' }),
    run: (alias: string, args: string[] = []) =>
      request<{ status: string; pid?: number; exit_code?: number }>(
        `/profiles/${alias}/run`,
        {
          method: 'POST',
          body: JSON.stringify({ args }),
        }
      ),
    env: (alias: string) =>
      request<Record<string, string>>(`/profiles/${alias}/env`),
  },

  // Hooks
  hooks: {
    list: (alias: string) => request<HooksConfig>(`/profiles/${alias}/hooks`),
    add: (alias: string, event: string, data: { matcher: string; command: string }) =>
      request<void>(`/profiles/${alias}/hooks`, {
        method: 'POST',
        body: JSON.stringify({ event, matcher: data.matcher, command: data.command }),
      }),
    remove: (alias: string, event: string, index: number) =>
      request<void>(`/profiles/${alias}/hooks/${event}/${index}`, {
        method: 'DELETE',
      }),
    import: (alias: string, config: HooksConfig) =>
      request<void>(`/profiles/${alias}/hooks/import`, {
        method: 'POST',
        body: JSON.stringify(config),
      }),
    export: (alias: string) =>
      request<HooksConfig>(`/profiles/${alias}/hooks/export`),
  },

  // Proxy
  proxy: {
    enable: (alias: string) =>
      request<void>(`/profiles/${alias}/proxy/enable`, { method: 'POST' }),
    disable: (alias: string) =>
      request<void>(`/profiles/${alias}/proxy/disable`, { method: 'POST' }),
    start: (alias: string) =>
      request<void>(`/profiles/${alias}/proxy/start`, { method: 'POST' }),
    stop: (alias: string) =>
      request<void>(`/profiles/${alias}/proxy/stop`, { method: 'POST' }),
    restart: (alias: string) =>
      request<void>(`/profiles/${alias}/proxy/restart`, { method: 'POST' }),
    status: (alias?: string) =>
      request<ProxyInstanceInfo[]>(
        alias ? `/profiles/${alias}/proxy/status` : '/proxy/status'
      ),
    stopAll: () => request<void>('/proxy/stop-all', { method: 'POST' }),
    config: (alias: string) =>
      request<ProfileProxyConfig>(`/profiles/${alias}/proxy/config`),
    logs: (alias: string, lines?: number) =>
      request<string>(
        `/profiles/${alias}/proxy/logs${lines ? `?lines=${lines}` : ''}`
      ),
    routes: {
      list: (alias: string) =>
        request<RoutingRule[]>(`/profiles/${alias}/proxy/routes`),
      add: (alias: string, rule: RoutingRule) =>
        request<void>(`/profiles/${alias}/proxy/routes`, {
          method: 'POST',
          body: JSON.stringify(rule),
        }),
      remove: (alias: string, name: string) =>
        request<void>(`/profiles/${alias}/proxy/routes/${name}`, {
          method: 'DELETE',
        }),
    },
    aliases: {
      list: (alias: string) =>
        request<Record<string, string>>(`/profiles/${alias}/proxy/aliases`),
      set: (alias: string, from: string, to: string) =>
        request<void>(`/profiles/${alias}/proxy/aliases/${from}`, {
          method: 'PUT',
          body: JSON.stringify({ to }),
        }),
      remove: (alias: string, from: string) =>
        request<void>(`/profiles/${alias}/proxy/aliases/${from}`, {
          method: 'DELETE',
        }),
    },
  },

  // Registry
  registry: {
    status: () => request<RegistryStatus>('/registry'),
    sync: (force = false, offline = false) =>
      request<RegistryStatus>('/registry/sync', {
        method: 'POST',
        body: JSON.stringify({ force, offline }),
      }),
    pin: (ref: string) =>
      request<void>('/registry/pin', {
        method: 'POST',
        body: JSON.stringify({ ref }),
      }),
  },

  // Stats (legacy)
  stats: {
    get: (agentId?: string, providerId?: string) => {
      const params = new URLSearchParams()
      if (agentId) params.set('agent', agentId)
      if (providerId) params.set('provider', providerId)
      const query = params.toString()
      return request<StatsResponse>(`/stats${query ? `?${query}` : ''}`)
    },
  },

  // Usage (token/cost tracking)
  usage: {
    get: (options?: { period?: string; profile?: string; model?: string }) => {
      const params = new URLSearchParams()
      if (options?.period) params.set('period', options.period)
      if (options?.profile) params.set('profile', options.profile)
      if (options?.model) params.set('model', options.model)
      const query = params.toString()
      return request<UsageStatsResponse>(`/usage${query ? `?${query}` : ''}`)
    },
    importClaude: (claudeDir?: string) => {
      const params = new URLSearchParams()
      if (claudeDir) params.set('claude_dir', claudeDir)
      const query = params.toString()
      return request<string>(`/usage/import-claude${query ? `?${query}` : ''}`, {
        method: 'POST',
      })
    },
  },

  // System
  system: {
    ping: () => request<{ status: string; version: string }>('/ping'),
    shutdown: () => request<void>('/shutdown', { method: 'POST' }),
  },
}

export { ApiError }
