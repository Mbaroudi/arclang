// The change-impact report as `arclang impact -f json` computes it, and what
// the impact view does with it without VS Code: the command line of a
// query, the filters, the rings of the graph, the marks on the workbench
// diagrams, the text copied as a table, the what-if requests of the form.
import type { EditRequest } from './workbench';

export type Severity = 'blocker' | 'major' | 'minor';
export type Category = 'requirement' | 'test' | 'safety' | 'interface' | 'budget' | 'view' | 'type_user' | 'architecture';

export const LINK_KINDS = ['realization', 'allocation', 'deployment', 'exchange', 'interface', 'chain', 'satisfies', 'verifies', 'safety', 'containment', 'typing', 'constraint', 'view', 'behavior', 'reference'] as const;
export type LinkKind = (typeof LINK_KINDS)[number];

export const LINK_LABELS: Record<LinkKind, string> = {
  realization: 'Realizes / implements',
  allocation: 'Allocations',
  deployment: 'Deployments',
  exchange: 'Exchanges',
  interface: 'Ports and links',
  chain: 'Functional chains',
  satisfies: 'Satisfies',
  verifies: 'Verification',
  safety: 'Hazards and threats',
  containment: 'Owner and children',
  typing: 'Types',
  constraint: 'Constraints',
  view: 'Diagrams and tables',
  behavior: 'Scenarios and modes',
  reference: 'Other references',
};

export const CATEGORIES: { id: Category; label: string }[] = [
  { id: 'requirement', label: 'Requirements' },
  { id: 'test', label: 'Tests' },
  { id: 'safety', label: 'Safety' },
  { id: 'interface', label: 'Interfaces / ICD' },
  { id: 'budget', label: 'Budgets and constraints' },
  { id: 'type_user', label: 'Type users' },
  { id: 'architecture', label: 'Architecture' },
  { id: 'view', label: 'Views' },
];

export const LAYERS = ['OA', 'SA', 'LA', 'PA', 'EPBS', 'Data', 'Transverse'];
export const LAYER_VIEW: Record<string, string> = { OA: 'oab', SA: 'sab', LA: 'lab', PA: 'pab', EPBS: 'pbs', Data: 'cdb' };

export interface PathStep {
  from: string;
  relation: string;
  kind: string;
  to: string;
}

export interface ImpactedItem {
  id: string;
  uuid: string;
  name: string;
  kind: string;
  layer: string;
  category: Category;
  severity: Severity;
  reason: string;
  distance: number;
  why: string;
  path: PathStep[];
  integrity?: string;
  removed?: boolean;
}

export interface SeedItem {
  id: string;
  uuid: string;
  name: string;
  kind: string;
  layer: string;
}

export interface GateState {
  standard: string;
  passed: boolean;
  blockers: string[];
}

export interface Coverage {
  status: string;
  satisfied_by: string[];
  verified_by: string[];
}

export interface WhatIfResult {
  requests: EditRequest[];
  gate_before: GateState;
  gate_after: GateState;
  new_blockers: string[];
  removed_blockers: string[];
  constraints: { id: string; name: string; expression: string; before: boolean | null; after: boolean | null; left_before: string; left_after: string; right: string; flipped: boolean }[];
  budgets: { chain: string; name: string; budget_ms: number | null; before_ms: number | null; after_ms: number | null; ok_before: boolean; ok_after: boolean; missing: string[] }[];
  coverage_before: { requirements: number; satisfied: number; verified: number };
  coverage_after: { requirements: number; satisfied: number; verified: number };
  coverage: { requirement: string; name: string; before: Coverage; after: Coverage }[];
  verdicts: { test: string; name: string; before: string; after: string; stale: boolean; why: string }[];
  diff: {
    added: { id: string; name: string; element_type: string }[];
    removed: { id: string; name: string; element_type: string }[];
    modified: { id: string; name: string; element_type: string; changes: { field: string; old: string; new: string }[] }[];
  };
}

export interface ImpactReport {
  success: true;
  model: string;
  seeds: SeedItem[];
  depth: number;
  kinds: LinkKind[];
  direction: 'both' | 'downstream' | 'upstream';
  impacted: ImpactedItem[];
  summary: { total: number; blockers: number; majors: number; minors: number; by_layer: Record<string, number>; by_category: Record<string, number>; max_distance: number };
  gate: GateState;
  what_if?: WhatIfResult;
  since?: string;
  links: Record<string, number>;
}

