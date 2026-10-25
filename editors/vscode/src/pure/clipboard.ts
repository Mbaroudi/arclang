// The system clipboard of the workbench: a copy carries the ArcLang source
// of what is selected, so it can be pasted into another model's workbench
// or a text editor; ArcLang text pasted on the canvas becomes declarations
// of this model with ids of their own. The compiler checks every paste.
import { EditRequest, ElementRecord } from './workbench';

/** Where a declaration stands in a text: [start, end) offsets. */
export interface Span {
  start: number;
  end: number;
}

/** The offsets that are code (not inside a string or a comment), as a mask. */
function codeMask(text: string): Uint8Array {
  const mask = new Uint8Array(text.length);
  let i = 0;
  while (i < text.length) {
    const c = text[i];
    if (c === '"') {
      i++;
      while (i < text.length && text[i] !== '"') i += text[i] === '\\' ? 2 : 1;
      i++;
    } else if (c === '/' && text[i + 1] === '/') {
      while (i < text.length && text[i] !== '\n') i++;
    } else if (c === '/' && text[i + 1] === '*') {
      const close = text.indexOf('*/', i + 2);
      i = close < 0 ? text.length : close + 2;
    } else {
      mask[i] = 1;
      i++;
    }
  }
  return mask;
}

/** The `}` that closes the `{` at `open`, or -1. */
function closing(text: string, mask: Uint8Array, open: number): number {
  let depth = 0;
  for (let i = open; i < text.length; i++) {
    if (!mask[i]) continue;
    if (text[i] === '{') depth++;
    else if (text[i] === '}' && --depth === 0) return i;
  }
  return -1;
}

/** The innermost `{` still open at offset `at`, or -1. */
function enclosing(text: string, mask: Uint8Array, at: number): number {
  const open: number[] = [];
  for (let i = 0; i < at; i++) {
    if (!mask[i]) continue;
    if (text[i] === '{') open.push(i);
    else if (text[i] === '}') open.pop();
  }
  return open.at(-1) ?? -1;
}

const lineStart = (text: string, at: number): number => text.lastIndexOf('\n', at - 1) + 1;

/** From the start of the line of `open` to past its closing brace (and what ends that line). */
function blockSpan(text: string, mask: Uint8Array, open: number): Span | null {
  const close = closing(text, mask, open);
  if (close < 0) return null;
  const lineEnd = text.indexOf('\n', close);
  const rest = text.slice(close + 1, lineEnd < 0 ? text.length : lineEnd);
  // A comment after the brace stays with it; anything else on the line is not this declaration's.
  const end = /^\s*(\/\/.*)?$/.test(rest) ? (lineEnd < 0 ? text.length : lineEnd) : close + 1;
  return { start: lineStart(text, open), end };
}

const escapeRegExp = (s: string) => s.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
const quoted = (s: string) => `"${s.replace(/\\/g, '\\\\').replace(/"/g, '\\"')}"`;

const RELATIONSHIP_KEYWORDS = ['component_exchange', 'functional_exchange', 'exchange', 'connection', 'physical_link', 'link', 'physical_exchange', 'interaction', 'operational_exchange'];

/**
 * Where the declaration of an element is: the block that writes its `id:`,
 * else the block whose declaration line names it. Null when the text does
 * not declare it (an element from an import, a port written on its line).
 */
export function declarationSpan(text: string, element: Pick<ElementRecord, 'id' | 'name'>): Span | null {
  const mask = codeMask(text);
  const byId = new RegExp(`\\bid\\s*:\\s*${escapeRegExp(quoted(element.id))}`, 'g');
  for (const match of text.matchAll(byId)) {
    if (!mask[match.index]) continue;
    const open = enclosing(text, mask, match.index);
    if (open >= 0) return blockSpan(text, mask, open);
  }
  if (!element.name) return null;
  const byName = new RegExp(`^[ \\t]*[a-z_]+(?:[ \\t]+[a-z_]+)*[ \\t]+${escapeRegExp(quoted(element.name))}[^\\n{]*\\{`, 'gm');
  for (const match of text.matchAll(byName)) {
    const keyword = match[0].trim().split(/\s+/)[0];
    if (!mask[match.index + match[0].search(/\S/)] || RELATIONSHIP_KEYWORDS.includes(keyword) || keyword === 'trace') continue;
    return blockSpan(text, mask, match.index + match[0].length - 1);
  }
  return null;
}

/** The declaration of a named exchange or link, declared as a block. */
export function relationshipSpan(text: string, name: string): Span | null {
  const mask = codeMask(text);
  const re = new RegExp(`^[ \\t]*(?:${RELATIONSHIP_KEYWORDS.join('|')})[ \\t]+${escapeRegExp(quoted(name))}[^\\n{]*\\{`, 'gm');
  for (const match of text.matchAll(re)) {
    if (!mask[match.index + match[0].search(/\S/)]) continue;
    return blockSpan(text, mask, match.index + match[0].length - 1);
  }
  return null;
}

/** A declaration taken out of its text, its lines back to the indentation of its first. */
function dedented(text: string, span: Span): string {
  const lines = text.slice(span.start, span.end).replace(/\s+$/, '').split('\n');
  const indent = /^[ \t]*/.exec(lines[0])?.[0] ?? '';
  return lines.map((line) => (line.startsWith(indent) ? line.slice(indent.length) : line.trimStart())).join('\n');
}

/**
 * The ArcLang source of a selection: each element's declaration with what
 * it holds, then the exchanges named between them. Elements whose
 * declaration is not found are listed in `missing`.
 */
