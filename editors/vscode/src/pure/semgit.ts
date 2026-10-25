// Semantic git, without VS Code: the shapes `arclang diff --from`,
// `arclang history`, `arclang baseline` print, the marks the workbench draws
// in compare mode, the ghosts of removed elements put back on the old
// diagrams, the element-level conflict markers `arclang merge` writes and
// how a choice resolves them. Pure functions, tested with node:test.
import { LAYER_VIEW } from './impact';
import type { ViewerEdge, ViewerGraph, ViewerNode, ViewerPayload } from './workbench';

export interface FieldChange {
  field: string;
  old: string;
  new: string;
}

export interface Transition {
  from: string;
  to: string;
}

export interface ElementChange {
  id: string;
  uuid: string;
  name: string;
  kind: string;
  status: 'added' | 'removed' | 'modified';
  renamed?: Transition;
  moved?: Transition;
  changes: FieldChange[];
}

export interface LayerGroup {
  layer: string;
  label: string;
  counts: { added: number; removed: number; modified: number };
  kinds: { kind: string; elements: ElementChange[] }[];
}

export interface RelationChange {
  status: 'added' | 'removed' | 'retargeted' | 'modified';
  category: string;
  kind: string;
  name: string;
  source: string;
  target: string;
  was?: Transition;
  changes: FieldChange[];
}

export interface GateState {
  standard: string;
  passed: boolean;
  blockers: string[];
}

export interface CoverageDelta {
  requirement: string;
  name: string;
  before: { status: string };
  after: { status: string };
}

export interface Consequences {
  gate_before: GateState;
  gate_after: GateState;
  new_blockers: string[];
  removed_blockers: string[];
  newly_unsatisfied: CoverageDelta[];
  newly_unverified: CoverageDelta[];
  coverage_before: { requirements: number; satisfied: number; verified: number };
  coverage_after: { requirements: number; satisfied: number; verified: number };
  constraints: { id: string; name: string; before: boolean | null; after: boolean | null }[];
  budgets: { chain: string; name: string; before_ms: number | null; after_ms: number | null; budget_ms: number | null; ok_after: boolean }[];
  impacted: number;
  impacted_blockers: number;
}

export interface LayoutChange {
  status: 'added' | 'removed' | 'changed';
  what: string;
  view?: string;
  element: string;
}

export interface RichDiff {
  model: string;
  from: string;
  to: string;
  from_commit?: string;
  to_commit?: string;
  model_changed: boolean;
  layout_only: boolean;
  summary: Record<string, number>;
  layers: LayerGroup[];
  relationships: RelationChange[];
  consequences: Consequences;
  layout: LayoutChange[];
}

/** The command line of a diff of a model file against a revision (exit 1 means "they differ"). */
export function diffArgs(file: string, from: string, to: string | null, gateStandard: string): string[] {
  const args = ['diff', file, '--from', from];
  if (to) args.push('--to', to);
  args.push('--gate-standard', gateStandard, '-f', 'json');
  return args;
}

export type ParsedDiff = { ok: true; diff: RichDiff } | { ok: false; error: string };

export function parseDiff(stdout: string, stderr = ''): ParsedDiff {
  try {
    const value = JSON.parse(stdout) as RichDiff;
    if (value && Array.isArray(value.layers) && value.consequences) return { ok: true, diff: value };
  } catch {
    // not JSON: the reason is on stderr
  }
  const text = (stderr || stdout).trim().replace(/^Error:\s*/, '').replace(/^Compilation error:\s*/, '');
  return { ok: false, error: text || 'arclang diff gave no answer' };
}

// --- Marks on the diagrams ---------------------------------------------------

export type MarkStatus = 'added' | 'removed' | 'modified' | 'moved' | 'renamed';

export interface CompareMark {
  id: string;
  uuid: string;
  name: string;
  kind: string;
  layer: string;
  status: MarkStatus;
  /** Lines saying what changed. */
  details: string[];
}

export function detailsOf(change: ElementChange): string[] {
  const lines: string[] = [];
  if (change.renamed) lines.push(`renamed: "${change.renamed.from}" → "${change.renamed.to}"`);
  if (change.moved) lines.push(`moved: ${change.moved.from || '(model)'} → ${change.moved.to || '(model)'}`);
  for (const c of change.changes) lines.push(!c.old ? `${c.field} = "${c.new}"` : !c.new ? `${c.field} removed (was "${c.old}")` : `${c.field}: "${c.old}" → "${c.new}"`);
  return lines;
}

