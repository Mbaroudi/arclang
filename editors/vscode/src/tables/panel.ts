// A table of a model in its own editor tab: a webview panel, so it sits
// beside a diagram and moves into a new window with VS Code's own command.
// The table is computed by `arclang table` on the text in the editor, saved
// or not, again after every change (debounced); every edit made from it is
// an `arclang edit` request, one undoable step of the document.
import { randomBytes } from 'node:crypto';
import * as fs from 'node:fs';
import * as path from 'node:path';
import * as vscode from 'vscode';
import { run, tempDir, withTempFile, workspaceTrusted, UNTRUSTED_MESSAGE } from '../cli';
import { History } from '../pure/history';
import { FromTableView, parseTable, removalRequests, TableData, TableSpec, tableArgs, ToTableView, traceRequest } from '../pure/tables';
import { EditRequest } from '../pure/workbench';
import { applyRequests, EditQueue, refusalMessage } from '../workbench/edits';
import { WorkbenchPanel } from '../workbench/panel';
import { snapshotOf } from '../workbench/snapshot';
import { arcTextOf, documentDir, replaceWhole } from '../workbench/source';
import { graphsDrawing } from '../workbench/tree';
import { goToSource, highlightElement } from '../workbench/go-to-source'; // nav hook

export const TABLE_VIEW_TYPE = 'arclang.table';
const REFRESH_MS = 250;

const panels = new Set<TablePanel>();
let active: TablePanel | null = null;

function gateStandard(): string {
  return vscode.workspace.getConfiguration('arclang').get<string>('gate.standard', 'iso26262');
}

export class TablePanel implements vscode.Disposable {
  private table: TableData | null = null;
  private timer: NodeJS.Timeout | undefined;
  private computing: Promise<void> | null = null;
  private again = false;
  private readonly edits = new EditQueue();
  private readonly history = new History();
  private own = 0;
  private readonly disposables: vscode.Disposable[] = [];

  /** Open a table of a document: a declared one by name, else the spec given. */
  static async open(context: vscode.ExtensionContext, document: vscode.TextDocument, spec: TableSpec, declared: string | null, column = vscode.ViewColumn.Beside): Promise<TablePanel> {
    const key = declared ?? JSON.stringify(spec);
    const existing = [...panels].find((p) => p.document.uri.toString() === document.uri.toString() && p.key === key);
    if (existing) {
      existing.panel.reveal(undefined, false);
      return existing;
    }
    const panel = vscode.window.createWebviewPanel(TABLE_VIEW_TYPE, declared ?? spec.name ?? 'Table', { viewColumn: column, preserveFocus: false }, {
      enableScripts: true,
      retainContextWhenHidden: true,
      localResourceRoots: [vscode.Uri.joinPath(context.extensionUri, 'dist'), vscode.Uri.joinPath(context.extensionUri, 'media')],
    });
    return new TablePanel(context, document, panel, spec, declared);
  }

  static activePanel(): TablePanel | null {
    return active;
  }

  static all(): TablePanel[] {
    return [...panels];
  }

  private constructor(
    private readonly context: vscode.ExtensionContext,
    readonly document: vscode.TextDocument,
    readonly panel: vscode.WebviewPanel,
    private spec: TableSpec,
    private declared: string | null,
  ) {
    panels.add(this);
    active = this;
    panel.iconPath = new vscode.ThemeIcon('table');
    panel.webview.html = this.html();
    this.disposables.push(
      panel.webview.onDidReceiveMessage((message: FromTableView) => void this.handle(message)),
      panel.onDidChangeViewState(() => {
        if (panel.active) active = this;
        void vscode.commands.executeCommand('setContext', 'arclang.tableActive', panel.active);
      }),
      panel.onDidDispose(() => this.dispose()),
      vscode.workspace.onDidChangeTextDocument((event) => {
        if (event.document.uri.toString() !== this.document.uri.toString() || event.contentChanges.length === 0) return;
        // A change of the text made elsewhere: this view's steps no longer apply.
        if (this.own === 0 && (this.history.canUndo || this.history.canRedo)) {
          this.history.clear();
          this.announce();
        }
        this.schedule();
      }),
    );
    void vscode.commands.executeCommand('setContext', 'arclang.tableActive', true);
  }

