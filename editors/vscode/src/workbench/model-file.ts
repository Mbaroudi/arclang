// The model's file from the workbench: Save (the document's own save: an
// .arc as it is, a .sysml as the SysML v2 export the edits already wrote),
// Save as… another format through `arclang export`, and the state the
// Save button shows (unsaved changes, auto save on).
import * as fs from 'node:fs';
import * as path from 'node:path';
import * as vscode from 'vscode';
import { failureText, report, run, withTempFile } from '../cli';
import { arcTextOf, modelLanguage } from './source';

export interface SaveAsFormat {
  id: 'sysml' | 'capella' | 'reqif' | 'arc';
  label: string;
  detail: string;
  extension: string;
  /** `arclang export -f`; none for a copy of the ArcLang text. */
  format: string | null;
}

export const SAVE_AS_FORMATS: SaveAsFormat[] = [
  { id: 'sysml', label: 'SysML v2 (.sysml)', detail: 'The textual notation of SysML v2, which `arclang import -f sysml` reads back', extension: 'sysml', format: 'sys-ml' },
  { id: 'capella', label: 'Capella (.capella, with its .aird and .afm)', detail: 'A Capella 7 project: the model, its diagrams file and its fragment manifest', extension: 'capella', format: 'capella' },
  { id: 'reqif', label: 'ReqIF requirements (.reqif)', detail: 'The requirements only: the traces to other elements are left out', extension: 'reqif', format: 'req-if' },
  { id: 'arc', label: 'A copy as ArcLang (.arc)', detail: 'The model as ArcLang text, unsaved changes included', extension: 'arc', format: null },
];

export interface FileState {
  dirty: boolean;
  autoSave: boolean;
  language: 'arclang' | 'sysml';
}

/** Whether VS Code saves on its own (files.autoSave, for this document). */
export function autoSaveOn(document: vscode.TextDocument): boolean {
  return vscode.workspace.getConfiguration('files', document.uri).get<string>('autoSave', 'off') !== 'off';
}

export function fileStateOf(document: vscode.TextDocument): FileState {
  return { dirty: document.isDirty, autoSave: autoSaveOn(document), language: modelLanguage(document) === 'sysml' ? 'sysml' : 'arclang' };
}

/** Save the document: an .arc as it is; a .sysml holds the SysML v2 export of every edit already. */
export async function saveModel(document: vscode.TextDocument): Promise<boolean> {
  if (!document.isDirty && !document.isUntitled) return true;
  const saved = await document.save();
  if (!saved) void vscode.window.showErrorMessage(`ArcLang: ${path.basename(document.uri.fsPath)} could not be saved.`);
  else vscode.window.setStatusBarMessage(`ArcLang: saved ${path.basename(document.uri.fsPath)}${modelLanguage(document) === 'sysml' ? ' (SysML v2)' : ''}`, 3000);
  return saved;
}

/**
 * The model in another format. An .arc saved and on disk is exported from
 * its file (its imports resolve beside it); otherwise its current text,
 * imported first when it is SysML v2, from a private temporary file.
 */
export async function saveModelAs(document: vscode.TextDocument, wanted?: string): Promise<string | null> {
  const picked = SAVE_AS_FORMATS.find((f) => f.id === wanted) ?? (await vscode.window.showQuickPick(SAVE_AS_FORMATS.map((f) => ({ label: f.label, detail: f.detail, format: f })), { title: 'ArcLang: save the model as', placeHolder: 'Format' }))?.format;
  if (!picked) return null;
  const folder = document.uri.scheme === 'file' ? path.dirname(document.uri.fsPath) : vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
  const base = path.basename(document.uri.fsPath).replace(/\.(arc|sysml)$/i, '') || 'model';
  const ownExtension = path.extname(document.uri.fsPath).slice(1).toLowerCase();
  const suggested = `${base}${picked.extension === ownExtension ? '.copy' : ''}.${picked.extension}`;
  const target = await vscode.window.showSaveDialog({ defaultUri: folder ? vscode.Uri.file(path.join(folder, suggested)) : undefined, filters: { [picked.label]: [picked.extension] }, saveLabel: 'Save', title: `Save the model as ${picked.label}` });
  if (!target || target.scheme !== 'file') return null;
  const out = target.fsPath;
  if (out === document.uri.fsPath) {
    void vscode.window.showErrorMessage('ArcLang: Save as writes another file; Save writes this one.');
    return null;
  }
  const source = await arcTextOf(document);
  if (!source.ok) {
    void vscode.window.showErrorMessage(`ArcLang: ${source.error}`);
    return null;
  }
  if (picked.format === null) {
    fs.writeFileSync(out, source.text);
  } else {
    const onDisk = modelLanguage(document) === 'arclang' && document.uri.scheme === 'file' && !document.isDirty;
    const exportFrom = (file: string, cwd: string) => run(['export', file, '-f', picked.format!, '-o', out], { cwd });
    const result = await vscode.window.withProgress({ location: vscode.ProgressLocation.Notification, title: `ArcLang: saving as ${picked.label}` }, () =>
      onDisk ? exportFrom(document.uri.fsPath, path.dirname(document.uri.fsPath)) : withTempFile(source.text, `${base}.arc`, (file, dir) => exportFrom(file, dir)),
    );
    report(`arclang export ${path.basename(document.uri.fsPath)} -f ${picked.format} -o ${out}`, result, false);
    if (result.code !== 0 || !fs.existsSync(out)) {
      void vscode.window.showErrorMessage(`ArcLang: the model could not be saved as ${picked.label}. ${failureText(result).split('\n')[0]}`);
      return null;
    }
  }
  void vscode.window.showInformationMessage(`ArcLang: the model is saved as ${path.basename(out)}.`, 'Open', 'Reveal').then(async (choice) => {
    if (choice === 'Open') await vscode.commands.executeCommand('vscode.open', target);
    if (choice === 'Reveal') await vscode.commands.executeCommand('revealFileInOS', target);
  });
  return out;
}
