// API Response wrapper
export interface ApiResponse<T> {
  success: boolean
  data?: T
  error?: {
    code: number
    message: string
  }
}

// Agent types
export interface AgentInfo {
  id: string
  name: string
  installed: boolean
  version: string | null
  binary_path: string | null
  profile_count: number
  default_model: string | null
  default_provider: string | null
  supports_hooks: boolean
  last_used: string | null
}

// Provider types
export interface ProviderInfo {
  id: string
  name: string
  provider_type: string
  default_model: string | null
  endpoints: EndpointInfo[]
  default_endpoint: string
  auth_required: boolean
  auth_prompt: string
}

export interface EndpointInfo {
  id: string
  url: string
  is_default: boolean
}

// Profile types
export interface ProfileInfo {
  alias: string
  agent_id: string
  provider_id: string
  endpoint_id: string
  model: string
  last_used: string | null
  total_runs: number
}

export interface ProfileCreateRequest {
  agent_id: string
  alias: string
  provider_id: string
  endpoint_id?: string
  model?: string
  api_key: string
  hooks: string[]
  mcp_servers: string[]
  args: string[]
  working_dir?: string
  bare: boolean
  proxy: boolean
}

// Hooks types
export interface HooksConfig {
  PreToolUse: HookRule[]
  PostToolUse: HookRule[]
  Notification: HookRule[]
  Stop: HookRule[]
}

export interface HookRule {
  matcher: string
  hooks: HookAction[]
}

export interface HookAction {
  type: 'command' | 'url'
  command?: string
  url?: string
  timeout?: number
}

// Proxy types
export interface ProxyInstanceInfo {
  alias: string
  port: number
  pid: number
  status: ProxyStatus
  started_at: string
  restart_count: number
}

export type ProxyStatus =
  | { status: 'starting' }
  | { status: 'running' }
  | { status: 'unhealthy'; since: string; reason: string }
  | { status: 'stopping' }
  | { status: 'stopped' }
  | { status: 'failed'; reason: string }

export interface ProfileProxyConfig {
  enabled: boolean
  port: number | null
  routing: RoutingConfig
  model_aliases: Record<string, ModelTarget>
}

export interface RoutingConfig {
  strategy: string
  rules: RoutingRule[]
}

export interface RoutingRule {
  name: string
  condition: RoutingCondition
  target: string
  priority: number
  weight?: number
}

export type RoutingCondition =
  | { type: 'always' }
  | { type: 'token_count'; min?: number; max?: number }
  | { type: 'has_tools'; min_count?: number }
  | { type: 'thinking_mode' }
  | { type: 'model_pattern'; pattern: string }

export interface ModelTarget {
  provider: string
  model: string
  api_base?: string
}

// Stats types
export interface StatsResponse {
  by_agent: Record<string, AgentStats>
  by_provider: Record<string, ProviderStats>
  by_profile: Record<string, ProfileStats>
  total_sessions: number
  total_runtime_secs: number
}

export interface AgentStats {
  sessions: number
  runtime_secs: number
  profiles: number
}

export interface ProviderStats {
  sessions: number
  runtime_secs: number
}

export interface ProfileStats {
  sessions: number
  runtime_secs: number
  last_used: string | null
}

// Usage types (token/cost tracking)
export type AgentType = 'claude' | 'codex' | 'opencode'

export interface TokenUsage {
  input_tokens: number
  output_tokens: number
  cache_creation_input_tokens: number
  cache_read_input_tokens: number
}

export interface CostBreakdown {
  input_cost: number
  output_cost: number
  cache_creation_cost: number
  cache_read_cost: number
  total_cost: number
}

export interface UsageStatsResponse {
  period: string
  total_tokens: TokenUsage
  total_cost: CostBreakdown | null
  total_sessions: number
  total_runtime_secs: number
  aggregates: UsageAggregates
}

export interface UsageAggregates {
  total_tokens: TokenUsage
  total_cost: CostBreakdown | null
  by_date: Record<string, DailyUsage>
  by_model: Record<string, ModelUsage>
  by_profile: Record<string, ProfileUsage>
  by_agent?: Record<AgentType, AgentUsage>
}

export interface AgentUsage {
  agent: AgentType
  tokens: TokenUsage
  cost: CostBreakdown | null
  sessions: number
}

export interface DailyUsage {
  date: string
  tokens: TokenUsage
  cost: CostBreakdown | null
  sessions: number
}

export interface ModelUsage {
  model: string
  tokens: TokenUsage
  cost: CostBreakdown | null
  sessions: number
}

export interface ProfileUsage {
  profile: string
  provider_id: string
  tokens: TokenUsage
  cost: CostBreakdown | null
  sessions: number
  runtime_secs: number
  last_used: string | null
}

// Registry types
export interface RegistryStatus {
  commit: string | null
  channel: string
  last_sync: string | null
  offline: boolean
  cached_agents: number
  cached_providers: number
  cached_scripts: number
}

// WebSocket event types
export type Event =
  | { type: 'connected'; data: { version: string; timestamp: string } }
  | { type: 'heartbeat'; data: { timestamp: number } }
  | { type: 'profile_created'; data: { alias: string } }
  | { type: 'profile_deleted'; data: { alias: string } }
  | { type: 'profile_run_started'; data: { alias: string; pid: number } }
  | { type: 'profile_run_completed'; data: { alias: string; exit_code: number } }
  | { type: 'proxy_started'; data: { alias: string; port: number } }
  | { type: 'proxy_stopped'; data: { alias: string } }
  | { type: 'proxy_status_changed'; data: { alias: string; status: ProxyStatus } }
  | { type: 'registry_sync_started'; data: Record<string, never> }
  | { type: 'registry_sync_completed'; data: { commit: string | null } }
  | { type: 'usage_updated'; data: { agent: AgentType; profile: string | null; tokens: TokenUsage; cost: CostBreakdown | null } }

export interface ServerMessage {
  type: 'event' | 'pong' | 'error'
  event?: Event
  error?: string
}

export interface ClientMessage {
  type: 'subscribe' | 'unsubscribe' | 'ping'
  topics?: string[]
}

// Terminal session types
export type TerminalSessionState = 'starting' | 'running' | 'terminated'

export interface TerminalSessionInfo {
  id: string
  profile_alias: string
  state: TerminalSessionState | { terminated: { exit_code: number | null } }
  created_at: string
  pid: number | null
  cols: number
  rows: number
  client_count: number
}

export interface CreateTerminalSessionRequest {
  profile_alias: string
  args?: string[]
  cols?: number
  rows?: number
  working_dir?: string
  no_sandbox?: boolean
  bwrap_flags?: string[]
  sandbox_exec_profile?: string
}

export interface CreateTerminalSessionResponse {
  session_id: string
  ws_url: string
}

// Terminal WebSocket message types
export type TerminalClientMessage =
  | { type: 'resize'; cols: number; rows: number }
  | { type: 'signal'; signal: number }

export type TerminalServerMessage =
  | { type: 'connected'; session_id: string }
  | { type: 'state_changed'; state: string; exit_code: number | null }
  | { type: 'resized'; cols: number; rows: number }
  | { type: 'error'; message: string }

// Shell session types
export interface CreateShellRequest {
  shell?: string
  cols?: number
  rows?: number
  working_dir?: string
  no_sandbox?: boolean
  bwrap_flags?: string[]
  sandbox_exec_profile?: string
}

// Filesystem types
export interface DirEntry {
  name: string
  path: string
  is_dir: boolean
}

export interface ListDirResponse {
  path: string
  parent: string | null
  entries: DirEntry[]
}
