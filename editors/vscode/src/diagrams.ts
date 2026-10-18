// The compiler's own diagram viewer (`arclang diagram -f html`) beside the
// source, redrawn when the model is saved.
import * as fs from 'node:fs';
import * as os from 'node:os';
import * as path from 'node:path';
import * as vscode from 'vscode';
import { failureText, run } from './cli';

let panel: vscode.WebviewPanel | null = null;
let shownFile: string | null = null;

async function render(file: string): Promise<{ html?: string; error?: string }> {
  const out = path.join(os.tmpdir(), `arclang-diagrams-${process.pid}-${Date.now()}.html`);
  const result = await run(['diagram', file, '-f', 'html', '-o', out], { cwd: path.dirname(file) });
  if (result.code !== 0 || !fs.existsSync(out)) return { error: failureText(result) };
  const html = fs.readFileSync(out, 'utf8');
  fs.rmSync(out, { force: true });
  return { html };
}

function escapeHtml(text: string): string {
  return text.replace(/[&<>"]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' })[c] ?? c);
}

export async function showDiagrams(file: string): Promise<void> {
  shownFile = file;
  if (!panel) {
    panel = vscode.window.createWebviewPanel('arclangDiagrams', 'ArcLang diagrams', vscode.ViewColumn.Beside, {
      enableScripts: true,
      retainContextWhenHidden: true,
    });
    panel.onDidDispose(() => {
      panel = null;
      shownFile = null;
    });
  }
  panel.title = `Diagrams: ${path.basename(file)}`;
  const outcome = await render(file);
  if (outcome.error !== undefined || outcome.html === undefined) {
    panel.webview.html = `<!doctype html><body style="font-family:sans-serif;padding:1rem"><h2>No diagrams</h2><pre style="white-space:pre-wrap">${escapeHtml(outcome.error ?? '')}</pre></body>`;
    return;
  }
  // The page is self-contained (the layout engine is inlined by native builds)
  // and runs inside the webview's own origin.
  panel.webview.html = outcome.html;
  panel.reveal(vscode.ViewColumn.Beside, true);
}

export function shownDiagramFile(): string | null {
  return shownFile;
}

export function registerDiagramRefresh(context: vscode.ExtensionContext): void {
  context.subscriptions.push(
    vscode.workspace.onDidSaveTextDocument(async (document) => {
      const refresh = vscode.workspace.getConfiguration('arclang').get<boolean>('diagrams.refreshOnSave', true);
      if (refresh && panel && shownFile && document.uri.fsPath === shownFile) await showDiagrams(shownFile);
    }),
  );
}
