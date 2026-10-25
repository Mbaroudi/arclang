// What the explorer's menus and the model view's welcome need to know of the
// workspace, as context keys: the folders that hold SysML v2 files or a
// Capella model (their "Import" entries), the .json files that are listings
// of SysML v2 API records, and whether the workspace has a model at all.
import * as fs from 'node:fs';
import * as path from 'node:path';
import * as vscode from 'vscode';
import { isCapellaModel, isSysmlText, looksLikeApiRecords } from '../pure/importing';

export const SYSML_FOLDERS = 'arclang.sysmlFolders';
export const CAPELLA_FOLDERS = 'arclang.capellaFolders';
export const API_JSON = 'arclang.apiRecordFiles';
export const NO_MODEL = 'arclang.noModelFiles';

const EXCLUDE = '**/{node_modules,target,.git}/**';
const SCAN_LIMIT = 5000;
const JSON_LIMIT = 500;
const HEAD_BYTES = 8192;

/** The first bytes of a file, as text ('' when it cannot be read). */
export function headOf(file: string, bytes = HEAD_BYTES): string {
  let handle: number | null = null;
  try {
    handle = fs.openSync(file, 'r');
    const buffer = Buffer.alloc(bytes);
    const read = fs.readSync(handle, buffer, 0, bytes, 0);
    return buffer.subarray(0, read).toString('utf8');
  } catch {
    return '';
  } finally {
    if (handle !== null) fs.closeSync(handle);
  }
}

/** A folder and the folders above it, up to the workspace folder that holds it. */
function withParents(folder: string, roots: string[]): string[] {
  const root = roots.find((r) => folder === r || folder.startsWith(r + path.sep));
  if (!root) return [folder];
  const out: string[] = [];
  for (let at = folder; ; at = path.dirname(at)) {
    out.push(at);
    if (at === root || path.dirname(at) === at) break;
  }
  return out;
}

async function scan(): Promise<void> {
  const roots = (vscode.workspace.workspaceFolders ?? []).filter((f) => f.uri.scheme === 'file').map((f) => f.uri.fsPath);
  const [sysml, capella, json, arc] = await Promise.all([
    vscode.workspace.findFiles('**/*.{sysml,kerml}', EXCLUDE, SCAN_LIMIT),
    vscode.workspace.findFiles('**/*.{capella,melodymodeller}', EXCLUDE, SCAN_LIMIT),
    vscode.workspace.findFiles('**/*.json', '**/{node_modules,target,.git,.vscode}/**', JSON_LIMIT),
    vscode.workspace.findFiles('**/*.arc', EXCLUDE, 1),
  ]);
  const sysmlFolders = new Set<string>();
  for (const uri of sysml) if (isSysmlText(uri.fsPath)) for (const f of withParents(path.dirname(uri.fsPath), roots)) sysmlFolders.add(f);
  const capellaFolders = new Set(capella.filter((uri) => isCapellaModel(uri.fsPath)).map((uri) => path.dirname(uri.fsPath)));
  const records = json.map((uri) => uri.fsPath).filter((file) => !/(\.layout|package(-lock)?|tsconfig[^/]*)\.json$/.test(file) && looksLikeApiRecords(headOf(file)));
  await vscode.commands.executeCommand('setContext', SYSML_FOLDERS, [...sysmlFolders]);
  await vscode.commands.executeCommand('setContext', CAPELLA_FOLDERS, [...capellaFolders]);
  await vscode.commands.executeCommand('setContext', API_JSON, records);
  await vscode.commands.executeCommand('setContext', NO_MODEL, arc.length === 0);
}

/** Keep the context keys up to date as files come and go; returns a "scan again soon". */
export function registerExplorerContext(context: vscode.ExtensionContext): () => void {
  let timer: NodeJS.Timeout | undefined;
  const later = () => {
    clearTimeout(timer);
    timer = setTimeout(() => void scan().catch(() => undefined), 800);
  };
  const watcher = vscode.workspace.createFileSystemWatcher('**/*.{sysml,kerml,capella,melodymodeller,json,arc}', false, true, false);
  context.subscriptions.push(watcher, watcher.onDidCreate(later), watcher.onDidDelete(later), vscode.workspace.onDidChangeWorkspaceFolders(later), { dispose: () => clearTimeout(timer) });
  void scan().catch(() => undefined);
  return later;
}
