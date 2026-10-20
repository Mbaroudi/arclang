// The AI side of the workbench page: its messages (`ai-*`) handled here,
// and a way to reach every page that shows a document. The panel hands
// over the messages it does not know (one hook in panel.ts).
import * as vscode from 'vscode';
import { applySession, diffSession, dropSession, runAsk } from './assist';
import { showOpenFailure } from './client';
import type { FromPageAi, ToPageAi } from './core/protocol';
import { isFromPageAi } from './core/protocol';
import { describeConfig, missingPiece } from './providers';
import { currentConfig, suggestMode } from './settings';
import type { SuggestionEngine } from './suggest';

type Post = (message: ToPageAi) => void;

const pages = new Map<string, Set<Post>>();

/** Every page showing the document (a page says hello when its AI side starts). */
export function postTo(uri: vscode.Uri, message: ToPageAi): void {
  for (const post of [...(pages.get(uri.toString()) ?? [])]) {
    try {
      post(message);
    } catch {
      pages.get(uri.toString())?.delete(post); // a page that went away
    }
  }
}

export function hasPage(uri: vscode.Uri): boolean {
  return (pages.get(uri.toString())?.size ?? 0) > 0;
}

export async function statusMessage(context: vscode.ExtensionContext): Promise<ToPageAi> {
  const config = await currentConfig(context);
  const problem = missingPiece(config);
  return { type: 'ai-status', configured: problem === null, label: describeConfig(config), suggestOnEdit: suggestMode(), problem: problem ?? undefined };
}

export async function broadcastStatus(context: vscode.ExtensionContext): Promise<void> {
  const status = await statusMessage(context);
  for (const key of pages.keys()) postTo(vscode.Uri.parse(key), status);
}

let engine: SuggestionEngine | null = null;
let extension: vscode.ExtensionContext | null = null;

export function connectBridge(context: vscode.ExtensionContext, suggestions: SuggestionEngine): void {
  extension = context;
  engine = suggestions;
}

/**
 * A message of the page; true when it was an AI message. `post` answers the
 * page that sent it. Hook: `WorkbenchPanel.handle` calls this for the
 * messages it does not handle.
 */
export function handlePageMessage(document: vscode.TextDocument, post: Post, message: unknown): boolean {
  if (!isFromPageAi(message)) return false;
  void handle(document, post, message);
  return true;
}

async function handle(document: vscode.TextDocument, post: Post, message: FromPageAi): Promise<void> {
  const context = extension;
  if (!context) return;
  switch (message.type) {
    case 'ai-hello': {
      const key = document.uri.toString();
      pages.set(key, new Set([...(pages.get(key) ?? []), post]));
      post(await statusMessage(context));
      engine?.replay(document, post);
      return;
    }
    case 'ai-ask':
    case 'ai-refine': {
      const request = message.type === 'ai-ask' ? { document, ids: message.ids, prompt: message.prompt, action: message.action } : { refine: message.prompt };
      const outcome = await runAsk(message.session, request, context, (text, chars) => post({ type: 'ai-progress', session: message.session, text, chars }));
      if (!outcome.ok) {
        post({ type: 'ai-error', session: message.session, error: outcome.error });
        if (outcome.fix) void showOpenFailure({ ok: false, error: outcome.error, fix: outcome.fix });
        return;
      }
      if (outcome.session.proposal) post({ type: 'ai-proposal', session: message.session, provider: outcome.session.provider, proposal: outcome.session.proposal.view });
      return;
    }
    case 'ai-apply': {
      const result = await applySession(message.session);
      post({ type: 'ai-applied', session: message.session, ok: result.ok, error: result.error });
      return;
    }
    case 'ai-diff':
      await diffSession(message.session);
      return;
    case 'ai-reject':
    case 'ai-cancel':
      dropSession(message.session);
      return;
    case 'ai-impact':
      await engine?.analyseSelection(document, message.ids);
      return;
    case 'ai-suggestion':
      await engine?.act(document, message.id, message.action);
      return;
    case 'ai-propose':
      await engine?.propose(document);
      return;
    case 'ai-configure':
      await vscode.commands.executeCommand('arclang.ai.configure');
      return;
    default:
      return;
  }
}
