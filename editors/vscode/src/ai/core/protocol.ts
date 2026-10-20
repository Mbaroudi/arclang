// The messages between the workbench page and the extension host for the
// AI features. Pure types: the page and the host both import them.

export type QuickAction = 'explain' | 'fix' | 'complete' | 'refine' | 'requirements';

export const QUICK_ACTIONS: { id: QuickAction; label: string; prompt: string }[] = [
  { id: 'explain', label: 'Explain', prompt: 'Explain the selection: its role in the architecture, what it exchanges with, and what the findings say about it. Propose no change.' },
  { id: 'fix', label: 'Fix the findings', prompt: 'Fix the findings and the gate blockers on the selection, minimally: keep ids, names and everything that is not concerned.' },
  { id: 'complete', label: 'Complete', prompt: 'Complete the selection: add the missing ports, exchanges, allocations and traces that the Arcadia method expects around it, and nothing else.' },
  { id: 'refine', label: 'Refine', prompt: 'Refine the selection into sub-components or sub-functions (two to four), with the ports and exchanges between them and the realizations or allocations that keep the model consistent.' },
  { id: 'requirements', label: 'Check against requirements', prompt: 'Check the selection against the requirements it satisfies or should satisfy: say which are met, which are not, and propose the traces or attribute changes that close the gaps.' },
];

/** A change shown before it is applied: what the AI said, the edits, the diff of the source. */
export interface ProposalView {
  explanation: string;
  /** The edits in words, one line each. */
  edits: string[];
  /** A unified diff of the model source (lines starting with ' ', '+', '-' or '@@'). */
  diff: string[];
  added: number;
  removed: number;
  /** Drawing ids to mark: changed, created (listed, not drawn yet), removed. */
  changed: string[];
  created: string[];
  removedIds: string[];
  /** The compiler's findings on the proposed model that the current one does not have. */
  newFindings: string[];
  /** Whether the compiler accepted the edits; false: `error` says why. */
  ok: boolean;
  error?: string;
  /** Nothing to apply: the answer only explains. */
  explainOnly: boolean;
}

export interface SuggestionView {
  id: string;
  title: string;
  rationale: string;
  resolves: string;
  proposal: ProposalView;
}

export type SuggestionsState = 'idle' | 'analysing' | 'thinking' | 'ready' | 'none' | 'error' | 'off';

/** Page → host. */
export type FromPageAi =
  | { type: 'ai-hello' }
  | { type: 'ai-ask'; session: string; ids: string[]; prompt: string; action?: QuickAction }
  | { type: 'ai-refine'; session: string; prompt: string }
  | { type: 'ai-apply'; session: string }
  | { type: 'ai-reject'; session: string }
  | { type: 'ai-diff'; session: string }
  | { type: 'ai-cancel'; session: string }
  | { type: 'ai-impact'; ids: string[] }
  | { type: 'ai-suggestion'; id: string; action: 'apply' | 'dismiss' | 'diff' }
  | { type: 'ai-propose' }
  | { type: 'ai-configure' };

/** Host → page. */
export type ToPageAi =
  | { type: 'ai-status'; configured: boolean; label: string; suggestOnEdit: string; problem?: string }
  | { type: 'ai-progress'; session: string; text: string; chars: number }
  | { type: 'ai-proposal'; session: string; provider: string; proposal: ProposalView }
  | { type: 'ai-error'; session: string; error: string }
  | { type: 'ai-applied'; session: string; ok: boolean; error?: string }
  | {
      type: 'ai-suggestions';
      state: SuggestionsState;
      trigger: string;
      consequences: string[];
      suggestions: SuggestionView[];
      provider: string;
      error?: string;
      /** What changed, in words. */
      change?: string[];
      /** `arclang impact` on the changed or selected elements, a line each. */
      impact?: string[];
      /** Consequences shown, the AI not asked yet (suggestOnEdit: onRequest). */
      canPropose?: boolean;
      /** Proposals the compiler refused, with the reason. */
      dropped?: string[];
    };

export function isFromPageAi(message: unknown): message is FromPageAi {
  return !!message && typeof message === 'object' && typeof (message as { type?: unknown }).type === 'string' && (message as { type: string }).type.startsWith('ai-');
}
