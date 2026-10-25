// The workbench beside the source: the compiler's diagram viewer with a
// palette, a properties view and a semantic browser, in a webview that talks
// to the extension host only through typed messages. Every gesture becomes
// `arclang edit` requests applied to the text; the text is the model.
//
// The page is a custom text editor (`arclang.workbench`, "Reopen Editor
// With..."): undo, redo, save and the dirty state are the document's own.
import { randomBytes } from 'node:crypto';
import * as path from 'node:path';
import * as vscode from 'vscode';
import { CanvasCommand, EditRequest, ElementRecord, FromWebview, isLayoutFile, layoutPathOf, Snapshot, ToWebview } from '../pure/workbench';
import { looksLikeArcLang, selectionSource } from '../pure/clipboard';
import { CanvasHistory } from './canvas-history';
import { applyRequests, EditQueue, refusalMessage } from './edits';
import { metamodelJson, onDidChangeSnapshot, snapshotOf } from './snapshot';
import { arcTextOf } from './source';
import { elementAt, revealElement } from './tree';
import { handlePageMessage, pageClosed } from '../ai'; // ai-design hook
import { handlePanesMessage, isPanesMessage } from './panes-host'; // panes hook
import { handleImpactMessage, isImpactMessage } from '../impact'; // impact hook
import { DiagramExports, ExportArgs, themeStore } from './diagram-export';
import type { DiagramAction } from '../pure/workbench';
import { fileStateOf, saveModel, saveModelAs } from './model-file';
import { droppedModelFiles } from '../pure/importing';

export const VIEW_TYPE = 'arclang.workbench';

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
  'arcviz-route.js',
  'arcviz-edit.js',
  'arcviz-toolbar.js',
  'arcviz-theme-presets.js',
  'arcviz-color.js',
  'arcviz-theme.js',
  'arcviz-layers.js', // zorder hook
  'arcviz-viewer.js',
];

/** The commands the webview may ask the host to run on its document. */
const ALLOWED_COMMANDS = new Set(['arclang.transitionPlan', 'arclang.transitionApply', 'arclang.runGate', 'arclang.check', 'arclang.format']);

const panels = new Map<string, Set<WorkbenchPanel>>();
let active: WorkbenchPanel | null = null;
const selectionEmitter = new vscode.EventEmitter<{ uri: vscode.Uri; id: string | null }>();

/** Fires when the user selects an element in a workbench. */
export const onDidSelectElement = selectionEmitter.event;

/** What a workbench last put on the system clipboard: pasted back in the same model, it is that workbench's own copy. */
let lastCopied: { text: string; uri: string } | null = null;

/** The context key that tells VS Code a field of the workbench has the keyboard. */
const TYPING = 'arclang.workbenchTyping';

export class WorkbenchPanel {
  private readonly disposables: vscode.Disposable[] = [];
  private ready = false;
  private selected: string | null = null;
  private busy = 0;
  private maximized = false;
  /** One edit at a time: two quick gestures both land, the second on the text the first left. */
  private readonly edits = new EditQueue();
  /** The arrangement the page shows: kept here, the layout file is written a moment later. */
  private layoutNow: unknown = undefined;
  private readonly history: CanvasHistory;
  /** Copy as image, PowerPoint, the colour themes. */
  private readonly exports: DiagramExports;
  /** A diagram in a tab of its own (a webview panel, not the custom editor): its title shows the unsaved changes. */
  private ownTab = false;
  private ownTitle: string | null = null;
  /** How the AI side answers this page: one function for the page's whole life. */
  private readonly aiPost = (message: unknown): void => this.post(message as ToWebview);

  /** Open (or reveal) the workbench of a document beside the text. */
  static async open(document: vscode.TextDocument): Promise<void> {
    const existing = WorkbenchPanel.for(document.uri);
    if (existing) {
      existing.panel.reveal(undefined, true);
      return;
    }
    await vscode.commands.executeCommand('vscode.openWith', document.uri, VIEW_TYPE, { viewColumn: vscode.ViewColumn.Beside, preserveFocus: true });
  }

  /** The document of the workbench the user looked at last. */
  static activeDocument(): vscode.TextDocument | null {
    return active?.document ?? null;
  }

  /** The workbench the user looked at last. */
  static activePanel(): WorkbenchPanel | null {
    return active;
  }

  static for(uri: vscode.Uri): WorkbenchPanel | undefined {
    const set = panels.get(uri.toString());
    return set ? [...set][0] : undefined;
  }

