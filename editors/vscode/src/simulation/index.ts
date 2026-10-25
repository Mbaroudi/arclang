// Simulation with OpenModelica, for someone who has never set it up: a
// status bar item that says where things stand, a menu, a checklist of what
// is missing with a button for each fix, and one command to run the
// simulations. Discovery is the compiler's (`arclang verify --check-tools
// --json`); this file asks it and shows the answer. Nothing runs in an
// untrusted workspace or in the browser build.
import * as path from 'node:path';
import * as vscode from 'vscode';
import { UNTRUSTED_MESSAGE, currentBackend, failureText, output, report, run, workspaceTrusted } from '../cli';
import {
  Checklist,
  Counts,
  DOCS_URL,
  MODEX_ID,
  SimModel,
  checklist,
  countOutcomes,
  nextStep,
  outcomesLog,
  parseDiagnosis,
  parseOutcomes,
  simView,
  simulates,
  simulationEnv,
  summaryText,
  toolLabel,
  toolMissing,
} from '../pure/simulation';
import { checklistHtml } from '../pure/simulation-html';
import { applyFix } from './fixes';

const SKIP_DIRS = '**/{node_modules,.git,target,.arclang,dist,out}/**';
const MAX_FILES = 300;
const MAX_BYTES = 400_000;
const STALE_MS = 60_000;

function settings() {
  const c = vscode.workspace.getConfiguration('arclang.simulation');
  return { image: c.get<string>('image', ''), omc: c.get<string>('omc', ''), timeout: c.get<number>('timeout', 900) };
}

const modexInstalled = () => vscode.extensions.getExtension(MODEX_ID) !== undefined;
const isWeb = () => currentBackend().name === 'wasm';

/** The model file the user is looking at: a text editor, or the workbench's tab. */
function activeModelUri(): vscode.Uri | null {
  const doc = vscode.window.activeTextEditor?.document;
  if (doc && doc.languageId === 'arclang' && doc.uri.scheme === 'file') return doc.uri;
  const input = vscode.window.tabGroups.activeTabGroup.activeTab?.input as { uri?: vscode.Uri } | undefined;
  return input?.uri instanceof vscode.Uri && input.uri.scheme === 'file' && input.uri.fsPath.endsWith('.arc') ? input.uri : null;
}

class Simulation implements vscode.Disposable {
  private readonly item = vscode.window.createStatusBarItem('arclang.simulation', vscode.StatusBarAlignment.Left, 90);
  private model: SimModel = { relevant: false, checking: false, running: false, list: null, toolLabel: null, last: null };
  /** .arc files that simulate, .mo files: by path. */
  private readonly simulating = new Set<string>();
  private readonly modelica = new Set<string>();
  private panel: vscode.WebviewPanel | null = null;
  private list: Checklist | null = null;
  private lastCheck = 0;
  private scanTimer: NodeJS.Timeout | undefined;

  constructor() {
    this.item.name = 'ArcLang Simulation';
    this.item.command = 'arclang.simulation.menu';
  }

  start(context: vscode.ExtensionContext): void {
    const watcher = vscode.workspace.createFileSystemWatcher('**/*.{arc,mo}');
    context.subscriptions.push(
      this,
      watcher,
      watcher.onDidCreate((uri) => void this.scanFile(uri)),
      watcher.onDidChange((uri) => void this.scanFile(uri)),
      watcher.onDidDelete((uri) => {
        this.simulating.delete(uri.fsPath);
        this.modelica.delete(uri.fsPath);
        this.render();
      }),
      vscode.workspace.onDidChangeTextDocument((e) => {
        if (e.document.languageId === 'arclang') this.noteText(e.document.uri, e.document.getText());
      }),
      vscode.window.onDidChangeActiveTextEditor(() => this.contextKeys()),
      vscode.window.tabGroups.onDidChangeTabs(() => this.contextKeys()),
      vscode.workspace.onDidChangeConfiguration((e) => {
        if (e.affectsConfiguration('arclang.simulation') || e.affectsConfiguration('arclang.binary')) void this.refresh();
      }),
      vscode.workspace.onDidGrantWorkspaceTrust(() => void this.scanAll()),
      vscode.window.onDidChangeWindowState((state) => {
        if (state.focused && this.model.relevant && Date.now() - this.lastCheck > STALE_MS) void this.refresh();
      }),
      vscode.extensions.onDidChange(() => this.panelRefresh()),
    );
    void this.scanAll();
  }