  /** What the panel shows: a declared table's name, else its spec. */
  get key(): string {
    return this.declared ?? JSON.stringify(this.spec);
  }

  dispose(): void {
    panels.delete(this);
    if (active === this) active = null;
    if (this.timer) clearTimeout(this.timer);
    this.disposables.forEach((d) => d.dispose());
    void vscode.commands.executeCommand('setContext', 'arclang.tableActive', false);
  }

  private post(message: ToTableView): void {
    void this.panel.webview.postMessage(message);
  }

  /** Compute again a moment after the last change. */
  schedule(): void {
    if (this.timer) clearTimeout(this.timer);
    this.timer = setTimeout(() => void this.refresh(), REFRESH_MS);
  }

  /** Compute the table on the document's current text; one run at a time. */
  async refresh(): Promise<void> {
    if (this.computing) {
      this.again = true;
      return this.computing;
    }
    this.computing = this.compute().finally(() => {
      this.computing = null;
      if (this.again) {
        this.again = false;
        void this.refresh();
      }
    });
    return this.computing;
  }

  private async compute(): Promise<void> {
    const file = path.basename(this.document.uri.fsPath);
    if (!workspaceTrusted()) return this.post({ type: 'error', error: UNTRUSTED_MESSAGE, file });
    this.post({ type: 'busy' });
    const source = await arcTextOf(this.document);
    if (!source.ok) return this.post({ type: 'error', error: source.error, file });
    const started = Date.now();
    const name = file.replace(/\.sysml$/i, '.arc') || 'model.arc';
    const answer = await withTempFile(source.text, name, async (temp) => run(tableArgs(temp, this.spec, this.declared, gateStandard()), { cwd: documentDir(this.document) }));
    const parsed = answer.code === 0 ? parseTable(answer.stdout) : { ok: false as const, error: (answer.stderr || answer.stdout).trim().replace(/^Error:\s*/, '') };
    if (!parsed.ok) {
      // A declared table renamed or removed: keep showing it by its spec.
      if (this.declared && /declares no table/.test(parsed.error) && this.table) {
        this.declared = null;
        this.spec = { ...this.table.spec, name: this.table.name };
        return this.compute();
      }
      return this.post({ type: 'error', error: parsed.error, file });
    }
    this.table = parsed.table;
    this.panel.title = parsed.table.name;
    const { undo, redo } = this.history.labels();
    this.post({ type: 'table', table: parsed.table, computedMs: Date.now() - started, file, history: { undo, redo } });
  }

  private announce(): void {
    const { undo, redo } = this.history.labels();
    this.post({ type: 'history', undo, redo });
  }

  /** Apply requests as one step of the document and of this view's history. */
  private async edit(requests: EditRequest[], label: string): Promise<boolean> {
    return this.edits.run(async () => {
      this.own += 1;
      try {
        const outcome = await applyRequests(this.document, requests);
        if (!outcome.ok) {
          this.post({ type: 'result', ok: false, message: refusalMessage(label, outcome) });
          this.schedule();
          return false;
        }
        if (outcome.applied && outcome.before !== undefined && outcome.after !== undefined) this.history.recordModel(label, outcome.before, outcome.after, Date.now());
        this.post({ type: 'result', ok: true, message: outcome.applied ? `Done: ${label}` : `Nothing to change: ${label}` });
        this.announce();
        await this.refresh();
        return true;
      } finally {
        setTimeout(() => (this.own -= 1), 50);
      }
    });
  }

  async undo(): Promise<void> {
    await this.step('undo');
  }

  async redo(): Promise<void> {
    await this.step('redo');
  }

  private async step(way: 'undo' | 'redo'): Promise<void> {
    const entry = way === 'undo' ? this.history.undo() : this.history.redo();
    if (!entry || entry.kind !== 'model') {
      if (way === 'undo') this.post({ type: 'result', ok: true, message: 'Nothing to undo from this table (⌘Z in the text editor undoes the text)' });
      return this.announce();
    }
    const [from, to] = way === 'undo' ? [entry.after, entry.before] : [entry.before, entry.after];
    if (this.document.isClosed || this.document.getText() !== from) {
      this.history.clear();
      this.post({ type: 'result', ok: false, message: `The text changed since "${entry.label}": this table's history was dropped` });
      return this.announce();
    }
    this.own += 1;
    try {
      const applied = await replaceWhole(this.document, to);
      this.post({ type: 'result', ok: applied, message: applied ? `${way === 'undo' ? 'Undone' : 'Redone'}: ${entry.label}` : `VS Code did not apply the ${way}` });
    } finally {
      setTimeout(() => (this.own -= 1), 50);
    }
    this.announce();
    await this.refresh();
  }

