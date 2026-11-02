// One import flow for every entry point (the model view's button and its
// welcome, the explorer's menus, the workbench's File menu, a file dropped
// on the sheet): what to import, where the model goes, the compiler run with
// a progress that can be canceled, then a report that says whether what was
// written compiles.
import * as fs from 'node:fs';
import * as path from 'node:path';
import * as zlib from 'node:zlib';
import * as vscode from 'vscode';
import { output, report, run, tempDir, withTempFile } from '../cli';
import { parseCheckReport } from '../pure/arc';
import { declarationsAfterModel, DirEntry, folderImports, ImportOutcome, isSysmlText, OPEN_FILTERS, parseImportOutput, reqifEntryOf, ResolvedImport, Resolution, resolveFile, selectionBase, SOURCE_KINDS, SourceKind, sysmlSelection } from '../pure/importing';
import { unzip } from '../pure/zip';
import { ApiError, chooseApiCommit, fetchElements } from './api';
import { headOf, registerExplorerContext } from './explorer';
import { ReportData, showReport, verdict } from './report';

/** An input ready for the compiler, and what to remove once it ran. */
interface Prepared {
  resolved: ResolvedImport;
  /** Where the result is offered by default. */
  folder: string;
  kind: SourceKind;
  cleanup: () => void;
}

const noop = () => undefined;
const MAX_INFLATED = 512 * 1024 * 1024;

function entriesOf(folder: string): DirEntry[] {
  try {
    return fs.readdirSync(folder, { withFileTypes: true }).map((e) => ({ name: e.name, isDirectory: e.isDirectory() }));
  } catch {
    return [];
  }
}

/** Whether a folder holds .sysml or .kerml files, at any depth (links not followed). */
function holdsSysml(folder: string, depth = 8): boolean {
  if (depth < 0) return false;
  for (const entry of entriesOf(folder)) {
    if (!entry.isDirectory && isSysmlText(entry.name)) return true;
    if (entry.isDirectory && !entry.name.startsWith('.') && entry.name !== 'node_modules' && holdsSysml(path.join(folder, entry.name), depth - 1)) return true;
  }
  return false;
}

function workspaceFolder(): string | undefined {
  return vscode.workspace.workspaceFolders?.find((f) => f.uri.scheme === 'file')?.uri.fsPath;
}

function kindOf(resolved: ResolvedImport): SourceKind {
  return resolved.format === 'capella' ? 'capella' : resolved.format === 'req-if' ? 'reqif' : 'sysml';
}

/** A .reqifz: its ReqIF document, extracted in a private temporary folder. */
function unpackReqifz(resolved: ResolvedImport): Prepared | { error: string } {
  const dir = tempDir();
  const cleanup = () => fs.rmSync(dir, { recursive: true, force: true });
  try {
    const entries = unzip(new Uint8Array(fs.readFileSync(resolved.input)), (data) => new Uint8Array(zlib.inflateRawSync(data, { maxOutputLength: MAX_INFLATED })));
    const entry = reqifEntryOf(entries);
    if ('error' in entry) { cleanup(); return entry; }
    const file = path.join(dir, path.basename(entry.name));
    fs.writeFileSync(file, entry.data, { mode: 0o600 });
    return { resolved: { ...resolved, input: file, note: `${path.basename(resolved.input)} is a ReqIF archive: ${entry.name} inside it is read.` }, folder: path.dirname(resolved.input), kind: 'reqif', cleanup };
  } catch (error) {
    cleanup();
    return { error: `${path.basename(resolved.input)} could not be opened as a ReqIF archive: ${error instanceof Error ? error.message : String(error)}` };
  }
}

/** Ready a resolution: an archive unpacked; else as it is. */
function prepare(resolution: Resolution): Prepared | { error: string } {
  if ('error' in resolution) return resolution;
  if (resolution.unzip) return unpackReqifz(resolution);
  return { resolved: resolution, folder: path.dirname(resolution.input), kind: kindOf(resolution), cleanup: noop };
}

