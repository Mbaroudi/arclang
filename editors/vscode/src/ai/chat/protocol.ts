// The messages between the chat page and the extension host. Pure types and
// the checks of what a page may send: a page is not trusted with the host's
// memory (sizes are bounded, only known fields are copied).
import type { ProposalView } from '../core/protocol';

export type ChatKind = 'arc' | 'modelica' | 'plain';

export interface ChatContextView {
  /** The file the chat sees, or null. */
  file: string | null;
  kind: ChatKind;
  /** Lines of the selection, 0 when there is none. */
  selectionLines: number;
  /** Elements of an ArcLang selection. */
  ids: string[];
}

export interface ChatStateView {
  provider: string;
  /** The provider's id, for the key button. */
  providerId: string;
  /** Why nothing can be sent yet; null when ready. */
  problem: string | null;
  context: ChatContextView;
  /** Running in a browser: no Copilot, keys go to the browser's secret storage. */
  web: boolean;
}

/** Page → host. */
export type FromChatPage =
  | { type: 'chat-hello' }
  | { type: 'chat-send'; text: string }
  | { type: 'chat-cancel' }
  | { type: 'chat-new' }
  | { type: 'chat-configure' }
  | { type: 'chat-setkey' }
  | { type: 'chat-apply'; session: string }
  | { type: 'chat-diff'; session: string }
  | { type: 'chat-reject'; session: string }
  | { type: 'chat-refine'; session: string; text: string }
  | { type: 'chat-undo' }
  | { type: 'chat-insert'; code: string; where: 'selection' | 'cursor' | 'file' }
  | { type: 'chat-copy'; code: string };

/** Host → page. */
export type ToChatPage =
  | ({ type: 'chat-state' } & ChatStateView)
  | { type: 'chat-start'; id: string; kind: ChatKind; file: string | null; label: string }
  | { type: 'chat-delta'; id: string; text: string }
  | { type: 'chat-proposal'; id: string; session: string; provider: string; proposal: ProposalView }
  | { type: 'chat-done'; id: string; kind: ChatKind; text: string; notVerified?: { label: string; where: string } }
  | { type: 'chat-error'; id: string; error: string; fix?: 'key' | 'configure' }
  | { type: 'chat-applied'; session: string; ok: boolean; error?: string }
  | { type: 'chat-note'; text: string };

const MAX_TEXT = 40000;
const MAX_SESSION = 300;
const isText = (v: unknown, max = MAX_TEXT): v is string => typeof v === 'string' && v.length <= max;
const isSession = (v: unknown): v is string => typeof v === 'string' && v.length > 0 && v.length <= MAX_SESSION;

/** A message of the page as the host may act on it, or null. */
export function parseFromChatPage(message: unknown): FromChatPage | null {
  if (!message || typeof message !== 'object') return null;
  const m = message as Record<string, unknown>;
  switch (m.type) {
    case 'chat-hello':
    case 'chat-cancel':
    case 'chat-new':
    case 'chat-configure':
    case 'chat-setkey':
    case 'chat-undo':
      return { type: m.type };
    case 'chat-send':
      return isText(m.text) && m.text.trim().length > 0 ? { type: 'chat-send', text: m.text } : null;
    case 'chat-apply':
    case 'chat-diff':
    case 'chat-reject':
      return isSession(m.session) ? { type: m.type, session: m.session } : null;
    case 'chat-refine':
      return isSession(m.session) && isText(m.text) && m.text.trim().length > 0 ? { type: 'chat-refine', session: m.session, text: m.text } : null;
    case 'chat-insert':
      return isText(m.code) && (m.where === 'selection' || m.where === 'cursor' || m.where === 'file') ? { type: 'chat-insert', code: m.code, where: m.where } : null;
    case 'chat-copy':
      return isText(m.code) ? { type: 'chat-copy', code: m.code } : null;
    default:
      return null;
  }
}
