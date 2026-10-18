// The workbench beside the source: the compiler's diagram viewer with a
// palette, a properties view and a semantic browser, in a webview that talks
// to the extension host only through typed messages. Every gesture becomes
// `arclang edit` requests applied to the text; the text is the model.
import { randomBytes } from 'node:crypto';
import * as path from 'node:path';
import * as vscode from 'vscode';
import { EditRequest, ElementRecord, FromWebview, Snapshot, ToWebview } from '../pure/workbench';
import { applyRequests, refusalMessage } from './edits';
import { metamodelJson, onDidChangeSnapshot, snapshotOf } from './snapshot';
import { elementAt, revealElement } from './tree';

/** The viewer's own files, copied from the compiler at build time (esbuild.mjs). */
const VIEWER_SCRIPTS = [
  'vendor/elk.bundled.js',
  'arcviz-render.js',
  'arcviz-links.js',
  'arcviz-navigation.js',
  'arcviz-collapse.js',
  'arcviz-scope.js',
  'arcviz-export.js',
  'arcviz-stage.js',
  'arcviz-place.js',
  'arcviz-viewer.js',
];

/** The commands the webview may ask the host to run on its document. */
const ALLOWED_COMMANDS = new Set(['arclang.transitionPlan', 'arclang.transitionApply', 'arclang.runGate', 'arclang.check', 'arclang.format']);

const panels = new Map<string, WorkbenchPanel>();

export class WorkbenchPanel {
  private readonly panel: vscode.WebviewPanel;
  private readonly disposables: vscode.Disposable[] = [];
  private ready = false;
  private selected: string | null = null;
  private busy = 0;

  static open(context: vscode.ExtensionContext, document: vscode.TextDocument): WorkbenchPanel {
    const key = document.uri.toString();
    const existing = panels.get(key);
    if (existing) {
      existing.panel.reveal(undefined, true);
      return existing;
    }
    const created = new WorkbenchPanel(context, document);
    panels.set(key, created);
    return created;
  }

  static for(uri: vscode.Uri): WorkbenchPanel | undefined {
    return panels.get(uri.toString());
  }

  private constructor(
    private readonly context: vscode.ExtensionContext,
    private readonly document: vscode.TextDocument,
  ) {
    const media = vscode.Uri.joinPath(context.extensionUri, 'media');
    const dist = vscode.Uri.joinPath(context.extensionUri, 'dist');
    this.panel = vscode.window.createWebviewPanel('arclangWorkbench', `Workbench: ${path.basename(document.uri.fsPath)}`, vscode.ViewColumn.Beside, {
      enableScripts: true,
      retainContextWhenHidden: true,
      localResourceRoots: [media, dist],
      enableCommandUris: false,
    });
    this.panel.iconPath = vscode.Uri.joinPath(media, 'icon.svg');
    this.panel.webview.html = this.html();
    this.disposables.push(
      this.panel.webview.onDidReceiveMessage((message: unknown) => void this.handle(message as FromWebview)),
      onDidChangeSnapshot((uri) => {
        if (uri.toString() === document.uri.toString()) void this.push();
      }),
      vscode.window.onDidChangeTextEditorSelection(async (event) => {
        if (event.textEditor.document.uri.toString() !== document.uri.toString()) return;
        if (event.kind !== vscode.TextEditorSelectionChangeKind.Mouse && event.kind !== vscode.TextEditorSelectionChangeKind.Keyboard) return;
        const snapshot = await snapshotOf(document);
        const element = await elementAt(document, event.selections[0].active, snapshot.elements?.elements ?? []);
        if (element && element.id !== this.selected) this.post({ type: 'select', id: element.id });
      }),
      vscode.workspace.onDidCloseTextDocument((closed) => {
        if (closed.uri.toString() === document.uri.toString()) this.panel.dispose();
      }),
    );
    this.panel.onDidDispose(() => {
      panels.delete(document.uri.toString());
      for (const d of this.disposables) d.dispose();
    });
  }

  /** The element selected in the views, for commands that act on it. */
  get selectedId(): string | null {
    return this.selected;
  }

  select(id: string | null): void {
    this.selected = id;
    this.post({ type: 'select', id });
  }

