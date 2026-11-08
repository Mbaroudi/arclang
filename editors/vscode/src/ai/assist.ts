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
import { askPrompt, CREATE_MODEL, IMPACT, PROPOSE_EDITS, READ_ELEMENTS, systemRulesFor, WRITE_FILE, writablePath } from './core/prompts';
import { checkModelText } from './compiler';
import { sampleModel } from './context';
import { currentWorkspace } from './workspace';
import type { ProposalView, QuickAction } from './core/protocol';
import { QUICK_ACTIONS } from './core/protocol';
import { describeEdit, isQuestion } from './core/requests';
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
  /** The files attached in the chat, as one block of data (attachmentsBlock). */
  attachments?: string;
  /** The conversation so far (questions and answers in words), so a follow-up ("yes, a new one") keeps its meaning. */
  history?: ChatMessage[];
}

export interface Proposal {
  view: ProposalView;
  requests: EditRequest[];
  /** A whole model text proposed instead of requests. */
  source: string | null;
  /** The proposed ArcLang text (for the diff), null when nothing changes. */
  proposedText: string | null;
  /** The version of the document the proposal was computed on. */
  version?: number;
  /** Files to write next to the model with the edits (write_file), by path relative to its folder. */
  files?: { path: string; content: string }[];
}

/** A file written with a proposal, and what it held before (null: it did not exist), for Undo. */
export interface WrittenFile {
  uri: vscode.Uri;
  previous: Uint8Array | null;
}

