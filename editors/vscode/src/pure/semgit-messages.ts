// The messages between the workbench page and the host for semantic git.
export type SemgitFromWorkbench =
  | { type: 'semgit-compare'; op: 'with' | 'baseline' | 'exit'; ref?: string }
  | { type: 'semgit-refresh' }
  | { type: 'semgit-open-diff'; id?: string }
  | { type: 'semgit-history'; id: string }
  | { type: 'semgit-history-show'; hash: string }
  | { type: 'semgit-history-diff'; hash: string }
  | { type: 'semgit-baseline'; op: 'create' | 'compare' }
  | { type: 'semgit-merge-install' };
