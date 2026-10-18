// The language client: `arclang lsp --stdio` gives diagnostics, outline,
// completion, hover, definition and rename.
import * as vscode from 'vscode';
import { LanguageClient } from 'vscode-languageclient/node';
import { binary, output, workspaceTrusted } from './cli';

let client: LanguageClient | null = null;

export async function startClient(context: vscode.ExtensionContext): Promise<void> {
  if (client || !workspaceTrusted()) return;
  // No `transport`: with TransportKind.stdio the client would add a second
  // `--stdio`, which the compiler's argument parser rejects.
  const serverOptions = { command: binary(), args: ['lsp', '--stdio'] };
  const clientOptions = {
    documentSelector: [{ scheme: 'file', language: 'arclang' }, { scheme: 'untitled', language: 'arclang' }],
    synchronize: { fileEvents: vscode.workspace.createFileSystemWatcher('**/*.arc') },
  };
  const started = new LanguageClient('arclang', 'ArcLang language server', serverOptions, clientOptions);
  try {
    await started.start();
    client = started;
    context.subscriptions.push(started);
  } catch (error) {
    const message = error instanceof Error ? `${error.message}\n${error.stack ?? ''}` : String(error);
    console.error(`ArcLang: cannot start ${binary()} lsp --stdio: ${message}`);
    output().appendLine(`cannot start ${binary()} lsp --stdio: ${message}`);
    vscode.window.showErrorMessage(`ArcLang: cannot start \`${binary()} lsp --stdio\` (${message.split('\n')[0]}).`);
  }
}

export async function stopClient(): Promise<void> {
  if (!client) return;
  const stopping = client;
  client = null;
  await stopping.stop();
}

export async function restartClient(context: vscode.ExtensionContext): Promise<void> {
  await stopClient();
  await startClient(context);
}

export function clientRunning(): boolean {
  return client !== null;
}