/** What the view asks for: seeds, or a change, or the changes since a revision. */
export interface ImpactQuery {
  elements: string[];
  /** Edit requests of a what-if, never written. */
  change?: EditRequest[] | null;
  /** A what-if delete, with the traces and list entries naming it. */
  delete?: string | null;
  since?: string | null;
  depth: number;
  kinds: LinkKind[];
  direction: 'both' | 'downstream' | 'upstream';
  /** What the what-if is, in words (the header). */
  label?: string | null;
}

export interface ImpactFilters {
  layers: string[];
  categories: Category[];
  severities: Severity[];
  maxDistance: number;
  text: string;
}

export const DEFAULT_DEPTH = 4;
export const MAX_DEPTH = 8;

export function defaultQuery(elements: string[]): ImpactQuery {
  return { elements, depth: DEFAULT_DEPTH, kinds: [...LINK_KINDS], direction: 'both' };
}

export function isWhatIf(query: ImpactQuery): boolean {
  return !!(query.change?.length || query.delete);
}

/** The `arclang impact` command line of a query on a model file. */
export function impactArgs(file: string, query: ImpactQuery, gateStandard: string, format = 'json'): string[] {
  const args = ['impact', file];
  for (const element of query.elements) args.push('--element', element);
  if (query.change?.length) args.push('--change', JSON.stringify(query.change));
  if (query.delete) args.push('--delete', query.delete);
  if (query.since && !isWhatIf(query)) args.push('--since', query.since);
  args.push('--depth', String(Math.max(1, Math.min(MAX_DEPTH, Math.round(query.depth)))));
  if (query.kinds.length && query.kinds.length < LINK_KINDS.length) args.push('--kinds', query.kinds.join(','));
  if (query.direction !== 'both') args.push('--direction', query.direction);
  args.push('--gate-standard', gateStandard, '-f', format);
  return args;
}

export type Parsed = { ok: true; report: ImpactReport } | { ok: false; error: string };

/** The JSON answer of `arclang impact`, or why there is none. */
export function parseReport(stdout: string, stderr = ''): Parsed {
  try {
    const value = JSON.parse(stdout) as ImpactReport | { success: false; error: string };
    if (value && value.success === true && Array.isArray(value.impacted)) return { ok: true, report: value };
    if (value && value.success === false) return { ok: false, error: value.error };
  } catch {
    // not JSON: the error is on stderr
  }
  const text = (stderr || stdout).trim().replace(/^Error:\s*/, '').replace(/^Compilation error:\s*/, '');
  return { ok: false, error: text || 'arclang impact gave no answer' };
}

export function emptyFilters(): ImpactFilters {
  return { layers: [], categories: [], severities: [], maxDistance: MAX_DEPTH, text: '' };
}

/** The items the filters keep: none chosen in a group means every one. */
export function visibleItems(items: ImpactedItem[], filters: ImpactFilters): ImpactedItem[] {
  const text = filters.text.trim().toLowerCase();
  return items.filter(
    (item) =>
      (!filters.layers.length || filters.layers.includes(item.layer)) &&
      (!filters.categories.length || filters.categories.includes(item.category)) &&
      (!filters.severities.length || filters.severities.includes(item.severity)) &&
      item.distance <= filters.maxDistance &&
      (!text || [item.id, item.name, item.kind, item.why, item.reason].some((t) => t.toLowerCase().includes(text))),
  );
}

export interface Counts {
  blocker: number;
  major: number;
  minor: number;
}

export function countsOf(items: ImpactedItem[]): Counts {
  const counts: Counts = { blocker: 0, major: 0, minor: 0 };
  for (const item of items) counts[item.severity] += 1;
  return counts;
}

/** The segments of a path, each element clickable: `A`, `satisfies`, `B`... */
export function pathSegments(item: ImpactedItem): { element?: string; relation?: string }[] {
  if (!item.path.length) return [{ relation: item.why || item.reason }];
  const out: { element?: string; relation?: string }[] = [{ element: item.path[0].from }];
  for (const step of item.path) out.push({ relation: step.relation }, { element: step.to });
  return out;
}

export interface RingNode {
  id: string;
  label: string;
  kind: string;
  layer: string;
  severity: Severity | 'seed';
  distance: number;
  x: number;
  y: number;
}

export interface RingEdge {
  from: string;
  to: string;
  label: string;
}

/**
 * The impact set as rings around the seeds: one ring per distance, the
 * elements of a ring spread by layer then id so the same report always
 * draws the same, each edge the last step of an element's path.
 */
