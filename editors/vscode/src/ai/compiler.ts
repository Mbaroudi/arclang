// The compiler's checks on what an AI proposes, on a temporary copy of the
// model: the edit requests applied one after the other, a whole model text
// compiled, a snapshot of the result, `arclang impact` on an element.
import * as path from 'node:path';
import * as vscode from 'vscode';
import { failureText, run, withTempFile } from '../cli';
import { parseCheckReport } from '../pure/arc';
import { EditRequest, parseSnapshot, Snapshot } from '../pure/workbench';
import { editText } from '../workbench/edits';
import { arcTextOf, documentDir } from '../workbench/source';

export function modelFileName(document: vscode.TextDocument): string {
  return path.basename(document.uri.fsPath).replace(/\.sysml$/i, '.arc') || 'model.arc';
}

/** The ArcLang text of the document (a SysML v2 document is imported). */
export async function arcText(document: vscode.TextDocument): Promise<{ ok: true; text: string } | { ok: false; error: string }> {
  const source = await arcTextOf(document);
  return source.ok ? { ok: true, text: source.text } : { ok: false, error: source.error };
}

export type Checked = { ok: true; text: string } | { ok: false; error: string; at: number };

/** The requests applied by `arclang edit` on a copy of `text`; refused at the first one the compiler refuses. */
export async function checkRequests(document: vscode.TextDocument, text: string, requests: EditRequest[]): Promise<Checked> {
  const outcome = await editText(text, requests, modelFileName(document), documentDir(document));
  return outcome.ok ? { ok: true, text: outcome.source } : { ok: false, error: outcome.error, at: outcome.at };
}

/** A whole model text: accepted when it compiles without error. */
export async function checkSource(document: vscode.TextDocument, text: string): Promise<Checked> {
  return withTempFile(text, modelFileName(document), async (file, dir) => {
    const result = await run(['check', '--json', file], { cwd: dir });
    const parsed = parseCheckReport(result.stdout);
    if (result.code === 127) return { ok: false, error: failureText(result), at: 0 };
    if (!parsed) return { ok: false, error: failureText(result), at: 0 };
    if (parsed.errors.length) return { ok: false, error: parsed.errors.slice(0, 8).join('\n'), at: 0 };
    return { ok: true, text };
  });
}

/** The snapshot of a model text that is not in an editor. */
export async function snapshotOfText(document: vscode.TextDocument, text: string): Promise<Snapshot> {
  const standard = vscode.workspace.getConfiguration('arclang').get<string>('gate.standard', 'iso26262');
  return withTempFile(text, modelFileName(document), async (file, dir) => {
    const result = await run(['workbench', file, '--gate-standard', standard], { cwd: dir });
    return parseSnapshot(result.stdout || result.stderr);
  });
}

/** What `arclang impact` says about an element of the text. */
export async function impactOf(document: vscode.TextDocument, text: string, element: string, token?: vscode.CancellationToken): Promise<string> {
  return withTempFile(text, modelFileName(document), async (file, dir) => {
    const result = await run(['impact', '--', file, element], { cwd: dir, token });
    return result.code === 0 ? result.stdout.trim() : `impact ${element}: ${failureText(result)}`;
  });
}
