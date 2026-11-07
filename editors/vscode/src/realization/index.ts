// The commands of the realization ladder: open the ladder of a model (the
// active editor, a tree item, a file), refresh it.
import * as vscode from 'vscode';
import { isModelDocument } from '../workbench/source';
import { RealizationPanel } from './panel';

async function documentOf(arg: unknown): Promise<vscode.TextDocument | null> {
  const object = (typeof arg === 'object' && arg !== null ? arg : {}) as { uri?: unknown };
  if (arg instanceof vscode.Uri) return vscode.workspace.openTextDocument(arg);
  if (object.uri instanceof vscode.Uri) return vscode.workspace.openTextDocument(object.uri);
  const editor = vscode.window.activeTextEditor;
  if (editor && isModelDocument(editor.document)) return editor.document;
  const open = RealizationPanel.activePanel()?.document ?? vscode.workspace.textDocuments.find(isModelDocument) ?? null;
  if (!open) vscode.window.showInformationMessage('ArcLang: open a .arc or .sysml file first.');
  return open;
}

export function registerRealization(context: vscode.ExtensionContext): void {
  const register = (command: string, callback: (...args: unknown[]) => unknown) => context.subscriptions.push(vscode.commands.registerCommand(command, callback));
  register('arclang.realization.open', async (arg?: unknown) => {
    const document = await documentOf(arg);
    if (document) await RealizationPanel.open(context, document);
  });
  register('arclang.realization.refresh', () => RealizationPanel.activePanel()?.refresh());
}