/** A path chosen in the explorer, dropped on the sheet or picked: file or folder. */
async function fromPath(file: string, kind?: SourceKind): Promise<Prepared | { error: string } | null> {
  let stat: fs.Stats;
  try {
    stat = fs.statSync(file);
  } catch {
    return { error: `${file} cannot be read` };
  }
  if (stat.isDirectory()) {
    const options = folderImports(file, entriesOf(file), holdsSysml(file)).filter((o) => !kind || kindOf(o) === kind);
    if (options.length === 0) return { error: `${path.basename(file)} holds no Capella model (.capella) and no SysML v2 file (.sysml, .kerml)` };
    if (options.length === 1) return prepare(options[0]);
    const picked = await vscode.window.showQuickPick(options.map((o) => ({ label: o.format === 'capella' ? `$(file) Capella model ${path.basename(o.input)}` : '$(folder) Every SysML v2 file of the folder, as one model', detail: o.note, option: o })), { placeHolder: `Import ${path.basename(file)} as` });
    return picked ? prepare(picked.option) : null;
  }
  const head = /\.json$/i.test(file) ? headOf(file) : '';
  return prepare(resolveFile(file, entriesOf(path.dirname(file)).map((e) => e.name), head));
}

/** Several SysML v2 files chosen together: one model (their folder, or a copy of them). */
function fromSysmlFiles(files: string[]): Prepared | { error: string } {
  const folder = path.dirname(files[0]);
  const selection = sysmlSelection(files, entriesOf(folder));
  if ('error' in selection) return selection;
  const base = selectionBase(files);
  if (!selection.copy) return { resolved: { format: 'sysml', input: selection.folder, base, note: `The ${files.length} files of ${path.basename(selection.folder)} are read as one model.` }, folder, kind: 'sysml', cleanup: noop };
  const dir = tempDir();
  const used = new Set<string>();
  files.forEach((file, index) => {
    let name = path.basename(file);
    if (used.has(name)) name = `${index}-${name}`;
    used.add(name);
    fs.copyFileSync(file, path.join(dir, name));
  });
  return { resolved: { format: 'sysml', input: dir, base, note: `The ${files.length} files chosen (${files.map((f) => path.basename(f)).join(', ')}) are read as one model.` }, folder, kind: 'sysml', cleanup: () => fs.rmSync(dir, { recursive: true, force: true }) };
}

async function pickFiles(kind: Exclude<SourceKind, 'sysml-api'>, many: boolean, folders = false): Promise<string[] | null> {
  const chosen = await vscode.window.showOpenDialog({
    canSelectMany: many,
    canSelectFiles: !folders,
    canSelectFolders: folders,
    filters: folders ? undefined : OPEN_FILTERS[kind],
    openLabel: 'Import',
    title: SOURCE_KINDS.find((k) => k.kind === kind)?.label,
    defaultUri: vscode.workspace.workspaceFolders?.[0]?.uri,
  });
  return chosen && chosen.length > 0 ? chosen.filter((u) => u.scheme === 'file').map((u) => u.fsPath) : null;
}

/** Ask what to import, for a kind (or after asking the kind). */
async function chooseSource(context: vscode.ExtensionContext, asked?: SourceKind): Promise<Prepared | { error: string } | null> {
  let kind: SourceKind | undefined = asked;
  if (!kind) {
    const picked = await vscode.window.showQuickPick(SOURCE_KINDS.map((k) => ({ label: k.label, detail: k.detail, source: k.kind })), { title: 'ArcLang: import a model', placeHolder: 'What do you import?', matchOnDetail: true });
    if (!picked) return null;
    kind = picked.source;
  }
  if (kind === 'sysml-api') return fromApi(context);
  if (kind === 'sysml') {
    const how = await vscode.window.showQuickPick([
      { label: '$(files) Files…', detail: 'One or more .sysml or .kerml files (several chosen: one model), or a .json listing of API records', folder: false },
      { label: '$(folder) A folder…', detail: 'Every .sysml and .kerml file under it, as one model', folder: true },
    ], { title: 'ArcLang: import SysML v2', placeHolder: 'Files or a folder?' });
    if (!how) return null;
    const files = await pickFiles('sysml', !how.folder, how.folder);
    if (!files) return null;
    if (how.folder) return fromPath(files[0], 'sysml');
    return files.length > 1 ? fromSysmlFiles(files) : fromPath(files[0], 'sysml');
  }
  const files = await pickFiles(kind, false);
  if (!files) return null;
  const prepared = await fromPath(files[0], kind === 'doors' ? 'reqif' : kind);
  return prepared && !('error' in prepared) && kind === 'doors' ? { ...prepared, kind: 'doors' } : prepared;
}

