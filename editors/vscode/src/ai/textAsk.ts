// "Ask AI about the selection" in the .arc text editor: a command, a code
// action on a selection (and on a diagnostic: fix it), the same engine as
// the workbench, the diff in VS Code's diff view, Apply / Refine / Reject.
import * as vscode from 'vscode';
import { elementsInRange } from './core/context';
import type { QuickAction } from './core/protocol';
import { QUICK_ACTIONS } from './core/protocol';
import { applySession, diffSession, dropSession, runAsk, session } from './assist';
import { showOpenFailure } from './client';
import { showExplanation } from './preview';
import { arcText } from './compiler';
import { snapshotOf } from '../workbench/snapshot';

let counter = 0;

/** The ids of the elements a range of the document declares or sits in. */
export async function idsInSelection(document: vscode.TextDocument, range: vscode.Range): Promise<string[]> {
  const snapshot = await snapshotOf(document);
  const source = await arcText(document);
  if (!snapshot.elements || !source.ok || document.languageId !== 'arclang') return [];
  return elementsInRange(source.text, snapshot.elements.elements, document.offsetAt(range.start), document.offsetAt(range.end)).map((e) => e.id);
}

/** The question: a quick action, or the user's own words. */
async function askQuestion(ids: string[]): Promise<{ prompt: string; action?: QuickAction } | undefined> {
  return new Promise((resolve) => {
    const pick = vscode.window.createQuickPick<vscode.QuickPickItem & { action?: QuickAction }>();
    pick.title = `Ask AI about ${ids.length ? ids.slice(0, 4).join(', ') + (ids.length > 4 ? '…' : '') : 'the selected text'}`;
    pick.placeholder = 'Type a request (modify, correct, add…) or pick a quick action';
    const actions = QUICK_ACTIONS.map((a) => ({ label: `$(sparkle) ${a.label}`, description: a.prompt.split(':')[0], action: a.id }));
    pick.items = actions;
    pick.onDidChangeValue((value) => {
      pick.items = value.trim() ? [{ label: `$(comment) ${value.trim()}`, description: 'ask' }, ...actions] : actions;
    });
    let done = false;
    pick.onDidAccept(() => {
      done = true;
      const chosen = pick.selectedItems[0];
      const typed = pick.value.trim();
      pick.hide();
      if (chosen?.action) resolve({ prompt: typed && !chosen.label.includes(typed) ? typed : '', action: chosen.action });
      else resolve(typed ? { prompt: typed } : undefined);
    });
    pick.onDidHide(() => {
      if (!done) resolve(undefined);
      pick.dispose();
    });
    pick.show();
  });
}

async function runWithProgress(sessionId: string, request: Parameters<typeof runAsk>[1], context: vscode.ExtensionContext, title: string) {
  return vscode.window.withProgress({ location: vscode.ProgressLocation.Notification, title, cancellable: true }, async (progress, token) => {
    token.onCancellationRequested(() => session(sessionId)?.controller.abort());
    let last = 0;
    return runAsk(sessionId, request, context, (_text, chars) => {
      if (chars - last > 200) {
        last = chars;
        progress.report({ message: `${chars} characters received` });
      }
    });
  });
}

