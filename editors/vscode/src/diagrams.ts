// The compiler's own diagram viewer (`arclang diagram -f html`) beside the
// source, redrawn when the model is saved.
import { randomBytes } from 'node:crypto';
import * as fs from 'node:fs';
import * as path from 'node:path';
import * as vscode from 'vscode';
import { failureText, run, tempDir } from './cli';
import { nonceFrom, secureViewerHtml } from './pure/arc';

let panel: vscode.WebviewPanel | null = null;
let shownFile: string | null = null;

async function render(file: string): Promise<{ html?: string; error?: string }> {
  // The viewer is written in a private temporary directory, never at a
  // predictable path of a shared one.
  const dir = tempDir();
  const out = path.join(dir, 'diagrams.html');
  try {
    const result = await run(['diagram', file, '-f', 'html', '-o', out], { cwd: path.dirname(file) });
    if (result.code !== 0 || !fs.existsSync(out)) return { error: failureText(result) };
    return { html: fs.readFileSync(out, 'utf8') };
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
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
      // The page is self-contained: no access to local resources, no command URIs.
      localResourceRoots: [],
      enableCommandUris: false,
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
  // and runs inside the webview's own origin, under a content security policy
  // that allows only its own scripts, marked with a fresh nonce. The webview
  // has no message channel to the extension.
  panel.webview.html = secureViewerHtml(outcome.html, nonceFrom(randomBytes(16)));
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