/** The mark an element gets: added, removed, or the most telling of its modifications. */
export function markStatus(change: ElementChange): MarkStatus {
  if (change.status !== 'modified') return change.status;
  if (change.moved && !change.renamed && change.changes.length === 0) return 'moved';
  if (change.renamed && !change.moved && change.changes.length === 0) return 'renamed';
  return 'modified';
}

export function compareMarks(diff: RichDiff): CompareMark[] {
  const marks: CompareMark[] = [];
  for (const layer of diff.layers) {
    for (const group of layer.kinds) {
      for (const e of group.elements) marks.push({ id: e.id, uuid: e.uuid, name: e.name, kind: e.kind, layer: layer.layer, status: markStatus(e), details: detailsOf(e) });
    }
  }
  return marks;
}

export interface ViewCounts {
  added: number;
  removed: number;
  modified: number;
}

/** What each diagram's tab counts: the changes of the layer it shows. */
export function compareCounts(diff: RichDiff): Record<string, ViewCounts> {
  const counts: Record<string, ViewCounts> = {};
  for (const layer of diff.layers) {
    const view = LAYER_VIEW[layer.layer];
    if (!view) continue;
    counts[view] = { ...layer.counts };
  }
  return counts;
}

/** What the page receives to draw the comparison. */
export interface CompareState {
  ref: string;
  /** What it is compared with, for the header: `main`, `baseline v1`. */
  label: string;
  marks: CompareMark[];
  counts: Record<string, ViewCounts>;
  relationships: RelationChange[];
  consequences: Consequences;
  layoutOnly: boolean;
  layoutChanges: number;
  /** The element ids drawn as ghosts (removed in the model now). */
  ghosts: string[];
  changed: boolean;
  /** The drawing ids of the ghosts on the page (set by the host once the old diagrams are put back). */
  ghostDrawn?: string[];
}

export function compareState(diff: RichDiff, ref: string, label: string): CompareState {
  const marks = compareMarks(diff);
  return { ref, label, marks, counts: compareCounts(diff), relationships: diff.relationships, consequences: diff.consequences, layoutOnly: diff.layout_only, layoutChanges: diff.layout.length, ghosts: marks.filter((m) => m.status === 'removed').map((m) => m.id), changed: diff.model_changed };
}

// --- Ghosts of removed elements ----------------------------------------------

interface LayoutArrangement {
  view: string;
  open: string | null;
  folded?: string[];
  places: Record<string, unknown>;
  [key: string]: unknown;
}

interface LayoutLike {
  arclang_layout?: string;
  views?: Record<string, unknown>;
  arrangements?: LayoutArrangement[];
  [key: string]: unknown;
}

function clone<T>(value: T): T {
  return JSON.parse(JSON.stringify(value)) as T;
}

function nodeIds(nodes: ViewerNode[] | undefined, into: Set<string>): Set<string> {
  for (const n of nodes ?? []) {
    into.add(n.id);
    for (const p of n.ports ?? []) into.add(p.id);
    nodeIds(n.children, into);
  }
  return into;
}

function findNode(nodes: ViewerNode[] | undefined, id: string): ViewerNode | null {
  for (const n of nodes ?? []) {
    if (n.id === id) return n;
    const inner = findNode(n.children, id);
    if (inner) return inner;
  }
  return null;
}

/** Outermost nodes of `nodes` that are removed, with the id of the node that held them. */
function removedNodes(nodes: ViewerNode[] | undefined, removed: Set<string>, parent: string | null, out: { node: ViewerNode; parent: string | null }[]): void {
  for (const n of nodes ?? []) {
    if (removed.has(n.id) || (n.arc.uuid && removed.has(n.arc.uuid))) out.push({ node: n, parent });
    else removedNodes(n.children, removed, n.id, out);
  }
}

export interface Ghosted {
  diagrams: ViewerPayload;
  /** The ids drawn as ghosts, by drawing id. */
  drawn: string[];
}

/**
 * The diagrams as they are, with what the old ones drew of the removed
 * elements put back where they stood (inside the box that held them when it
 * still exists): drawn ghosted by the page, never part of the model. Their
 * places in the old layout file, when it had some, come with them.
 */