  // ---- What there is to simulate ------------------------------------------------

  private available(): boolean {
    return workspaceTrusted() && !isWeb();
  }

  private async scanAll(): Promise<void> {
    this.simulating.clear();
    this.modelica.clear();
    if (this.available()) {
      const files = await vscode.workspace.findFiles('**/*.{arc,mo}', SKIP_DIRS, MAX_FILES);
      await Promise.all(files.map((uri) => this.scanFile(uri, false)));
    }
    this.render();
    if (this.model.relevant) await this.refresh();
  }

  private async scanFile(uri: vscode.Uri, render = true): Promise<void> {
    if (!this.available() || uri.scheme !== 'file') return;
    if (uri.fsPath.endsWith('.mo')) {
      this.modelica.add(uri.fsPath);
    } else {
      try {
        const stat = await vscode.workspace.fs.stat(uri);
        if (stat.size <= MAX_BYTES) this.noteText(uri, Buffer.from(await vscode.workspace.fs.readFile(uri)).toString('utf8'), false);
      } catch {
        this.simulating.delete(uri.fsPath);
      }
    }
    if (render) this.render();
  }

  private noteText(uri: vscode.Uri, text: string, render = true): void {
    if (uri.scheme !== 'file' || !this.available()) return;
    if (simulates(text)) this.simulating.add(uri.fsPath);
    else this.simulating.delete(uri.fsPath);
    if (render) {
      clearTimeout(this.scanTimer);
      this.scanTimer = setTimeout(() => {
        const was = this.model.relevant;
        this.render();
        if (!was && this.model.relevant) void this.refresh();
      }, 400);
    }
  }

  private contextKeys(): void {
    const uri = activeModelUri();
    void vscode.commands.executeCommand('setContext', 'arclang.simulatesHere', !!uri && this.simulating.has(uri.fsPath));
  }

  // ---- The status bar item ---------------------------------------------------------

  private render(): void {
    this.model = { ...this.model, relevant: this.available() && (this.simulating.size > 0 || this.modelica.size > 0), list: this.list };
    const view = simView(this.model);
    if (view.state === 'hidden') {
      this.item.hide();
    } else {
      this.item.text = view.text;
      this.item.tooltip = view.tooltip;
      this.item.backgroundColor = view.error ? new vscode.ThemeColor('statusBarItem.errorBackground') : view.warning ? new vscode.ThemeColor('statusBarItem.warningBackground') : undefined;
      this.item.show();
    }
    this.contextKeys();
  }

  /** Asks the compiler what is installed. */
  private async diagnose(): Promise<Checklist> {
    this.model = { ...this.model, checking: true };
    this.render();
    try {
      const result = await run(['verify', '--check-tools', '--json'], { env: simulationEnv(settings()) });
      const diagnosis = result.code === 0 ? parseDiagnosis(result.stdout) : null;
      this.list = checklist(diagnosis, { platform: process.platform, modexInstalled: modexInstalled() });
      this.model = { ...this.model, toolLabel: toolLabel(diagnosis) };
      this.lastCheck = Date.now();
      return this.list;
    } finally {
      this.model = { ...this.model, checking: false };
      this.render();
    }
  }

  async refresh(): Promise<void> {
    if (!this.available() || this.model.running) return;
    await this.diagnose();
    this.panelRefresh();
  }

  // ---- The menu ----------------------------------------------------------------------

