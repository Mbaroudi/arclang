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
  const name = path.basename(document.uri.fsPath).replace(/\.sysml$/i, '.arc') || 'model.arc';
  // The text may change while the compiler works (a keystroke, another
  // edit): the result is written only onto the text it was computed from,
  // else computed again on the new text.
  for (let attempt = 0; attempt < MAX_ATTEMPTS; attempt++) {
    const version = document.version;
    const source = await arcTextOf(document);
    if (!source.ok) return { ok: false, error: source.error, at: 0 };
    const outcome = await editText(source.text, requests, name, documentDir(document));
    if (!outcome.ok) return outcome;
    const back = await documentTextFor(document, outcome.source);
    if (!back.ok) return { ok: false, error: back.error, at: requests.length };
    if (document.version !== version) continue;
    if (back.text === document.getText()) return { ...outcome, applied: false };
    const applied = await replaceWhole(document, back.text);
    if (!applied) return { ok: false, error: 'VS Code did not apply the edit', at: requests.length };
    return { ...outcome, applied: true };
  }
  return { ok: false, error: 'the text kept changing while the edit was computed: try again', at: 0 };
}

const MAX_ATTEMPTS = 3;

/** Run tasks one after the other, each on what the previous one left. */
export class EditQueue {
  private last: Promise<unknown> = Promise.resolve();

  run<T>(task: () => Promise<T>): Promise<T> {
    const next = this.last.then(task, task);
    this.last = next.catch(() => undefined);
    return next;
  }
}

/** A human line for a refused gesture. */
export function refusalMessage(label: string, outcome: Extract<EditOutcome, { ok: false }>): string {
  return `ArcLang refused "${label}": ${outcome.error}`;
}
