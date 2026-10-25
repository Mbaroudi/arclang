// CI/CD in the editor: "ArcLang: Configure CI" writes the pipeline files
// with `arclang ci init` (a preview first), and a status bar item shows the
// latest GitHub Actions run of the current branch, read through the `gh`
// CLI. No token is stored or asked for: `gh` has its own login. Nothing runs
// in an untrusted workspace, and nothing here is part of the web extension.
import { execFile } from 'node:child_process';
import * as path from 'node:path';
import * as vscode from 'vscode';
import { UNTRUSTED_MESSAGE, failureText, output, run, workspaceTrusted } from '../cli';
import {
  GhOutcome,
  PlannedFile,
  ProviderId,
  PROVIDERS,
  Run,
  STANDARDS,
  Standard,
  StatusView,
  ageText,
  describeFiles,
  ghMessage,
  ghProblem,
  githubSlug,
  initArgs,
  parseDryRun,
  parseRun,
  parseWritten,
  pollSeconds,
  providerFromRemote,
  runListArgs,
  safeBranch,
  statusView,
} from '../pure/cicd';
import { currentBranch, git, repoTop } from '../semgit/git';

const PREVIEW_SCHEME = 'arclang-ci-preview';
const GITHUB_WORKFLOW = 'arclang.yml';
const GITHUB_WORKFLOW_PATH = `.github/workflows/${GITHUB_WORKFLOW}`;

/** `arclang-ci-preview:/<path>`: the text `ci init --dry-run` printed, read-only. */
class PreviewText implements vscode.TextDocumentContentProvider {
  readonly texts = new Map<string, string>();
  provideTextDocumentContent(uri: vscode.Uri): string {
    return this.texts.get(uri.path) ?? '';
  }
}
const preview = new PreviewText();

function config<T>(key: string, fallback: T): T {
  return vscode.workspace.getConfiguration('arclang').get<T>(key, fallback);
}

/** The repository the command is about: the active file's, else the first workspace folder's. */
async function repository(): Promise<string | null> {
  const active = vscode.window.activeTextEditor?.document.uri;
  const dirs = [active?.scheme === 'file' ? path.dirname(active.fsPath) : null, ...(vscode.workspace.workspaceFolders ?? []).map((f) => f.uri.fsPath)].filter((d): d is string => !!d);
  for (const dir of dirs) {
    const top = await repoTop(dir);
    if (top) return top;
  }
  return null;
}

async function remoteUrl(top: string): Promise<string> {
  const out = await git(['remote', 'get-url', 'origin'], top);
  return out.code === 0 ? out.stdout.trim() : '';
}

function standardSetting(): Standard {
  const value = config<string>('gate.standard', 'iso26262');
  return (STANDARDS as readonly string[]).includes(value) ? (value as Standard) : 'iso26262';
}

// ------------------------------------------------------------ Configure CI

interface Plan {
  files: PlannedFile[];
  notes: string[];
  /** The provider each file belongs to, to write them one provider at a time. */
  owners: Map<string, ProviderId>;
}

async function plan(top: string, providers: ProviderId[], standard: Standard): Promise<Plan | string> {
  const files: PlannedFile[] = [];
  const notes: string[] = [];
  const owners = new Map<string, ProviderId>();
  for (const provider of providers) {
    const result = await run(initArgs({ provider, standard, dryRun: true }), { cwd: top });
    if (result.code !== 0) {
      const text = failureText(result);
      return /unrecognized subcommand|unexpected argument/i.test(text) ? 'This arclang does not have "ci init" yet: install a release that does (4.6.0 or later).' : text.split('\n')[0];
    }
    const parsed = parseDryRun(result.stdout);
    for (const file of parsed.files) {
      if (!owners.has(file.path)) {
        files.push(file);
        owners.set(file.path, provider);
      }
    }
    notes.push(...parsed.notes);
  }
  return { files, notes, owners };
}

async function showPreview(file: PlannedFile): Promise<void> {
  preview.texts.set(`/${file.path}`, file.content);
  const uri = vscode.Uri.from({ scheme: PREVIEW_SCHEME, path: `/${file.path}` });
  const document = await vscode.workspace.openTextDocument(uri);
  const language = file.path.endsWith('.yml') ? 'yaml' : file.path === 'Jenkinsfile' ? 'groovy' : 'plaintext';
  await vscode.languages.setTextDocumentLanguage(document, language).then(undefined, () => undefined);
  await vscode.window.showTextDocument(document, { preview: true, preserveFocus: false });
}

type Choice = vscode.QuickPickItem & { file?: PlannedFile; action?: 'write' | 'cancel' };