  private async handle(message: FromTableView): Promise<void> {
    switch (message.type) {
      case 'ready':
        return this.refresh();
      case 'typing':
        return void vscode.commands.executeCommand('setContext', 'arclang.tableTyping', message.on);
      case 'undo':
        return this.undo();
      case 'redo':
        return this.redo();
      case 'reveal':
        return this.reveal(message.id, message.name, message.open);
      case 'edit': {
        const request: EditRequest = message.edit.op === 'rename' ? { op: 'rename', element: message.edit.element, name: JSON.parse(message.value) as string } : { op: 'set', element: message.edit.element, key: message.edit.key ?? '', value: message.value };
        await this.edit([request], message.label);
        return;
      }
      case 'add-trace': {
        const matrix = this.table?.matrix;
        const request = matrix ? traceRequest(matrix, message.row, message.column, message.kind) : null;
        if (!request) return this.post({ type: 'result', ok: false, message: 'No trace can be created in this cell' });
        await this.edit([request], `trace ${request.from} ${request.kind} ${request.to}`);
        return;
      }
      case 'remove-trace':
        return this.removeTrace(message.row, message.column);
      case 'export':
        return this.export(message.format);
      case 'copied':
        if (!message.ok) await vscode.env.clipboard.writeText(message.tsv);
        vscode.window.setStatusBarMessage(`ArcLang: copied ${message.rows} row${message.rows === 1 ? '' : 's'} as a table`, 4000);
        return;
      case 'save-declaration':
        return this.saveDeclaration(message.spec);
      case 'spec':
        this.spec = { ...message.spec, name: this.declared ?? message.spec.name ?? this.table?.name ?? null };
        // A declared table changed in the view is shown by its spec: the declaration stays as written.
        this.declared = null;
        return this.refresh();
    }
  }

  private async removeTrace(row: number, column: number): Promise<void> {
    const matrix = this.table?.matrix;
    const cell = matrix?.cells.find((c) => c.row === row && c.column === column);
    if (!matrix || !cell) return;
    const { requests, kept } = removalRequests(cell);
    const between = `${matrix.rows[row]?.name} and ${matrix.columns[column]?.name}`;
    if (requests.length === 0) {
      const how = kept.map((l) => `${l.kind} (${l.source})`).join(', ');
      return this.post({ type: 'result', ok: false, message: `${how} between ${between} is not a trace declaration: change it where it is written` });
    }
    const what = requests.map((r) => `${r.from} ${r.kind} ${r.to}`).join(', ');
    const choice = await vscode.window.showWarningMessage(`Remove the trace${requests.length > 1 ? 's' : ''} ${what}?`, { modal: true, detail: kept.length ? `${kept.length} other link${kept.length > 1 ? 's' : ''} of this cell (${kept.map((l) => l.kind).join(', ')}) stay${kept.length > 1 ? '' : 's'}: ${kept.length > 1 ? 'they are' : 'it is'} not written as a trace.` : 'Undo (⌘Z) takes it back.' }, 'Remove');
    if (choice !== 'Remove') return;
    await this.edit(requests, `remove ${what}`);
  }

