// Impact-driven suggestions while editing: after an edit (by the
// arclang.ai.suggestOnEdit setting) or on "Analyse impact" of a selection,
// the compiler says what the change touched and broke (traces, findings,
// gate, `arclang impact`); when something broke, the AI proposes one to
// three architecture changes, each compiled on a copy before it is shown in
// the Suggestions pane. Debounced, cancelled when a newer edit comes, never
// in the way of editing, and never applied without the user's click.
import * as vscode from 'vscode';
import type { Snapshot } from '../pure/workbench';
import { applyRequests } from '../workbench/edits';
import { lastSnapshot, metamodelJson, onDidChangeSnapshot, snapshotOf } from '../workbench/snapshot';
import { isModelDocument } from '../workbench/source';
import { proposalView } from './assist';
import { hasPage, postTo } from './bridge';
import { openProvider } from './client';
import { parseSuggestions } from './core/answer';
import { buildSelectionContext } from './core/context';
import { converse } from './core/conversation';
import { consequencesOf, describeChange, diffSnapshots, isEmptyChange, touchedIds } from './core/impact';
import { IMPACT, PROPOSE_CHANGES, READ_ELEMENTS, suggestPrompt, SUGGEST_RULES } from './core/prompts';
import type { SuggestionView, ToPageAi } from './core/protocol';
import { describeEdit } from './core/requests';
import { arcText, checkRequests, impactOf } from './compiler';
import { showDiff } from './preview';
import type { ChatMessage } from './providers';
import { logLine, readSettings, suggestMode } from './settings';
import type { EditRequest } from '../pure/workbench';

const DEBOUNCE_MS = 1200;

interface Analysis {
  trigger: string;
  change: string[];
  consequences: string[];
  impact: string[];
  /** The ids the analysis is about, for the context sent to the AI. */
  ids: string[];
}

interface Kept {
  view: SuggestionView;
  requests: EditRequest[];
  proposedText: string | null;
}

type SuggestionsMessage = Extract<ToPageAi, { type: 'ai-suggestions' }>;

export class SuggestionEngine {
  private readonly baselines = new Map<string, Snapshot>();
  private readonly timers = new Map<string, NodeJS.Timeout>();
  private readonly running = new Map<string, AbortController>();
  private readonly shown = new Map<string, SuggestionsMessage>();
  private readonly kept = new Map<string, Map<string, Kept>>();
  private readonly analyses = new Map<string, Analysis>();
  private counter = 0;

  constructor(private readonly context: vscode.ExtensionContext) {
    context.subscriptions.push(
      onDidChangeSnapshot((uri) => this.snapshotChanged(uri)),
      vscode.workspace.onDidCloseTextDocument((document) => this.forget(document.uri.toString())),
    );
  }

  private forget(key: string): void {
    clearTimeout(this.timers.get(key));
    this.running.get(key)?.abort();
    for (const map of [this.baselines, this.timers, this.running, this.shown, this.kept, this.analyses]) map.delete(key);
  }

  /** A page that opens gets the suggestions already shown elsewhere. */
  replay(document: vscode.TextDocument, post: (message: ToPageAi) => void): void {
    const shown = this.shown.get(document.uri.toString());
    if (shown) post(shown);
  }

  private publish(document: vscode.TextDocument, message: Omit<SuggestionsMessage, 'type'>): void {
    const full: SuggestionsMessage = { type: 'ai-suggestions', ...message };
    this.shown.set(document.uri.toString(), full);
    postTo(document.uri, full);
  }

  private snapshotChanged(uri: vscode.Uri): void {
    const document = vscode.workspace.textDocuments.find((d) => d.uri.toString() === uri.toString());
    if (!document || !isModelDocument(document)) return;
    const snapshot = lastSnapshot(document);
    if (!snapshot?.success || !snapshot.elements) return;
    const key = uri.toString();
    if (!this.baselines.has(key) || suggestMode() === 'off') {
      this.baselines.set(key, snapshot);
      return;
    }
    clearTimeout(this.timers.get(key));
    this.timers.set(
      key,
      setTimeout(() => {
        this.timers.delete(key);
        const before = this.baselines.get(key);
        const after = lastSnapshot(document);
        if (!before || !after?.success) return;
        this.baselines.set(key, after);
        void this.afterEdit(document, before, after).catch((error: unknown) => logLine(`suggestions: ${error instanceof Error ? error.message : String(error)}`));
      }, DEBOUNCE_MS),
    );
  }

  private async impactLines(document: vscode.TextDocument, text: string, ids: string[], signal: AbortSignal): Promise<string[]> {
    const lines: string[] = [];
    for (const id of ids.slice(0, 3)) {
      if (signal.aborted) break;
      const out = await impactOf(document, text, id);
      lines.push(...out.split('\n').map((l) => l.trim()).filter((l) => l && !/^Distance/.test(l)).slice(0, 12));
    }
    return lines;
  }

