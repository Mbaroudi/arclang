// The diagram color themes the workbench offers: the compiler's presets,
// the team's (`.arclang/themes/*.json` in the workspace, versioned with the
// model) and the user's own (in the extension's global storage). Every one
// is checked by the viewer's validator (media/viewer/arcviz-theme.js, the
// same rules the CLI applies); one that fails is listed with its reasons and
// never drawn. `arclang.diagram.theme` names the one in use.
import * as path from 'node:path';
import * as vscode from 'vscode';
import type { ThemeEntry, ThemeSource } from '../pure/workbench';

interface Verdict {
  ok: boolean;
  errors: { rule: string; message: string }[];
  warnings: { rule: string; message: string }[];
  theme: Record<string, unknown> | null;
}

interface ThemeModule {
  presets(): { id: string; name: string; description: string }[];
  preset(id: string): Record<string, unknown> | null;
  validate(theme: unknown): Verdict;
  parse(text: string): Verdict;
  slug(text: string): string;
}

export const SETTING = 'arclang.diagram.theme';
export const DEFAULT_THEME = 'arclang-default';
const TEAM_FOLDER = ['.arclang', 'themes'];
const MAX_FILE = 256 * 1024;

/** How the viewer's theme module is loaded; the web build bundles it instead of reading it. */
let themeModuleLoader: ((context: vscode.ExtensionContext) => ThemeModule) | null = null; // web hook
export function setThemeModuleLoader(loader: (context: vscode.ExtensionContext) => ThemeModule): void { // web hook
  themeModuleLoader = loader;
}

export class ThemeStore implements vscode.Disposable {
  private readonly module: ThemeModule;
  private readonly emitter = new vscode.EventEmitter<void>();
  private readonly disposables: vscode.Disposable[] = [];
  /** Fires when a theme file, the setting or the list changed. */
  readonly onDidChange = this.emitter.event;

  constructor(private readonly context: vscode.ExtensionContext) {
    // The viewer's own module, copied with the viewer at build time: one validator for page, host and CLI.
    this.module = themeModuleLoader
      ? themeModuleLoader(context)
      : // eslint-disable-next-line @typescript-eslint/no-require-imports
        (require(path.join(context.extensionPath, 'media', 'viewer', 'arcviz-theme.js')) as ThemeModule);
    const watcher = vscode.workspace.createFileSystemWatcher(`**/${TEAM_FOLDER.join('/')}/*.json`);
    this.disposables.push(
      watcher,
      watcher.onDidCreate(() => this.emitter.fire()),
      watcher.onDidChange(() => this.emitter.fire()),
      watcher.onDidDelete(() => this.emitter.fire()),
      vscode.workspace.onDidChangeConfiguration((event) => {
        if (event.affectsConfiguration(SETTING) || event.affectsConfiguration('arclang.diagram.copyBackground')) this.emitter.fire();
      }),
      this.emitter,
    );
  }

  dispose(): void {
    for (const d of this.disposables) d.dispose();
  }

  validate(theme: unknown): Verdict {
    return this.module.validate(theme);
  }

  private teamFolder(): vscode.Uri | null {
    const folder = vscode.workspace.workspaceFolders?.[0];
    return folder ? vscode.Uri.joinPath(folder.uri, ...TEAM_FOLDER) : null;
  }

  private userFolder(): vscode.Uri {
    return vscode.Uri.joinPath(this.context.globalStorageUri, 'themes');
  }

  private async readFolder(folder: vscode.Uri | null, source: ThemeSource): Promise<ThemeEntry[]> {
    if (!folder) return [];
    let names: [string, vscode.FileType][] = [];
    try {
      names = await vscode.workspace.fs.readDirectory(folder);
    } catch {
      return [];
    }
    const out: ThemeEntry[] = [];
    for (const [name, type] of names.sort(([a], [b]) => a.localeCompare(b))) {
      if (type !== vscode.FileType.File || !name.endsWith('.json')) continue;
      out.push(await this.readFile(vscode.Uri.joinPath(folder, name), source));
    }
    return out;
  }

  private async readFile(uri: vscode.Uri, source: ThemeSource): Promise<ThemeEntry> {
    const fallbackId = path.basename(uri.fsPath, '.json');
    try {
      const bytes = await vscode.workspace.fs.readFile(uri);
      if (bytes.byteLength > MAX_FILE) throw new Error(`larger than ${MAX_FILE / 1024} KB`);
      const verdict = this.module.parse(Buffer.from(bytes).toString('utf8'));
      const raw = (() => { try { return JSON.parse(Buffer.from(bytes).toString('utf8')) as Record<string, unknown>; } catch { return null; } })();
      const id = typeof raw?.id === 'string' ? raw.id : fallbackId;
      return { key: `${source}:${id}`, id, name: typeof raw?.name === 'string' ? raw.name : fallbackId, source, path: uri.fsPath, theme: raw, ok: verdict.ok, errors: verdict.errors.map((e) => e.message), warnings: verdict.warnings.map((w) => w.message) };
    } catch (error) {
      return { key: `${source}:${fallbackId}`, id: fallbackId, name: fallbackId, source, path: uri.fsPath, theme: null, ok: false, errors: [`cannot be read: ${error instanceof Error ? error.message : String(error)}`], warnings: [] };
    }
  }