  async menu(): Promise<void> {
    if (!this.guard()) return;
    type Entry = vscode.QuickPickItem & { id: string };
    const state = this.list ? (this.list.ready ? this.list.headline : 'Needs setup: choose "Check my setup".') : 'Looking for OpenModelica…';
    const entries: Entry[] = [
      { id: 'run', label: '$(play) Run simulations on this model', detail: 'Simulates the test cases of the model in OpenModelica and shows their verdicts.' },
      { id: 'check', label: '$(checklist) Check my setup', detail: 'What is installed, what is missing, and a button to fix each thing.' },
      { id: 'setup', label: '$(tools) Set up OpenModelica (guided)', detail: 'Step by step, from nothing to a first simulation.' },
      { id: 'export', label: '$(export) Export Modelica', detail: 'Write the physical architecture as a Modelica package.' },
      { id: 'drawing', label: '$(symbol-structure) Open the Modelica drawing', detail: 'A .mo file as a diagram (needs the Modex extension).' },
      { id: 'docs', label: '$(book) Open the documentation', detail: 'How simulation works and how to write a test case.' },
    ];
    const picked = await vscode.window.showQuickPick(entries, { title: 'ArcLang: Simulation', placeHolder: state });
    switch (picked?.id) {
      case 'run': return this.runSimulations();
      case 'check': return this.showSetup();
      case 'setup': return this.guidedSetup();
      case 'export': return this.exportModelica();
      case 'drawing': return this.openDrawing();
      case 'docs': return void vscode.env.openExternal(vscode.Uri.parse(DOCS_URL));
    }
  }

  private guard(): boolean {
    if (isWeb()) {
      vscode.window.showInformationMessage('ArcLang: simulation needs the desktop extension. It runs OpenModelica on your computer, which a browser cannot do.');
      return false;
    }
    if (!workspaceTrusted()) {
      vscode.window.showWarningMessage(UNTRUSTED_MESSAGE);
      return false;
    }
    return true;
  }

  // ---- Running ---------------------------------------------------------------------

  private async pickModel(arg: unknown): Promise<string | null> {
    const explicit = arg instanceof vscode.Uri && arg.scheme === 'file' && arg.fsPath.endsWith('.arc') ? arg : null;
    const here = explicit ?? activeModelUri();
    const doc = here ? await vscode.workspace.openTextDocument(here) : null;
    if (here && doc && simulates(doc.getText())) {
      if (doc.isDirty) {
        const choice = await vscode.window.showInformationMessage('ArcLang: the model has unsaved changes. Save it before simulating?', 'Save and run', 'Run the saved version');
        if (!choice) return null;
        if (choice === 'Save and run') await doc.save();
      }
      return here.fsPath;
    }
    const others = [...this.simulating].sort();
    if (others.length === 0) {
      const choice = await vscode.window.showInformationMessage(
        'ArcLang: no model here has a test case that simulates yet. A test case names a Modelica model and what to check (simulate: and expects:).',
        'Open the documentation',
      );
      if (choice) await vscode.env.openExternal(vscode.Uri.parse(DOCS_URL));
      return null;
    }
    if (others.length === 1) return others[0];
    const picked = await vscode.window.showQuickPick(
      others.map((file) => ({ label: path.basename(file), description: vscode.workspace.asRelativePath(file), file })),
      { placeHolder: 'Which model do you want to simulate?' },
    );
    return picked?.file ?? null;
  }

  async runSimulations(arg?: unknown): Promise<void> {
    if (!this.guard() || this.model.running) return;
    const file = await this.pickModel(arg);
    if (!file) return;
    const list = await this.vscodeProgress('Looking for OpenModelica…', () => this.diagnose());
    if (!list.ready) {
      await this.showSetup('Simulation needs OpenModelica first. Here is what to do; then run the simulations again.');
      return;
    }
    this.model = { ...this.model, running: true };
    this.render();
    const title = list.firstRun
      ? 'Simulating… the first run installs the Modelica library (a few minutes); later runs are cached'
      : 'Simulating…';
    let counts: Counts | null = null;
    let missingTool = false;
    try {
      const result = await vscode.window.withProgress({ location: vscode.ProgressLocation.Notification, title, cancellable: true }, (_progress, token) =>
        run(['verify', file, '--json'], { cwd: path.dirname(file), token, env: simulationEnv(settings()) }),
      );
      const outcomes = parseOutcomes(result.stdout);
      if (!outcomes) {
        if (/OpenModelica not available/i.test(result.stderr)) missingTool = true;
        else if (result.code === 127 || result.code === 126) vscode.window.showErrorMessage(`ArcLang: ${failureText(result)}`);
        else if (result.code === null || result.code > 128) vscode.window.showInformationMessage('ArcLang: the simulation was cancelled.');
        else {
          report(`arclang verify ${path.basename(file)}`, result);
          vscode.window.showErrorMessage('ArcLang: the model could not be simulated. See the ArcLang output.');
        }
      } else {
        counts = countOutcomes(outcomes);
        missingTool = toolMissing(outcomes, result.stderr);
        const out = output();
        out.appendLine(`$ arclang verify ${path.basename(file)}`);
        out.appendLine(outcomesLog(outcomes));
        out.appendLine(summaryText(counts));
        out.appendLine('');
        if (!missingTool) await this.announce(file, counts);
      }
    } finally {
      this.model = { ...this.model, running: false, last: counts ?? this.model.last };
      this.render();
    }
    if (missingTool) await this.showSetup('OpenModelica could not be started. Here is what to fix.');
  }

