// ArcLang for VS Code. Everything comes from the `arclang` binary: the
// language server for what the editor shows, the CLI for commands and tools,
// the diagram viewer, and the compiler's verdict on what the assistant writes.
import * as vscode from 'vscode';
import { registerParticipant } from './ai/participant';
import { registerTools } from './ai/tools';
import { installBinary, warnIfMissing } from './binary';
import { restartClient, startClient, stopClient } from './client';
import * as commands from './commands';
import { registerDiagramRefresh, showDiagrams } from './diagrams';
import { registerFormatter } from './format';
import { registerMcpProvider } from './mcp';

export async function activate(context: vscode.ExtensionContext): Promise<void> {
  const register = (command: string, callback: (...args: unknown[]) => unknown) =>
    context.subscriptions.push(vscode.commands.registerCommand(command, callback));

  register('arclang.openDiagrams', async (uri?: unknown) => {
    const file = await commands.fileFrom(uri instanceof vscode.Uri ? uri : undefined);
    if (file) await showDiagrams(file);
  });
  register('arclang.check', (uri?: unknown) => commands.check(uri instanceof vscode.Uri ? uri : undefined));
  register('arclang.runGate', (uri?: unknown) => commands.runGate(uri instanceof vscode.Uri ? uri : undefined));
  register('arclang.format', () => commands.formatActive());
  register('arclang.transitionPlan', (uri?: unknown) => commands.transitionPlan(uri instanceof vscode.Uri ? uri : undefined));
  register('arclang.transitionApply', (uri?: unknown, step?: unknown) =>
    commands.transitionApply(uri instanceof vscode.Uri ? uri : undefined, typeof step === 'string' ? step : undefined),
  );
  register('arclang.impact', (uri?: unknown, element?: unknown) =>
    commands.impact(uri instanceof vscode.Uri ? uri : undefined, typeof element === 'string' ? element : undefined),
  );
  register('arclang.diffWithHead', (uri?: unknown) => commands.diffWithHead(uri instanceof vscode.Uri ? uri : undefined));
  register('arclang.export', (uri?: unknown) => commands.exportModel(uri instanceof vscode.Uri ? uri : undefined));
  register('arclang.import', () => commands.importModel());
  register('arclang.showMetamodel', () => commands.showMetamodel());
  register('arclang.showRules', () => commands.showRules());
  register('arclang.restartServer', () => restartClient(context));
  register('arclang.installBinary', () => installBinary());

  registerFormatter(context);
  registerDiagramRefresh(context);
  registerTools(context);
  registerParticipant(context);
  registerMcpProvider(context);

  // The language server starts with the first .arc document, not at startup.
  const startForArc = (document: vscode.TextDocument) => {
    if (document.languageId === 'arclang') void startClient(context);
  };
  context.subscriptions.push(vscode.workspace.onDidOpenTextDocument(startForArc));
  context.subscriptions.push(
    vscode.workspace.onDidChangeConfiguration(async (event) => {
      if (event.affectsConfiguration('arclang.binary')) await restartClient(context);
    }),
  );
  if (vscode.workspace.textDocuments.some((d) => d.languageId === 'arclang')) {
    if (await warnIfMissing()) await startClient(context);
  }
}

export async function deactivate(): Promise<void> {
  await stopClient();
}
