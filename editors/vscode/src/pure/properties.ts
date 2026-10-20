// The properties pane without the DOM: which attributes a kind has and how
// they are grouped, what each control writes, what is wrong with a value
// before the compiler is asked, what an element inherits from its type,
// what it is related and traced to, and what the findings propose. Pure
// functions, tested with node:test; the page builds its controls on them.
import type { AttributeType, EditRequest, ElementRecord, Finding, Metamodel, MetamodelAttribute, Snapshot } from './workbench';

// --- Groups -------------------------------------------------------------------

export type GroupId = 'identity' | 'safety' | 'timing' | 'resources' | 'links' | 'structure' | 'general' | 'description';

export const GROUP_LABELS: Record<GroupId, string> = {
  identity: 'Identity',
  safety: 'Safety and assurance',
  timing: 'Timing',
  resources: 'Resources',
  links: 'References',
  structure: 'Structure',
  general: 'General',
  description: 'Description',
};

const GROUP_ORDER: GroupId[] = ['identity', 'general', 'safety', 'timing', 'resources', 'links', 'structure', 'description'];
const IDENTITY_KEYS = new Set(['id', 'name', 'is', 'tags']);
export const DESCRIPTION_KEYS = new Set(['description', 'rationale', 'summary', 'notes', 'comment', 'procedure', 'expected']);
const SAFETY_KEYS = /^(safety_level|asil|dal|sil|severity|exposure|controllability|failure_condition|integrity|criticality|mitigated_by|hazard)/;

/** The attribute's kind of value, read through a list (`List of Reference` is a list of references). */
export function valueShape(type: AttributeType | undefined): { list: boolean; item: string; of?: string } {
  if (!type) return { list: false, item: 'Text' };
  if (type.type === 'List') {
    const inner = (type.of as unknown as AttributeType | string | undefined) ?? 'Text';
    if (typeof inner === 'string') return { list: true, item: inner };
    return { list: true, item: inner.type, of: inner.of };
  }
  return { list: false, item: type.type, of: type.of };
}

/** The group an attribute goes in, as its type and its key say. */
export function groupOf(attribute: MetamodelAttribute): GroupId {
  const key = attribute.key;
  if (IDENTITY_KEYS.has(key)) return 'identity';
  if (DESCRIPTION_KEYS.has(key)) return 'description';
  if (SAFETY_KEYS.test(key)) return 'safety';
  const shape = valueShape(attribute.type);
  if (shape.item === 'Quantity') return shape.of === 'Time' || shape.of === 'Frequency' ? 'timing' : 'resources';
  if (shape.item === 'Reference') return 'links';
  if (shape.item === 'Boolean' || shape.item === 'Multiplicity' || shape.item === 'Number') return 'structure';
  return 'general';
}

export interface AttributeGroup {
  id: GroupId;
  label: string;
  attributes: MetamodelAttribute[];
}

/**
 * The attributes of an element in groups, in method order: the kind's
 * attributes from the metamodel, then those the element carries that the
 * kind does not declare (they show as text). Empty groups are left out.
 */
export function attributeGroups(element: ElementRecord, metamodel: Metamodel | null): AttributeGroup[] {
  const declared = metamodel?.kinds.find((k) => k.name === element.kind)?.attributes ?? [];
  const known = new Set(declared.map((a) => a.key));
  const extra: MetamodelAttribute[] = Object.keys(element.attributes)
    .filter((key) => !known.has(key) && !['from', 'to', 'from_port', 'to_port', 'label'].includes(key))
    .map((key) => ({ key, type: { type: 'Text' } }));
  const byGroup = new Map<GroupId, MetamodelAttribute[]>();
  for (const attribute of [...declared, ...extra]) {
    const group = groupOf(attribute);
    byGroup.set(group, [...(byGroup.get(group) ?? []), attribute]);
  }
  return GROUP_ORDER.filter((id) => (byGroup.get(id) ?? []).length > 0).map((id) => ({ id, label: GROUP_LABELS[id], attributes: byGroup.get(id) ?? [] }));
}

/** The attributes several elements all have, for one value set on all of them. */
export function commonAttributes(elements: ElementRecord[], metamodel: Metamodel | null): MetamodelAttribute[] {
  const kinds = [...new Set(elements.map((e) => e.kind))].map((kind) => metamodel?.kinds.find((k) => k.name === kind)?.attributes ?? []);
  if (kinds.length === 0) return [];
  const [first, ...rest] = kinds;
  return first.filter((attribute) => !['id', 'name'].includes(attribute.key) && rest.every((other) => other.some((a) => a.key === attribute.key)));
}

