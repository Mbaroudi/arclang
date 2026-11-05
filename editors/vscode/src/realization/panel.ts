// The realization ladder of a model in its own editor tab: what each stage
// says of each requirement, from the `realization` field of the workbench
// snapshot (the compiler reads the cached verdicts; nothing runs). The view
// follows the text as it is edited, and the cache as the commands fill it:
// a run of `arclang sil` in the terminal refreshes it when it writes its
// verdict. "Run" sends the next command to a terminal.
import { randomBytes } from 'node:crypto';
import * as path from 'node:path';
import * as vscode from 'vscode';
import { FromRealizationView, Ladder, onModel, ToRealizationView, withBinary } from '../pure/realization';
import { goToSource } from '../workbench/go-to-source'; // nav hook: the one way to the source
import { forgetSnapshot, onDidChangeSnapshot, snapshotOf } from '../workbench/snapshot';
import { documentDir } from '../workbench/source';

export const REALIZATION_VIEW_TYPE = 'arclang.realization';
const REFRESH_MS = 400;

const panels = new Set<RealizationPanel>();
let active: RealizationPanel | null = null;

function binary(): string {
  return vscode.workspace.getConfiguration('arclang').get<string>('binary', 'arclang');
}

export class RealizationPanel implements vscode.Disposable {
  private timer: NodeJS.Timeout | undefined;
  private terminal: vscode.Terminal | null = null;
  private readonly disposables: vscode.Disposable[] = [];
  private ladders: Ladder[] = [];

  /** Show the ladder of a document: the panel of the document, revealed. */
  static async open(context: vscode.ExtensionContext, document: vscode.TextDocument): Promise<RealizationPanel> {
    const existing = [...panels].find((p) => p.document.uri.toString() === document.uri.toString());
    if (existing) {
      existing.panel.reveal(undefined, false);
      void existing.refresh();
      return existing;
    }
    const panel = vscode.window.createWebviewPanel(REALIZATION_VIEW_TYPE, `Realization: ${path.basename(document.uri.fsPath)}`, { viewColumn: vscode.ViewColumn.Beside, preserveFocus: false }, {
      enableScripts: true,
      retainContextWhenHidden: true,
      localResourceRoots: [vscode.Uri.joinPath(context.extensionUri, 'dist'), vscode.Uri.joinPath(context.extensionUri, 'media')],
    });
    return new RealizationPanel(context, document, panel);
  }

  static activePanel(): RealizationPanel | null {
    return active;
  }

  static all(): RealizationPanel[] {
    return [...panels];
  }

  private constructor(
    private readonly context: vscode.ExtensionContext,
    readonly document: vscode.TextDocument,
    readonly panel: vscode.WebviewPanel,
  ) {
    panels.add(this);
    active = this;
    panel.iconPath = new vscode.ThemeIcon('list-tree');
    panel.webview.html = this.html();
    const uri = document.uri.toString();
    this.disposables.push(
      panel.webview.onDidReceiveMessage((message: FromRealizationView) => void this.handle(message)),
      panel.onDidChangeViewState(() => {
        if (panel.active) active = this;
      }),
      panel.onDidDispose(() => this.dispose()),
      onDidChangeSnapshot((changed) => {
        if (changed.toString() === uri) this.schedule();
      }),
      vscode.workspace.onDidChangeTextDocument((event) => {
        if (event.document.uri.toString() === uri && event.contentChanges.length) this.post({ type: 'busy' });
      }),
      vscode.window.onDidCloseTerminal((terminal) => {
        if (terminal === this.terminal) this.terminal = null;
      }),
    );
    // The verdicts the commands write beside the model: a new one is a new ladder.
    const dir = documentDir(document);
    if (document.uri.scheme === 'file' && dir) {
      const watcher = vscode.workspace.createFileSystemWatcher(new vscode.RelativePattern(vscode.Uri.file(dir), '.arclang/sim-cache/**'));
      const changed = () => {
        forgetSnapshot(document.uri);
        this.schedule();
      };
      this.disposables.push(watcher, watcher.onDidChange(changed), watcher.onDidCreate(changed), watcher.onDidDelete(changed));
    }
  }

