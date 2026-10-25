// The change-impact analysis of a model in its own editor tab, as the
// tables are: `arclang impact` computed on the text in the editor, saved or
// not, again after every change (debounced). A what-if is computed on a
// copy and never written; "Apply the change" sends its requests through the
// edit path of every view, as one undoable step.
import { randomBytes } from 'node:crypto';
import * as fs from 'node:fs';
import * as path from 'node:path';
import * as vscode from 'vscode';
import { run, tempDir, withTempFile, workspaceTrusted, UNTRUSTED_MESSAGE } from '../cli';
import { askPrompt, emptyFilters, FromImpactView, ImpactMarksMessage, ImpactQuery, ImpactReport, impactArgs, isWhatIf, parseReport, queryTitle, ToImpactView, visibleItems, whatIfQuery, workbenchMarks } from '../pure/impact';
import { applyRequests, EditQueue, refusalMessage } from '../workbench/edits';
import { WorkbenchPanel } from '../workbench/panel';
import { snapshotOf } from '../workbench/snapshot';
import { arcTextOf, documentDir } from '../workbench/source';
import { graphsDrawing, revealElement } from '../workbench/tree';
import { statusMessage } from '../ai/bridge';

export const IMPACT_VIEW_TYPE = 'arclang.impact';
const REFRESH_MS = 350;

const panels = new Set<ImpactPanel>();
let active: ImpactPanel | null = null;

function gateStandard(): string {
  return vscode.workspace.getConfiguration('arclang').get<string>('gate.standard', 'iso26262');
}

export class ImpactPanel implements vscode.Disposable {
  private report: ImpactReport | null = null;
  private timer: NodeJS.Timeout | undefined;
  private computing: Promise<void> | null = null;
  private again = false;
  private highlight = true;
  private applied = false;
  private readonly edits = new EditQueue();
  private readonly disposables: vscode.Disposable[] = [];

  /** Show the impact of a query on a document: the panel of the document, its query replaced. */
  static async open(context: vscode.ExtensionContext, document: vscode.TextDocument, query: ImpactQuery): Promise<ImpactPanel> {
    const existing = [...panels].find((p) => p.document.uri.toString() === document.uri.toString());
    if (existing) {
      existing.panel.reveal(undefined, false);
      existing.setQuery(query);
      return existing;
    }
    const panel = vscode.window.createWebviewPanel(IMPACT_VIEW_TYPE, queryTitle(query), { viewColumn: vscode.ViewColumn.Beside, preserveFocus: false }, {
      enableScripts: true,
      retainContextWhenHidden: true,
      localResourceRoots: [vscode.Uri.joinPath(context.extensionUri, 'dist'), vscode.Uri.joinPath(context.extensionUri, 'media')],
    });
    return new ImpactPanel(context, document, panel, query);
  }

  static activePanel(): ImpactPanel | null {
    return active;
  }

  static all(): ImpactPanel[] {
    return [...panels];
  }

  private constructor(
    private readonly context: vscode.ExtensionContext,
    readonly document: vscode.TextDocument,
    readonly panel: vscode.WebviewPanel,
    private query: ImpactQuery,
  ) {
    panels.add(this);
    active = this;
    panel.iconPath = new vscode.ThemeIcon('pulse');
    panel.webview.html = this.html();
    this.disposables.push(
      panel.webview.onDidReceiveMessage((message: FromImpactView) => void this.handle(message)),
      panel.onDidChangeViewState(() => {
        if (panel.active) active = this;
      }),
      panel.onDidDispose(() => this.dispose()),
      vscode.workspace.onDidChangeTextDocument((event) => {
        if (event.document.uri.toString() !== this.document.uri.toString() || event.contentChanges.length === 0) return;
        this.post({ type: 'busy', stale: true });
        this.schedule();
      }),
    );
  }

  get currentQuery(): ImpactQuery {
    return this.query;
  }

  get currentReport(): ImpactReport | null {
    return this.report;
  }

  dispose(): void {
    panels.delete(this);
    if (active === this) active = null;
    if (this.timer) clearTimeout(this.timer);
    this.clearMarks();
    this.disposables.forEach((d) => d.dispose());
  }

  setQuery(query: ImpactQuery): void {
    this.query = query;
    this.applied = false;
    this.panel.title = queryTitle(query);
    void this.refresh();
  }

