export type * from './generated'

// Workspace types
export interface RecentWorkspace {
  path: string
  last_opened: string
  open_count: number
}

export interface WorkspaceBookmark {
  path: string
  name: string
  pinned: boolean
  added_at: string
}