  private async export(format: 'csv' | 'xlsx' | 'html' | 'md'): Promise<void> {
    if (!this.table) return;
    const base = this.table.name.replace(/[^\p{L}\p{N} _-]+/gu, '').trim().replace(/\s+/g, '_') || 'table';
    const folder = this.document.uri.scheme === 'file' ? vscode.Uri.file(path.dirname(this.document.uri.fsPath)) : undefined;
    const filters: Record<string, Record<string, string[]>> = { csv: { CSV: ['csv'] }, xlsx: { 'Excel workbook': ['xlsx'] }, html: { HTML: ['html'] }, md: { Markdown: ['md'] } };
    const target = await vscode.window.showSaveDialog({ defaultUri: folder ? vscode.Uri.joinPath(folder, `${base}.${format}`) : undefined, filters: filters[format], saveLabel: `Export ${format.toUpperCase()}` });
    if (!target) return;
    const source = await arcTextOf(this.document);
    if (!source.ok) return this.post({ type: 'result', ok: false, message: source.error });
    const dir = tempDir();
    try {
      const model = path.join(dir, path.basename(this.document.uri.fsPath).replace(/\.sysml$/i, '.arc') || 'model.arc');
      fs.writeFileSync(model, source.text, { mode: 0o600 });
      const out = path.join(dir, `table.${format}`);
      const args = tableArgs(model, this.spec, this.declared, gateStandard()).map((a, i, all) => (all[i - 1] === '-f' ? format : a));
      const result = await run([...args, '-o', out], { cwd: documentDir(this.document) });
      if (result.code !== 0 || !fs.existsSync(out)) return this.post({ type: 'result', ok: false, message: (result.stderr || result.stdout).trim() || 'the export failed' });
      await vscode.workspace.fs.writeFile(target, fs.readFileSync(out));
      this.post({ type: 'result', ok: true, message: `Exported ${path.basename(target.fsPath)}` });
      vscode.window.setStatusBarMessage(`ArcLang: exported ${target.fsPath}`, 5000);
    } finally {
      fs.rmSync(dir, { recursive: true, force: true });
    }
  }

  private async saveDeclaration(spec: TableSpec): Promise<void> {
    const proposed = spec.name ?? this.table?.name ?? 'Table';
    const name = await vscode.window.showInputBox({ prompt: 'Name of the table declared in the model', value: proposed, validateInput: (v) => (v.trim() ? null : 'a name is needed') });
    if (!name) return;
    const named = { ...spec, name: name.trim(), id: null };
    const text = await withTempFile('model M {}\n', 'model.arc', async (file) => run([...tableArgs(file, named, null, gateStandard()), '--declaration']));
    if (text.code !== 0) return this.post({ type: 'result', ok: false, message: (text.stderr || text.stdout).trim() });
    const ok = await this.edit([{ op: 'add_raw', declaration: text.stdout.trim(), top_level: true }], `declare table "${name.trim()}"`);
    if (ok) {
      this.declared = name.trim();
      this.spec = named;
      await this.refresh();
    }
  }

  /** Show an element in the text and, when the workbench is open, in its diagram; `open` opens the workbench. */
  private async reveal(id: string, name: string, open: boolean): Promise<void> {
    // nav hook: a click on a row only selects (the text is highlighted if shown); a double-click goes to the source
    if (open) await goToSource(this.document, { id, name });
    else await highlightElement(this.document, { id, name });
    const snapshot = await snapshotOf(this.document);
    const record = (snapshot.elements?.elements ?? []).find((e) => e.id === id);
    const drawn = record ? graphsDrawing(snapshot.diagrams?.graphs ?? [], record) : [];
    let workbench = WorkbenchPanel.for(this.document.uri);
    if (!workbench && open && drawn.length) {
      await WorkbenchPanel.open(this.document);
      workbench = WorkbenchPanel.for(this.document.uri);
    }
    if (workbench && drawn.length) workbench.show(drawn[0].id, id);
    else if (workbench) workbench.select(id);
    else if (open && !drawn.length) vscode.window.setStatusBarMessage(`ArcLang: ${name} is drawn on no diagram`, 4000);
  }

  private html(): string {
    const webview = this.panel.webview;
    const nonce = randomBytes(16).toString('base64url');
    const uri = (...segments: string[]) => webview.asWebviewUri(vscode.Uri.joinPath(this.context.extensionUri, ...segments)).toString();
    const csp = [
      "default-src 'none'",
      `style-src ${webview.cspSource}`,
      `font-src ${webview.cspSource}`,
      `script-src 'nonce-${nonce}'`,
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
<link rel="stylesheet" href="${uri('media', 'tables.css')}">
<title>ArcLang table</title>
</head>
<body>
<div id="tables"></div>
<script nonce="${nonce}" src="${uri('dist', 'tables.js')}"></script>
</body>
</html>`;
  }
}
