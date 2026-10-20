// The AI features of the extension: settings and keys, the provider layer,
// Ask AI on a selection (workbench and text), impact-driven suggestions.
import * as vscode from 'vscode';
import { connectBridge } from './bridge';
import { testConnection } from './client';
import { registerPreviews } from './preview';
import { aiLog, clearApiKey, configure, pickModel, setApiKey } from './settings';
import { registerAiStatus } from './status';
import { SuggestionEngine } from './suggest';
import { askAboutSelection, AskCodeActions, idsInSelection } from './textAsk';

export { handlePageMessage } from './bridge';

let engine: SuggestionEngine | null = null;

export function registerAi(context: vscode.ExtensionContext): void {
  registerPreviews(context);
  engine = new SuggestionEngine(context);
  connectBridge(context, engine);
  registerAiStatus(context);
  const register = (command: string, callback: (...args: unknown[]) => unknown) => context.subscriptions.push(vscode.commands.registerCommand(command, callback));
  register('arclang.ai.setApiKey', (provider?: unknown) => setApiKey(context, provider));
  register('arclang.ai.clearApiKey', () => clearApiKey(context));
  register('arclang.ai.testConnection', () => testConnection(context));
  register('arclang.ai.configure', () => configure(context));
  register('arclang.ai.pickModel', () => pickModel(context));
  register('arclang.ai.showLog', () => aiLog().show(true));
  register('arclang.ai.askSelection', (uri?: unknown, range?: unknown, action?: unknown, prompt?: unknown) => askAboutSelection(context, uri, range, action, prompt));
  register('arclang.ai.analyseImpact', async (uri?: unknown, element?: unknown) => {
    const editor = vscode.window.activeTextEditor;
    const document = uri instanceof vscode.Uri ? await vscode.workspace.openTextDocument(uri) : editor?.document;
    if (!document) return;
    const ids = typeof element === 'string' ? [element] : editor ? await idsInSelection(document, editor.selection) : [];
    if (ids.length === 0) {
      vscode.window.showInformationMessage('ArcLang: put the cursor in an element of the model, or select one in the workbench.');
      return;
    }
    await engine?.analyseSelection(document, ids);
    await vscode.commands.executeCommand('arclang.openWorkbench', document.uri);
  });
  context.subscriptions.push(vscode.languages.registerCodeActionsProvider([{ language: 'arclang' }, { language: 'sysml' }], new AskCodeActions(), { providedCodeActionKinds: AskCodeActions.kinds }));
}