  get current(): Ladder[] {
    return this.ladders;
  }

  dispose(): void {
    panels.delete(this);
    if (active === this) active = null;
    if (this.timer) clearTimeout(this.timer);
    this.disposables.forEach((d) => d.dispose());
  }

  private post(message: ToRealizationView): void {
    void this.panel.webview.postMessage(message);
  }

  schedule(): void {
    if (this.timer) clearTimeout(this.timer);
    this.timer = setTimeout(() => void this.refresh(), REFRESH_MS);
  }

  async refresh(): Promise<void> {
    const file = path.basename(this.document.uri.fsPath);
    this.post({ type: 'busy' });
    const started = Date.now();
    const snapshot = await snapshotOf(this.document);
    if (!snapshot.success) {
      this.ladders = [];
      return this.post({ type: 'error', error: snapshot.errors.join('\n') || 'the model does not compile', file });
    }
    // The compiler saw a copy of the text: the commands run on the file itself.
    this.ladders = this.document.uri.scheme === 'file' ? onModel(snapshot.realization ?? [], this.document.uri.fsPath) : (snapshot.realization ?? []);
    this.post({ type: 'ladders', ladders: this.ladders, file, binary: binary(), computedMs: Date.now() - started });
  }

  private async handle(message: FromRealizationView): Promise<void> {
    switch (message.type) {
      case 'ready':
        return this.refresh();
      case 'refresh':
        forgetSnapshot(this.document.uri);
        return this.refresh();
      case 'run':
        return this.run(message.command);
      case 'realize': {
        const file = this.document.uri.scheme === 'file' ? this.document.uri.fsPath : null;
        if (!file) return this.post({ type: 'result', ok: false, message: 'Save the model to a file first' });
        const quoted = /[\s"']/.test(file) ? `"${file.replace(/"/g, '\\"')}"` : file;
        return this.run(`arclang realize ${quoted}${message.requirement ? ` --requirement ${message.requirement}` : ''}`);
      }
      case 'reveal':
        return this.reveal(message.id);
      case 'table':
        await vscode.commands.executeCommand('arclang.tables.new', { uri: this.document.uri, kind: 'realization' });
        return;
    }
  }

  /** Run a `arclang …` command the ladder names, in a terminal beside the model. */
  private run(command: string): void {
    if (!command.startsWith('arclang ')) return this.post({ type: 'result', ok: false, message: `Not a compiler command: ${command}` });
    if (!this.terminal) this.terminal = vscode.window.createTerminal({ name: 'ArcLang realization', cwd: documentDir(this.document) });
    this.terminal.show(true);
    this.terminal.sendText(withBinary(command, binary()), true);
    this.post({ type: 'result', ok: true, message: `Running: ${command}. The ladder refreshes when its verdict is written.` });
  }

  private async reveal(id: string): Promise<void> {
    const snapshot = await snapshotOf(this.document);
    const records = snapshot.elements?.elements ?? [];
    const record = records.find((e) => e.id === id) ?? records.find((e) => e.name === id);
    if (record) await goToSource(this.document, { id: record.id, name: record.name }, { preserveFocus: true });
  }

  private html(): string {
    const webview = this.panel.webview;
    const nonce = randomBytes(16).toString('base64url');
    const uri = (...segments: string[]) => webview.asWebviewUri(vscode.Uri.joinPath(this.context.extensionUri, ...segments)).toString();
    const csp = ["default-src 'none'", `style-src ${webview.cspSource}`, `font-src ${webview.cspSource}`, `img-src ${webview.cspSource} data:`, `script-src 'nonce-${nonce}'`, "connect-src 'none'", "frame-src 'none'", "form-action 'none'", "base-uri 'none'"].join('; ');
    return `<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta http-equiv="Content-Security-Policy" content="${csp}">
<meta name="viewport" content="width=device-width, initial-scale=1">
<link rel="stylesheet" href="${uri('media', 'realization.css')}">
<title>ArcLang realization ladder</title>
</head>
<body>
<main id="realization" aria-label="Realization ladder"></main>
<script nonce="${nonce}" src="${uri('dist', 'realization.js')}"></script>
</body>
</html>`;
  }
}
