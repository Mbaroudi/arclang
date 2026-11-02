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
import { registerSimulationWeb } from '../simulation/web';
import { registerSemgit } from '../semgit';
import { registerDebug } from '../debug';
import { registerDocuments } from '../document';
import { registerTraces } from '../traces';
import { setGitRunner } from '../semgit/git';
import { setClipboardImageWriter } from '../workbench/clipboard-image';
import { registerWorkbench } from '../workbench';
import { setThemeModuleLoader } from '../workbench/themes';
import { registerWebDiagnostics } from './diagnostics';
import { createWasmBackend, NOT_AVAILABLE } from './wasm-backend';
import { loadWasm } from './wasm';
import { wasmDebugAdapter } from './debug-adapter';
import { probeRemote } from './remote';

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
  // The viewer's theme validator is bundled here: a worker cannot require a file of the extension.
  // eslint-disable-next-line @typescript-eslint/no-require-imports
  setThemeModuleLoader(() => require('../../media/viewer/arcviz-theme.js'));

  const register = (command: string, callback: (...args: unknown[]) => unknown) =>
    context.subscriptions.push(vscode.commands.registerCommand(command, callback));
  const uriOf = (value?: unknown) => (value instanceof vscode.Uri ? value : undefined);

  register('arclang.openDiagrams', async (uri?: unknown) => {
    const file = await commands.fileFrom(uriOf(uri));
    if (file) await vscode.commands.executeCommand('arclang.openWorkbench', vscode.Uri.file(file));
  });
  register('arclang.check', (uri?: unknown) => commands.check(uriOf(uri)));
  register('arclang.runGate', (uri?: unknown) => commands.runGate(uriOf(uri)));
  register('arclang.safetyReport', (uri?: unknown) => commands.safetyReport(uriOf(uri)));
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

  // git: the signed-in user's own repository (every save is a commit), through the page; signed out, there is none.
  setGitRunner(async (args) => {
    const answer = await Promise.resolve(vscode.commands.executeCommand<{ code: number; stdout: string; stderr: string } | null>('arclangWeb.git.run', args)).catch(() => null);
    return answer ?? { code: 128, stdout: '', stderr: 'fatal: not a git repository (sign in: your work is then a git repository, every save a version)' };
  });

  // An image on the clipboard: the page writes it (the browser's clipboard), the worker cannot.
  setClipboardImageWriter(async (png) => {
    const answer = await Promise.resolve(vscode.commands.executeCommand<string | null>('arclangWeb.clipboard.writeImage', png.toString('base64'))).catch((error: unknown) => String(error));
    if (answer) throw new Error(answer);
  });

  // The compiler online: the page registers its command once the account is known.
  for (const delay of [0, 2000, 6000, 15000]) setTimeout(() => void probeRemote(), delay);

  attempt('formatting', () => registerFormatter(context));
  attempt('diagnostics', () => registerWebDiagnostics(context));
  attempt('the workbench', () => registerWorkbench(context));
  attempt('import', () => registerImport(context));
  attempt('tables', () => registerTables(context));
  attempt('impact analysis', () => registerImpact(context));
  attempt('simulation', () => registerSimulationWeb(context));
  attempt('semantic git', () => registerSemgit(context));
  attempt('documents', () => registerDocuments(context));
  attempt('traces', () => registerTraces(context));
  attempt('the debugger', () => registerDebug(context, wasmDebugAdapter(wasm)));
  attempt('the AI assistant', () => registerAi(context)); // ai-design hook
}

export function deactivate(): void {}