/** The value all the elements share for a key, '' when they differ or none has one. */
export function sharedValue(elements: ElementRecord[], key: string): { value: string; mixed: boolean } {
  const values = elements.map((e) => displayValue(e.attributes[key]));
  const distinct = [...new Set(values)];
  return distinct.length === 1 ? { value: distinct[0], mixed: false } : { value: '', mixed: true };
}

// --- Values ---------------------------------------------------------------------

export interface Unit {
  symbol: string;
  dimension: string;
  factor?: number;
}

/** The units the metamodel knows for a dimension (`Time`: ns, us, ms, s...). */
export function unitsOf(metamodel: Metamodel | null, dimension: string | undefined): Unit[] {
  const units = Array.isArray(metamodel?.units) ? (metamodel!.units as Unit[]) : [];
  if (!dimension) return [];
  const wanted = normaliseDimension(dimension);
  return units.filter((u) => normaliseDimension(u.dimension) === wanted);
}

/** `data rate`, `DataRate` and `datarate` are one dimension. */
export function normaliseDimension(dimension: string): string {
  return dimension.replace(/[\s_-]/g, '').toLowerCase();
}

/** A quantity as the snapshot carries it (`{ value, unit }`) or as text (`25 ms`). */
export function quantityOf(value: unknown): { value: string; unit: string } | null {
  if (value && typeof value === 'object' && 'value' in (value as Record<string, unknown>)) {
    const record = value as Record<string, unknown>;
    return { value: String(record.value), unit: String(record.unit ?? '') };
  }
  if (typeof value === 'number') return { value: String(value), unit: '' };
  if (typeof value === 'string') {
    const match = /^\s*([-+]?\d+(?:\.\d+)?(?:e[-+]?\d+)?)\s*(.*?)\s*$/i.exec(value);
    if (match) return { value: match[1], unit: match[2] };
  }
  return null;
}

/** The value as the pane shows it: lists joined, quantities with their unit, texts bare. */
export function displayValue(value: unknown): string {
  if (value === null || value === undefined) return '';
  if (Array.isArray(value)) return value.map((v) => displayValue(v)).join(', ');
  if (typeof value === 'object') {
    const quantity = quantityOf(value);
    if (quantity) return `${Number(quantity.value)} ${quantity.unit}`.trim();
    return JSON.stringify(value);
  }
  return String(value);
}

/** The items of a list value, as texts. */
export function listItems(value: unknown): string[] {
  if (Array.isArray(value)) return value.map((v) => displayValue(v));
  if (value === null || value === undefined || value === '') return [];
  return String(value)
    .replace(/^\[|\]$/g, '')
    .split(',')
    .map((s) => s.trim().replace(/^"|"$/g, ''))
    .filter(Boolean);
}

const quote = (text: string): string => JSON.stringify(text);
const isIdentifier = (text: string): boolean => /^[A-Za-z_][A-Za-z0-9_]*$/.test(text);

/** What a control writes, as ArcLang source for `set`: texts, references and enumeration members with a dash are quoted. */
export function fieldLiteral(type: AttributeType | undefined, raw: string | string[]): string {
  const shape = valueShape(type);
  const item = (text: string): string => {
    const t = text.trim();
    switch (shape.item) {
      case 'Number':
      case 'Integer':
      case 'Boolean':
        return t;
      case 'Quantity': {
        const q = quantityOf(t);
        return q && q.unit ? `${q.value} ${q.unit}` : t;
      }
      case 'Enum':
      case 'Enumeration':
        return isIdentifier(t) ? t : quote(t.replace(/^"|"$/g, ''));
      default:
        return /^".*"$/.test(t) ? t : quote(t);
    }
  };
  if (shape.list) {
    const items = Array.isArray(raw) ? raw : listItems(raw);
    return `[${items.filter((s) => s.trim()).map(item).join(', ')}]`;
  }
  return item(Array.isArray(raw) ? raw.join(', ') : raw);
}

export function enumValues(metamodel: Metamodel | null, name: string | undefined): string[] {
  const enums = metamodel?.enums;
  if (!name || !Array.isArray(enums)) return [];
  return (enums as { name: string; values: string[] }[]).find((e) => e.name === name)?.values ?? [];
}