  private post(message: ToImpactView): void {
    void this.panel.webview.postMessage(message);
  }

  schedule(): void {
    if (this.timer) clearTimeout(this.timer);
    this.timer = setTimeout(() => void this.refresh(), REFRESH_MS);
  }

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

  /** Run `arclang impact` on the current text: a copy for seeds and what-ifs, the file with git for `--since`. */
  private async answer(format: string, out?: string): Promise<{ code: number; stdout: string; stderr: string }> {
    const source = await arcTextOf(this.document);
    if (!source.ok) return { code: 1, stdout: '', stderr: source.error };
    const name = path.basename(this.document.uri.fsPath).replace(/\.sysml$/i, '.arc') || 'model.arc';
    const cwd = documentDir(this.document);
    const extra = out ? ['-o', out] : [];
    if (this.query.since && !isWhatIf(this.query) && this.document.uri.scheme === 'file' && /\.arc$/i.test(this.document.uri.fsPath)) {
      return withTempFile(source.text, name, (temp) => run([...impactArgs(this.document.uri.fsPath, this.query, gateStandard(), format), '--current', temp, ...extra], { cwd }));
    }
    return withTempFile(source.text, name, (temp) => run([...impactArgs(temp, this.query, gateStandard(), format), ...extra], { cwd }));
  }

  private async compute(): Promise<void> {
    const file = path.basename(this.document.uri.fsPath);
    if (!workspaceTrusted()) return this.post({ type: 'error', error: UNTRUSTED_MESSAGE, query: this.query, file });
    this.post({ type: 'busy', stale: true });
    const started = Date.now();
    const result = await this.answer('json');
    const parsed = parseReport(result.stdout, result.stderr);
    if (!parsed.ok) {
      this.report = null;
      this.clearMarks();
      return this.post({ type: 'error', error: parsed.error, query: this.query, file });
    }
    this.report = parsed.report;
    const ai = await statusMessage(this.context).then((s) => s.type === 'ai-status' && s.configured).catch(() => false);
    this.post({ type: 'report', report: parsed.report, query: this.query, file, computedMs: Date.now() - started, aiAvailable: ai, applied: this.applied });
    this.pushMarks();
  }

  private marksMessage(on: boolean): ImpactMarksMessage {
    if (!on || !this.report) return { type: 'impact-marks', marks: [], counts: {}, title: '' };
    const { marks, counts } = workbenchMarks(this.report, visibleItems(this.report.impacted, emptyFilters()));
    return { type: 'impact-marks', marks, counts, title: queryTitle(this.query) };
  }

  /** The impacted elements outlined on the workbench diagrams, counts on the layer tabs. */
  private pushMarks(): void {
    const message = this.marksMessage(this.highlight);
    for (const workbench of WorkbenchPanel.all(this.document.uri)) workbench.postPanes(message);
  }

  private clearMarks(): void {
    for (const workbench of WorkbenchPanel.all(this.document.uri)) workbench.postPanes(this.marksMessage(false));
  }

  /** A workbench opened after the analysis gets the marks too. */
  remark(): void {
    this.pushMarks();
  }

  private async handle(message: FromImpactView): Promise<void> {
    switch (message.type) {
      case 'ready':
        return this.refresh();
      case 'query':
        this.query = message.query;
        this.panel.title = queryTitle(message.query);
        return this.refresh();
      case 'reveal':
        return this.reveal(message.id, message.open);
      case 'analyze':
        return this.setQuery({ ...this.query, elements: [message.id], change: null, delete: null, since: null, label: null });
      case 'open-what-if':
        return void vscode.commands.executeCommand('arclang.impact.whatIf', { uri: this.document.uri, id: message.id });
      case 'what-if':
        return this.setQuery(whatIfQuery(message.form, this.query));
      case 'apply':
        return this.apply();
      case 'discard':
        return this.discard();
      case 'export':
        return this.export(message.format);
      case 'copied':
        if (!message.ok) await vscode.env.clipboard.writeText(message.tsv);
        vscode.window.setStatusBarMessage(`ArcLang: copied ${message.rows} impacted element${message.rows === 1 ? '' : 's'} as a table`, 4000);
        return;
      case 'ask':
        if (!this.report) return;
        await vscode.commands.executeCommand('workbench.action.chat.open', { query: askPrompt(this.report, this.query, this.report.impacted) });
        return;
      case 'highlight':
        this.highlight = message.on;
        return this.pushMarks();
    }
  }

