// ArcLang as a web extension: the same workbench, the same views and
// commands, the compiler running in WebAssembly in this worker. There is no
// language server and no native binary here; `src/cli.ts` sends every
// compiler call to the WebAssembly backend (`wasm-backend.ts`).
import * as vscode from 'vscode';
import { registerAi } from '../ai'; // ai-design hook
import * as commands from '../commands';
import { setBackend } from '../cli';
import { registerFormatter } from '../format';
import { registerImport } from '../importing';
import { registerImpact } from '../impact';
import { registerTables } from '../tables';
import { registerWorkbench } from '../workbench';
import { registerWebDiagnostics } from './diagnostics';
import { createWasmBackend, NOT_AVAILABLE } from './wasm-backend';
import { loadWasm } from './wasm';

/** Registers a part of the extension; one that cannot start in a browser must not stop the others. */
function attempt(name: string, start: () => void): void {
  try {
    start();
  } catch (error) {
    console.warn(`ArcLang: ${name} is not available in the browser: ${error instanceof Error ? error.message : String(error)}`);
  }
}

export async function activate(context: vscode.ExtensionContext): Promise<void> {
  const wasm = await loadWasm(context);
  setBackend(createWasmBackend(wasm));

  const register = (command: string, callback: (...args: unknown[]) => unknown) =>
    context.subscriptions.push(vscode.commands.registerCommand(command, callback));
  const uriOf = (value?: unknown) => (value instanceof vscode.Uri ? value : undefined);

  register('arclang.openDiagrams', async (uri?: unknown) => {
    const file = await commands.fileFrom(uriOf(uri));
    if (file) await vscode.commands.executeCommand('arclang.openWorkbench', vscode.Uri.file(file));
  });
  register('arclang.check', (uri?: unknown) => commands.check(uriOf(uri)));
  register('arclang.runGate', (uri?: unknown) => commands.runGate(uriOf(uri)));
  register('arclang.format', () => commands.formatActive());
  register('arclang.transitionPlan', (uri?: unknown) => commands.transitionPlan(uriOf(uri)));
  register('arclang.transitionApply', (uri?: unknown, step?: unknown) => commands.transitionApply(uriOf(uri), typeof step === 'string' ? step : undefined));
  register('arclang.impact', (uri?: unknown, element?: unknown) => commands.impact(uriOf(uri), typeof element === 'string' ? element : undefined));
  register('arclang.diffWithHead', (uri?: unknown) => commands.diffWithHead(uriOf(uri)));
  register('arclang.export', (uri?: unknown) => commands.exportModel(uriOf(uri)));
  register('arclang.showMetamodel', () => commands.showMetamodel());
  register('arclang.showRules', () => commands.showRules());
  register('arclang.restartServer', () => vscode.window.showInformationMessage('ArcLang: the compiler runs in this page, there is no server to restart.'));
  register('arclang.installBinary', () => vscode.window.showInformationMessage(NOT_AVAILABLE('Installing the native compiler')));

  attempt('formatting', () => registerFormatter(context));
  attempt('diagnostics', () => registerWebDiagnostics(context));
  attempt('the workbench', () => registerWorkbench(context));
  attempt('import', () => registerImport(context));
  attempt('tables', () => registerTables(context));
  attempt('impact analysis', () => registerImpact(context));
  attempt('the AI assistant', () => registerAi(context)); // ai-design hook
}

export function deactivate(): void {}