/** A commit of a Systems Modeling API server, fetched to a listing of records. */
async function fromApi(context: vscode.ExtensionContext): Promise<Prepared | { error: string } | null> {
  try {
    const choice = await chooseApiCommit(context);
    if (!choice) return null;
    const dir = tempDir();
    const cleanup = () => fs.rmSync(dir, { recursive: true, force: true });
    const base = choice.projectName.replace(/[^A-Za-z0-9._-]+/g, '_').replace(/^_+|_+$/g, '') || 'api_model';
    try {
      const text = await vscode.window.withProgress({ location: vscode.ProgressLocation.Notification, title: `ArcLang: reading ${choice.projectName}`, cancellable: true }, (progress, cancel) => fetchElements(choice, (message) => progress.report({ message }), cancel));
      const file = path.join(dir, `${base}.json`);
      fs.writeFileSync(file, text, { mode: 0o600 });
      const records = (JSON.parse(text) as unknown[]).length;
      return { resolved: { format: 'sysml', input: file, base, note: `${records} elements of the commit ${choice.commitId} of "${choice.projectName}", read from ${choice.session.base}.` }, folder: workspaceFolder() ?? path.dirname(file), kind: 'sysml-api', cleanup };
    } catch (error) {
      cleanup();
      throw error;
    }
  } catch (error) {
    if (error instanceof ApiError && error.message === 'canceled') return null;
    return { error: `the API server: ${error instanceof Error ? error.message : String(error)}` };
  }
}

/** The open ArcLang model a ReqIF import may go into. */
function openModel(): vscode.TextDocument | null {
  const editor = vscode.window.activeTextEditor;
  if (editor?.document.languageId === 'arclang' && editor.document.uri.scheme === 'file') return editor.document;
  const tab = vscode.window.tabGroups.activeTabGroup.activeTab?.input as { uri?: vscode.Uri } | undefined;
  if (tab?.uri?.scheme === 'file' && tab.uri.fsPath.endsWith('.arc')) return vscode.workspace.textDocuments.find((d) => d.uri.toString() === tab.uri!.toString()) ?? null;
  return null;
}

interface Target {
  file: string;
  /** Requirements merged into an open model: the CLI writes to a private folder, the model takes its declarations. */
  into: vscode.TextDocument | null;
  cleanup: () => void;
}

async function chooseTarget(prepared: Prepared, presetTarget?: string): Promise<Target | null> {
  if (presetTarget) return { file: presetTarget, into: null, cleanup: noop };
  const model = prepared.kind === 'reqif' || prepared.kind === 'doors' ? openModel() : null;
  if (model) {
    const name = path.basename(model.uri.fsPath);
    const picked = await vscode.window.showQuickPick([
      { label: `$(merge) Into ${name}`, detail: `The requirements are added at the end of ${name}, checked with it, as one edit (Undo takes it back; save when you are happy with it)`, into: true },
      { label: '$(new-file) A new model', detail: 'The requirements alone, in a new .arc file', into: false },
    ], { title: 'ArcLang: import requirements', placeHolder: 'Where do the requirements go?' });
    if (!picked) return null;
    if (picked.into) {
      const dir = tempDir();
      return { file: path.join(dir, `${prepared.resolved.base.replace(/[^A-Za-z0-9._-]+/g, '_')}.arc`), into: model, cleanup: () => fs.rmSync(dir, { recursive: true, force: true }) };
    }
  }
  const suggested = `${prepared.resolved.base}.arc`.replace(/[^A-Za-z0-9._ -]+/g, '_');
  const target = await vscode.window.showSaveDialog({ defaultUri: vscode.Uri.file(path.join(prepared.folder, suggested)), filters: { 'ArcLang model': ['arc'] }, saveLabel: 'Write the ArcLang model', title: 'Where the imported model is written' });
  if (!target || target.scheme !== 'file') return null;
  return { file: target.fsPath.endsWith('.arc') ? target.fsPath : `${target.fsPath}.arc`, into: null, cleanup: noop };
}

interface CheckResult {
  compiles: boolean | null;
  errors: string[];
  findings: number;
  /** Findings of severity error (the model compiles, a rule says it is wrong). */
  errorFindings: number;
  metrics: Record<string, number>;
}

