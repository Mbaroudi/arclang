// Proposed model texts as read-only documents (`arclang-ai:` URIs), for VS
// Code's diff view: the current source on the left, the proposal on the right.
import * as path from 'node:path';
import * as vscode from 'vscode';

export const PREVIEW_SCHEME = 'arclang-ai';

const texts = new Map<string, string>();
const emitter = new vscode.EventEmitter<vscode.Uri>();

export function registerPreviews(context: vscode.ExtensionContext): void {
  context.subscriptions.push(
    vscode.workspace.registerTextDocumentContentProvider(PREVIEW_SCHEME, {
      onDidChange: emitter.event,
      provideTextDocumentContent: (uri) => texts.get(uri.toString()) ?? '',
    }),
  );
}

/** A read-only document holding `text`, named after the model. */
export function previewUri(key: string, document: vscode.TextDocument, text: string, label = 'AI proposal'): vscode.Uri {
  const name = path.basename(document.uri.fsPath) || 'model.arc';
  const uri = vscode.Uri.from({ scheme: PREVIEW_SCHEME, path: `/${key}/${name}`, query: label });
  texts.set(uri.toString(), text);
  emitter.fire(uri);
  return uri;
}

export function forgetPreview(key: string): void {
  for (const uri of [...texts.keys()]) if (uri.includes(`/${key}/`)) texts.delete(uri);
}

/** The diff of the document and a proposal, beside the views. */
export async function showDiff(key: string, document: vscode.TextDocument, proposed: string, title: string): Promise<void> {
  const right = previewUri(key, document, proposed);
  await vscode.commands.executeCommand('vscode.diff', document.uri, right, title, { preview: true, viewColumn: vscode.ViewColumn.Beside, preserveFocus: true });
}

/** An explanation in a Markdown document beside the views. */
export async function showExplanation(key: string, document: vscode.TextDocument, markdown: string): Promise<void> {
  const uri = vscode.Uri.from({ scheme: PREVIEW_SCHEME, path: `/${key}/Explanation.md` });
  texts.set(uri.toString(), markdown);
  emitter.fire(uri);
  const shown = await vscode.workspace.openTextDocument(uri);
  await vscode.languages.setTextDocumentLanguage(shown, 'markdown');
  await vscode.window.showTextDocument(shown, { viewColumn: vscode.ViewColumn.Beside, preview: true, preserveFocus: true });
  void document;
}