export function enumDoc(metamodel: Metamodel | null, name: string | undefined): string {
  const enums = metamodel?.enums;
  if (!name || !Array.isArray(enums)) return '';
  return (enums as { name: string; doc?: string }[]).find((e) => e.name === name)?.doc ?? '';
}

/** The elements a reference attribute may point at: of its kind, any element when it names none of the model's kinds. */
export function referenceCandidates(of: string | undefined, records: ElementRecord[], self?: ElementRecord): ElementRecord[] {
  const others = records.filter((r) => r.uuid !== self?.uuid && r.kind !== 'Model' && !/Port$/.test(r.kind));
  if (!of || of === 'Element') return others;
  const kinds = of.split(/[,\s]+|\bor\b/).map((k) => k.trim().toLowerCase()).filter(Boolean);
  const matching = others.filter((r) => kinds.some((k) => r.kind.toLowerCase() === k || r.kind.toLowerCase().endsWith(k)));
  return matching.length ? matching : others;
}

/** Whether a reference names an element of the model (by id or by name). */
export function resolveReference(text: string, records: ElementRecord[]): ElementRecord | null {
  const t = text.trim().replace(/^"|"$/g, '');
  return records.find((r) => r.id === t) ?? records.find((r) => r.name === t) ?? null;
}

export interface CheckContext {
  metamodel: Metamodel | null;
  records: ElementRecord[];
}

/**
 * What is wrong with a value before the compiler sees it, in the reader's
 * words; null when the page has nothing to say (the compiler still checks).
 * The compiler accepts a quantity of the wrong dimension with a warning: the
 * page refuses it here.
 */
export function checkValue(attribute: MetamodelAttribute, raw: string | string[], context: CheckContext): string | null {
  const shape = valueShape(attribute.type);
  const items = Array.isArray(raw) ? raw : shape.list ? listItems(raw) : [raw];
  for (const text of items.map((s) => s.trim()).filter(Boolean)) {
    const problem = checkItem(attribute, shape, text, context);
    if (problem) return problem;
  }
  return null;
}

function checkItem(attribute: MetamodelAttribute, shape: ReturnType<typeof valueShape>, text: string, context: CheckContext): string | null {
  switch (shape.item) {
    case 'Number':
    case 'Integer':
      return /^[-+]?\d+(\.\d+)?$/.test(text) ? null : `${attribute.key} is a number: "${text}" is not one`;
    case 'Boolean':
      return /^(true|false)$/.test(text) ? null : `${attribute.key} is true or false`;
    case 'Multiplicity':
      return /^(\*|\d+(\.\.(\d+|\*))?)$/.test(text) ? null : `a multiplicity is written 1, 0..1, 1..* or *`;
    case 'Enum':
    case 'Enumeration': {
      const values = enumValues(context.metamodel, shape.of);
      return values.length === 0 || values.includes(text.replace(/^"|"$/g, '')) ? null : `${text} is not a ${shape.of}: one of ${values.join(', ')}`;
    }
    case 'Quantity': {
      const quantity = quantityOf(text);
      if (!quantity) return `${attribute.key} is a quantity: a number and a unit, e.g. 25 ${unitsOf(context.metamodel, shape.of)[0]?.symbol ?? 'ms'}`;
      if (!quantity.unit) return `${attribute.key} needs a unit (${unitsOf(context.metamodel, shape.of).map((u) => u.symbol).slice(0, 6).join(', ')})`;
      const units = unitsOf(context.metamodel, shape.of);
      if (units.length === 0 || units.some((u) => u.symbol === quantity.unit)) return null;
      const all = Array.isArray(context.metamodel?.units) ? (context.metamodel!.units as Unit[]) : [];
      const actual = all.find((u) => u.symbol === quantity.unit);
      const expected = `${shape.of?.toLowerCase()} (${units.map((u) => u.symbol).slice(0, 8).join(', ')})`;
      return actual ? `${quantity.unit} measures ${actual.dimension.toLowerCase()}; ${attribute.key} is a ${expected}` : `${quantity.unit} is not a unit the language knows; ${attribute.key} is a ${expected}`;
    }
    case 'Reference': {
      const target = resolveReference(text, context.records);
      if (!target) return `no element "${text.replace(/^"|"$/g, '')}" in the model`;
      // A reference to one kind of the metamodel (a Type, a Requirement) is checked against it.
      const kind = shape.of && context.metamodel?.kinds.some((k) => k.name === shape.of) ? shape.of : null;
      return kind && target.kind !== kind ? `${target.name || target.id} is ${article(kindLabel(target.kind).toLowerCase())}; ${humanise(attribute.key)} points at ${article(kindLabel(kind).toLowerCase())}` : null;
    }
    default:
      return null;
  }
}