  /** The custom editor that VS Code opens for `.arc` and `.sysml` files on request. */
  static register(context: vscode.ExtensionContext): vscode.Disposable {
    const provider: vscode.CustomTextEditorProvider = {
      resolveCustomTextEditor(document, webviewPanel) {
        const created = new WorkbenchPanel(context, document, webviewPanel);
        const key = document.uri.toString();
        panels.set(key, new Set([...(panels.get(key) ?? []), created]));
      },
    };
    return vscode.window.registerCustomEditorProvider(VIEW_TYPE, provider, {
      webviewOptions: { retainContextWhenHidden: true, enableFindWidget: false },
      supportsMultipleEditorsPerDocument: true,
    });
  }

  private constructor(
    private readonly context: vscode.ExtensionContext,
    private readonly document: vscode.TextDocument,
    private readonly panel: vscode.WebviewPanel,
  ) {
    const media = vscode.Uri.joinPath(context.extensionUri, 'media');
    const dist = vscode.Uri.joinPath(context.extensionUri, 'dist');
    this.panel.webview.options = {
      enableScripts: true,
      localResourceRoots: [media, dist],
      enableCommandUris: false,
    };
    this.panel.iconPath = vscode.Uri.joinPath(media, 'icon.svg');
    this.panel.webview.html = this.html();
    this.history = new CanvasHistory({ document, restoreLayout: (state, show) => this.restoreLayout(state, show), post: (message) => this.post(message) });
    this.exports = new DiagramExports(themeStore(context), document, (message) => this.post(message));
    this.disposables.push(
      this.history,
      this.exports,
      this.panel.webview.onDidReceiveMessage((message: unknown) => void this.handle(message as FromWebview)),
      onDidChangeSnapshot((uri) => {
        if (uri.toString() === document.uri.toString()) void this.push();
      }),
      // The Save button follows the document: unsaved changes, saved, auto save switched.
      vscode.workspace.onDidChangeTextDocument((event) => {
        if (event.document === document) this.postFileState();
      }),
      vscode.workspace.onDidSaveTextDocument((saved) => {
        if (saved === document) this.postFileState();
      }),
      vscode.workspace.onDidChangeConfiguration((event) => {
        if (event.affectsConfiguration('files.autoSave')) this.postFileState();
      }),
      vscode.window.onDidChangeTextEditorSelection(async (event) => {
        if (event.textEditor.document.uri.toString() !== document.uri.toString()) return;
        if (event.kind !== vscode.TextEditorSelectionChangeKind.Mouse && event.kind !== vscode.TextEditorSelectionChangeKind.Keyboard) return;
        const snapshot = await snapshotOf(document);
        const element = await elementAt(document, event.selections[0].active, snapshot.elements?.elements ?? []);
        if (element && element.id !== this.selected) this.post({ type: 'select', id: element.id });
      }),
    );
    active = this;
    this.disposables.push(
      this.panel.onDidChangeViewState((event) => {
        if (event.webviewPanel.active) active = this;
        // A field left open in a workbench that is no longer in front does not hold VS Code's keys.
        else void vscode.commands.executeCommand('setContext', TYPING, false);
      }),
    );
    this.panel.onDidDispose(() => {
      if (active === this) active = null;
      // Closed in full screen: the window comes back as it was.
      if (this.maximized) {
        void vscode.commands.executeCommand('workbench.action.toggleZenMode');
        void vscode.commands.executeCommand('workbench.action.toggleMaximizeEditorGroup');
      }
      const key = document.uri.toString();
      const set = panels.get(key);
      set?.delete(this);
      if (set && set.size === 0) panels.delete(key);
      for (const d of this.disposables) d.dispose();
      pageClosed(document, this.aiPost);
    });
    // The snapshot of the text as it is: the custom editor may open before the
    // typing pause recomputes it.
    void snapshotOf(document);
  }

  // panes hook: a diagram in a tab of its own, the pages of a document, the panes' messages (src/workbench/panes-host.ts).
  static adopt(context: vscode.ExtensionContext, document: vscode.TextDocument, panel: vscode.WebviewPanel): WorkbenchPanel {
    const created = new WorkbenchPanel(context, document, panel);
    created.ownTab = true;
    const key = document.uri.toString();
    panels.set(key, new Set([...(panels.get(key) ?? []), created]));
    return created;
  }
  static all(uri: vscode.Uri): WorkbenchPanel[] {
    return [...(panels.get(uri.toString()) ?? [])];
  }
  get documentOf(): vscode.TextDocument {
    return this.document;
  }
  postPanes(message: unknown): void {
    this.post(message as ToWebview);
  }
  submitEdit(requests: EditRequest[], label: string): Promise<void> {
    return this.edits.run(() => this.edit(requests, label));
  }
  // end panes hook