/** Ask, preview, then Apply / Refine / Reject until the user decides. */
export async function askAboutSelection(context: vscode.ExtensionContext, uriArg?: unknown, rangeArg?: unknown, actionArg?: unknown, promptArg?: unknown): Promise<void> {
  const editor = vscode.window.activeTextEditor;
  const document = uriArg instanceof vscode.Uri ? await vscode.workspace.openTextDocument(uriArg) : editor?.document;
  if (!document || (document.languageId !== 'arclang' && document.languageId !== 'sysml')) {
    vscode.window.showInformationMessage('ArcLang: select part of a .arc model first.');
    return;
  }
  const range = rangeArg instanceof vscode.Range ? rangeArg : editor && editor.document === document ? editor.selection : new vscode.Range(0, 0, 0, 0);
  const ids = await idsInSelection(document, range);
  const asked = typeof actionArg === 'string' || typeof promptArg === 'string' ? { action: (typeof actionArg === 'string' ? actionArg : undefined) as QuickAction | undefined, prompt: typeof promptArg === 'string' ? promptArg : '' } : await askQuestion(ids);
  if (!asked) return;
  const sessionId = `t${++counter}`;
  const selectedText = range.isEmpty ? undefined : document.getText(range);
  let outcome = await runWithProgress(sessionId, { document, ids, prompt: asked.prompt, action: asked.action, selectedText }, context, 'ArcLang AI is thinking about the selection');
  for (;;) {
    if (!outcome.ok) {
      if (outcome.fix) await showOpenFailure({ ok: false, error: outcome.error, fix: outcome.fix });
      else vscode.window.showErrorMessage(`ArcLang AI: ${outcome.error}`);
      dropSession(sessionId);
      return;
    }
    const proposal = outcome.session.proposal;
    if (!proposal) return;
    const view = proposal.view;
    const head = `ArcLang AI (${outcome.session.provider}): ${view.explanation.split('\n')[0].slice(0, 240)}`;
    let choice: string | undefined;
    if (!view.ok) {
      choice = await vscode.window.showWarningMessage(`${head} — the compiler refused the change: ${view.error}`, 'Refine…', 'Dismiss');
    } else if (view.explainOnly || !proposal.proposedText) {
      await showExplanation(sessionId, document, `# ArcLang AI on ${ids.join(', ') || 'the selection'}\n\n_${outcome.session.provider}: an AI answer, check it._\n\n${view.explanation}\n`);
      choice = await vscode.window.showInformationMessage(head, 'Refine…', 'Dismiss');
    } else {
      await diffSession(sessionId);
      const summary = `${view.edits.length} edit${view.edits.length === 1 ? '' : 's'} checked by the compiler, +${view.added} −${view.removed} lines${view.newFindings.length ? `, ${view.newFindings.length} new finding(s)` : ''}`;
      choice = await vscode.window.showInformationMessage(`${head} (${summary})`, 'Apply', 'Refine…', 'Reject');
    }
    if (choice === 'Apply') {
      const applied = await applySession(sessionId);
      if (applied.ok) vscode.window.setStatusBarMessage('ArcLang AI: applied as one change (⌘Z undoes it)', 5000);
      else vscode.window.showErrorMessage(`ArcLang AI: not applied: ${applied.error}`);
      // The diff of a proposal now applied is closed.
      for (const tab of vscode.window.tabGroups.all.flatMap((g) => g.tabs)) {
        if (tab.input instanceof vscode.TabInputTextDiff && tab.input.modified.scheme === 'arclang-ai') await vscode.window.tabGroups.close(tab);
      }
      dropSession(sessionId);
      return;
    }
    if (choice !== 'Refine…') {
      dropSession(sessionId);
      return;
    }
    const refine = await vscode.window.showInputBox({ prompt: 'Refine the proposal (a follow-up question)', placeHolder: 'for example: name the ports after the data they carry' });
    if (!refine) {
      dropSession(sessionId);
      return;
    }
    outcome = await runWithProgress(sessionId, { refine }, context, 'ArcLang AI is refining the proposal');
  }
}

/** The light bulb on a selection of a model: ask about it; on a diagnostic: fix it. */
export class AskCodeActions implements vscode.CodeActionProvider {
  static readonly kinds = [vscode.CodeActionKind.QuickFix, vscode.CodeActionKind.RefactorRewrite];

  provideCodeActions(document: vscode.TextDocument, range: vscode.Range | vscode.Selection, context: vscode.CodeActionContext): vscode.CodeAction[] {
    const actions: vscode.CodeAction[] = [];
    if (!range.isEmpty) {
      const ask = new vscode.CodeAction('Ask ArcLang AI about the selection…', vscode.CodeActionKind.RefactorRewrite);
      ask.command = { command: 'arclang.ai.askSelection', title: 'Ask ArcLang AI', arguments: [document.uri, range] };
      actions.push(ask);
    }
    for (const diagnostic of context.diagnostics.slice(0, 3)) {
      const fix = new vscode.CodeAction(`Fix with ArcLang AI: ${diagnostic.message.slice(0, 80)}`, vscode.CodeActionKind.QuickFix);
      fix.diagnostics = [diagnostic];
      fix.command = { command: 'arclang.ai.askSelection', title: 'Fix with ArcLang AI', arguments: [document.uri, diagnostic.range, 'fix', `Fix this diagnostic: ${diagnostic.message}`] };
      actions.push(fix);
    }
    return actions;
  }
}