// --- Inherited from the type --------------------------------------------------------

const TYPE_META = new Set(['required', 'description', 'from_kind', 'to_kind', 'id', 'name']);

/** The types an element is (`is: "Periodic task"` or a list of them). */
export function typesOf(element: ElementRecord, records: ElementRecord[]): ElementRecord[] {
  const written = element.attributes.is;
  const names = Array.isArray(written) ? written.map(String) : written ? [String(written)] : [];
  return names.map((name) => records.find((r) => r.kind === 'Type' && (r.id === name || r.name === name))).filter((t): t is ElementRecord => !!t);
}

export interface Inherited {
  key: string;
  /** The type's value. */
  value: unknown;
  from: ElementRecord;
  /** The element writes its own value, different from the type's. */
  redefined: boolean;
}

const same = (a: unknown, b: unknown): boolean => JSON.stringify(a) === JSON.stringify(b);

/**
 * What an element takes from its types: the type's attributes, each either
 * inherited (the element shows the type's value) or redefined (the element
 * writes another). The compiler merges the type's values into the element's
 * attributes; a value equal to the type's is read as inherited.
 */
export function inheritedOf(element: ElementRecord, records: ElementRecord[]): Inherited[] {
  const out: Inherited[] = [];
  const seen = new Set<string>();
  for (const type of typesOf(element, records)) {
    for (const [key, value] of Object.entries(type.attributes)) {
      if (TYPE_META.has(key) || seen.has(key)) continue;
      seen.add(key);
      const own = element.attributes[key];
      out.push({ key, value, from: type, redefined: own !== undefined && own !== null && !same(own, value) });
    }
  }
  return out;
}

// --- Relationships and traces ----------------------------------------------------------

export interface RelationRow {
  /** The relationship as the reader names it: `satisfies`, `Component exchange LaneFlow`, `owns`. */
  label: string;
  kind: string;
  other: ElementRecord | null;
  /** The edits that take the relationship away, when the pane can name them. */
  remove?: EditRequest[];
  /** A verification's verdict, from the test case. */
  verdict?: string;
}

const TRACE_INVERSE: Record<string, string> = {
  satisfies: 'satisfied by',
  realizes: 'realized by',
  implements: 'implemented by',
  allocates: 'allocated by',
  verifies: 'verified by',
  refines: 'refined by',
  validates: 'validated by',
};

/** `ComponentExchange` reads "Component exchange". */
export function kindLabel(kind: string): string {
  const words = kind.replace(/([a-z])([A-Z])/g, '$1 $2').replace(/_/g, ' ');
  return words.charAt(0).toUpperCase() + words.slice(1).toLowerCase();
}

/** A relationship designated as the edits designate it: its written id, else its name. */
function relationshipId(relationship: { name: string | null; attributes?: Record<string, unknown> }): string | null {
  const id = relationship.attributes?.id;
  return typeof id === 'string' ? id : relationship.name || null;
}

const TRACE_LIKE = new Set(['Trace', 'Realization', 'Verification', 'Typing', 'Involvement', 'Mitigation', 'Deployment', 'ThreatApplication', 'ThreatInvolvement', 'AssetMember', 'Specialization']);

