// "Ask AI" on a selection, from the diagram or the text: the selection's
// context goes to the configured provider, the answer comes back as edit
// requests, the compiler checks them on a copy (sending its refusal back to
// the AI a few times), and the user sees a preview: the explanation, the
// edits, the diff of the source, the elements marked on the diagram. Only
// Apply changes the model, as one undoable edit.
import * as vscode from 'vscode';
import type { EditRequest, Snapshot } from '../pure/workbench';
import { applyRequests } from '../workbench/edits';
import { metamodelJson, snapshotOf } from '../workbench/snapshot';
import { documentTextFor, replaceWhole } from '../workbench/source';
import { openProvider } from './client';
import { parseAssistAnswer } from './core/answer';
import { buildSelectionContext, ContextScope } from './core/context';
import { converse } from './core/conversation';
import { lineDiff } from './core/diff';
import { consequencesOf, diffSnapshots } from './core/impact';
import { askPrompt, IMPACT, PROPOSE_EDITS, READ_ELEMENTS, SYSTEM_RULES } from './core/prompts';
import type { ProposalView, QuickAction } from './core/protocol';
import { QUICK_ACTIONS } from './core/protocol';
import { describeEdit } from './core/requests';
import { arcText, checkRequests, checkSource, impactOf, snapshotOfText } from './compiler';
import { forgetPreview, showDiff } from './preview';
import type { ChatMessage, ToolCall } from './providers';
import { contextScope, logLine, readSettings, repairAttempts } from './settings';

export interface AskRequest {
  document: vscode.TextDocument;
  ids: string[];
  prompt: string;
  action?: QuickAction;
  selectedText?: string;
  scope?: ContextScope;
}

export interface Proposal {
  view: ProposalView;
  requests: EditRequest[];
  /** A whole model text proposed instead of requests. */
  source: string | null;
  /** The proposed ArcLang text (for the diff), null when nothing changes. */
  proposedText: string | null;
}

export interface AskSession {
  id: string;
  document: vscode.TextDocument;
  messages: ChatMessage[];
  ids: string[];
  provider: string;
  proposal: Proposal | null;
  controller: AbortController;
}

const sessions = new Map<string, AskSession>();

export function session(id: string): AskSession | undefined {
  return sessions.get(id);
}

export function cancelSession(id: string): void {
  sessions.get(id)?.controller.abort();
}

export function dropSession(id: string): void {
  cancelSession(id);
  sessions.delete(id);
  forgetPreview(id);
}

/** The prompt of a quick action, the user's words after it. */
export function promptFor(prompt: string, action?: QuickAction): string {
  const quick = action ? QUICK_ACTIONS.find((a) => a.id === action)?.prompt : undefined;
  return [quick, prompt.trim()].filter(Boolean).join('\n\n') || 'Explain the selection.';
}

/** The read-only tools of an ask: more of the model, and the compiler's impact analysis. */
function toolRunner(document: vscode.TextDocument, text: string, snapshot: Snapshot, metamodel: Awaited<ReturnType<typeof metamodelJson>>) {
  return async (call: ToolCall): Promise<string> => {
    if (call.name === 'read_elements') {
      const ids = Array.isArray(call.input.ids) ? call.input.ids.map(String).slice(0, 12) : [];
      return buildSelectionContext({ snapshot, metamodel, text, fileName: 'model', ids, scope: 'selection', maxChars: 12000 }).text;
    }
    if (call.name === 'impact') return impactOf(document, text, String(call.input.element ?? ''));
    return `There is no tool named ${call.name}.`;
  };
}