export function withGhosts(now: ViewerPayload, before: ViewerPayload | null, removedIds: string[], removedUuids: string[]): Ghosted {
  const result = clone(now);
  const drawn: string[] = [];
  if (!before || (removedIds.length === 0 && removedUuids.length === 0)) return { diagrams: result, drawn };
  const removed = new Set([...removedIds, ...removedUuids]);
  for (const old of before.graphs) {
    const graph = result.graphs.find((g) => g.id === old.id);
    if (!graph) continue;
    const found: { node: ViewerNode; parent: string | null }[] = [];
    removedNodes(old.children, removed, null, found);
    if (found.length === 0) continue;
    graph.children = graph.children ?? [];
    for (const { node, parent } of found) {
      const holder = parent ? findNode(graph.children, parent) : null;
      const ghost = clone(node);
      if (holder) (holder.children = holder.children ?? []).push(ghost);
      else graph.children.push(ghost);
      nodeIds([ghost], new Set()).forEach((id) => drawn.push(id));
    }
    // Exchanges that joined them to boxes still drawn.
    const present = nodeIds(graph.children, new Set());
    const known = new Set((graph.edges ?? []).map((e) => e.id));
    const ghostEdges: ViewerEdge[] = (old.edges ?? []).filter((e: ViewerEdge) => !known.has(e.id) && e.sources.every((s) => present.has(s)) && e.targets.every((t) => present.has(t)));
    if (ghostEdges.length) {
      graph.edges = [...(graph.edges ?? []), ...clone(ghostEdges)];
      for (const e of ghostEdges) drawn.push(e.id);
    }
  }
  const oldLayout = before.layout as LayoutLike | undefined;
  if (oldLayout?.arrangements && drawn.length) {
    const layout: LayoutLike = clone((result.layout as LayoutLike | undefined) ?? { arclang_layout: '1', views: {}, arrangements: [] });
    layout.arrangements = layout.arrangements ?? [];
    const ghostIds = new Set(drawn);
    for (const arrangement of oldLayout.arrangements) {
      const places = Object.fromEntries(Object.entries(arrangement.places ?? {}).filter(([id]) => ghostIds.has(id)));
      if (Object.keys(places).length === 0) continue;
      let target = layout.arrangements.find((a) => a.view === arrangement.view && (a.open ?? null) === (arrangement.open ?? null));
      if (!target) {
        target = { view: arrangement.view, open: arrangement.open ?? null, folded: [], places: {} };
        layout.arrangements.push(target);
      }
      target.places = { ...target.places, ...places };
    }
    result.layout = layout;
  }
  return { diagrams: result, drawn };
}

// --- Git refs ------------------------------------------------------------------

export interface GitRef {
  name: string;
  kind: 'branch' | 'remote' | 'tag' | 'baseline' | 'head';
  detail: string;
}

/** `git for-each-ref --format=%(refname)%09%(objectname:short)%09%(subject)` as a pick list: HEAD, branches, baselines, tags. */
export function parseRefs(output: string, current: string | null): GitRef[] {
  const refs: GitRef[] = [{ name: 'HEAD', kind: 'head', detail: 'the last commit' }];
  for (const line of output.split('\n')) {
    const [name, hash, subject] = line.split('\t');
    if (!name || !hash) continue;
    const detail = `${hash} ${subject ?? ''}`.trim();
    if (name.startsWith('refs/heads/')) refs.push({ name: name.slice('refs/heads/'.length), kind: 'branch', detail: current === name.slice('refs/heads/'.length) ? `current branch, ${detail}` : detail });
    else if (name.startsWith('refs/remotes/')) {
      const short = name.slice('refs/remotes/'.length);
      if (!short.endsWith('/HEAD')) refs.push({ name: short, kind: 'remote', detail });
    } else if (name.startsWith('refs/tags/baseline/')) refs.push({ name: name.slice('refs/tags/'.length), kind: 'baseline', detail });
    else if (name.startsWith('refs/tags/')) refs.push({ name: name.slice('refs/tags/'.length), kind: 'tag', detail });
  }
  const rank = { head: 0, branch: 1, baseline: 2, tag: 3, remote: 4 } as const;
  return refs.sort((a, b) => rank[a.kind] - rank[b.kind] || a.name.localeCompare(b.name));
}

// --- Merge conflicts -----------------------------------------------------------

export interface Conflict {
  /** 0-based line of the opening marker and of the closing marker. */
  start: number;
  middle: number;
  end: number;
  /** `component 'Controller' [LC-001]`. */
  label: string;
  element: string;
  reason: string;
  ours: string[];
  theirs: string[];
}

