// What the AI is shown about a selection: the source of the selected
// elements, their relationships and neighbors, the findings on them, the
// metamodel of their kinds and the gate. Computed from the snapshot and the
// text; nothing here reads a file.
import type { ElementRecord, Metamodel, Snapshot } from '../../pure/workbench';

export type ContextScope = 'selection' | 'selectionAndNeighbours' | 'wholeModel';

export interface Span {
  start: number;
  end: number;
  startLine: number;
  endLine: number;
  text: string;
}

function lineOf(text: string, offset: number): number {
  let line = 0;
  for (let i = 0; i < offset && i < text.length; i++) if (text.charCodeAt(i) === 10) line++;
  return line;
}

function lineStart(text: string, offset: number): number {
  const at = text.lastIndexOf('\n', offset - 1);
  return at < 0 ? 0 : at + 1;
}

function lineEnd(text: string, offset: number): number {
  const at = text.indexOf('\n', offset);
  return at < 0 ? text.length : at;
}

/** Offsets of the code outside strings and comments, as a mask: true for code. */
function codeMask(text: string): Uint8Array {
  const mask = new Uint8Array(text.length);
  let i = 0;
  while (i < text.length) {
    const c = text[i];
    if (c === '/' && text[i + 1] === '/') {
      while (i < text.length && text[i] !== '\n') i++;
      continue;
    }
    if (c === '/' && text[i + 1] === '*') {
      const end = text.indexOf('*/', i + 2);
      i = end < 0 ? text.length : end + 2;
      continue;
    }
    if (c === '"') {
      i++;
      while (i < text.length && text[i] !== '"' && text[i] !== '\n') i += text[i] === '\\' ? 2 : 1;
      i++;
      continue;
    }
    mask[i] = 1;
    i++;
  }
  return mask;
}

/** The `{` that opens the innermost block around `offset`, or -1. */
function openerBefore(text: string, mask: Uint8Array, offset: number): number {
  let depth = 0;
  for (let i = offset - 1; i >= 0; i--) {
    if (!mask[i]) continue;
    if (text[i] === '}') depth++;
    else if (text[i] === '{') {
      if (depth === 0) return i;
      depth--;
    }
  }
  return -1;
}

/** The `}` that closes the block opened at `open`, or the end. */
function closerOf(text: string, mask: Uint8Array, open: number): number {
  let depth = 0;
  for (let i = open; i < text.length; i++) {
    if (!mask[i]) continue;
    if (text[i] === '{') depth++;
    else if (text[i] === '}') {
      depth--;
      if (depth === 0) return i;
    }
  }
  return text.length - 1;
}

/** The top-level block around an offset: `logical_architecture "Logical Architecture"`. */
export function topBlockAt(text: string, offset: number, mask: Uint8Array = codeMask(text)): string | null {
  let open = openerBefore(text, mask, offset);
  let outer = -1;
  while (open >= 0) {
    outer = open;
    open = openerBefore(text, mask, open);
  }
  if (outer < 0) return null;
  const line = text.slice(lineStart(text, outer), outer).trim();
  return line || null;
}