export function ringLayout(seeds: SeedItem[], items: ImpactedItem[], radiusStep = 120): { nodes: RingNode[]; edges: RingEdge[]; radius: number } {
  const nodes: RingNode[] = [];
  const seedCount = Math.max(1, seeds.length);
  seeds.forEach((seed, i) => {
    const angle = (2 * Math.PI * i) / seedCount;
    const r = seeds.length > 1 ? 28 : 0;
    nodes.push({ id: seed.id, label: seed.name || seed.id, kind: seed.kind, layer: seed.layer, severity: 'seed', distance: 0, x: r * Math.cos(angle), y: r * Math.sin(angle) });
  });
  const rings = new Map<number, ImpactedItem[]>();
  for (const item of items) rings.set(item.distance, [...(rings.get(item.distance) ?? []), item]);
  const layerOrder = (layer: string) => {
    const at = LAYERS.indexOf(layer);
    return at < 0 ? LAYERS.length : at;
  };
  let radius = 0;
  for (const [distance, members] of [...rings.entries()].sort((a, b) => a[0] - b[0])) {
    const sorted = [...members].sort((a, b) => layerOrder(a.layer) - layerOrder(b.layer) || a.id.localeCompare(b.id));
    // A ring wide enough for its elements: 46 px of arc each at least.
    const r = Math.max(distance * radiusStep, (sorted.length * 46) / (2 * Math.PI));
    radius = Math.max(radius, r);
    const offset = distance % 2 === 0 ? Math.PI / Math.max(sorted.length, 1) : 0;
    sorted.forEach((item, i) => {
      const angle = -Math.PI / 2 + offset + (2 * Math.PI * i) / sorted.length;
      nodes.push({ id: item.id, label: item.name || item.id, kind: item.kind, layer: item.layer, severity: item.severity, distance, x: r * Math.cos(angle), y: r * Math.sin(angle) });
    });
  }
  const present = new Set(nodes.map((n) => n.id));
  const edges: RingEdge[] = [];
  for (const item of items) {
    const last = item.path[item.path.length - 1];
    if (last && present.has(last.from)) edges.push({ from: last.from, to: item.id, label: last.relation });
  }
  return { nodes, edges, radius: radius || radiusStep };
}

export interface ImpactMark {
  id: string;
  name: string;
  severity: Severity | 'seed';
}

/** The marks of the workbench diagrams: severity by element, the seeds apart, and the count per diagram view. */
export function workbenchMarks(report: ImpactReport, items: ImpactedItem[]): { marks: ImpactMark[]; counts: Record<string, Counts> } {
  const marks: ImpactMark[] = report.seeds.map((s) => ({ id: s.id, name: s.name, severity: 'seed' as const }));
  const counts: Record<string, Counts> = {};
  for (const item of items) {
    if (item.removed) continue;
    marks.push({ id: item.id, name: item.name, severity: item.severity });
    const view = LAYER_VIEW[item.layer];
    if (!view || item.category === 'view') continue;
    counts[view] = counts[view] ?? { blocker: 0, major: 0, minor: 0 };
    counts[view][item.severity] += 1;
  }
  return { marks, counts };
}

const TSV_HEADER = ['Element', 'Name', 'Kind', 'Layer', 'Category', 'Severity', 'Distance', 'Reason', 'Why'];
const cells = (i: ImpactedItem) => [i.id, i.name, i.kind, i.layer, i.category, i.severity, String(i.distance), i.reason, i.why];

/** Tab-separated rows (what pastes as a table into a spreadsheet). */
export function toTsv(items: ImpactedItem[]): string {
  const clean = (t: string) => t.replace(/[\t\r\n]+/g, ' ');
  return [TSV_HEADER, ...items.map(cells)].map((l) => l.map(clean).join('\t')).join('\n') + '\n';
}

/** An HTML table of the same rows, for the rich clipboard. */
export function toHtmlTable(items: ImpactedItem[]): string {
  const esc = (t: string) => t.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
  const head = TSV_HEADER.map((h) => `<th>${esc(h)}</th>`).join('');
  const body = items.map((i) => `<tr>${cells(i).map((c) => `<td>${esc(c)}</td>`).join('')}</tr>`).join('');
  return `<table><thead><tr>${head}</tr></thead><tbody>${body}</tbody></table>`;
}

export type WhatIfForm =
  | { op: 'delete'; element: string }
  | { op: 'set'; element: string; key: string; value: string }
  | { op: 'retype'; element: string; type: string }
  | { op: 'move'; element: string; container: string };