const OURS = /^<<<<<<< ours(?: (.*))?$/;
const THEIRS = /^>>>>>>> theirs(?: .*)?$/;

/** The element-level conflicts `arclang merge` left in a file. */
export function parseConflicts(text: string): Conflict[] {
  const lines = text.split(/\r?\n/);
  const out: Conflict[] = [];
  for (let i = 0; i < lines.length; i++) {
    const m = OURS.exec(lines[i]);
    if (!m) continue;
    let middle = -1;
    let end = -1;
    for (let j = i + 1; j < lines.length; j++) {
      if (lines[j] === '=======' && middle < 0) middle = j;
      else if (THEIRS.test(lines[j])) {
        end = j;
        break;
      }
    }
    if (middle < 0 || end < 0) continue;
    const header = m[1] ?? '';
    const colon = header.indexOf('): ') >= 0 ? header.indexOf(': ', header.indexOf(']') >= 0 ? header.indexOf(']') : 0) : header.indexOf(': ');
    const label = colon >= 0 ? header.slice(0, colon) : header;
    const reason = colon >= 0 ? header.slice(colon + 2) : '';
    const id = /\[([^\]]+)\]/.exec(label)?.[1] ?? label;
    out.push({ start: i, middle, end, label, element: id, reason, ours: lines.slice(i + 1, middle), theirs: lines.slice(middle + 1, end) });
    i = end;
  }
  return out;
}

export type Choice = 'ours' | 'theirs' | 'both';

/** The text with one conflict settled. */
export function resolveConflict(text: string, conflict: Conflict, choice: Choice): string {
  const eol = text.includes('\r\n') ? '\r\n' : '\n';
  const lines = text.split(/\r?\n/);
  const kept = choice === 'ours' ? conflict.ours : choice === 'theirs' ? conflict.theirs : [...conflict.ours, ...conflict.theirs];
  lines.splice(conflict.start, conflict.end - conflict.start + 1, ...kept);
  return lines.join(eol);
}

export function hasConflicts(text: string): boolean {
  return parseConflicts(text).length > 0;
}

// --- History and baselines ---------------------------------------------------------

export interface HistoryEntry {
  hash: string;
  short: string;
  author: string;
  date: string;
  subject: string;
  status: 'added' | 'removed' | 'modified';
  renamed?: Transition;
  moved?: Transition;
  changes: FieldChange[];
  relationships: RelationChange[];
}

export interface ElementHistory {
  model: string;
  element: string;
  id: string;
  name: string;
  entries: HistoryEntry[];
  skipped: string[];
}

export function parseHistory(stdout: string, stderr = ''): { ok: true; history: ElementHistory } | { ok: false; error: string } {
  try {
    const value = JSON.parse(stdout) as ElementHistory;
    if (value && Array.isArray(value.entries)) return { ok: true, history: value };
  } catch {
    // not JSON
  }
  return { ok: false, error: (stderr || stdout).trim().replace(/^Error:\s*/, '') || 'arclang history gave no answer' };
}

export function entryDetails(entry: HistoryEntry): string[] {
  return detailsOf({ id: '', uuid: '', name: '', kind: '', status: entry.status, renamed: entry.renamed, moved: entry.moved, changes: entry.changes });
}

export interface BaselineListed {
  name: string;
  tag: boolean;
  created: string;
  standard: string;
  commit: string;
  gate: string;
}

export function parseBaselines(stdout: string): BaselineListed[] {
  try {
    const value = JSON.parse(stdout) as BaselineListed[];
    return Array.isArray(value) ? value : [];
  } catch {
    return [];
  }
}

export const BASELINE_STANDARDS = ['iso26262', 'do178c', 'iec61508'] as const;

/** A baseline name git and the file system both accept. */
export function validBaselineName(name: string): string | null {
  if (!name.trim()) return 'Give the baseline a name.';
  if (!/^[A-Za-z0-9][A-Za-z0-9._-]{0,99}$/.test(name) || name.includes('..')) return 'Letters, digits, ".", "_" and "-" only, starting with a letter or a digit.';
  return null;
}

/** The gate of a baseline about to be made, in one line (what `arclang gate` printed). */
export function gateLine(passed: boolean, blockers: number, standard: string): string {
  return passed ? `Gate ${standard}: PASS` : `Gate ${standard}: FAIL (${blockers} blocker${blockers === 1 ? '' : 's'})`;
}

export type { ViewerGraph };