const UNCHECKED: CheckResult = { compiles: null, errors: [], findings: 0, errorFindings: 0, metrics: {} };

async function checkModel(file: string, token?: vscode.CancellationToken): Promise<CheckResult> {
  const result = await run(['check', '--json', file], { cwd: path.dirname(file), token });
  const parsed = parseCheckReport(result.stdout);
  let metrics: Record<string, number> = {};
  try {
    const raw = JSON.parse(result.stdout) as { metrics?: Record<string, unknown> };
    metrics = Object.fromEntries(Object.entries(raw.metrics ?? {}).filter((e): e is [string, number] => typeof e[1] === 'number'));
  } catch {
    // no metrics: the report says only whether it compiles
  }
  if (!parsed) return { compiles: false, errors: [(result.stderr || result.stdout).trim() || `arclang check exited with ${result.code}`], findings: 0, errorFindings: 0, metrics };
  return { compiles: parsed.errors.length === 0, errors: parsed.errors, findings: parsed.findings.filter((f) => f.severity !== 'info').length, errorFindings: parsed.findings.filter((f) => f.severity === 'error').length, metrics };
}

/** Check a model text: from a private copy, or beside the model when it imports other files. */
async function checkText(text: string, beside: string): Promise<CheckResult> {
  if (!/^\s*import\s+"/m.test(text)) return withTempFile(text, path.basename(beside), (file) => checkModel(file));
  const file = path.join(path.dirname(beside), `.${path.basename(beside, '.arc')}.merge-check.arc`);
  try {
    fs.writeFileSync(file, text, { mode: 0o600 });
    return await checkModel(file);
  } finally {
    fs.rmSync(file, { force: true });
  }
}

/** The requirements written by the import, added at the end of the open model as one edit, when the whole still compiles. */
async function mergeInto(document: vscode.TextDocument, file: string, source: string): Promise<{ ok: boolean; note: string; check: CheckResult }> {
  const name = path.basename(document.uri.fsPath);
  const declarations = declarationsAfterModel(fs.readFileSync(file, 'utf8'));
  if (!declarations) return { ok: false, note: `the import wrote no declaration to add to ${name}`, check: UNCHECKED };
  const merged = `${document.getText().trimEnd()}\n\n// Requirements imported from ${path.basename(source)} (ReqIF)\n${declarations}\n`;
  const check = await checkText(merged, document.uri.fsPath);
  if (!check.compiles) return { ok: false, note: `${name} would not compile with these requirements: it was left as it was.`, check };
  const edit = new vscode.WorkspaceEdit();
  edit.replace(document.uri, new vscode.Range(document.positionAt(0), document.positionAt(document.getText().length)), merged);
  const applied = await vscode.workspace.applyEdit(edit);
  return applied
    ? { ok: true, note: `The requirements are added at the end of ${name}, as one edit: Undo takes them back. ${name} is not saved yet.`, check }
    : { ok: false, note: `VS Code refused the edit of ${name}.`, check };
}

export interface ImportOptions {
  kind?: SourceKind;
  /** A path given by a menu or a drop. */
  path?: string;
  /** Several files chosen in the explorer. */
  paths?: string[];
  /** Where to write (tests and scripted runs); else asked. */
  target?: string;
}

/** The whole flow: source, target, the compiler, the report. Resolves to the report's data (null when abandoned). */
export async function importModel(context: vscode.ExtensionContext, options: ImportOptions = {}): Promise<ReportData | null> {
  const sysmlMany = (options.paths ?? []).filter(isSysmlText);
  const prepared = sysmlMany.length > 1
    ? fromSysmlFiles(sysmlMany)
    : options.path
      ? await fromPath(options.path, options.kind)
      : await chooseSource(context, options.kind);
  if (!prepared) return null;
  if ('error' in prepared) {
    void vscode.window.showErrorMessage(`ArcLang: ${prepared.error}`);
    return null;
  }
  let chosen: Target | null = null;
  try {
    chosen = await chooseTarget(prepared, options.target);
    if (!chosen) return null;
    const target = chosen;
    const { resolved } = prepared;
    const args = ['import', resolved.input, '-f', resolved.format, '-o', target.file];
    const done = await vscode.window.withProgress({ location: vscode.ProgressLocation.Notification, title: `ArcLang: importing ${path.basename(resolved.input)}`, cancellable: true }, async (progress, token) => {
      progress.report({ message: `arclang import -f ${resolved.format}` });
      const result = await run(args, { cwd: path.dirname(target.file), token });
      if (token.isCancellationRequested) return null;
      report(`arclang import ${path.basename(resolved.input)} -f ${resolved.format} -o ${target.file}`, result, false);
      const outcome: ImportOutcome = parseImportOutput(result.stdout, result.stderr, result.code);
      if (!outcome.ok) return { outcome, check: UNCHECKED, merge: null };
      progress.report({ message: 'checking the model written' });
      const written = await checkModel(target.file, token);
      if (!target.into || !written.compiles) return { outcome, check: written, merge: null };
      progress.report({ message: `adding the requirements to ${path.basename(target.into.uri.fsPath)}` });
      const merge = await mergeInto(target.into, target.file, resolved.input);
      // The report counts what was imported (the requirements file) and says whether the model still compiles.
      return { outcome, check: merge.ok ? { ...merge.check, metrics: written.metrics } : merge.check, merge };
    });
    if (!done) {
      void vscode.window.setStatusBarMessage('ArcLang: import canceled', 4000);
      return null;
    }
    const notes = [resolved.note, done.merge?.note].filter((n): n is string => !!n);
    const merged = target.into && done.merge?.ok ? target.into.uri.fsPath : undefined;
    const data: ReportData = {
      source: options.path ?? resolved.input,
      format: resolved.format,
      target: merged ?? target.file,
      mergedInto: merged,
      outcome: target.into && done.merge && !done.merge.ok ? { ...done.outcome, ok: false, error: done.merge.note } : done.outcome,
      compiles: done.check.compiles,
      checkErrors: done.check.errors,
      findings: done.check.findings,
      errorFindings: done.check.errorFindings,
      metrics: done.check.metrics,
      notes,
    };
    showReport(data, () => output().show(true));
    const text = `ArcLang: ${verdict(data)}`;
    if (done.outcome.ok && done.check.compiles && done.check.errorFindings === 0) void vscode.window.setStatusBarMessage(text, 6000);
    else void vscode.window.showWarningMessage(text);
    return data;
  } finally {
    prepared.cleanup();
    chosen?.cleanup();
  }
}

// "New model" lives in src/workbench/create.ts (blank, Arcadia skeleton or an example, opened beside the workbench).

/** A command argument that is a file URI (the explorer gives the clicked one and the selection). */
const fileUri = (value: unknown): vscode.Uri | undefined => (value instanceof vscode.Uri && value.scheme === 'file' ? value : undefined);

export function registerImport(context: vscode.ExtensionContext): void {
  const register = (command: string, callback: (...args: unknown[]) => unknown) => context.subscriptions.push(vscode.commands.registerCommand(command, callback));
  const rescan = registerExplorerContext(context);
  const after = (data: ReportData | null) => { if (data?.outcome.ok) rescan(); return data; };
  register('arclang.import', async (uri?: unknown, selection?: unknown) => {
    const one = fileUri(uri);
    const many = Array.isArray(selection) ? selection.map(fileUri).filter((u): u is vscode.Uri => !!u).map((u) => u.fsPath) : [];
    // Scripted: `arclang.import` with { path, kind, target }.
    if (!one && uri && typeof uri === 'object' && !(uri instanceof vscode.Uri)) return after(await importModel(context, uri as ImportOptions));
    return after(await importModel(context, { path: one?.fsPath, paths: many.length > 1 ? many : undefined }));
  });
  for (const kind of ['capella', 'sysml', 'reqif', 'doors'] as const) register(`arclang.import.${kind}`, async () => after(await importModel(context, { kind })));
  register('arclang.import.api', async () => after(await importModel(context, { kind: 'sysml-api' })));
  register('arclang.import.sysmlFolder', async (uri?: unknown) => after(await importModel(context, { path: fileUri(uri)?.fsPath, kind: 'sysml' })));
  register('arclang.import.capellaProject', async (uri?: unknown) => after(await importModel(context, { path: fileUri(uri)?.fsPath, kind: 'capella' })));
}
