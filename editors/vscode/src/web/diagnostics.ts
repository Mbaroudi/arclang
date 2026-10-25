// Diagnostics in the browser: no language server, so the editor's problems
// come from the compiler's `check` on the text, a moment after typing stops.
import * as vscode from 'vscode';
import { run } from '../cli';
import { parseCheckReport } from '../pure/arc';

const DELAY_MS = 350;

function lineOf(message: string): number {
  const at = /(?:line|:)\s*(\d+)(?::(\d+))?/i.exec(message);
  return at ? Math.max(0, Number(at[1]) - 1) : 0;
}

export function registerWebDiagnostics(context: vscode.ExtensionContext): void {
  const collection = vscode.languages.createDiagnosticCollection('arclang');
  const timers = new Map<string, ReturnType<typeof setTimeout>>();
  const versions = new Map<string, number>();

  async function refresh(document: vscode.TextDocument): Promise<void> {
    if (document.languageId !== 'arclang') return;
    const version = document.version;
    versions.set(document.uri.toString(), version);
    const tmp = `/tmp/check-${version}-${Math.random().toString(36).slice(2)}.arc`;
    const vfs = await import('./vfs');
    vfs.write(tmp, document.getText());
    const result = await run(['check', '--json', tmp], {});
    vfs.remove(tmp);
    if (versions.get(document.uri.toString()) !== version) return;
    const report = parseCheckReport(result.stdout);
    const diagnostics: vscode.Diagnostic[] = [];
    if (!report) {
      collection.set(document.uri, []);
      return;
    }
    for (const error of report.errors) {
      const line = Math.min(lineOf(error), document.lineCount - 1);
      const range = document.lineAt(line).range;
      const diagnostic = new vscode.Diagnostic(range, error, vscode.DiagnosticSeverity.Error);
      diagnostic.source = 'arclang';
      diagnostics.push(diagnostic);
    }
    for (const finding of report.findings) {
      const severity = finding.severity === 'error' ? vscode.DiagnosticSeverity.Error : finding.severity === 'info' ? vscode.DiagnosticSeverity.Information : vscode.DiagnosticSeverity.Warning;
      const element = finding.element ?? '';
      const at = element ? document.getText().indexOf(element) : -1;
      const range = at >= 0 ? new vscode.Range(document.positionAt(at), document.positionAt(at + element.length)) : document.lineAt(0).range;
      const diagnostic = new vscode.Diagnostic(range, finding.message, severity);
      diagnostic.source = 'arclang';
      diagnostic.code = finding.rule;
      diagnostics.push(diagnostic);
    }
    collection.set(document.uri, diagnostics);
  }

  const schedule = (document: vscode.TextDocument) => {
    const key = document.uri.toString();
    clearTimeout(timers.get(key));
    timers.set(key, setTimeout(() => void refresh(document), DELAY_MS));
  };
  context.subscriptions.push(
    collection,
    vscode.workspace.onDidOpenTextDocument(schedule),
    vscode.workspace.onDidChangeTextDocument((event) => schedule(event.document)),
    vscode.workspace.onDidCloseTextDocument((document) => collection.delete(document.uri)),
  );
  vscode.workspace.textDocuments.forEach(schedule);
}
