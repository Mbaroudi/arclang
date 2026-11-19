// What other extensions can use of ArcLang: `vscode.extensions.getExtension('arclang.arclang')?.exports`.
// A closed companion extension (the Enterprise navigator, for one) loads the compiler and the model the user is
// looking at from here, instead of finding the binary and watching the files itself. Version 1: it only grows.
import * as vscode from 'vscode';
import { binary, run, RunResult } from './cli';
import { debounced, parseWorkbenchAnswer, WorkbenchAnswer } from './pure/api';
import { WorkbenchPanel } from './workbench/panel';
import { goToSource } from './workbench/go-to-source';

export type { RunResult, WorkbenchAnswer };

export interface ArcLangApi {
  /** The version of this interface; a new method never changes the meaning of an old one. */
  readonly apiVersion: 1;
  /** The compiler the extension runs: the `arclang.binary` setting, or `arclang` on the PATH. */
  compilerPath(): string;
  /**
   * Run `arclang` with these arguments, as the extension does: nothing runs on an untrusted workspace (exit code
   * 126), a missing compiler is exit code 127, and neither throws.
   */
  run(args: string[], options?: { cwd?: string }): Promise<RunResult>;
  /** What `arclang workbench` says of a model file: its workbench document, or why there is none. */
  workbench(file: vscode.Uri): Promise<WorkbenchAnswer>;
  /** Fires with the file when a model is saved or edited (a burst of keystrokes is one event, 300 ms after the last). */
  readonly onDidChangeModel: vscode.Event<vscode.Uri>;
  /** The model the user is on: the active editor's, else the workbench opened last; undefined when there is none. */
  activeModel(): vscode.Uri | undefined;
  /** Show an element's declaration in the editor. False when the file does not declare it (it may be imported). */
  reveal(file: vscode.Uri, element: { id: string; name?: string }): Promise<boolean>;
}

export function createApi(context: vscode.ExtensionContext): ArcLangApi {
  const changed = new vscode.EventEmitter<vscode.Uri>();
  const burst = debounced<vscode.Uri>(300, (uri) => changed.fire(uri));
  const isModel = (document: vscode.TextDocument) => document.languageId === 'arclang' && document.uri.scheme === 'file';
  context.subscriptions.push(
    changed,
    { dispose: () => burst.dispose() },
    vscode.workspace.onDidChangeTextDocument((event) => { if (isModel(event.document)) burst.push(event.document.uri.toString(), event.document.uri); }),
    vscode.workspace.onDidSaveTextDocument((document) => { if (isModel(document)) burst.push(document.uri.toString(), document.uri); }),
  );
  return {
    apiVersion: 1,
    compilerPath: () => binary(),
    run: (args, options = {}) => run(args, { cwd: options.cwd }),
    async workbench(file) {
      const result = await run(['workbench', file.fsPath], { cwd: vscode.Uri.joinPath(file, '..').fsPath });
      return parseWorkbenchAnswer(result.code, result.stdout, result.stderr);
    },
    onDidChangeModel: changed.event,
    activeModel() {
      const editor = vscode.window.activeTextEditor;
      if (editor && isModel(editor.document)) return editor.document.uri;
      return WorkbenchPanel.activePanel()?.documentOf.uri;
    },
    async reveal(file, element) {
      const document = await vscode.workspace.openTextDocument(file);
      return goToSource(document, { id: element.id, name: element.name ?? element.id });
    },
  };
}