/** The blocks at the top of the model text (the layers, requirements, safety…), as they are written. */
export function topBlocks(text: string): string[] {
  const out: string[] = [];
  for (const match of text.matchAll(/^([a-z_]+)((?:[ \t]+[A-Za-z_]+)?(?:[ \t]+"[^"\n]*")?)[ \t]*\{/gm)) out.push(`${match[1]}${match[2]}`.trim());
  return out;
}

function spanOf(text: string, start: number, end: number): Span {
  return { start, end, startLine: lineOf(text, start), endLine: lineOf(text, end), text: text.slice(start, end) };
}

function escapeRegExp(text: string): string {
  return text.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

/**
 * The declaration of an element in the ArcLang text: the block that writes
 * its `id:`, else the line that declares its name (with its block when it
 * opens one). Null when the text does not spell it (an import, a SysML file).
 */
export function declarationSpan(text: string, element: Pick<ElementRecord, 'id' | 'name' | 'kind'>, mask: Uint8Array = codeMask(text)): Span | null {
  const blockFrom = (lineAt: number): Span => {
    const start = lineStart(text, lineAt);
    const end = lineEnd(text, lineAt);
    const brace = text.slice(lineAt, end).search(/\{/);
    const open = brace >= 0 ? lineAt + brace : -1;
    if (open >= 0 && mask[open]) return spanOf(text, start, lineEnd(text, closerOf(text, mask, open)));
    return spanOf(text, start, end);
  };
  if (element.id) {
    const idLine = new RegExp(`(^|\\n)[ \\t]*id:[ \\t]*"${escapeRegExp(element.id)}"`, 'g');
    const match = idLine.exec(text);
    if (match) {
      const at = match.index + match[1].length;
      const open = openerBefore(text, mask, at);
      if (open >= 0) return spanOf(text, lineStart(text, open), lineEnd(text, closerOf(text, mask, open)));
    }
  }
  // A port is `Owner.port`: its line inside the owner's block.
  if (/Port$/.test(element.kind) && element.id.includes('.')) {
    const owner = element.id.slice(0, element.id.lastIndexOf('.'));
    const port = element.id.slice(element.id.lastIndexOf('.') + 1);
    const ownerSpan = declarationSpan(text, { id: owner, name: owner, kind: '' }, mask);
    if (ownerSpan) {
      const inner = new RegExp(`\\bport[ \\t]+(?:in|out|inout)?[ \\t]*"${escapeRegExp(port)}"`).exec(ownerSpan.text);
      if (inner) return blockFrom(ownerSpan.start + inner.index);
    }
  }
  for (const name of [element.name, element.id].filter(Boolean)) {
    const declares = new RegExp(`(^|\\n)[ \\t]*[a-z_]+(?:[ \\t]+(?:in|out|inout|system|def))?[ \\t]+"${escapeRegExp(name)}"`, 'g');
    let match: RegExpExecArray | null;
    while ((match = declares.exec(text))) {
      const at = match.index + match[1].length;
      const keyword = at + match[0].slice(match[1].length).search(/[a-z_]/);
      if (mask[keyword]) return blockFrom(at);
    }
  }
  return null;
}

export interface ContextInput {
  snapshot: Snapshot;
  metamodel: Metamodel | null;
  text: string;
  fileName: string;
  /** The ids (or names) of the selected elements. */
  ids: string[];
  scope: ContextScope;
  /** The text the user selected in the editor, when the ask comes from it. */
  selectedText?: string;
  maxChars?: number;
}

export interface BuiltContext {
  text: string;
  /** The selected elements found in the model. */
  elements: ElementRecord[];
  /** The elements shown around them. */
  neighbors: ElementRecord[];
}

const clipText = (text: string, max: number) => (text.length > max ? `${text.slice(0, max)}\n… (${text.length - max} more characters)` : text);

function findElement(snapshot: Snapshot, key: string): ElementRecord | null {
  const records = snapshot.elements?.elements ?? [];
  return records.find((e) => e.id === key) ?? records.find((e) => e.name === key) ?? null;
}

function attributeText(value: unknown): string {
  if (value && typeof value === 'object' && (value as { '@type'?: string })['@type'] === 'Quantity') {
    const q = value as { value?: number; unit?: string };
    return `${q.value} ${q.unit ?? ''}`.trim();
  }
  return typeof value === 'string' ? JSON.stringify(value) : JSON.stringify(value);
}

/** One kind of the metamodel in a few lines: its layer and its attributes with their types. */
export function kindExcerpt(metamodel: Metamodel | null, kind: string): string | null {
  const found = metamodel?.kinds.find((k) => k.name === kind);
  if (!found) return null;
  const attributes = found.attributes.map((a) => `${a.key}: ${a.type.type}${a.type.of ? ` of ${a.type.of}` : ''}${a.type.values?.length ? ` (${a.type.values.join(' | ')})` : ''}`);
  return `- ${found.name} (${found.layer}${found.arcadia ? `, Arcadia ${found.arcadia}` : ''}): ${attributes.join('; ') || 'no attribute'}`;
}

/** The context of a selection, as one Markdown text for the AI. */
export function buildSelectionContext(input: ContextInput): BuiltContext {
  const { snapshot, text, scope } = input;
  const max = input.maxChars ?? 40000;
  const records = snapshot.elements?.elements ?? [];
  const byUuid = new Map(records.map((e) => [e.uuid, e]));
  const mask = codeMask(text);
  const elements = [...new Map(input.ids.map((id) => findElement(snapshot, id)).filter((e): e is ElementRecord => !!e).map((e) => [e.uuid, e])).values()];
  const uuids = new Set(elements.map((e) => e.uuid));
  const relationships = (snapshot.elements?.relationships ?? []).filter((r) => uuids.has(r.source) || uuids.has(r.target));
  const neighbourMap = new Map<string, ElementRecord>();
  if (scope !== 'selection') {
    for (const r of relationships) for (const end of [r.source, r.target]) if (!uuids.has(end) && byUuid.has(end)) neighbourMap.set(end, byUuid.get(end)!);
    for (const e of elements) {
      const owner = e.owner ? byUuid.get(e.owner) : undefined;
      if (owner && !uuids.has(owner.uuid) && owner.kind !== 'Model') neighbourMap.set(owner.uuid, owner);
    }
  }
  const neighbors = [...neighbourMap.values()];
  const label = (e: ElementRecord | undefined) => (e ? `${e.id}${e.name && e.name !== e.id ? ` "${e.name}"` : ''}` : '?');
  const sections: string[] = [];
  sections.push(`# Model ${input.fileName}`);
  if (input.selectedText) sections.push(`## The text the user selected\n\`\`\`arc\n${clipText(input.selectedText, Math.floor(max / 3))}\n\`\`\``);

  const declared: string[] = [];
  for (const e of elements) {
    const span = declarationSpan(text, e, mask);
    const owner = e.owner ? byUuid.get(e.owner) : undefined;
    const block = span ? topBlockAt(text, span.start, mask) : null;
    const head = `### ${e.kind} ${label(e)}${owner && owner.kind !== 'Model' ? `, owned by ${label(owner)} (${owner.kind})` : ''}${block && !(owner && owner.kind !== 'Model') ? `, in \`${block}\`` : ''}${span ? `, lines ${span.startLine + 1}-${span.endLine + 1}` : ''}`;
    const attributes = Object.entries(e.attributes).filter(([k]) => k !== 'id' && k !== 'name').map(([k, v]) => `${k}=${attributeText(v)}`).join(', ');
    declared.push(`${head}\n${span ? `\`\`\`arc\n${clipText(span.text, 6000)}\n\`\`\`` : `(not declared in this file)${attributes ? `\nattributes: ${attributes}` : ''}`}`);
  }
  sections.push(`## Selected elements (${elements.length})\n${declared.join('\n\n') || '(none found in the model: answer from the text)'}`);

  if (relationships.length) {
    const lines = relationships.slice(0, 80).map((r) => `- ${r.kind}${r.name ? ` "${r.name}"` : ''}: ${label(byUuid.get(r.source))} -> ${label(byUuid.get(r.target))}`);
    sections.push(`## Relationships of the selection (${relationships.length})\n${lines.join('\n')}`);
  }
  const ids = new Set(elements.flatMap((e) => [e.id, e.name]));
  const traces = (snapshot.diagrams?.links ?? []).filter((l) => ids.has(l.source) || ids.has(l.target));
  if (traces.length) sections.push(`## Traces of the selection\n${traces.map((l) => `- ${l.source} ${l.kind} ${l.target}`).join('\n')}`);

  if (scope === 'selectionAndNeighbours' && neighbors.length) {
    const shown = neighbors.slice(0, 20).map((n) => {
      const span = declarationSpan(text, n, mask);
      return `### ${n.kind} ${label(n)}\n${span ? `\`\`\`arc\n${clipText(span.text, 1500)}\n\`\`\`` : '(not declared in this file)'}`;
    });
    sections.push(`## Neighbors (${neighbors.length})\n${shown.join('\n\n')}`);
  } else if (scope === 'selection' && relationships.length) {
    sections.push(`## Neighbors (ids only)\n${[...new Set(relationships.flatMap((r) => [r.source, r.target]).filter((u) => !uuids.has(u)).map((u) => byUuid.get(u)).filter((e): e is ElementRecord => !!e).map((e) => `- ${e.kind} ${label(e)}`))].join('\n')}`);
  }

  const findings = snapshot.findings.filter((f) => f.element && ids.has(f.element));
  sections.push(`## Findings on the selection (${findings.length} of ${snapshot.findings.length} in the model)\n${findings.map((f) => `- [${f.rule}] ${f.severity}: ${f.message}`).join('\n') || 'none'}`);
  if (snapshot.gate) {
    const blockers = snapshot.gate.findings.slice(0, 12).map((f) => `- ${f.check} (${f.severity}): ${f.message}`).join('\n');
    sections.push(`## Production gate: ${snapshot.gate.passed ? 'PASS' : 'FAIL'}, ${snapshot.gate.requirements_verified}/${snapshot.gate.requirements_total} requirements verified\n${blockers}`);
  }
  const kinds = [...new Set([...elements, ...neighbors].map((e) => e.kind))];
  const excerpt = kinds.map((k) => kindExcerpt(input.metamodel, k)).filter((l): l is string => !!l);
  if (excerpt.length) sections.push(`## Metamodel of these kinds (from \`arclang metamodel\`)\n${excerpt.join('\n')}`);
  const blocks = topBlocks(text);
  if (blocks.length) sections.push(`## Top-level blocks of the model (the \`layer\` of an add is the first word)\n${blocks.slice(0, 40).map((l) => `- ${l}`).join('\n')}`);
  if (scope === 'wholeModel') sections.push(`## The whole model\n\`\`\`arc\n${clipText(text, Math.floor(max * 0.6))}\n\`\`\``);
  return { text: clipText(sections.join('\n\n'), max), elements, neighbors };
}

/** The elements whose declaration a range of the text covers or sits in (the innermost one around it). */
export function elementsInRange(text: string, records: ElementRecord[], start: number, end: number): ElementRecord[] {
  const mask = codeMask(text);
  const spans = records.map((e) => ({ e, span: declarationSpan(text, e, mask) })).filter((x): x is { e: ElementRecord; span: Span } => !!x.span && x.e.kind !== 'Model');
  const inside = spans.filter((x) => x.span.start >= lineStart(text, start) && x.span.end <= Math.max(end, lineEnd(text, start)));
  const around = spans.filter((x) => x.span.start <= start && x.span.end >= end).sort((a, b) => a.span.end - a.span.start - (b.span.end - b.span.start));
  // A selection that starts on a declaration's first line is about that element.
  const startsOn = around.filter((x) => x.span.startLine === lineOf(text, start));
  const pool = startsOn.length ? [...inside, startsOn[0]] : inside;
  const chosen = pool.length ? pool.filter((x) => !pool.some((o) => o !== x && o.span.start <= x.span.start && o.span.end >= x.span.end && (o.span.start !== x.span.start || o.span.end !== x.span.end))) : around.slice(0, 1);
  return [...new Map(chosen.map((x) => [x.e.uuid, x.e])).values()];
}