/** Previews the files one at a time until the user writes them or gives up. */
async function confirm(files: PlannedFile[], notes: string[]): Promise<boolean> {
  const existing = files.filter((f) => f.state !== 'new');
  for (;;) {
    const items: Choice[] = [
      ...files.map((f): Choice => ({ label: `$(file) ${f.path}`, description: f.state === 'new' ? 'new' : 'exists: would be replaced', detail: 'Show the file', file: f })),
      { label: '', kind: vscode.QuickPickItemKind.Separator },
      { label: `$(check) Write ${files.length} file${files.length === 1 ? '' : 's'}`, description: existing.length ? `replaces ${existing.length} existing` : 'nothing is overwritten', detail: notes[0], action: 'write' },
      { label: '$(close) Cancel', action: 'cancel' },
    ];
    const picked = await vscode.window.showQuickPick(items, { title: 'ArcLang: Configure CI, preview', placeHolder: 'Nothing is written until you choose Write. Pick a file to read it.', ignoreFocusOut: true });
    if (!picked || picked.action === 'cancel') return false;
    if (picked.file) {
      await showPreview(picked.file);
      continue;
    }
    if (existing.length) {
      const sure = await vscode.window.showWarningMessage(`${describeFiles(existing)}\n\nThese files exist. Replace them?`, { modal: true }, 'Replace');
      if (sure !== 'Replace') continue;
    }
    return true;
  }
}

export async function configureCi(): Promise<void> {
  if (!workspaceTrusted()) return void vscode.window.showInformationMessage(UNTRUSTED_MESSAGE);
  const top = await repository();
  if (!top) return void vscode.window.showInformationMessage('ArcLang: open a folder that is a git repository to configure its CI.');
  const detected = providerFromRemote(await remoteUrl(top));
  type Pick = vscode.QuickPickItem & { id: ProviderId };
  const picks = await vscode.window.showQuickPick<Pick>(
    PROVIDERS.map((p) => ({ id: p.id, label: p.label, description: p.file, detail: p.id === detected ? 'The git remote points here' : undefined, picked: p.id === detected })),
    { title: 'ArcLang: Configure CI', placeHolder: 'Which CI systems? (the gate, the SARIF file, the pull-request report)', canPickMany: true, ignoreFocusOut: true },
  );
  if (!picks || picks.length === 0) return;
  const standard = standardSetting();
  const planned = await vscode.window.withProgress({ location: vscode.ProgressLocation.Notification, title: 'ArcLang: preparing the pipeline files' }, () => plan(top, picks.map((p) => p.id), standard));
  if (typeof planned === 'string') return void vscode.window.showErrorMessage(`ArcLang: ${planned}`);
  if (!(await confirm(planned.files, planned.notes))) return;

  const existing = new Set(planned.files.filter((f) => f.state !== 'new').map((f) => planned.owners.get(f.path)));
  const written: string[] = [];
  for (const pick of picks) {
    const result = await run(initArgs({ provider: pick.id, standard, force: existing.has(pick.id) }), { cwd: top });
    output().appendLine(`$ arclang ${initArgs({ provider: pick.id, standard, force: existing.has(pick.id) }).join(' ')}\n${result.stdout}${result.stderr}`);
    if (result.code !== 0) return void vscode.window.showErrorMessage(`ArcLang: ${pick.label} was not written. ${failureText(result).split('\n')[0]}`);
    written.push(...parseWritten(result.stdout).map((w) => w.path));
  }
  for (const file of written) {
    const document = await vscode.workspace.openTextDocument(vscode.Uri.file(path.join(top, file)));
    await vscode.window.showTextDocument(document, { preview: false, preserveFocus: written.indexOf(file) > 0 });
  }
  void vscode.window.showInformationMessage(`ArcLang: ${written.length} pipeline file${written.length === 1 ? '' : 's'} written (not committed, not run). The first pipeline run is their test.`);
  void status.refresh();
}

// ------------------------------------------------------------- the status

function gh(args: string[], cwd: string): Promise<GhOutcome> {
  return new Promise((resolve) => {
    const command = config<string>('ci.gh', 'gh').trim() || 'gh';
    execFile(command, args, { cwd, timeout: 20_000, maxBuffer: 8 * 1024 * 1024, env: { ...process.env, GH_PROMPT_DISABLED: '1', GH_NO_UPDATE_NOTIFIER: '1' } }, (error, stdout, stderr) => {
      if (error && (error as NodeJS.ErrnoException).code === 'ENOENT') return resolve({ code: 127, stdout: '', stderr: `${command}: command not found` });
      resolve({ code: error && typeof error.code === 'number' ? error.code : error ? 1 : 0, stdout: String(stdout), stderr: String(stderr) });
    });
  });
}

interface Snapshot {
  view: StatusView;
  run: Run | null;
  problem: string | null;
  top: string;
}

class CiStatus implements vscode.Disposable {
  private readonly item = vscode.window.createStatusBarItem('arclang.ci', vscode.StatusBarAlignment.Right, 90);
  private timer: NodeJS.Timeout | undefined;
  private snapshot: Snapshot | null = null;
  private busy = false;
  private lastRefresh = 0;

