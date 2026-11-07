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

export type SuggestionsState = 'idle' | 'analyzing' | 'thinking' | 'ready' | 'none' | 'error' | 'off';

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

/** Whether a message of the page is meant for the AI side (its type starts with `ai-`). */
export function isAiMessage(message: unknown): boolean {
  return !!message && typeof message === 'object' && typeof (message as { type?: unknown }).type === 'string' && (message as { type: string }).type.startsWith('ai-');
}

/** Bounds of what a page may send: a page is not trusted with the host's memory. */
const MAX_ID = 300;
const MAX_IDS = 1000;
const MAX_PROMPT = 20000;

const isText = (value: unknown, max: number): value is string => typeof value === 'string' && value.length <= max;
const isSession = (value: unknown): value is string => typeof value === 'string' && value.length > 0 && value.length <= MAX_ID;
const isIds = (value: unknown): value is string[] => Array.isArray(value) && value.length <= MAX_IDS && value.every((id) => isText(id, MAX_ID));
const ACTIONS = new Set<string>(QUICK_ACTIONS.map((a) => a.id));
const SUGGESTION_ACTIONS = new Set(['apply', 'dismiss', 'diff']);

/**
 * A message of the page as the host may act on it, every field checked
 * (types, arrays, sizes, the known values), copied without anything else;
 * null when it is not a valid AI message.
 */
export function parseFromPageAi(message: unknown): FromPageAi | null {
  if (!isAiMessage(message)) return null;
  const m = message as Record<string, unknown>;
  switch (m.type) {
    case 'ai-hello':
    case 'ai-propose':
    case 'ai-configure':
      return { type: m.type };
    case 'ai-ask':
      if (!isSession(m.session) || !isIds(m.ids) || !isText(m.prompt, MAX_PROMPT)) return null;
      if (m.action !== undefined && !(typeof m.action === 'string' && ACTIONS.has(m.action))) return null;
      return { type: 'ai-ask', session: m.session, ids: [...m.ids], prompt: m.prompt, ...(m.action !== undefined ? { action: m.action as QuickAction } : {}) };
    case 'ai-refine':
      return isSession(m.session) && isText(m.prompt, MAX_PROMPT) ? { type: 'ai-refine', session: m.session, prompt: m.prompt } : null;
    case 'ai-apply':
    case 'ai-reject':
    case 'ai-diff':
    case 'ai-cancel':
      return isSession(m.session) ? { type: m.type, session: m.session } : null;
    case 'ai-impact':
      return isIds(m.ids) ? { type: 'ai-impact', ids: [...m.ids] } : null;
    case 'ai-suggestion':
      return isSession(m.id) && typeof m.action === 'string' && SUGGESTION_ACTIONS.has(m.action) ? { type: 'ai-suggestion', id: m.id, action: m.action as 'apply' | 'dismiss' | 'diff' } : null;
    default:
      return null;
  }
}

/** Whether the message is a valid AI message of the page. */
export function isFromPageAi(message: unknown): message is FromPageAi {
  return parseFromPageAi(message) !== null;
}