/** The preview of a proposal: the diff, the marked elements, the findings it adds. */
export async function proposalView(document: vscode.TextDocument, before: Snapshot, current: string, explanation: string, requests: EditRequest[], proposed: string | null): Promise<Omit<Proposal, 'requests' | 'source'>> {
  if (proposed === null || proposed === current) {
    return { view: { explanation, edits: requests.map(describeEdit), diff: [], added: 0, removed: 0, changed: [], created: [], removedIds: [], newFindings: [], ok: true, explainOnly: requests.length === 0 }, proposedText: null };
  }
  const diff = lineDiff(current, proposed);
  const after = await snapshotOfText(document, proposed);
  const change = diffSnapshots(before, after);
  const findings = consequencesOf(before, after, change).filter((c) => c.kind === 'new-finding' || c.kind === 'gate').map((c) => c.message);
  return {
    view: {
      explanation,
      edits: requests.map(describeEdit),
      diff: diff.lines.slice(0, 400),
      added: diff.added,
      removed: diff.removed,
      changed: change.changed.map((e) => e.id),
      created: change.created.map((e) => `${e.kind} ${e.id}`),
      removedIds: change.removed.map((e) => e.id),
      newFindings: findings,
      ok: true,
      explainOnly: false,
    },
    proposedText: proposed,
  };
}

/**
 * One round of an ask (the first question or a refinement): the AI answers,
 * the compiler checks, the AI repairs what was refused. The session keeps
 * the conversation for the next refinement.
 */
export async function runAsk(
  sessionId: string,
  request: AskRequest | { refine: string },
  context: vscode.ExtensionContext,
  onText: (text: string, chars: number) => void,
): Promise<{ ok: true; session: AskSession } | { ok: false; error: string; fix?: 'key' | 'configure' }> {
  let current = sessions.get(sessionId);
  if ('refine' in request && !current) return { ok: false, error: 'this question is no longer open: ask again' };
  const document = 'refine' in request ? current!.document : request.document;
  const source = await arcText(document);
  if (!source.ok) return { ok: false, error: source.error };
  const [snapshot, metamodel] = await Promise.all([snapshotOf(document), metamodelJson()]);
  if (!snapshot.elements) return { ok: false, error: `the model does not compile: ${snapshot.errors.slice(0, 2).join('; ') || 'no snapshot'}` };
  const settings = readSettings();
  let messages: ChatMessage[];
  let ids: string[];
  if ('refine' in request) {
    ids = current!.ids;
    const last = current!.proposal;
    messages = [...current!.messages, { role: 'user', content: `${last ? `The proposal is not applied yet. ` : ''}Refine it: ${request.refine}\n\nAnswer again with the complete set of requests (they replace the previous ones), with propose_edits.` }];
  } else {
    ids = request.ids;
    const built = buildSelectionContext({ snapshot, metamodel, text: source.text, fileName: document.uri.path.split('/').pop() ?? 'model.arc', ids, scope: request.scope ?? contextScope(), selectedText: request.selectedText });
    messages = [
      { role: 'system', content: SYSTEM_RULES },
      { role: 'user', content: askPrompt(promptFor(request.prompt, request.action), built.text) },
    ];
    current?.controller.abort();
  }
  const sentChars = messages.reduce((n, m) => n + m.content.length, 0);
  const opened = await openProvider(context, `ask about ${ids.slice(0, 6).join(', ') || 'the selection'}`, `The source of the selected elements (${ids.slice(0, 8).join(', ') || 'text selection'}), their relationships, findings, the metamodel of their kinds and the gate state: about ${sentChars} characters.`);
  if (!opened.ok) return opened;
  const controller = new AbortController();
  const live: AskSession = { id: sessionId, document, messages, ids, provider: opened.label, proposal: null, controller };
  sessions.set(sessionId, live);
  logLine(`ask ${sessionId} on ${document.uri.path.split('/').pop()}: ${ids.join(', ') || '(text)'}; ${sentChars} characters of context`);
  let chars = 0;
  const run = toolRunner(document, source.text, snapshot, metamodel);
  try {
    for (let attempt = 0; ; attempt++) {
      const result = await converse(opened.provider, messages, {
        tools: [PROPOSE_EDITS, READ_ELEMENTS, IMPACT],
        final: ['propose_edits'],
        run,
        signal: controller.signal,
        maxTokens: settings.maxTokens,
        onText: (text) => {
          chars += text.length;
          onText(text, chars);
        },
      });
      const answer = parseAssistAnswer(result.text, result.calls);
      messages.push({ role: 'assistant', content: result.text, toolCalls: result.calls.length ? result.calls : undefined });
      for (const call of result.calls) messages.push({ role: 'tool', content: 'Received: it is being compiled.', toolCallId: call.id, toolName: call.name });
      let refusal: string | null = answer.problem;
      let proposed: string | null = null;
      if (!refusal && answer.requests.length) {
        const checked = await checkRequests(document, source.text, answer.requests);
        if (checked.ok) proposed = checked.text;
        else refusal = `request ${checked.at + 1} (${describeEdit(answer.requests[checked.at])}) was refused by the compiler: ${checked.error}`;
      } else if (!refusal && answer.source) {
        const checked = await checkSource(document, answer.source);
        if (checked.ok) proposed = checked.text;
        else refusal = `the model text does not compile: ${checked.error}`;
      }
      if (refusal && attempt < repairAttempts()) {
        logLine(`ask ${sessionId}: ${refusal}; sent back to the AI`);
        messages.push({ role: 'user', content: `The arclang compiler refused the proposal: ${refusal}\nNothing was applied. Send the corrected, complete list of requests with propose_edits; change only what this error needs.` });
        onText(`\n(the compiler refused it: ${refusal.split('\n')[0]}; asking for a correction)\n`, chars);
        continue;
      }
      if (refusal) {
        live.proposal = { view: { explanation: answer.explanation, edits: answer.requests.map(describeEdit), diff: [], added: 0, removed: 0, changed: [], created: [], removedIds: [], newFindings: [], ok: false, error: refusal, explainOnly: false }, requests: [], source: null, proposedText: null };
        return { ok: true, session: live };
      }
      const view = await proposalView(document, snapshot, source.text, answer.explanation, answer.requests, proposed);
      live.proposal = { ...view, requests: answer.requests, source: answer.source && !answer.requests.length ? proposed : null };
      logLine(`ask ${sessionId}: ${answer.requests.length} request(s) accepted by the compiler on a copy; +${view.view.added} -${view.view.removed} lines; waiting for Apply`);
      return { ok: true, session: live };
    }
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    return { ok: false, error: message };
  }
}