  /** Every theme on offer: presets, then the team's, then the user's. */
  async list(): Promise<ThemeEntry[]> {
    const presets: ThemeEntry[] = this.module.presets().map((p) => ({ key: `preset:${p.id}`, id: p.id, name: p.name, source: 'preset', theme: this.module.preset(p.id), ok: true, errors: [], warnings: [], description: p.description }));
    return [...presets, ...(await this.readFolder(this.teamFolder(), 'team')), ...(await this.readFolder(this.userFolder(), 'user'))];
  }

  /**
   * The theme the setting names, among `entries` or as a file; with the
   * reason when it cannot be drawn (the default is then drawn instead).
   */
  async active(entries: ThemeEntry[]): Promise<{ entry: ThemeEntry; problem: string | null }> {
    const wanted = (vscode.workspace.getConfiguration().get<string>(SETTING) ?? DEFAULT_THEME).trim() || DEFAULT_THEME;
    const fallback = entries.find((e) => e.key === `preset:${DEFAULT_THEME}`)!;
    let found = entries.find((e) => e.key === wanted) ?? entries.find((e) => e.source === 'preset' && e.id === wanted) ?? entries.find((e) => e.source === 'team' && e.id === wanted) ?? entries.find((e) => e.source === 'user' && e.id === wanted);
    if (!found && /[\\/]|\.json$/i.test(wanted)) {
      const folder = vscode.workspace.workspaceFolders?.[0];
      const uri = path.isAbsolute(wanted) ? vscode.Uri.file(wanted) : folder ? vscode.Uri.joinPath(folder.uri, wanted) : null;
      if (uri) found = await this.readFile(uri, 'file');
    }
    if (!found) return { entry: fallback, problem: `ArcLang: no diagram theme "${wanted}" (${SETTING}); the ArcLang default is drawn.` };
    if (!found.ok) return { entry: fallback, problem: `ArcLang: the diagram theme "${found.name}" is refused, the ArcLang default is drawn: ${found.errors.join('; ')}` };
    return { entry: found, problem: null };
  }

  /** Make `entry` the theme in use: the workspace's when one is open, else the user's. */
  async choose(entry: ThemeEntry): Promise<void> {
    const value = entry.source === 'file' && entry.path ? entry.path : entry.source === 'preset' ? entry.id : entry.key;
    const target = vscode.workspace.workspaceFolders?.length ? vscode.ConfigurationTarget.Workspace : vscode.ConfigurationTarget.Global;
    await vscode.workspace.getConfiguration().update(SETTING, value, target);
  }

  /** Write a theme for the team (the workspace) or for the user; refused when it breaks a rule. */
  async save(scope: 'team' | 'user', theme: Record<string, unknown>): Promise<ThemeEntry> {
    const verdict = this.module.validate(theme);
    if (!verdict.ok) throw new Error(`the theme is refused: ${verdict.errors.map((e) => e.message).join('; ')}`);
    const folder = scope === 'team' ? this.teamFolder() : this.userFolder();
    if (!folder) throw new Error('a team theme is kept in the workspace: open a folder first');
    const id = this.module.slug(typeof theme.id === 'string' && theme.id ? theme.id : String(theme.name ?? 'theme'));
    const file = vscode.Uri.joinPath(folder, `${id}.json`);
    await vscode.workspace.fs.createDirectory(folder);
    await vscode.workspace.fs.writeFile(file, Buffer.from(`${JSON.stringify({ ...theme, id }, null, 2)}\n`, 'utf8'));
    this.emitter.fire();
    return this.readFile(file, scope);
  }

  /** A theme file chosen by the user, read and checked (it is not kept anywhere yet). */
  async importFile(): Promise<{ theme: unknown; verdict: Verdict; name: string } | null> {
    const picked = await vscode.window.showOpenDialog({ canSelectMany: false, filters: { 'ArcLang theme': ['json'] }, openLabel: 'Import theme' });
    if (!picked?.[0]) return null;
    const bytes = await vscode.workspace.fs.readFile(picked[0]);
    if (bytes.byteLength > MAX_FILE) throw new Error(`the file is larger than ${MAX_FILE / 1024} KB`);
    const text = Buffer.from(bytes).toString('utf8');
    let theme: unknown = null;
    try {
      theme = JSON.parse(text);
    } catch {
      // the verdict says it is not JSON
    }
    return { theme, verdict: this.module.parse(text), name: path.basename(picked[0].fsPath) };
  }

  /** Save a theme where the user says. */
  async exportFile(theme: Record<string, unknown>): Promise<vscode.Uri | null> {
    const id = this.module.slug(String(theme.id ?? theme.name ?? 'theme'));
    const folder = vscode.workspace.workspaceFolders?.[0]?.uri;
    const target = await vscode.window.showSaveDialog({ defaultUri: folder ? vscode.Uri.joinPath(folder, `${id}.arclang-theme.json`) : undefined, filters: { 'ArcLang theme': ['json'] }, saveLabel: 'Export theme' });
    if (!target) return null;
    await vscode.workspace.fs.writeFile(target, Buffer.from(`${JSON.stringify(theme, null, 2)}\n`, 'utf8'));
    return target;
  }
}
