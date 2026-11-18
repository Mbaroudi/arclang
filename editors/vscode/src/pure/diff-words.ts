// The part of a changed line that changed: in a diff, a line removed and the
// line added in its place usually differ by a word or an attribute's value.

/** A line of a diff, with the part to mark when it is one of a replaced pair: `[start, end)` in `text`. */
export interface DiffLine {
  text: string;
  kind: 'add' | 'del' | 'hunk' | 'ctx';
  changed?: [number, number];
}

const kindOf = (line: string): DiffLine['kind'] => (line.startsWith('@@') ? 'hunk' : line.startsWith('+') ? 'add' : line.startsWith('-') ? 'del' : 'ctx');

const WORD = /[A-Za-z0-9_]/;

/**
 * What differs between two texts once their common start and end are set
 * aside, widened to whole words: `[start, end)` in each. Null when one has
 * nothing in common with the other (marking it all would say nothing).
 */
export function changedPart(before: string, after: string): { before: [number, number]; after: [number, number] } | null {
  if (before === after) return null;
  const shortest = Math.min(before.length, after.length);
  let start = 0;
  while (start < shortest && before[start] === after[start]) start++;
  let tail = 0;
  while (tail < shortest - start && before[before.length - 1 - tail] === after[after.length - 1 - tail]) tail++;
  // Whole words: a letter changed inside a word marks the word (text added between two words marks only itself).
  const word = (text: string, at: number) => at >= 0 && at < text.length && WORD.test(text[at]);
  if (word(before, start - 1) && (word(before, start) || word(after, start))) while (word(before, start - 1)) start--;
  const endBefore = before.length - tail;
  const endAfter = after.length - tail;
  if (word(before, endBefore) && ((endBefore > start && word(before, endBefore - 1)) || (endAfter > start && word(after, endAfter - 1)))) while (tail > 0 && word(before, before.length - tail)) tail--;
  if (!before.slice(0, start).trim() && !before.slice(before.length - tail).trim()) return null;
  return { before: [start, before.length - tail], after: [start, after.length - tail] };
}

/**
 * The lines of a unified diff, each with its kind; where a run of removed
 * lines is followed by as many added lines, each pair says which part
 * changed.
 */
export function refineDiff(lines: string[]): DiffLine[] {
  const out: DiffLine[] = lines.map((text) => ({ text, kind: kindOf(text) }));
  let at = 0;
  while (at < out.length) {
    if (out[at].kind !== 'del') {
      at++;
      continue;
    }
    let dels = at;
    while (dels < out.length && out[dels].kind === 'del') dels++;
    let adds = dels;
    while (adds < out.length && out[adds].kind === 'add') adds++;
    const removed = dels - at;
    if (removed === adds - dels) {
      for (let i = 0; i < removed; i++) {
        // The sign of the diff is not part of the line.
        const part = changedPart(out[at + i].text.slice(1), out[dels + i].text.slice(1));
        if (!part) continue;
        // Text only added (or only removed) marks the one line that has it.
        if (part.before[1] > part.before[0]) out[at + i].changed = [part.before[0] + 1, part.before[1] + 1];
        if (part.after[1] > part.after[0]) out[dels + i].changed = [part.after[0] + 1, part.after[1] + 1];
      }
    }
    at = adds;
  }
  return out;
}