  private async announce(file: string, counts: Counts): Promise<void> {
    const text = `ArcLang: ${summaryText(counts)}`;
    const show = counts.fail || counts.other ? vscode.window.showWarningMessage : vscode.window.showInformationMessage;
    const choice = await show(text, 'Show the verdicts', 'Show the log');
    if (choice === 'Show the verdicts') await vscode.commands.executeCommand('arclang.table.coverage', vscode.Uri.file(file));
    else if (choice === 'Show the log') output().show(true);
  }

  private vscodeProgress<T>(title: string, task: () => Promise<T>): Promise<T> {
    return Promise.resolve(vscode.window.withProgress({ location: vscode.ProgressLocation.Window, title }, task));
  }

  // ---- The checklist -------------------------------------------------------------------

  async showSetup(banner = ''): Promise<void> {
    if (!this.guard()) return;
    const list = await this.diagnose();
    if (!this.panel) {
      this.panel = vscode.window.createWebviewPanel('arclang.simulationSetup', 'Simulation setup', vscode.ViewColumn.Active, { enableScripts: true });
      this.panel.onDidDispose(() => (this.panel = null));
      this.panel.webview.onDidReceiveMessage((message: unknown) => void this.fromPage(message));
    }
    this.banner = banner;
    this.panel.reveal(undefined, false);
    this.paint(list);
  }

  private banner = '';

  private paint(list: Checklist): void {
    if (!this.panel) return;
    const nonce = Buffer.from(`${Date.now()}${Math.random()}`).toString('base64').replace(/[^A-Za-z0-9]/g, '').slice(0, 24);
    this.panel.webview.html = checklistHtml(list, nonce, this.panel.webview.cspSource, this.banner);
  }

  private panelRefresh(): void {
    if (this.panel && this.list) {
      this.list = this.rebuilt(this.list);
      this.paint(this.list);
    }
  }

  /** The same diagnosis with the editor's own facts (Modex) read again. */
  private rebuilt(list: Checklist): Checklist {
    const modex = list.items.find((i) => i.id === 'modex');
    if (!modex || (modex.state === 'ok') === modexInstalled()) return list;
    return { ...list, items: list.items.map((i) => (i.id === 'modex' ? checklist(null, { platform: process.platform, modexInstalled: modexInstalled() }).items.find((x) => x.id === 'modex')! : i)) };
  }

  private async fromPage(message: unknown): Promise<void> {
    if (!this.available() || typeof message !== 'object' || message === null) return;
    const { type, fix } = message as { type?: unknown; fix?: unknown };
    if (type === 'docs') return void vscode.env.openExternal(vscode.Uri.parse(DOCS_URL));
    if (type === 'recheck') return void (await this.showSetup(this.banner));
    if (type !== 'fix' || typeof fix !== 'string' || !this.list) return;
    // Only a fix of the list being shown, found by its id: nothing the page sends is run as is.
    const [id, index] = fix.split(':');
    const found = this.list.items.find((i) => i.id === id)?.fixes[Number(index)];
    if (!found) return;
    await applyFix(found);
    await this.showSetup(this.banner);
  }

  // ---- The guided setup -------------------------------------------------------------------