  /** The requests of the what-if on screen, applied as one undoable edit. */
  private async apply(): Promise<void> {
    const requests = this.report?.what_if?.requests;
    if (!requests?.length) return this.post({ type: 'result', ok: false, message: 'Nothing to apply: this is not a what-if' });
    const label = queryTitle(this.query).replace(/^What if /, '');
    await this.edits.run(async () => {
      const outcome = await applyRequests(this.document, requests);
      if (!outcome.ok) return this.post({ type: 'result', ok: false, message: refusalMessage(label, outcome) });
      this.post({ type: 'result', ok: true, message: `Applied: ${label}. ⌘Z in the text undoes it.` });
      // What the change reaches, now that it is in the model.
      const seeds = [...new Set(this.report?.seeds.map((s) => s.id) ?? [])];
      this.setQuery({ ...this.query, change: null, delete: null, elements: seeds, label: `Impact of the applied change (${label})` });
      this.applied = true;
    });
  }

  private discard(): void {
    const seeds = this.report?.seeds.map((s) => s.id) ?? [];
    this.post({ type: 'result', ok: true, message: 'Discarded: the model is unchanged' });
    this.setQuery({ ...this.query, change: null, delete: null, label: null, elements: seeds.length ? seeds.slice(0, 1) : this.query.elements });
  }

  private async export(format: 'md' | 'html' | 'xlsx'): Promise<void> {
    if (!this.report) return;
    const base = queryTitle(this.query).replace(/[^\p{L}\p{N} _-]+/gu, '').trim().replace(/\s+/g, '_') || 'impact';
    const folder = this.document.uri.scheme === 'file' ? vscode.Uri.file(path.dirname(this.document.uri.fsPath)) : undefined;
    const filters: Record<string, Record<string, string[]>> = { xlsx: { 'Excel workbook': ['xlsx'] }, html: { HTML: ['html'] }, md: { Markdown: ['md'] } };
    const target = await vscode.window.showSaveDialog({ defaultUri: folder ? vscode.Uri.joinPath(folder, `${base}.${format}`) : undefined, filters: filters[format], saveLabel: `Export ${format.toUpperCase()}` });
    if (!target) return;
    const dir = tempDir();
    try {
      const out = path.join(dir, `impact.${format}`);
      const result = await this.answer(format, out);
      if (result.code !== 0 || !fs.existsSync(out)) return this.post({ type: 'result', ok: false, message: (result.stderr || result.stdout).trim() || 'the export failed' });
      await vscode.workspace.fs.writeFile(target, fs.readFileSync(out));
      this.post({ type: 'result', ok: true, message: `Exported ${path.basename(target.fsPath)}` });
      vscode.window.setStatusBarMessage(`ArcLang: exported ${target.fsPath}`, 5000);
    } finally {
      fs.rmSync(dir, { recursive: true, force: true });
    }
  }

  /** Show an element in the text and in its diagram; `open` opens the workbench. */
  private async reveal(id: string, open: boolean): Promise<void> {
    const snapshot = await snapshotOf(this.document);
    const records = snapshot.elements?.elements ?? [];
    const record = records.find((e) => e.id === id) ?? records.find((e) => e.name === id);
    if (record) await revealElement(this.document, { id: record.id, name: record.name }, { preserveFocus: true });
    const drawn = record ? graphsDrawing(snapshot.diagrams?.graphs ?? [], record) : [];
    let workbench = WorkbenchPanel.for(this.document.uri);
    if (!workbench && open && drawn.length) {
      await WorkbenchPanel.open(this.document);
      workbench = WorkbenchPanel.for(this.document.uri);
      this.pushMarks();
    }
    if (workbench && drawn.length && record) workbench.show(drawn[0].id, record.id);
    else if (workbench && record) workbench.select(record.id);
    else if (open && !drawn.length) vscode.window.setStatusBarMessage(`ArcLang: ${id} is drawn on no diagram`, 4000);
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
<link rel="stylesheet" href="${uri('media', 'impact.css')}">
<title>ArcLang impact analysis</title>
</head>
<body>
<main id="impact" aria-label="Impact analysis"></main>
<script nonce="${nonce}" src="${uri('dist', 'impact.js')}"></script>
</body>
</html>`;
  }
}
