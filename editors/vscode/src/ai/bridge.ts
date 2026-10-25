// The AI side of the workbench page: its messages (`ai-*`) handled here,
// and a way to reach every page that shows a document. The panel hands
// over the messages it does not know (one hook in panel.ts).
import * as vscode from 'vscode';
import { applySession, diffSession, dropSession, dropSessionsOf, runAsk } from './assist';
import { showOpenFailure } from './client';
import type { FromPageAi, ToPageAi } from './core/protocol';
import { isAiMessage, parseFromPageAi } from './core/protocol';
import { describeConfig, missingPiece } from './providers';
import { currentConfig, logLine, suggestMode } from './settings';
import type { SuggestionEngine } from './suggest';

type Post = (message: ToPageAi) => void;

/** What the page's host lends the AI: a change of the text that is one step of the canvas's history. */
export interface PageHost {
  asStep<T>(label: string, change: () => Promise<T>): Promise<T>;
}

const direct: PageHost = { asStep: (_label, change) => change() };

const pages = new Map<string, Set<Post>>();
/** The ask sessions each page started: dropped with the page. */
const pageSessions = new Map<Post, Set<string>>();

/** A page went away (its panel closed): its sessions and its place in the pages go with it. */
export function pageClosed(document: vscode.TextDocument, post: Post): void {
  for (const id of pageSessions.get(post) ?? []) dropSession(id);
  pageSessions.delete(post);
  const key = document.uri.toString();
  const set = pages.get(key);
  set?.delete(post);
  if (set && set.size === 0) pages.delete(key);
}

/** A document closed: every session on it, and every page left on it, go. */
export function documentClosed(document: vscode.TextDocument): void {
  for (const post of [...(pages.get(document.uri.toString()) ?? [])]) pageClosed(document, post);
  pages.delete(document.uri.toString());
  dropSessionsOf(document.uri);
}

/** How many pages and sessions are kept (for the tests). */
export function bridgeSizes(): { pages: number; sessions: number } {
  return { pages: [...pages.values()].reduce((n, s) => n + s.size, 0), sessions: [...pageSessions.values()].reduce((n, s) => n + s.size, 0) };
}

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
  context.subscriptions.push(vscode.workspace.onDidCloseTextDocument(documentClosed));
}

/**
 * A message of the page; true when it was an AI message. `post` answers the
 * page that sent it. Hook: `WorkbenchPanel.handle` calls this for the
 * messages it does not handle.
 */
export function handlePageMessage(document: vscode.TextDocument, post: Post, message: unknown, host: PageHost = direct): boolean {
  if (!isAiMessage(message)) return false;
  const valid = parseFromPageAi(message);
  if (!valid) {
    logLine(`a page message was dropped: ${String((message as { type: string }).type).slice(0, 40)} with fields of the wrong type`);
    return true;
  }
  handle(document, post, valid, host).catch((error: unknown) => {
    const text = error instanceof Error ? error.message : String(error);
    logLine(`${valid.type} failed: ${text}`);
    if ('session' in valid) {
      try {
        post({ type: 'ai-error', session: valid.session, error: text });
      } catch {
        // the page went away
      }
    }
  });
  return true;
}

async function handle(document: vscode.TextDocument, post: Post, message: FromPageAi, host: PageHost): Promise<void> {
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
      pageSessions.set(post, new Set([...(pageSessions.get(post) ?? []), message.session]));
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
      // One step of the canvas's history: ⌘Z on the workbench takes the whole change back.
      const result = await host.asStep('the AI proposal', () => applySession(message.session));
      post({ type: 'ai-applied', session: message.session, ok: result.ok, error: result.error });
      return;
    }
    case 'ai-diff':
      await diffSession(message.session);
      return;
    case 'ai-reject':
    case 'ai-cancel':
      dropSession(message.session);
      pageSessions.get(post)?.delete(message.session);
      return;
    case 'ai-impact':
      await engine?.analyseSelection(document, message.ids);
      return;
    case 'ai-suggestion':
      if (message.action === 'apply') await host.asStep('the AI suggestion', async () => engine?.act(document, message.id, message.action));
      else await engine?.act(document, message.id, message.action);
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