/** Apply the session's proposal to the document: one undoable edit, checked again on the text as it is now. */
export async function applySession(sessionId: string): Promise<{ ok: boolean; error?: string }> {
  const live = sessions.get(sessionId);
  const proposal = live?.proposal;
  if (!live || !proposal || !proposal.view.ok) return { ok: false, error: 'nothing to apply' };
  if (proposal.requests.length) {
    const outcome = await applyRequests(live.document, proposal.requests);
    if (!outcome.ok) return { ok: false, error: outcome.error };
  } else if (proposal.source) {
    const back = await documentTextFor(live.document, proposal.source);
    if (!back.ok) return { ok: false, error: back.error };
    if (!(await replaceWhole(live.document, back.text))) return { ok: false, error: 'VS Code did not apply the edit' };
  } else {
    return { ok: false, error: 'the answer has no change to apply' };
  }
  logLine(`ask ${sessionId}: applied by the user`);
  live.proposal = null;
  forgetPreview(sessionId);
  return { ok: true };
}

export async function diffSession(sessionId: string): Promise<void> {
  const live = sessions.get(sessionId);
  if (!live?.proposal?.proposedText) return;
  const proposed = await documentTextFor(live.document, live.proposal.proposedText);
  await showDiff(sessionId, live.document, proposed.ok ? proposed.text : live.proposal.proposedText, `${live.document.uri.path.split('/').pop()} ↔ AI proposal (${live.provider})`);
}
