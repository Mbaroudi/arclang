// What an edit changed in the model and what it broke: the difference of two
// snapshots (elements, relationships, traces, findings, gate). The
// suggestions ask the AI only when this finds a consequence.
import type { ElementRecord, Finding, Snapshot } from '../../pure/workbench';

export interface ModelChange {
  created: ElementRecord[];
  removed: ElementRecord[];
  changed: ElementRecord[];
  addedTraces: string[];
  removedTraces: string[];
  /** Relationships (exchanges, allocations…) that appeared or went, in words. */
  addedRelationships: string[];
  removedRelationships: string[];
}

const sameRecord = (a: ElementRecord, b: ElementRecord) => a.name === b.name && a.owner === b.owner && a.kind === b.kind && JSON.stringify(a.attributes) === JSON.stringify(b.attributes);

export function diffSnapshots(before: Snapshot, after: Snapshot): ModelChange {
  const old = new Map((before.elements?.elements ?? []).map((e) => [e.uuid, e]));
  const now = new Map((after.elements?.elements ?? []).map((e) => [e.uuid, e]));
  const created = [...now.values()].filter((e) => !old.has(e.uuid));
  const removed = [...old.values()].filter((e) => !now.has(e.uuid));
  const changed = [...now.values()].filter((e) => old.has(e.uuid) && !sameRecord(old.get(e.uuid)!, e));
  const traceKey = (l: { kind: string; source: string; target: string }) => `${l.source} ${l.kind} ${l.target}`;
  const oldTraces = new Set((before.diagrams?.links ?? []).map(traceKey));
  const newTraces = new Set((after.diagrams?.links ?? []).map(traceKey));
  const name = (map: Map<string, ElementRecord>, uuid: string) => map.get(uuid)?.id ?? '?';
  const relationKey = (map: Map<string, ElementRecord>) => (r: { kind: string; name: string; source: string; target: string }) => `${r.kind}${r.name ? ` "${r.name}"` : ''} ${name(map, r.source)} -> ${name(map, r.target)}`;
  const oldRelations = new Set((before.elements?.relationships ?? []).filter((r) => r.kind !== 'Trace').map(relationKey(old)));
  const newRelations = new Set((after.elements?.relationships ?? []).filter((r) => r.kind !== 'Trace').map(relationKey(now)));
  return {
    created,
    removed,
    changed,
    addedTraces: [...newTraces].filter((t) => !oldTraces.has(t)),
    removedTraces: [...oldTraces].filter((t) => !newTraces.has(t)),
    addedRelationships: [...newRelations].filter((t) => !oldRelations.has(t)),
    removedRelationships: [...oldRelations].filter((t) => !newRelations.has(t)),
  };
}

export function isEmptyChange(change: ModelChange): boolean {
  return !change.created.length && !change.removed.length && !change.changed.length && !change.addedTraces.length && !change.removedTraces.length && !change.addedRelationships.length && !change.removedRelationships.length;
}

/** The ids the change touches, for `arclang impact` and for the context. */
export function touchedIds(change: ModelChange): string[] {
  const fromTraces = [...change.addedTraces, ...change.removedTraces].flatMap((t) => {
    const [source, , target] = t.split(' ');
    return [source, target];
  });
  return [...new Set([...change.changed, ...change.created, ...change.removed].map((e) => e.id).concat(fromTraces))].filter(Boolean);
}

const findingKey = (f: Finding) => `${f.rule}|${f.element ?? ''}|${f.message}`;

export interface Consequence {
  kind: 'broken-trace' | 'new-finding' | 'gate' | 'unsatisfied-requirement' | 'removed-relationship';
  message: string;
  elements: string[];
}

/** The consequences of going from `before` to `after` that deserve a proposal. */
export function consequencesOf(before: Snapshot, after: Snapshot, change: ModelChange = diffSnapshots(before, after)): Consequence[] {
  const out: Consequence[] = [];
  for (const trace of change.removedTraces) {
    const [source, kind, target] = trace.split(' ');
    out.push({ kind: 'broken-trace', message: `the trace ${source} ${kind} ${target} is gone`, elements: [source, target] });
  }
  const oldFindings = new Set(before.findings.map(findingKey));
  for (const finding of after.findings.filter((f) => !oldFindings.has(findingKey(f)) && f.severity !== 'info')) {
    out.push({ kind: 'new-finding', message: `[${finding.rule}] ${finding.message}`, elements: finding.element ? [finding.element] : [] });
  }
  // A requirement satisfied before and by nothing now.
  const satisfied = (s: Snapshot) => new Set((s.diagrams?.links ?? []).filter((l) => l.kind === 'satisfies').map((l) => l.target));
  const was = satisfied(before);
  const is = satisfied(after);
  for (const requirement of [...was].filter((r) => !is.has(r))) {
    if ((after.elements?.elements ?? []).some((e) => e.id === requirement)) out.push({ kind: 'unsatisfied-requirement', message: `requirement ${requirement} is satisfied by no element any more`, elements: [requirement] });
  }
  if (before.gate && after.gate) {
    const oldGate = new Set(before.gate.findings.map((f) => `${f.check}|${f.message}`));
    for (const blocker of after.gate.findings.filter((f) => !oldGate.has(`${f.check}|${f.message}`))) {
      out.push({ kind: 'gate', message: `new gate ${blocker.severity.toLowerCase()}: ${blocker.message}`, elements: [] });
    }
    if (before.gate.passed && !after.gate.passed && !out.some((c) => c.kind === 'gate')) out.push({ kind: 'gate', message: 'the production gate passed before this change and fails now', elements: [] });
    if (after.gate.requirements_verified < before.gate.requirements_verified) out.push({ kind: 'gate', message: `${before.gate.requirements_verified - after.gate.requirements_verified} requirement(s) no longer verified`, elements: [] });
  }
  for (const relation of change.removedRelationships.filter((r) => /Exchange|Link|Deployment|Allocation|Realization/.test(r))) {
    if (!change.removed.some((e) => relation.includes(` ${e.id} `) || relation.endsWith(` ${e.id}`))) continue;
    out.push({ kind: 'removed-relationship', message: `${relation} went with the deleted element`, elements: [] });
  }
  return out;
}

/** The change in words, for the prompt and the pane. */
export function describeChange(change: ModelChange): string[] {
  const label = (e: ElementRecord) => `${e.kind} ${e.id}${e.name && e.name !== e.id ? ` "${e.name}"` : ''}`;
  return [
    ...change.created.map((e) => `created ${label(e)}`),
    ...change.removed.map((e) => `deleted ${label(e)}`),
    ...change.changed.map((e) => `changed ${label(e)}`),
    ...change.addedTraces.map((t) => `new trace ${t}`),
    ...change.removedTraces.map((t) => `removed trace ${t}`),
    ...change.addedRelationships.map((t) => `new ${t}`),
    ...change.removedRelationships.map((t) => `removed ${t}`),
  ].slice(0, 60);
}
