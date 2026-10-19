// Edits of a model from the views: each one is an `arclang edit` request,
// refused by the compiler unless the edited model compiles and shows what
// was asked. Several requests of one gesture are applied in order on one
// text and land in the editor as one undoable edit. A SysML v2 document is
// imported before and exported after: the gesture is the same.
import * as path from 'node:path';
import * as vscode from 'vscode';
import { run, withTempFile, workspaceTrusted, UNTRUSTED_MESSAGE } from '../cli';
import { EditRequest, parseEditAnswer } from '../pure/workbench';
import { arcTextOf, documentDir, documentTextFor, replaceWhole } from './source';

export type EditOutcome = { ok: true; source: string; applied: boolean } | { ok: false; error: string; at: number };

/** Run the requests on `source`, each on the result of the previous one. */
export async function editText(source: string, requests: EditRequest[], fileName: string, cwd?: string): Promise<EditOutcome> {
  if (!workspaceTrusted()) return { ok: false, error: UNTRUSTED_MESSAGE, at: 0 };
  let text = source;
  for (const [index, request] of requests.entries()) {
    const outcome = await withTempFile(text, fileName, async (file) => {
      // The request travels as one argument: no shell, no quoting, no file.
      const result = await run(['edit', file, '--request', JSON.stringify(request)], { cwd });
      return parseEditAnswer(result.stdout || result.stderr);
    });
    if (!outcome.ok) return { ok: false, error: outcome.error, at: index };
    text = outcome.source;
  }
  return { ok: true, source: text, applied: false };
}

/**
 * Apply the requests to the document: the whole text is replaced by the
 * compiler's result in one workspace edit, so one Undo takes the gesture back.
 */
export async function applyRequests(document: vscode.TextDocument, requests: EditRequest[]): Promise<EditOutcome> {
  const source = await arcTextOf(document);
  if (!source.ok) return { ok: false, error: source.error, at: 0 };
  const name = path.basename(document.uri.fsPath).replace(/\.sysml$/i, '.arc') || 'model.arc';
  const outcome = await editText(source.text, requests, name, documentDir(document));
  if (!outcome.ok) return outcome;
  const back = await documentTextFor(document, outcome.source);
  if (!back.ok) return { ok: false, error: back.error, at: requests.length };
  if (back.text === document.getText()) return { ...outcome, applied: false };
  const applied = await replaceWhole(document, back.text);
  if (!applied) return { ok: false, error: 'VS Code did not apply the edit', at: requests.length };
  return { ...outcome, applied: true };
}

/** A human line for a refused gesture. */
export function refusalMessage(label: string, outcome: Extract<EditOutcome, { ok: false }>): string {
  return `ArcLang refused "${label}": ${outcome.error}`;
}