  constructor() {
    this.item.name = 'ArcLang CI';
    this.item.command = 'arclang.ci.status';
  }

  start(context: vscode.ExtensionContext): void {
    context.subscriptions.push(
      this,
      vscode.window.onDidChangeWindowState((state) => {
        if (state.focused && Date.now() - this.lastRefresh > 30_000) void this.refresh();
      }),
      vscode.workspace.onDidChangeConfiguration((e) => {
        if (e.affectsConfiguration('arclang.ci')) void this.refresh();
      }),
      vscode.workspace.onDidGrantWorkspaceTrust(() => void this.refresh()),
    );
    void this.refresh();
  }

  async refresh(): Promise<void> {
    if (this.busy) return;
    this.busy = true;
    clearTimeout(this.timer);
    let next = pollSeconds(config<number>('ci.pollSeconds', 120));
    try {
      this.snapshot = await this.read();
      if (this.snapshot) {
        this.item.text = this.snapshot.view.text;
        this.item.tooltip = this.snapshot.problem ?? this.snapshot.view.tooltip;
        this.item.backgroundColor = this.snapshot.view.state === 'failed' ? new vscode.ThemeColor('statusBarItem.errorBackground') : undefined;
        this.item.show();
        if (this.snapshot.view.state === 'running') next = Math.min(next, 30);
      } else {
        this.item.hide();
      }
    } finally {
      this.lastRefresh = Date.now();
      this.busy = false;
      this.timer = setTimeout(() => void this.refresh(), next * 1000);
    }
  }

  /** The status, or null when there is nothing to show: untrusted, switched off, no ArcLang workflow, not GitHub. */
  private async read(): Promise<Snapshot | null> {
    if (!workspaceTrusted() || !config<boolean>('ci.statusBar', true)) return null;
    const top = await repository();
    if (!top) return null;
    const workflow = vscode.Uri.file(path.join(top, GITHUB_WORKFLOW_PATH));
    const exists = await vscode.workspace.fs.stat(workflow).then(() => true, () => false);
    if (!exists || !githubSlug(await remoteUrl(top))) return null;
    const branch = (await currentBranch(top)) ?? '';
    if (!safeBranch(branch) || branch === 'HEAD') return { top, run: null, problem: null, view: { state: 'none', text: '$(circle-slash) ArcLang CI', tooltip: 'No branch is checked out.' } };
    const result = await gh(runListArgs(GITHUB_WORKFLOW, branch), top);
    if (result.code !== 0) {
      const message = ghMessage(ghProblem(result), result.stderr);
      return { top, run: null, problem: message, view: { state: 'unknown', text: '$(question) ArcLang CI', tooltip: message } };
    }
    const found = parseRun(result.stdout);
    return { top, run: found, problem: null, view: statusView(found, branch) };
  }

  async show(): Promise<void> {
    await this.refresh();
    const snap = this.snapshot;
    if (!snap) {
      const choice = await vscode.window.showInformationMessage('ArcLang: no CI status to show. It appears for a git repository with a GitHub remote and the workflow written by "ArcLang: Configure CI", in a trusted workspace.', 'Configure CI');
      if (choice === 'Configure CI') await configureCi();
      return;
    }
    if (snap.problem) return void vscode.window.showWarningMessage(`ArcLang: ${snap.problem}`);
    const run = snap.run;
    type Action = vscode.QuickPickItem & { action: 'open' | 'refresh' };
    const items: Action[] = [];
    if (run?.url) items.push({ label: '$(link-external) Open the run on GitHub', detail: run.url, action: 'open' });
    items.push({ label: '$(refresh) Refresh', action: 'refresh' });
    const picked = await vscode.window.showQuickPick(items, {
      title: `${snap.view.text.replace(/\$\([^)]+\)\s*/, '')}: ${snap.view.state}`,
      placeHolder: run ? `${snap.view.tooltip}${run.createdAt ? `, ${ageText(run.createdAt, Date.now())}` : ''}` : snap.view.tooltip,
    });
    if (picked?.action === 'open' && run?.url) await vscode.env.openExternal(vscode.Uri.parse(run.url));
    else if (picked?.action === 'refresh') await this.refresh();
  }

  dispose(): void {
    clearTimeout(this.timer);
    this.item.dispose();
  }
}

const status = new CiStatus();

export function registerCi(context: vscode.ExtensionContext): void {
  const register = (command: string, callback: (...args: unknown[]) => unknown) => context.subscriptions.push(vscode.commands.registerCommand(command, callback));
  context.subscriptions.push(vscode.workspace.registerTextDocumentContentProvider(PREVIEW_SCHEME, preview));
  register('arclang.ci.configure', () => configureCi());
  register('arclang.ci.status', () => status.show());
  register('arclang.ci.refresh', () => status.refresh());
  status.start(context);
}