/** What is owned by the element, and the relationships into and out of it (exchanges, links, transitions, messages...). */
export function relationsOf(element: ElementRecord, snapshot: Snapshot): { owner: ElementRecord | null; owned: RelationRow[]; incoming: RelationRow[]; outgoing: RelationRow[] } {
  const records = snapshot.elements?.elements ?? [];
  const byUuid = new Map(records.map((e) => [e.uuid, e]));
  const owner = element.owner ? (byUuid.get(element.owner) ?? null) : null;
  const owned = records.filter((e) => e.owner === element.uuid).map((other) => ({ label: kindLabel(other.kind), kind: other.kind, other, remove: [{ op: 'remove' as const, element: other.id }] }));
  const incoming: RelationRow[] = [];
  const outgoing: RelationRow[] = [];
  // Ports carry the exchanges of their owner: they count as its own.
  const ends = new Set([element.uuid, ...records.filter((e) => e.owner === element.uuid && /Port$/.test(e.kind)).map((e) => e.uuid)]);
  for (const relationship of snapshot.elements?.relationships ?? []) {
    if (TRACE_LIKE.has(relationship.kind)) continue;
    const id = relationshipId(relationship);
    const label = `${kindLabel(relationship.kind)}${relationship.name ? ` ${relationship.name}` : ''}`;
    const remove = id ? [{ op: 'remove' as const, element: id }] : undefined;
    const ownerOf = (uuid: string): ElementRecord | null => {
      const end = byUuid.get(uuid) ?? null;
      return end && /Port$/.test(end.kind) && end.owner ? (byUuid.get(end.owner) ?? end) : end;
    };
    if (ends.has(relationship.source)) outgoing.push({ label, kind: relationship.kind, other: ownerOf(relationship.target), remove });
    if (ends.has(relationship.target)) incoming.push({ label, kind: relationship.kind, other: ownerOf(relationship.source), remove });
  }
  return { owner, owned, incoming, outgoing };
}

export interface TraceGroup {
  label: string;
  rows: RelationRow[];
}

/** The traces of an element, by kind and direction: satisfies, verified by (with the verdict), realizes, deployments... */
export function tracesOf(element: ElementRecord, snapshot: Snapshot): TraceGroup[] {
  const records = snapshot.elements?.elements ?? [];
  const byId = new Map(records.map((e) => [e.id, e]));
  const byUuid = new Map(records.map((e) => [e.uuid, e]));
  const groups = new Map<string, RelationRow[]>();
  const add = (label: string, row: RelationRow) => groups.set(label, [...(groups.get(label) ?? []), row]);
  for (const link of snapshot.diagrams?.links ?? []) {
    if (link.source === element.id) {
      add(link.kind, { label: link.kind, kind: link.kind, other: byId.get(link.target) ?? null, remove: [{ op: 'remove_trace', from: link.source, kind: link.kind, to: link.target }] });
    }
    if (link.target === element.id) {
      add(TRACE_INVERSE[link.kind] ?? `${link.kind} (from)`, { label: link.kind, kind: link.kind, other: byId.get(link.source) ?? null, remove: [{ op: 'remove_trace', from: link.source, kind: link.kind, to: link.target }] });
    }
  }
  const seenOthers = (label: string) => new Set((groups.get(label) ?? []).map((r) => r.other?.uuid));
  for (const relationship of snapshot.elements?.relationships ?? []) {
    const source = byUuid.get(relationship.source) ?? null;
    const target = byUuid.get(relationship.target) ?? null;
    const out = relationship.source === element.uuid;
    const into = relationship.target === element.uuid;
    if (!out && !into) continue;
    switch (relationship.kind) {
      case 'Verification':
        if (into && source) add('verified by', { label: 'verified by', kind: 'Verification', other: source, verdict: source.attributes.verdict ? String(source.attributes.verdict) : undefined });
        if (out && target) add('verifies', { label: 'verifies', kind: 'Verification', other: target });
        break;
      case 'Realization':
        if (out && target && !seenOthers('realizes').has(target.uuid)) add('realizes', { label: 'realizes', kind: 'Realization', other: target });
        if (into && source && !seenOthers('realized by').has(source.uuid)) add('realized by', { label: 'realized by', kind: 'Realization', other: source });
        break;
      case 'Deployment':
        if (out && target) add('deploys', { label: 'deploys', kind: 'Deployment', other: target });
        if (into && source) add('deployed on', { label: 'deployed on', kind: 'Deployment', other: source });
        break;
      case 'Mitigation':
        if (out && target) add('mitigates', { label: 'mitigates', kind: 'Mitigation', other: target });
        if (into && source) add('mitigated by', { label: 'mitigated by', kind: 'Mitigation', other: source });
        break;
      case 'Involvement':
        if (out && target) add('involves', { label: 'involves', kind: 'Involvement', other: target });
        if (into && source) add('involved in', { label: 'involved in', kind: 'Involvement', other: source });
        break;
      default:
        break;
    }
  }
  // The functions a component holds are allocated to it (Arcadia's allocation).
  if (/Component|Actor|Node|Entity/.test(element.kind)) {
    for (const fn of records.filter((e) => e.owner === element.uuid && /Function|Activity/.test(e.kind))) add('allocated functions', { label: 'allocated', kind: 'Allocation', other: fn });
  }
  const order = ['satisfies', 'satisfied by', 'verified by', 'verifies', 'realizes', 'realized by', 'implements', 'implemented by', 'allocated functions', 'deploys', 'deployed on', 'involves', 'involved in', 'mitigates', 'mitigated by'];
  const rank = (label: string) => (order.indexOf(label) < 0 ? order.length : order.indexOf(label));
  return [...groups.entries()].sort((a, b) => rank(a[0]) - rank(b[0])).map(([label, rows]) => ({ label, rows }));
}