/** The ArcLang source of a typed value: a quantity or a number as written, else a string. */
export function valueLiteral(raw: string): string {
  const text = raw.trim();
  if (/^-?\d+(\.\d+)?(\s*[A-Za-zµ°%/²³.]+)?$/.test(text) || /^(true|false)$/.test(text) || /^\[.*\]$/.test(text) || /^".*"$/.test(text)) return text;
  return JSON.stringify(text);
}

/** The query of a what-if form: never written, only computed. */
export function whatIfQuery(form: WhatIfForm, base: ImpactQuery): ImpactQuery {
  const common = { ...base, elements: [], since: null };
  switch (form.op) {
    case 'delete':
      return { ...common, change: null, delete: form.element, label: `What if ${form.element} is deleted` };
    case 'set':
      return { ...common, delete: null, change: [{ op: 'set', element: form.element, key: form.key.trim(), value: valueLiteral(form.value) }], label: `What if ${form.element}.${form.key.trim()} = ${form.value.trim()}` };
    case 'retype':
      return { ...common, delete: null, change: [{ op: 'set', element: form.element, key: 'is', value: JSON.stringify(form.type.trim()) }], label: `What if ${form.element} is a ${form.type.trim()}` };
    case 'move':
      return { ...common, delete: null, change: [{ op: 'move', element: form.element, container: form.container.trim() }], label: `What if ${form.element} moves into ${form.container.trim()}` };
  }
}

/** The title of the view: what is analysed. */
export function queryTitle(query: ImpactQuery): string {
  if (query.label) return query.label;
  if (query.since) return `Impact of my changes since ${query.since}`;
  if (query.elements.length === 1) return `Impact of ${query.elements[0]}`;
  return `Impact of ${query.elements.length} elements`;
}

/** What to ask the assistant about a report: the seeds, the counts, the worst items. */
export function askPrompt(report: ImpactReport, query: ImpactQuery, items: ImpactedItem[]): string {
  const rank = ['blocker', 'major', 'minor'];
  const worst = [...items].sort((a, b) => rank.indexOf(a.severity) - rank.indexOf(b.severity) || a.distance - b.distance).slice(0, 12);
  const lines = worst.map((i) => `- [${i.severity}] ${i.id} (${i.kind}, ${i.layer}): ${i.reason}; why: ${i.why}`);
  const w = report.what_if;
  const gate = w ? `Gate ${w.gate_before.passed ? 'PASS' : 'FAIL'} → ${w.gate_after.passed ? 'PASS' : 'FAIL'}${w.new_blockers.length ? `; new blockers: ${w.new_blockers.join(' | ')}` : ''}.` : `Gate ${report.gate.passed ? 'PASS' : 'FAIL'}.`;
  return [
    `@arclang Explain this change impact of the open model and propose mitigations (edits or verifications to plan). ${queryTitle(query)}.`,
    `${report.summary.blockers} blockers, ${report.summary.majors} majors, ${report.summary.minors} minors. ${gate}`,
    ...lines,
  ].join('\n');
}

/** The page's messages, host to page. */
export type ToImpactView =
  | { type: 'busy'; stale: boolean }
  | { type: 'report'; report: ImpactReport; query: ImpactQuery; file: string; computedMs: number; aiAvailable: boolean; applied: boolean }
  | { type: 'error'; error: string; query: ImpactQuery; file: string }
  | { type: 'result'; ok: boolean; message: string };

/** The page's messages, page to host. */
export type FromImpactView =
  | { type: 'ready' }
  | { type: 'query'; query: ImpactQuery }
  | { type: 'reveal'; id: string; open: boolean }
  | { type: 'analyze'; id: string }
  | { type: 'what-if'; form: WhatIfForm }
  | { type: 'open-what-if'; id: string }
  | { type: 'apply' }
  | { type: 'discard' }
  | { type: 'export'; format: 'md' | 'html' | 'xlsx' }
  | { type: 'copied'; ok: boolean; tsv: string; rows: number }
  | { type: 'ask' }
  | { type: 'highlight'; on: boolean };

/** The workbench page's marks (host to workbench page). */
export interface ImpactMarksMessage {
  type: 'impact-marks';
  marks: ImpactMark[];
  counts: Record<string, Counts>;
  title: string;
}

/** The workbench page's asks (workbench page to host). */
export type ImpactFromWorkbench = { type: 'impact-open'; id: string; whatIf?: boolean } | { type: 'impact-hello' };