export function selectionSource(text: string, elements: Pick<ElementRecord, 'id' | 'name'>[], relationships: string[] = []): { text: string; missing: string[] } {
  const parts: string[] = [];
  const missing: string[] = [];
  const spans: Span[] = [];
  for (const element of elements) {
    const span = declarationSpan(text, element);
    if (!span) {
      missing.push(element.id);
      continue;
    }
    // One inside another already taken comes with it.
    if (spans.some((s) => s.start <= span.start && span.end <= s.end)) continue;
    spans.push(span);
    parts.push(dedented(text, span));
  }
  for (const name of relationships) {
    const span = relationshipSpan(text, name);
    if (span && !spans.some((s) => s.start <= span.start && span.end <= s.end)) parts.push(dedented(text, span));
  }
  return { text: parts.length ? `${parts.join('\n')}\n` : '', missing };
}

/** The top-level declarations of a piece of ArcLang text, each as written. */
export function splitDeclarations(text: string): string[] {
  const mask = codeMask(text);
  const out: string[] = [];
  let depth = 0;
  let start = -1;
  for (let i = 0; i <= text.length; i++) {
    const c = text[i];
    if (i < text.length && mask[i]) {
      if (c === '{') depth++;
      else if (c === '}') depth = Math.max(0, depth - 1);
      else if (start < 0 && /\S/.test(c)) start = i;
    }
    if (start < 0 && i < text.length && !mask[i] && text[i] === '"' && depth === 0) start = i;
    const endOfLine = i === text.length || c === '\n';
    if (endOfLine && depth === 0 && start >= 0) {
      // Its lines back to the indentation of its first (copied from inside a layer, say).
      const column = start - lineStart(text, start);
      const chunk = text
        .slice(start, i)
        .split('\n')
        .map((line, at) => (at === 0 ? line : line.replace(new RegExp(`^[ \\t]{0,${column}}`), '')))
        .join('\n')
        .trim();
      // A line that only closes or carries a comment is not a declaration.
      if (chunk && !/^(\/\/.*)?$/.test(chunk)) out.push(chunk);
      start = -1;
    }
  }
  return out;
}

/** `LC-X` -> `LC-X-2`, `LC-X-2` -> `LC-X-3`, as the compiler names a copy; never one in `taken`. */
export function freshId(id: string, taken: Set<string>): string {
  const match = /^(.*\S)-(\d+)$/.exec(id);
  const base = match ? match[1] : id;
  const width = match ? match[2].length : 1;
  for (let n = match ? Number(match[2]) + 1 : 2; ; n++) {
    const candidate = `${base}-${String(n).padStart(width, '0')}`;
    if (!taken.has(candidate)) return candidate;
  }
}

/** Whether a piece of text reads like ArcLang declarations (a keyword, then a name or a block). */
export function looksLikeArcLang(text: string): boolean {
  return splitDeclarations(text).some((d) => /^[a-z_]+(\s+[a-z_]+)*\s+("[^"]*"|[A-Za-z_][\w.-]*)/.test(d));
}

/**
 * Text from the clipboard as declarations of this model: every `id:` that
 * is taken here gets a fresh one (the references to it in the pasted text
 * follow), a top-level element named like one here is named apart
 * ("Camera 2"). One `add_raw` per declaration, into the container or the
 * layer: one gesture, one undo.
 */
export function pasteTextRequests(text: string, elements: ElementRecord[], where: { container?: string; layer?: string }): { requests: EditRequest[]; ids: string[] } | { error: string } {
  const declarations = splitDeclarations(text);
  if (declarations.length === 0) return { error: 'the clipboard holds no ArcLang declaration' };
  const taken = new Set(elements.flatMap((e) => [e.id, e.name]));
  const names = new Set(elements.map((e) => e.name));
  const renamed = new Map<string, string>();
  for (const match of text.matchAll(/\bid\s*:\s*"((?:[^"\\]|\\.)*)"/g)) {
    const id = match[1];
    if (renamed.has(id) || !taken.has(id)) continue;
    const next = freshId(id, taken);
    taken.add(next);
    renamed.set(id, next);
  }
  const ids: string[] = [];
  const requests: EditRequest[] = declarations.map((declaration) => {
    let written = declaration.replace(/"((?:[^"\\]|\\.)*)"/g, (whole, inner: string) => {
      for (const [from, to] of renamed) {
        if (inner === from) return quoted(to);
        if (inner.startsWith(`${from}.`)) return quoted(`${to}${inner.slice(from.length)}`);
      }
      return whole;
    });
    const header = written.split('\n')[0];
    const keyword = header.trim().split(/\s+/)[0];
    const isRelationship = keyword === 'trace' || keyword === 'transition' || keyword === 'message' || /->/.test(header);
    const name = /"((?:[^"\\]|\\.)*)"/.exec(header);
    if (name && !isRelationship && !RELATIONSHIP_KEYWORDS.includes(keyword) && names.has(name[1])) {
      let n = 2;
      while (names.has(`${name[1]} ${n}`)) n++;
      const fresh = `${name[1]} ${n}`;
      names.add(fresh);
      written = written.replace(name[0], quoted(fresh));
    } else if (name && RELATIONSHIP_KEYWORDS.includes(keyword) && names.has(name[1])) {
      let n = 2;
      while (names.has(`${name[1]} ${n}`)) n++;
      written = written.replace(name[0], quoted(`${name[1]} ${n}`));
    }
    const id = /\bid\s*:\s*"((?:[^"\\]|\\.)*)"/.exec(written);
    if (id) ids.push(id[1]);
    return where.container ? { op: 'add_raw', declaration: written, container: where.container } : { op: 'add_raw', declaration: written, layer: where.layer };
  });
  if (!where.container && !where.layer) return { error: 'this view has no layer to paste into: select an element to paste into' };
  return { requests, ids };
}