  async guidedSetup(): Promise<void> {
    if (!this.guard()) return;
    for (let step = 1; step <= 8; step++) {
      const list = await this.vscodeProgress('Looking at what is installed…', () => this.diagnose());
      const next = nextStep(list);
      if (!next) {
        this.panelRefresh();
        const choice = await vscode.window.showInformationMessage(`ArcLang: ${list.headline}`, ...(list.ready ? ['Run simulations'] : []));
        if (choice === 'Run simulations') await this.runSimulations();
        return;
      }
      const choice = await vscode.window.showInformationMessage(`Step ${step}: ${next.item.title}. ${next.item.detail}`, next.fix.label, 'Stop');
      if (choice !== next.fix.label) return;
      const done = await applyFix(next.fix);
      if (next.fix.kind === 'openUrl') {
        vscode.window.showInformationMessage('ArcLang: install it, start it, then choose "Set up OpenModelica" again to continue.');
        return;
      }
      if (!done) return;
    }
  }

  // ---- Export and drawing ------------------------------------------------------------------

  async exportModelica(arg?: unknown): Promise<void> {
    if (!this.guard()) return;
    const explicit = arg instanceof vscode.Uri && arg.scheme === 'file' ? arg.fsPath : null;
    const file = explicit ?? activeModelUri()?.fsPath ?? (this.simulating.size === 1 ? [...this.simulating][0] : null);
    if (!file) return void vscode.window.showInformationMessage('ArcLang: open a .arc model first.');
    const target = path.join(path.dirname(file), 'modelica');
    const result = await run(['export', file, '-f', 'modelica', '-o', target], { cwd: path.dirname(file) });
    report(`arclang export ${path.basename(file)} -f modelica -o ${target}`, result, result.code !== 0);
    if (result.code !== 0) return void vscode.window.showErrorMessage(`ArcLang: the Modelica export failed. ${failureText(result).split('\n')[0]}`);
    const written = /Output: (.+)/.exec(result.stdout)?.[1]?.trim();
    const choice = await vscode.window.showInformationMessage(`ArcLang: Modelica package written to ${vscode.workspace.asRelativePath(written ?? target)}.`, 'Open');
    if (choice === 'Open' && written) await vscode.commands.executeCommand('vscode.open', vscode.Uri.file(written));
  }

  async openDrawing(): Promise<void> {
    if (!this.guard()) return;
    if (!modexInstalled()) {
      const choice = await vscode.window.showInformationMessage('ArcLang: Modex draws .mo files as diagrams. Install it?', 'Install Modex', 'Open as text');
      if (choice === 'Install Modex') {
        await applyFix({ kind: 'installExtension', label: 'Install Modex', id: MODEX_ID });
        return;
      }
      if (!choice) return;
    }
    const files = [...this.modelica].sort();
    if (files.length === 0) {
      const choice = await vscode.window.showInformationMessage('ArcLang: this workspace has no .mo file yet. Export the model to Modelica first.', 'Export Modelica');
      if (choice) await this.exportModelica();
      return;
    }
    const file =
      files.length === 1
        ? files[0]
        : (await vscode.window.showQuickPick(files.map((f) => ({ label: path.basename(f), description: vscode.workspace.asRelativePath(f), f })), { placeHolder: 'Which Modelica file?' }))?.f;
    if (file) await vscode.commands.executeCommand('vscode.open', vscode.Uri.file(file));
  }

  dispose(): void {
    clearTimeout(this.scanTimer);
    this.item.dispose();
    this.panel?.dispose();
  }
}

export function registerSimulation(context: vscode.ExtensionContext): void {
  const simulation = new Simulation();
  const register = (command: string, callback: (...args: unknown[]) => unknown) => context.subscriptions.push(vscode.commands.registerCommand(command, callback));
  register('arclang.simulation.menu', () => simulation.menu());
  register('arclang.simulation.run', (uri?: unknown) => simulation.runSimulations(uri));
  register('arclang.simulation.check', () => simulation.showSetup());
  register('arclang.simulation.setup', () => simulation.guidedSetup());
  register('arclang.simulation.export', (uri?: unknown) => simulation.exportModelica(uri));
  register('arclang.simulation.drawing', () => simulation.openDrawing());
  register('arclang.simulation.docs', () => vscode.env.openExternal(vscode.Uri.parse(DOCS_URL)));
  simulation.start(context);
}