  private async afterEdit(document: vscode.TextDocument, before: Snapshot, after: Snapshot): Promise<void> {
    // The pane is in the workbench: nothing is computed for a model no workbench shows.
    if (!this.watched(document)) return;
    const change = diffSnapshots(before, after);
    if (isEmptyChange(change)) return;
    const key = document.uri.toString();
    this.running.get(key)?.abort();
    const controller = new AbortController();
    this.running.set(key, controller);
    const source = await arcText(document);
    if (!source.ok || controller.signal.aborted) return;
    const consequences = consequencesOf(before, after, change);
    const ids = touchedIds(change).filter((id) => (after.elements?.elements ?? []).some((e) => e.id === id));
    const impact = consequences.length ? await this.impactLines(document, source.text, ids, controller.signal) : [];
    if (controller.signal.aborted) return;
    const analysis: Analysis = { trigger: 'after your edit', change: describeChange(change), consequences: consequences.map((c) => c.message), impact, ids: [...new Set([...ids, ...consequences.flatMap((c) => c.elements)])] };
    this.analyses.set(key, analysis);
    const provider = (await this.providerLabel()) ?? '';
    if (consequences.length === 0) {
      this.publish(document, { state: 'none', trigger: analysis.trigger, consequences: [], suggestions: [], provider, change: analysis.change });
      return;
    }
    logLine(`suggestions: ${consequences.length} consequence(s) of the edit of ${document.uri.path.split('/').pop()}: ${analysis.consequences.join('; ')}`);
    if (suggestMode() === 'afterEachEdit') return this.ask(document, analysis, controller);
    this.publish(document, { state: 'ready', trigger: analysis.trigger, consequences: analysis.consequences, suggestions: [], provider, change: analysis.change, impact, canPropose: true });
  }

  private async providerLabel(): Promise<string | null> {
    const settings = readSettings();
    return settings.provider === 'off' ? null : `${settings.provider}${settings.model ? ` · ${settings.model}` : ''}`;
  }

  /** "Propose changes with AI" on consequences already shown. */
  async propose(document: vscode.TextDocument): Promise<void> {
    const analysis = this.analyses.get(document.uri.toString());
    if (!analysis) return;
    const controller = new AbortController();
    this.running.get(document.uri.toString())?.abort();
    this.running.set(document.uri.toString(), controller);
    await this.ask(document, analysis, controller);
  }