  /** The element selected in the views, for commands that act on it. */
  get selectedId(): string | null {
    return this.selected;
  }
  /** In full screen, its editor group fills the window: nothing else may be shown beside it. */
  get fullScreen(): boolean {
    return this.maximized;
  }

  select(id: string | null): void {
    this.selected = id;
    this.post({ type: 'select', id });
  }

  /** A diagram operation on an element, done by the page (hide, show related, new diagram, rename in place). */
  diagramOp(op: 'hide' | 'related' | 'new' | 'rename', id: string): void {
    this.post({ type: 'diagram-op', op, id });
  }

  /** The next (+1) or previous (-1) Arcadia layer that has a diagram. */
  stepLayer(step: number): void {
    this.post({ type: 'layer', step });
  }

  /** Show one diagram of the model (a graph id of the snapshot). */
  show(view: string, selectId?: string): void {
    this.post({ type: 'show', view, select: selectId ?? null });
    this.panel.reveal(undefined, true);
  }

  private lastFileState = '';
  /** What the Save button shows; a diagram's own tab also says it in its title. */
  private postFileState(): void {
    const state = fileStateOf(this.document);
    const key = JSON.stringify(state);
    if (key === this.lastFileState) return;
    this.lastFileState = key;
    this.post({ type: 'file-state', ...state });
    if (this.ownTab) {
      this.ownTitle ??= this.panel.title.replace(/^● /, '');
      this.panel.title = state.dirty && !state.autoSave ? `● ${this.ownTitle}` : this.ownTitle;
    }
  }

  /** Save the model (⌘S on the canvas, the Save button). */
  saveFile(): Promise<boolean> {
    return saveModel(this.document);
  }

  /** The model in another format (Save as…). */
  saveAs(format?: string): Promise<string | null> {
    return saveModelAs(this.document, format);
  }

  /** Model files dropped on the sheet: offered for import. */
  private async offerImport(uris: string[]): Promise<void> {
    const files = droppedModelFiles(uris).map((u) => vscode.Uri.parse(u, true)).filter((u) => u.scheme === 'file');
    if (files.length === 0) return;
    const names = files.map((f) => path.basename(f.fsPath)).join(', ');
    const choice = await vscode.window.showInformationMessage(`ArcLang: import ${names} as an ArcLang model?`, 'Import');
    if (choice !== 'Import') return;
    await vscode.commands.executeCommand('arclang.import', files[0], files);
  }

  private post(message: ToWebview): void {
    void this.panel.webview.postMessage(message);
  }

  private async push(): Promise<void> {
    if (!this.ready) return;
    const [snapshot, metamodel, layout] = await Promise.all([snapshotOf(this.document), metamodelJson(), this.layoutNow !== undefined ? this.layoutNow : this.readLayout()]);
    this.layoutNow = layout ?? null;
    if (snapshot.diagrams && this.layoutUri()) {
      // The arrangement kept next to the model: what the views show, as the CLI draws them.
      snapshot.diagrams = { ...snapshot.diagrams, layout: layout ?? undefined, layout_name: path.basename(this.layoutUri()!.fsPath) };
    }
    this.post({ type: 'snapshot', snapshot, metamodel, file: path.basename(this.document.uri.fsPath), selected: this.selected });
  }

  /** A diagram command (copy as image, PowerPoint, theme), run by the page. */
  diagramAction(action: DiagramAction, args?: ExportArgs): void {
    this.panel.reveal(undefined, false);
    this.exports.run(action, args);
  }

