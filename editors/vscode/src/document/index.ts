// *ArcLang: Generate a document*: a design review of the model, or a
// template of the user's (Markdown with {{ ... }} directives), filled by the
// compiler (`arclang document`) and written as Word, HTML or Markdown.
import * as path from 'node:path';
import * as vscode from 'vscode';
import { failureText, run } from '../cli';
import { documentArgs, DOCUMENT_FORMATS, DocumentFormat } from '../pure/document';

/** The model the command acts on: given, the active editor's, or the workbench tab's. */
function modelOf(arg: unknown): vscode.Uri | null {
  if (arg instanceof vscode.Uri) return arg;
  const doc = vscode.window.activeTextEditor?.document;
  if (doc?.languageId === 'arclang' && doc.uri.scheme === 'file') return doc.uri;
  const input = vscode.window.tabGroups.activeTabGroup.activeTab?.input as { uri?: vscode.Uri } | undefined;
  return input?.uri instanceof vscode.Uri && input.uri.fsPath.endsWith('.arc') ? input.uri : null;
}

async function generate(arg: unknown): Promise<void> {
  const model = modelOf(arg);
  if (!model) {
    void vscode.window.showInformationMessage('ArcLang: open a model (.arc) to generate its document.');
    return;
  }
  const source = await vscode.window.showQuickPick(
    [
      { label: '$(book) Design review', detail: 'Gate, requirements and their coverage, verification, constraints, traceability, suspect links, rules, interfaces.', template: null as string | null },
      { label: '$(file) A template of mine…', detail: 'Markdown with {{ ... }} directives: model, date, value, table, elements, verdicts, constraints, gate, suspects, rules.', template: '' },
    ],
    { title: 'ArcLang: Generate a document' },
  );
  if (!source) return;
  let template: string | null = null;
  if (source.template !== null) {
    const picked = await vscode.window.showOpenDialog({ canSelectMany: false, defaultUri: vscode.Uri.file(path.dirname(model.fsPath)), filters: { Templates: ['md', 'markdown', 'txt'] }, title: 'The template' });
    if (!picked?.length) return;
    template = picked[0].fsPath;
  }
  const format = await vscode.window.showQuickPick(DOCUMENT_FORMATS.map((f) => ({ label: f.label, detail: f.detail, value: f.id })), { title: 'ArcLang: Generate a document', placeHolder: 'The format' });
  if (!format) return;
  const base = path.basename(model.fsPath, '.arc');
  const target = await vscode.window.showSaveDialog({ defaultUri: vscode.Uri.file(path.join(path.dirname(model.fsPath), `${base}-review.${format.value}`)), filters: { [format.label]: [format.value] } });
  if (!target) return;
  const document = vscode.workspace.textDocuments.find((d) => d.uri.toString() === model.toString());
  if (document?.isDirty) await document.save();
  const result = await run(documentArgs(model.fsPath, target.fsPath, template), { cwd: path.dirname(model.fsPath) });
  if (result.code !== 0) {
    void vscode.window.showErrorMessage(`ArcLang: the document was not written. ${result.stderr.trim().split('\n').pop() || failureText(result)}`);
    return;
  }
  const open = await vscode.window.showInformationMessage(`ArcLang: wrote ${path.basename(target.fsPath)}.`, format.value === 'md' ? 'Open' : 'Open with the default app');
  if (!open) return;
  if (format.value === 'md') await vscode.window.showTextDocument(target);
  else await vscode.env.openExternal(target);
}

export function registerDocuments(context: vscode.ExtensionContext): void {
  context.subscriptions.push(vscode.commands.registerCommand('arclang.document.generate', (arg?: unknown) => generate(arg)));
}

export type { DocumentFormat };