  /** "Analyse impact" of a selection: what depends on it, what the compiler says on it, and proposals. */
  async analyseSelection(document: vscode.TextDocument, ids: string[]): Promise<void> {
    const key = document.uri.toString();
    this.running.get(key)?.abort();
    const controller = new AbortController();
    this.running.set(key, controller);
    const provider = (await this.providerLabel()) ?? '';
    this.publish(document, { state: 'analysing', trigger: `impact of ${ids.join(', ')}`, consequences: [], suggestions: [], provider });
    const [snapshot, source] = await Promise.all([snapshotOf(document), arcText(document)]);
    if (!source.ok || !snapshot.elements) {
      this.publish(document, { state: 'error', trigger: `impact of ${ids.join(', ')}`, consequences: [], suggestions: [], provider, error: source.ok ? 'the model does not compile' : source.error });
      return;
    }
    const impact = await this.impactLines(document, source.text, ids, controller.signal);
    const names = new Set(ids.flatMap((id) => [id, snapshot.elements!.elements.find((e) => e.id === id)?.name ?? id]));
    const affected = new Set(impact.map((l) => /\[([^\]]+)\]/.exec(l)?.[1]).filter((x): x is string => !!x));
    const consequences = [
      ...snapshot.findings.filter((f) => f.element && (names.has(f.element) || affected.has(f.element))).map((f) => `[${f.rule}] ${f.message}`),
      ...(snapshot.gate?.findings ?? []).filter((f) => [...names, ...affected].some((n) => f.message.includes(n))).map((f) => `gate ${f.severity.toLowerCase()}: ${f.message}`),
    ];
    const analysis: Analysis = { trigger: `impact of ${ids.join(', ')}`, change: [], consequences, impact, ids: [...ids, ...affected] };
    this.analyses.set(key, analysis);
    if (controller.signal.aborted) return;
    if (consequences.length === 0) {
      this.publish(document, { state: 'none', trigger: analysis.trigger, consequences: [], suggestions: [], provider, impact });
      return;
    }
    await this.ask(document, analysis, controller);
  }

  private async ask(document: vscode.TextDocument, analysis: Analysis, controller: AbortController): Promise<void> {
    const base = { trigger: analysis.trigger, consequences: analysis.consequences, change: analysis.change, impact: analysis.impact };
    const opened = await openProvider(this.context, `suggestions (${analysis.trigger})`, `The edit just made, its consequences found by the compiler, \`arclang impact\` and the source of the elements concerned (${analysis.ids.slice(0, 8).join(', ')}).`);
    if (!opened.ok) {
      this.publish(document, { ...base, state: opened.error.includes('off') ? 'off' : 'error', suggestions: [], provider: '', error: opened.error, canPropose: true });
      return;
    }
    if (controller.signal.aborted) return;
    this.publish(document, { ...base, state: 'thinking', suggestions: [], provider: opened.label });
    const [snapshot, metamodel, source] = await Promise.all([snapshotOf(document), metamodelJson(), arcText(document)]);
    if (!source.ok || !snapshot.elements) return;
    const context = buildSelectionContext({ snapshot, metamodel, text: source.text, fileName: document.uri.path.split('/').pop() ?? 'model.arc', ids: analysis.ids, scope: 'selectionAndNeighbours', maxChars: 30000 });
    const messages: ChatMessage[] = [
      { role: 'system', content: SUGGEST_RULES },
      { role: 'user', content: suggestPrompt(analysis.change, analysis.consequences, analysis.impact.join('\n'), context.text) },
    ];
    const run = async (call: { name: string; input: Record<string, unknown> }) => {
      if (call.name === 'read_elements') return buildSelectionContext({ snapshot, metamodel, text: source.text, fileName: 'model', ids: Array.isArray(call.input.ids) ? call.input.ids.map(String) : [], scope: 'selection', maxChars: 12000 }).text;
      if (call.name === 'impact') return impactOf(document, source.text, String(call.input.element ?? ''));
      return `There is no tool named ${call.name}.`;
    };
    try {
      let kept: Kept[] = [];
      let dropped: string[] = [];
      for (let attempt = 0; attempt < 2 && kept.length === 0; attempt++) {
        const result = await converse(opened.provider, messages, { tools: [PROPOSE_CHANGES, READ_ELEMENTS, IMPACT], final: ['propose_changes'], run, signal: controller.signal, maxTokens: readSettings().maxTokens });
        if (controller.signal.aborted) return;
        const parsed = parseSuggestions(result.text, result.calls);
        dropped = [...parsed.dropped];
        kept = [];
        for (const suggestion of parsed.suggestions) {
          const checked = await checkRequests(document, source.text, suggestion.requests);
          if (!checked.ok) {
            dropped.push(`${suggestion.title}: request ${checked.at + 1} (${describeEdit(suggestion.requests[checked.at])}) refused by the compiler: ${checked.error}`);
            continue;
          }
          const view = await proposalView(document, snapshot, source.text, suggestion.rationale, suggestion.requests, checked.text);
          kept.push({ view: { id: `s${++this.counter}`, title: suggestion.title, rationale: suggestion.rationale, resolves: suggestion.resolves, proposal: view.view }, requests: suggestion.requests, proposedText: view.proposedText });
        }
        if (kept.length === 0 && dropped.length) {
          messages.push({ role: 'assistant', content: result.text, toolCalls: result.calls.length ? result.calls : undefined });
          for (const call of result.calls) messages.push({ role: 'tool', content: 'Received.', toolCallId: call.id, toolName: call.name });
          messages.push({ role: 'user', content: `The compiler refused every proposal:\n${dropped.join('\n')}\nSend corrected proposals with propose_changes.` });
        } else break;
      }
      this.kept.set(document.uri.toString(), new Map(kept.map((k) => [k.view.id, k])));
      logLine(`suggestions: ${kept.length} proposal(s) accepted by the compiler on a copy${dropped.length ? `, ${dropped.length} refused` : ''}`);
      this.publish(document, { ...base, state: kept.length ? 'ready' : 'none', suggestions: kept.map((k) => k.view), provider: opened.label, dropped });
    } catch (error) {
      if (controller.signal.aborted) return;
      this.publish(document, { ...base, state: 'error', suggestions: [], provider: opened.label, error: error instanceof Error ? error.message : String(error), canPropose: true });
    }
  }

  async act(document: vscode.TextDocument, id: string, action: 'apply' | 'dismiss' | 'diff'): Promise<void> {
    const key = document.uri.toString();
    const kept = this.kept.get(key)?.get(id);
    const shown = this.shown.get(key);
    if (!kept || !shown) return;
    if (action === 'diff') {
      if (kept.proposedText) await showDiff(id, document, kept.proposedText, `${document.uri.path.split('/').pop()} ↔ AI suggestion: ${kept.view.title}`);
      return;
    }
    if (action === 'apply') {
      const outcome = await applyRequests(document, kept.requests);
      if (!outcome.ok) {
        const view = { ...kept.view, proposal: { ...kept.view.proposal, ok: false, error: `refused on the model as it is now: ${outcome.error}` } };
        this.kept.get(key)!.set(id, { ...kept, view });
        this.publish(document, { ...shown, suggestions: shown.suggestions.map((s) => (s.id === id ? view : s)) });
        return;
      }
      logLine(`suggestions: "${kept.view.title}" applied by the user`);
    }
    this.kept.get(key)!.delete(id);
    const rest = shown.suggestions.filter((s) => s.id !== id);
    this.publish(document, { ...shown, suggestions: rest, state: rest.length ? shown.state : 'idle', canPropose: false });
  }

  /** Whether a page will show what the engine finds (nothing is computed for nobody). */
  watched(document: vscode.TextDocument): boolean {
    return hasPage(document.uri);
  }
}