  private post(message: ToWebview): void {
    void this.panel.webview.postMessage(message);
  }

  private async push(): Promise<void> {
    if (!this.ready) return;
    const [snapshot, metamodel] = await Promise.all([snapshotOf(this.document), metamodelJson()]);
    this.post({ type: 'snapshot', snapshot, metamodel, file: path.basename(this.document.uri.fsPath), selected: this.selected });
  }

  private async handle(message: FromWebview): Promise<void> {
    switch (message.type) {
      case 'ready':
        this.ready = true;
        await this.push();
        return;
      case 'selected':
        this.selected = message.id;
        return;
      case 'reveal': {
        const element = await this.element(message.id);
        if (element) await revealElement(this.document, element);
        return;
      }
      case 'open-text':
        await vscode.window.showTextDocument(this.document, { viewColumn: vscode.ViewColumn.One, preserveFocus: false });
        return;
      case 'ask':
        await vscode.commands.executeCommand('workbench.action.chat.open', {
          query: `@arclang /explain the element ${message.name} (${message.id}, ${message.kind}) of the open model: its role, what it exchanges with, and what the findings say about it.`,
        });
        return;
      case 'command':
        if (ALLOWED_COMMANDS.has(message.command)) await vscode.commands.executeCommand(message.command, this.document.uri);
        return;
      case 'edit':
        await this.edit(message.requests, message.label);
        return;
      default:
        return;
    }
  }

  private async edit(requests: EditRequest[], label: string): Promise<void> {
    this.busy += 1;
    this.post({ type: 'busy', busy: true, label });
    try {
      const outcome = await applyRequests(this.document, requests);
      if (!outcome.ok) {
        this.post({ type: 'edit-result', ok: false, label, error: outcome.error });
        vscode.window.setStatusBarMessage(refusalMessage(label, outcome), 6000);
        return;
      }
      this.post({ type: 'edit-result', ok: true, label });
      // The edited document gets its snapshot right away, not after the typing pause.
      void snapshotOf(this.document);
    } finally {
      this.busy -= 1;
      if (this.busy === 0) this.post({ type: 'busy', busy: false });
    }
  }

  private async element(id: string): Promise<ElementRecord | null> {
    const snapshot: Snapshot = await snapshotOf(this.document);
    const records = snapshot.elements?.elements ?? [];
    return records.find((e) => e.id === id) ?? records.find((e) => e.name === id) ?? null;
  }

  private html(): string {
    const webview = this.panel.webview;
    const nonce = randomBytes(16).toString('base64url');
    const uri = (...segments: string[]) => webview.asWebviewUri(vscode.Uri.joinPath(this.context.extensionUri, ...segments)).toString();
    const scripts = [...VIEWER_SCRIPTS.map((file) => uri('media', 'viewer', ...file.split('/'))), uri('dist', 'webview.js')];
    // Only the extension's own files run or style the page, scripts marked
    // with a fresh nonce; the viewer draws with inline styles on its SVG,
    // hence 'unsafe-inline' for styles only. No network, no frames, no forms.
    const csp = [
      "default-src 'none'",
      `img-src ${webview.cspSource} data: blob:`,
      `style-src ${webview.cspSource} 'unsafe-inline'`,
      `font-src ${webview.cspSource} data:`,
      `script-src 'nonce-${nonce}'`,
      'worker-src blob:',
      "connect-src 'none'",
      "frame-src 'none'",
      "form-action 'none'",
      "base-uri 'none'",
    ].join('; ');
    return `<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta http-equiv="Content-Security-Policy" content="${csp}">
<meta name="viewport" content="width=device-width, initial-scale=1">
<link rel="stylesheet" href="${uri('media', 'viewer', 'arcviz-viewer.css')}">
<link rel="stylesheet" href="${uri('media', 'workbench.css')}">
<title>ArcLang workbench</title>
</head>
<body>
<div id="workbench" data-file="${escapeAttribute(path.basename(this.document.uri.fsPath))}"></div>
${scripts.map((src) => `<script nonce="${nonce}" src="${src}"></script>`).join('\n')}
</body>
</html>`;
  }
}

function escapeAttribute(text: string): string {
  return text.replace(/[&<>"]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' })[c] ?? c);
}
