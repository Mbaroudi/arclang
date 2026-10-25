// What a tool asks the user to confirm, as Markdown that shows exactly what
// the model sent: every value escaped (no link, no emphasis, no code span a
// value can close), every declaration whole, in a fence it cannot close.
// No VS Code here.
import type { EditRequest } from './workbench';

/** Text shown as is in Markdown: every character Markdown or HTML could read is escaped. */
export function mdText(value: unknown): string {
  const text = typeof value === 'string' ? value : JSON.stringify(value) ?? String(value);
  return text
    .replace(/[\\`*_{}[\]()#+\-.!|<>~&"'=:$]/g, (c) => `\\${c}`)
    .replace(/\r?\n/g, ' ⏎ ');
}

/** A fenced code block holding `text` whole: the fence is longer than any run of backticks inside. */
export function mdFence(text: string, language = 'arclang'): string {
  const longest = Math.max(0, ...[...String(text).matchAll(/`+/g)].map((m) => m[0].length));
  const fence = '`'.repeat(Math.max(3, longest + 1));
  return `${fence}${language}\n${String(text)}\n${fence}`;
}

const where = (r: { container?: string; layer?: string }): string =>
  r.container ? `inside ${mdText(r.container)}` : r.layer ? `in ${mdText(r.layer)}` : 'at the top level';

/** One request as a Markdown list item; a raw declaration follows in full, fenced. */
export function confirmRequest(request: EditRequest): string {
  switch (request.op) {
    case 'set':
      return `- set **${mdText(request.key)}** to ${mdText(request.value)} on ${mdText(request.element)}`;
    case 'unset':
      return `- remove **${mdText(request.key)}** from ${mdText(request.element)}`;
    case 'rename':
      return `- rename ${mdText(request.element)} to ${mdText(`"${request.name}"`)}`;
    case 'add':
      return `- add ${mdText(request.keyword)} ${mdText(`"${request.name}"`)} ${where(request)}${(request.attributes ?? []).length ? ` with ${(request.attributes ?? []).map(([k, v]) => `${mdText(k)}: ${mdText(v)}`).join(', ')}` : ''}`;
    case 'add_raw':
      return `- add this declaration ${where(request)}:\n\n${mdFence(String(request.declaration ?? '')).replace(/^/gm, '  ')}`;
    case 'remove':
      return `- delete ${mdText(request.element)}`;
    case 'move':
      return `- move ${mdText(request.element)} into ${mdText(request.container ?? request.layer ?? '?')}`;
    case 'add_trace':
      return `- trace ${mdText(request.from)} ${mdText(request.kind)} ${mdText(request.to)}`;
    case 'remove_trace':
      return `- remove the trace ${mdText(request.from)} ${mdText(request.kind)} ${mdText(request.to)}`;
    default:
      return `- the request:\n\n${mdFence(JSON.stringify(request, null, 2), 'json').replace(/^/gm, '  ')}`;
  }
}

/** The confirmation of a list of edits on a file (its path relative to the workspace). */
export function confirmEdits(file: string, requests: EditRequest[]): string {
  return `Apply to ${mdText(file)}:\n\n${requests.map(confirmRequest).join('\n')}\n\nThe compiler refuses the gesture unless the edited model compiles; it is applied in the editor as one undoable change, not saved.`;
}