  private async handle(message: FromWebview): Promise<void> {
    if (await this.exports.handle(message)) return;
    switch (message.type) {
      case 'ready':
        this.ready = true;
        // The theme first: the first drawing is already in its colours.
        await this.exports.pushThemes();
        this.lastFileState = '';
        this.postFileState();
        await this.push();
        return;
      case 'selected':
        this.selected = message.id;
        selectionEmitter.fire({ uri: this.document.uri, id: message.id });
        return;
      case 'reveal': {
        const element = await this.element(message.id);
        if (element) await revealElement(this.document, element, { preserveFocus: true });
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
        await this.edits.run(() => this.edit(message.requests, message.label));
        return;
      case 'save':
        await this.save(message.name, message.data);
        return;
      case 'layout': {
        if (!isLayoutFile(message.file)) return;
        const before = this.layoutNow ?? null;
        this.layoutNow = message.file;
        this.history.recordLayout(before, message.file, message.join === true);
        this.writeLayout(message.file);
        return;
      }
      case 'model-file':
        if (message.op === 'save') await this.saveFile();
        else if (message.op === 'save-as') await this.saveAs(message.format);
        else await vscode.commands.executeCommand('arclang.import');
        return;
      case 'import-drop':
        await this.offerImport(message.uris);
        return;
      case 'history':
        await this.edits.run(() => (message.op === 'undo' ? this.history.undo() : this.history.redo()));
        return;
      case 'copied':
        await this.copied(message.elements, message.relationships);
        return;
      case 'clipboard-text':
        await vscode.env.clipboard.writeText(message.text);
        return;
      case 'typing':
        await vscode.commands.executeCommand('setContext', TYPING, message.on);
        return;
      case 'focus-mode':
        // The workbench takes the whole screen, its tools with it: its editor
        // group fills the window and Zen mode puts the window full screen,
        // without side bars, panel or status bar. And gives it all back.
        if (message.on !== this.maximized) {
          this.maximized = message.on;
          this.panel.reveal(undefined, false);
          await vscode.commands.executeCommand('workbench.action.toggleMaximizeEditorGroup');
          await vscode.commands.executeCommand('workbench.action.toggleZenMode');
          // Zen mode centers the editor by default: the diagram wants the full width.
          // Leaving Zen mode puts the centering back as it was.
          if (message.on && vscode.workspace.getConfiguration('zenMode').get<boolean>('centerLayout', true)) {
            await vscode.commands.executeCommand('workbench.action.toggleCenteredLayout');
          }
        }
        return;
      default:
        if (isPanesMessage(message)) return void handlePanesMessage(this, message); // panes hook
        if (isImpactMessage(message)) return void handleImpactMessage(this.document, message); // impact hook
        // ai-design hook: the AI's messages (ask, apply, suggestions) go to src/ai.
        handlePageMessage(this.document, this.aiPost, message, { asStep: (label, change) => this.asStep(label, change) });
        return;
    }
  }

  private async edit(requests: EditRequest[], label: string): Promise<void> {
    this.busy += 1;
    this.post({ type: 'busy', busy: true, label });
    try {
      const outcome = await this.history.ownChange(() => applyRequests(this.document, requests));
      if (!outcome.ok) {
        this.post({ type: 'edit-result', ok: false, label, error: outcome.error });
        vscode.window.setStatusBarMessage(refusalMessage(label, outcome), 6000);
        return;
      }
      if (outcome.applied && outcome.before !== undefined && outcome.after !== undefined) this.history.recordModel(label, outcome.before, outcome.after);
      this.post({ type: 'edit-result', ok: true, label });
      // The edited document gets its snapshot right away, not after the typing pause.
      void snapshotOf(this.document);
    } finally {
      this.busy -= 1;
      if (this.busy === 0) this.post({ type: 'busy', busy: false });
    }
  }

  /** `<model>.layout.json` beside a model on disk; none for an unsaved one. */
  private layoutUri(): vscode.Uri | null {
    return this.document.uri.scheme === 'file' ? vscode.Uri.file(layoutPathOf(this.document.uri.fsPath)) : null;
  }

  private async readLayout(): Promise<unknown> {
    const uri = this.layoutUri();
    if (!uri) return null;
    try {
      const parsed: unknown = JSON.parse(Buffer.from(await vscode.workspace.fs.readFile(uri)).toString('utf8'));
      return isLayoutFile(parsed) ? parsed : null;
    } catch {
      return null; // no arrangement yet, or one that is not a layout file: the automatic layout
    }
  }

  private layoutTimer: NodeJS.Timeout | undefined;
  /** Boxes placed by hand: kept next to the model, a moment after the last move. */
  private writeLayout(file: unknown): void {
    const uri = this.layoutUri();
    if (!uri || !isLayoutFile(file)) return;
    clearTimeout(this.layoutTimer);
    this.layoutTimer = setTimeout(() => {
      vscode.workspace.fs.writeFile(uri, Buffer.from(`${JSON.stringify(file, null, 2)}\n`, 'utf8')).then(
        () => vscode.window.setStatusBarMessage(`ArcLang: layout kept in ${path.basename(uri.fsPath)}`, 2500),
        (error: unknown) => void vscode.window.showErrorMessage(`ArcLang: the layout could not be written to ${uri.fsPath}: ${error instanceof Error ? error.message : String(error)}`),
      );
    }, 400);
  }

  /** A change of the text made for the page (an AI proposal applied): one step of the canvas's history. */
  private async asStep<T>(label: string, change: () => Promise<T>): Promise<T> {
    const before = this.document.getText();
    const result = await this.history.ownChange(change);
    const after = this.document.getText();
    if (after !== before) this.history.recordModel(label, before, after);
    return result;
  }

  /** Show an arrangement now (an undo, a redo): written at once, the page redrawn with it. */
  private async restoreLayout(state: unknown, show: boolean): Promise<void> {
    const uri = this.layoutUri();
    clearTimeout(this.layoutTimer);
    this.layoutNow = state ?? null;
    if (uri) {
      try {
        if (state) await vscode.workspace.fs.writeFile(uri, Buffer.from(`${JSON.stringify(state, null, 2)}\n`, 'utf8'));
        else await vscode.workspace.fs.delete(uri);
      } catch (error) {
        if (state) void vscode.window.showErrorMessage(`ArcLang: the layout could not be written to ${uri.fsPath}: ${error instanceof Error ? error.message : String(error)}`);
      }
    }
    if (show) await this.push();
  }

  /** ⌘Z on the canvas: the last edit of the model or of the arrangement. */
  undo(): Promise<void> {
    return this.edits.run(() => this.history.undo());
  }

  redo(): Promise<void> {
    return this.edits.run(() => this.history.redo());
  }

  /** A key of the canvas bound in VS Code: the page runs it; a paste looks at the system clipboard first. */
  async canvas(command: CanvasCommand): Promise<void> {
    if (command !== 'paste') return this.post({ type: 'key', command });
    const text = await vscode.env.clipboard.readText();
    // This workbench's own copy (its clip knows what was cut), or ArcLang text from elsewhere.
    const own = lastCopied !== null && lastCopied.text === text && lastCopied.uri === this.document.uri.toString();
    if (!own && text.trim() && looksLikeArcLang(text)) return this.post({ type: 'key', command: 'paste-text', text });
    this.post({ type: 'key', command: 'paste' });
  }

  /** A copy on the canvas: the source of what it took, on the system clipboard. */
  private async copied(elements: { id: string; name: string }[], relationships: string[]): Promise<void> {
    const source = await arcTextOf(this.document);
    if (!source.ok) return;
    const { text } = selectionSource(source.text, elements, relationships);
    if (!text) return;
    lastCopied = { text, uri: this.document.uri.toString() };
    await vscode.env.clipboard.writeText(text);
  }

  /** An export of the viewer (SVG, PNG, layout): saved where the user says, next to the model by default. */
  private async save(name: string, base64: string): Promise<void> {
    const safe = path.basename(name).replace(/[^A-Za-z0-9._-]+/g, '-') || 'diagram';
    const folder = this.document.uri.scheme === 'file' ? vscode.Uri.file(path.dirname(this.document.uri.fsPath)) : undefined;
    const extension = path.extname(safe).slice(1).toLowerCase();
    const filters: Record<string, string[]> = extension === 'png' ? { 'PNG image': ['png'] } : extension === 'svg' ? { 'SVG drawing': ['svg'] } : extension === 'json' ? { 'Layout': ['json'] } : {};
    const target = await vscode.window.showSaveDialog({ defaultUri: folder ? vscode.Uri.joinPath(folder, safe) : undefined, filters, saveLabel: 'Save' });
    if (!target) return;
    await vscode.workspace.fs.writeFile(target, Buffer.from(base64, 'base64'));
    const open = await vscode.window.showInformationMessage(`ArcLang: saved ${path.basename(target.fsPath)}`, 'Open', 'Reveal');
    if (open === 'Open') await vscode.commands.executeCommand('vscode.open', target);
    if (open === 'Reveal') await vscode.commands.executeCommand('revealFileInOS', target);
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
<link rel="stylesheet" href="${uri('media', 'panes.css')}"><!-- panes hook -->
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