export interface AskSession {
  id: string;
  document: vscode.TextDocument;
  messages: ChatMessage[];
  ids: string[];
  provider: string;
  proposal: Proposal | null;
  controller: AbortController;
  /** A new model for another system (create_model), compiled: for a new file, never the open one. */
  newModel?: { source: string; explanation: string; compiled: boolean };
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

/** Drop every session on a document (it closed). */
export function dropSessionsOf(uri: vscode.Uri): void {
  for (const [id, live] of [...sessions]) if (live.document.uri.toString() === uri.toString()) dropSession(id);
}

/** How many sessions are open (for the tests). */
export function sessionCount(): number {
  return sessions.size;
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

/** The id of an owner (by uuid) in either snapshot; none for the model itself. */
function ownerId(before: Snapshot, after: Snapshot, uuid: string): string | null {
  const owner = (before.elements?.elements ?? []).find((e) => e.uuid === uuid) ?? (after.elements?.elements ?? []).find((e) => e.uuid === uuid);
  return owner && owner.kind !== 'Model' ? owner.id : null;
}

/** The preview of a proposal: the diff, the marked elements, the findings it adds. */
export async function proposalView(document: vscode.TextDocument, before: Snapshot, current: string, explanation: string, requests: EditRequest[], proposed: string | null): Promise<Omit<Proposal, 'requests' | 'source'> & { newErrors: string[] }> {
  if (proposed === null || proposed === current) {
    return { view: { explanation, edits: requests.map(describeEdit), diff: [], added: 0, removed: 0, changed: [], created: [], removedIds: [], newFindings: [], ok: true, explainOnly: requests.length === 0 }, proposedText: null, newErrors: [] };
  }
  const diff = lineDiff(current, proposed);
  const after = await snapshotOfText(document, proposed);
  const change = diffSnapshots(before, after);
  const findings = consequencesOf(before, after, change).filter((c) => c.kind === 'new-finding' || c.kind === 'gate').map((c) => c.message);
  // The errors the change brings (a model that compiles can still be wrong: a trace the wrong way...): sent back to the AI.
  const key = (f: { rule: string; element?: string | null; message: string }) => `${f.rule}|${f.element ?? ''}|${f.message}`;
  const had = new Set(before.findings.map(key));
  const newErrors = after.findings.filter((f) => f.severity === 'error' && !had.has(key(f))).map((f) => `[${f.rule}]${f.element ? ` ${f.element}:` : ''} ${f.message}${f.fix ? ` (fix: ${f.fix})` : ''}`);
  return {
    view: {
      explanation,
      edits: requests.map(describeEdit),
      diff: diff.lines.slice(0, 400),
      added: diff.added,
      removed: diff.removed,
      // A box gains a port or loses one: the box is what changes on the drawing.
      changed: [...new Set([...change.changed, ...change.created, ...change.removed].map((e) => (e.owner ? ownerId(before, after, e.owner) : null)).filter((id): id is string => !!id).concat(change.changed.map((e) => e.id)))],
      created: change.created.map((e) => `${e.kind} ${e.id}`),
      removedIds: change.removed.map((e) => e.id),
      newFindings: findings,
      ok: true,
      explainOnly: false,
    },
    proposedText: proposed,
    newErrors,
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
  // The version the proposal is computed on: a whole text is applied only on it.
  const version = document.version;
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
      { role: 'system', content: `${systemRulesFor(await currentWorkspace())}\n\n# Reference model (compiled by the extension's tests): the syntax for a new model\n\`\`\`arc\n${await sampleModel(context)}\n\`\`\`` },
      ...(request.history ?? []),
      { role: 'user', content: askPrompt(promptFor(request.prompt, request.action), request.attachments ? `${built.text}\n\n${request.attachments}` : built.text) },
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
  const answerTools = toolRunner(document, source.text, snapshot, metamodel);
  // write_file keeps the file for the proposal: written only when the engineer keeps the change.
  const files = new Map<string, string>(('refine' in request ? current?.proposal?.files : undefined)?.map((f) => [f.path, f.content]) ?? []);
  const run = async (call: ToolCall): Promise<string> => {
    if (call.name !== 'write_file') return answerTools(call);
    const path = writablePath(call.input.path);
    const content = typeof call.input.content === 'string' ? call.input.content : '';
    if (!path) return 'Refused: the path must be relative to the model\'s folder, without "..", ending in .mo, .mos, .csv, .md, .txt or .json.';
    if (!content.trim() || content.length > 200_000) return 'Refused: the content is empty or larger than 200 000 characters.';
    files.set(path, content);
    return `Received: ${path} (${content.split('\n').length} lines) is written next to the model when the engineer keeps the change. Now send the .arc edits with propose_edits (the test case's simulate: names this file in sources:).`;
  };
  try {
    for (let attempt = 0; ; attempt++) {
      const result = await converse(opened.provider, messages, {
        tools: [WRITE_FILE, PROPOSE_EDITS, CREATE_MODEL, READ_ELEMENTS, IMPACT],
        final: ['propose_edits', 'create_model'],
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
      if (answer.newModel) {
        const checked = await checkModelText(answer.newModel);
        if (!checked.ok && attempt < repairAttempts()) {
          logLine(`ask ${sessionId}: the new model does not compile; sent back to the AI`);
          messages.push({ role: 'user', content: `The arclang compiler refuses the new model:\n${checked.error}\nCall create_model again with the corrected WHOLE model (declare every type you use).` });
          onText(`\n(the compiler finds errors in the new model: ${checked.error.split('\n')[0].slice(0, 140)}; correcting it)\n`, chars);
          continue;
        }
        live.newModel = { source: answer.newModel, explanation: answer.explanation, compiled: checked.ok };
        live.proposal = null;
        return { ok: true, session: live };
      }
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
      const { newErrors, ...view } = await proposalView(document, snapshot, source.text, answer.explanation, answer.requests, proposed);
      if (newErrors.length && attempt < repairAttempts()) {
        logLine(`ask ${sessionId}: the proposal compiles but adds ${newErrors.length} error(s); sent back to the AI`);
        messages.push({ role: 'user', content: `Your requests compile, but the compiler finds ${newErrors.length} new error(s) in the result:\n${newErrors.slice(0, 20).map((e) => `- ${e}`).join('\n')}\nNothing was applied. Send the corrected, complete list of requests with propose_edits, so that these errors are gone.` });
        onText(`\n(the compiler finds ${newErrors.length} error${newErrors.length > 1 ? 's' : ''} in the result: ${newErrors[0].slice(0, 120)}…; asking for a correction)\n`, chars);
        continue;
      }
      // A simulation needs its Modelica: a source the proposal names that is neither next to the model nor written now goes back to the AI.
      const missing = await missingSources(document, source.text, proposed, files);
      // Asked for a simulation, the answer brings one: a test case that simulates, with its Modelica.
      const asked = 'refine' in request ? request.refine : request.prompt;
      const simulates = (text: string) => (text.match(/simulate:\s*\{/g) ?? []).length;
      // Only a request to do it ("rajoute la simulation"), not a question about it ("peut-on faire une co-simulation ?").
      const question = isQuestion(asked);
      if (!question && /simul|openmodelica|modelica/i.test(asked) && simulates(proposed ?? source.text) <= simulates(source.text) && ![...files.keys()].some((p) => /\.mo$/i.test(p)) && attempt < repairAttempts()) {
        logLine(`ask ${sessionId}: a simulation was asked and the proposal has none; sent back to the AI`);
        messages.push({ role: 'user', content: 'The engineer asked for the simulation too, and your proposal has none. Add it now, in this answer: write the Modelica package with write_file (physics/<Name>Physics.mo), give the port types their physics if needed, and add a test_case with simulate: { model: ... stop_time: ... sources: [...] } and expects: on a requirement; keep everything else of your proposal. Send the complete propose_edits again.' });
        onText('\n(the simulation was asked too; asking for it now)\n', chars);
        continue;
      }
      if (missing.length && attempt < repairAttempts()) {
        logLine(`ask ${sessionId}: the proposal names ${missing.join(', ')}, which is not written; sent back to the AI`);
        messages.push({ role: 'user', content: `The test case's simulate: names ${missing.map((m) => `"${m}"`).join(', ')} in sources:, but no such file is next to the model and you did not write it. Write it now with write_file (the whole Modelica package: models extending the classes \`arclang export -f modelica\` generates for the elements, equations balanced, the variables the expects: read), then send the complete propose_edits again.` });
        onText(`\n(the simulation names ${missing.join(', ')}, which is not written yet; asking for it)\n`, chars);
        continue;
      }
      const written = [...files].map(([path, content]) => ({ path, content }));
      if (written.length) {
        view.view.files = written.map((f) => ({ path: f.path, lines: f.content.split('\n').length }));
        view.view.explainOnly = false;
      }
      live.proposal = { ...view, requests: answer.requests, source: answer.source && !answer.requests.length ? proposed : null, version, files: written };
      logLine(`ask ${sessionId}: ${answer.requests.length} request(s) accepted by the compiler on a copy; +${view.view.added} -${view.view.removed} lines; waiting for Apply`);
      return { ok: true, session: live };
    }
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    return { ok: false, error: message };
  }
}

/** Apply the session's proposal to the document: one undoable edit, checked again on the text as it is now. */
export async function applySession(sessionId: string): Promise<{ ok: boolean; error?: string; files?: WrittenFile[] }> {
  const live = sessions.get(sessionId);
  const proposal = live?.proposal;
  if (!live || !proposal || !proposal.view.ok) return { ok: false, error: 'nothing to apply' };
  const written: WrittenFile[] = [];
  const folder = vscode.Uri.joinPath(live.document.uri, '..');
  for (const file of proposal.files ?? []) {
    const uri = vscode.Uri.joinPath(folder, ...file.path.split('/'));
    const previous = await vscode.workspace.fs.readFile(uri).then((b) => b, () => null);
    await vscode.workspace.fs.createDirectory(vscode.Uri.joinPath(uri, '..'));
    await vscode.workspace.fs.writeFile(uri, new TextEncoder().encode(file.content.endsWith('\n') ? file.content : `${file.content}\n`));
    written.push({ uri, previous });
  }
  if (!proposal.requests.length && !proposal.source && written.length) {
    logLine(`ask ${sessionId}: ${written.length} file(s) written by the user`);
    live.proposal = null;
    return { ok: true, files: written };
  }
  if (proposal.requests.length) {
    const outcome = await applyRequests(live.document, proposal.requests);
    if (!outcome.ok) {
      await restoreFiles(written);
      return { ok: false, error: outcome.error };
    }
  } else if (proposal.source) {
    // A whole text replaces the document: only on the text it was computed on.
    if (proposal.version !== undefined && live.document.version !== proposal.version) {
      return { ok: false, error: 'the model changed since this proposal was made: it would undo those changes, so it is not applied. Ask again.' };
    }
    const back = await documentTextFor(live.document, proposal.source);
    if (!back.ok) return { ok: false, error: back.error };
    if (!(await replaceWhole(live.document, back.text))) return { ok: false, error: 'VS Code did not apply the edit' };
  } else {
    return { ok: false, error: 'the answer has no change to apply' };
  }
  logLine(`ask ${sessionId}: applied by the user`);
  live.proposal = null;
  forgetPreview(sessionId);
  return { ok: true, files: written };
}

/** The sources a proposal's simulations name (new to it) that are neither next to the model nor among the files written with it. */
async function missingSources(document: vscode.TextDocument, current: string, proposed: string | null, files: Map<string, string>): Promise<string[]> {
  if (!proposed) return [];
  const named = (text: string) => new Set([...text.matchAll(/sources:\s*\[([^\]]*)\]/g)].flatMap((m) => [...m[1].matchAll(/"([^"]+)"/g)].map((q) => q[1].replace(/^\.\//, ''))));
  const before = named(current);
  const folder = vscode.Uri.joinPath(document.uri, '..');
  const missing: string[] = [];
  for (const path of named(proposed)) {
    if (before.has(path) || files.has(path)) continue;
    const exists = await vscode.workspace.fs.stat(vscode.Uri.joinPath(folder, ...path.split('/'))).then(() => true, () => false);
    if (!exists) missing.push(path);
  }
  // A new simulate: with no sources and no Modelica written: the model it names exists nowhere the run can read.
  const simulations = (text: string) => [...text.matchAll(/simulate:\s*\{[^}]*\}/g)].map((m) => m[0]);
  const old = new Set(simulations(current));
  const fresh = simulations(proposed).filter((block) => !old.has(block));
  const writesModelica = [...files.keys()].some((p) => /\.mo$/i.test(p));
  if (fresh.some((block) => !/sources:/.test(block)) && !writesModelica) {
    const hasPackage = await vscode.workspace.findFiles(new vscode.RelativePattern(folder, '**/*.mo'), undefined, 1).then((f) => f.length > 0, () => false);
    if (!hasPackage) missing.push('the Modelica package (physics/<Name>Physics.mo, named in sources:)');
  }
  return missing;
}

/** Files written with a proposal, as they were before (removed when they did not exist). */
export async function restoreFiles(files: WrittenFile[]): Promise<void> {
  for (const file of files) {
    if (file.previous) await vscode.workspace.fs.writeFile(file.uri, file.previous);
    else await vscode.workspace.fs.delete(file.uri).then(undefined, () => undefined);
  }
}

export async function diffSession(sessionId: string): Promise<void> {
  const live = sessions.get(sessionId);
  if (!live?.proposal?.proposedText) return;
  const proposed = await documentTextFor(live.document, live.proposal.proposedText);
  await showDiff(sessionId, live.document, proposed.ok ? proposed.text : live.proposal.proposedText, `${live.document.uri.path.split('/').pop()} ↔ AI proposal (${live.provider})`);
}
