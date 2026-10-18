// The model behind a document. An `.arc` file is the model text itself. A
// `.sysml` file is imported by the compiler into ArcLang for the views and
// the edits, and written back through the SysML v2 export: the same gesture
// works on both, the compiler stays the only one that reads and writes.
import * as fs from 'node:fs';
import * as path from 'node:path';
import * as vscode from 'vscode';
import { run, withTempFile, workspaceTrusted, UNTRUSTED_MESSAGE } from '../cli';

export type ModelLanguage = 'arclang' | 'sysml';

/** Whether the document holds a model the workbench can show. */
export function modelLanguage(document: vscode.TextDocument): ModelLanguage | null {
  if (document.languageId === 'arclang') return 'arclang';
  if (document.languageId === 'sysml') return 'sysml';
  const name = document.uri.fsPath.toLowerCase();
  if (name.endsWith('.arc')) return 'arclang';
  if (name.endsWith('.sysml')) return 'sysml';
  return null;
}

export function isModelDocument(document: vscode.TextDocument): boolean {
  return modelLanguage(document) !== null;
}

export type SourceOutcome = { ok: true; text: string; imported: boolean } | { ok: false; error: string };

/** The document's directory, for compiler runs that resolve relative paths. */
export function documentDir(document: vscode.TextDocument): string | undefined {
  return document.uri.scheme === 'file' ? path.dirname(document.uri.fsPath) : undefined;
}

function baseName(document: vscode.TextDocument, fallback: string): string {
  return path.basename(document.uri.fsPath) || fallback;
}

/** The ArcLang text of the model in the document, imported when the document is SysML v2. */
export async function arcTextOf(document: vscode.TextDocument): Promise<SourceOutcome> {
  const language = modelLanguage(document);
  if (language === 'arclang') return { ok: true, text: document.getText(), imported: false };
  if (language !== 'sysml') return { ok: false, error: 'not a model file (.arc or .sysml)' };
  if (!workspaceTrusted()) return { ok: false, error: UNTRUSTED_MESSAGE };
  return withTempFile(document.getText(), baseName(document, 'model.sysml'), async (file, dir) => {
    const out = path.join(dir, 'imported.arc');
    const result = await run(['import', '-f', 'sysml', file, '-o', out], { cwd: dir });
    if (result.code !== 0 || !fs.existsSync(out)) {
      return { ok: false, error: `the SysML v2 text could not be imported: ${(result.stderr || result.stdout).trim().split('\n').slice(-3).join(' ')}` };
    }
    return { ok: true, text: fs.readFileSync(out, 'utf8'), imported: true };
  });
}

/** The text the document should hold for an ArcLang model text: itself, or its SysML v2 export. */
export async function documentTextFor(document: vscode.TextDocument, arcText: string): Promise<SourceOutcome> {
  const language = modelLanguage(document);
  if (language === 'arclang') return { ok: true, text: arcText, imported: false };
  if (language !== 'sysml') return { ok: false, error: 'not a model file (.arc or .sysml)' };
  return withTempFile(arcText, 'model.arc', async (file, dir) => {
    const out = path.join(dir, 'exported.sysml');
    const result = await run(['export', '-f', 'sys-ml', file, '-o', out], { cwd: dir });
    if (result.code !== 0 || !fs.existsSync(out)) {
      return { ok: false, error: `the edited model could not be exported to SysML v2: ${(result.stderr || result.stdout).trim().split('\n').slice(-3).join(' ')}` };
    }
    return { ok: true, text: fs.readFileSync(out, 'utf8'), imported: true };
  });
}

/** Replace the whole document with `text` as one undoable edit; false when VS Code refused. */
export async function replaceWhole(document: vscode.TextDocument, text: string): Promise<boolean> {
  if (text === document.getText()) return true;
  const whole = new vscode.Range(document.positionAt(0), document.positionAt(document.getText().length));
  const edit = new vscode.WorkspaceEdit();
  edit.replace(document.uri, whole, text);
  return vscode.workspace.applyEdit(edit);
}
