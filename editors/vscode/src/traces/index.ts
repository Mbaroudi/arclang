import * as vscode from 'vscode';
import { run } from '../cli';
import { SUSPECT_RULE, SuspectTrace, quickFixTitle, reviewArgs, suspectOf } from '../pure/suspect';

/**
 * A suspect trace (one of its ends changed since it was reviewed) offers a
 * quick fix: once the author has checked that it still holds, mark it
 * reviewed. The compiler writes the new fingerprints; the extension applies
 * them as an edit of the document, so ⌘Z takes them back.
 */
class SuspectActions implements vscode.CodeActionProvider {
  static readonly kinds = [vscode.CodeActionKind.QuickFix];

  provideCodeActions(document: vscode.TextDocument, _range: vscode.Range, context: vscode.CodeActionContext): vscode.CodeAction[] {
    return context.diagnostics
      .filter((d) => (typeof d.code === 'object' ? d.code.value : d.code) === SUSPECT_RULE)
      .flatMap((diagnostic) => {
        const trace = suspectOf(diagnostic.message);
        if (!trace) return [];
        const action = new vscode.CodeAction(quickFixTitle(trace), vscode.CodeActionKind.QuickFix);
        action.diagnostics = [diagnostic];
        action.command = { command: 'arclang.trace.review', title: 'Mark the trace as reviewed', arguments: [{ uri: document.uri, trace }] };
        return [action];
      });
  }
}

async function review(arg?: unknown): Promise<void> {
  const given = arg as { uri?: vscode.Uri; trace?: SuspectTrace } | undefined;
  const document = given?.uri ? await vscode.workspace.openTextDocument(given.uri) : vscode.window.activeTextEditor?.document;
  if (!document || document.languageId !== 'arclang') {
    void vscode.window.showInformationMessage('Open an ArcLang model to review its traces.');
    return;
  }
  if (!given?.trace) {
    const yes = 'Mark them reviewed';
    const answer = await vscode.window.showWarningMessage(
      'Mark every trace of this model as reviewed? Each records what its two ends say now; a later change to either end makes it suspect.',
      { modal: true },
      yes,
    );
    if (answer !== yes) return;
  }
  // The compiler reads the file: what the editor holds is saved first.
  if (document.isDirty && !(await document.save())) return;
  const result = await run(reviewArgs(document.uri.fsPath, given?.trace), { cwd: vscode.Uri.joinPath(document.uri, '..').fsPath });
  if (result.code !== 0) {
    void vscode.window.showErrorMessage(`The trace could not be marked reviewed: ${result.stderr.trim() || result.stdout.trim()}`);
    return;
  }
  if (result.stdout === document.getText()) {
    void vscode.window.showInformationMessage('Nothing to review: every trace is up to date.');
    return;
  }
  const edit = new vscode.WorkspaceEdit();
  edit.replace(document.uri, new vscode.Range(document.positionAt(0), document.positionAt(document.getText().length)), result.stdout);
  await vscode.workspace.applyEdit(edit);
  await document.save();
  void vscode.window.setStatusBarMessage(given?.trace ? `Reviewed: ${given.trace.from} ${given.trace.type} ${given.trace.to}` : 'Every trace marked reviewed', 4000);
}

export function registerTraces(context: vscode.ExtensionContext): void {
  context.subscriptions.push(vscode.commands.registerCommand('arclang.trace.review', review));
  context.subscriptions.push(vscode.languages.registerCodeActionsProvider({ language: 'arclang' }, new SuspectActions(), { providedCodeActionKinds: SuspectActions.kinds }));
}