// --- Findings ---------------------------------------------------------------------------

export function findingsOf(element: ElementRecord, findings: Finding[]): Finding[] {
  return findings.filter((f) => f.element && (f.element === element.id || f.element === element.name));
}

export type QuickFix = { label: string; action: 'focus'; key: string } | { label: string; action: 'trace' } | { label: string; action: 'identify' };

/** What the pane offers for a finding, from the compiler's fix hint (`{ kind: set | trace | identify }`). */
export function quickFixes(finding: Finding): QuickFix[] {
  const hint = finding.fix as unknown as { kind?: string; key?: string } | string | null | undefined;
  if (!hint || typeof hint !== 'object') return [];
  switch (hint.kind) {
    case 'set':
      return hint.key ? [{ label: `Edit ${hint.key}`, action: 'focus', key: hint.key }] : [];
    case 'trace':
      return [{ label: 'Add a trace…', action: 'trace' }];
    case 'identify':
      return [{ label: 'Give it an id…', action: 'identify' }];
    default:
      return [];
  }
}

// --- Description --------------------------------------------------------------------------

export type Inline = { text: string; bold?: boolean; italic?: boolean; code?: boolean };
export type Block = { type: 'p' | 'h' | 'li' | 'code'; inline: Inline[] };

/** A little Markdown, as blocks of runs of text: no markup string ever reaches the page. */
export function markdownBlocks(text: string): Block[] {
  const blocks: Block[] = [];
  const lines = text.replace(/\r\n/g, '\n').split('\n');
  let paragraph: string[] = [];
  let code: string[] | null = null;
  const flush = () => {
    if (paragraph.length) blocks.push({ type: 'p', inline: inlineRuns(paragraph.join(' ')) });
    paragraph = [];
  };
  for (const line of lines) {
    if (/^```/.test(line)) {
      if (code) {
        blocks.push({ type: 'code', inline: [{ text: code.join('\n'), code: true }] });
        code = null;
      } else {
        flush();
        code = [];
      }
      continue;
    }
    if (code) {
      code.push(line);
      continue;
    }
    const heading = /^#{1,6}\s+(.*)$/.exec(line);
    const item = /^\s*(?:[-*+]|\d+\.)\s+(.*)$/.exec(line);
    if (heading) {
      flush();
      blocks.push({ type: 'h', inline: inlineRuns(heading[1]) });
    } else if (item) {
      flush();
      blocks.push({ type: 'li', inline: inlineRuns(item[1]) });
    } else if (!line.trim()) flush();
    else paragraph.push(line.trim());
  }
  if (code) blocks.push({ type: 'code', inline: [{ text: code.join('\n'), code: true }] });
  flush();
  return blocks;
}

export function inlineRuns(text: string): Inline[] {
  const runs: Inline[] = [];
  const pattern = /(\*\*[^*]+\*\*|__[^_]+__|`[^`]+`|\*[^*]+\*|_[^_]+_)/g;
  let at = 0;
  for (const match of text.matchAll(pattern)) {
    if (match.index! > at) runs.push({ text: text.slice(at, match.index) });
    const token = match[0];
    if (token.startsWith('**') || token.startsWith('__')) runs.push({ text: token.slice(2, -2), bold: true });
    else if (token.startsWith('`')) runs.push({ text: token.slice(1, -1), code: true });
    else runs.push({ text: token.slice(1, -1), italic: true });
    at = match.index! + token.length;
  }
  if (at < text.length) runs.push({ text: text.slice(at) });
  return runs;
}

/** `an actor`, `a type`. */
export function article(noun: string): string {
  return `${/^[aeiou]/i.test(noun) ? 'an' : 'a'} ${noun}`;
}

/** `safety_level` reads "Safety level". */
export function humanise(key: string): string {
  const words = key.replace(/_/g, ' ').trim();
  return words.charAt(0).toUpperCase() + words.slice(1);
}
