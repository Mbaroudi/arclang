// Marks on the drawing: the findings of an element on its box (with the
// findings as its tooltip), and what the last edit changed, pulsing once.
import type { ElementRecord, Finding, Snapshot } from '../src/pure/workbench';
import { button, clear, el } from './dom';
import { VIEWPOINTS, ViewerGraph } from '../src/pure/workbench';

export function markFindings(host: HTMLElement, findings: Finding[], drawingOf: (element: string) => string): void {
  const byElement = new Map<string, string[]>();
  for (const finding of findings) {
    if (!finding.element) continue;
    const key = drawingOf(finding.element);
    byElement.set(key, [...(byElement.get(key) ?? []), `${finding.severity}: ${finding.rule}: ${finding.message}`]);
  }
  for (const [id, lines] of byElement) {
    const node = host.querySelector<SVGGElement>(`[data-id="${CSS.escape(id)}"]`);
    if (!node || node.classList.contains('wb-has-finding')) continue;
    node.classList.add('wb-has-finding');
    if (lines.some((l) => l.startsWith('error'))) node.classList.add('is-error');
    const title = document.createElementNS('http://www.w3.org/2000/svg', 'title');
    title.textContent = lines.join('\n');
    node.prepend(title);
  }
}

/** What changed between two snapshots: the one element created (to bring forward), the elements changed (to pulse). */
export function changesOf(before: Snapshot, after: Snapshot): { created: ElementRecord | null; changed: ElementRecord[] } {
  const known = new Set((before.elements?.elements ?? []).map((e) => e.uuid));
  const created = (after.elements?.elements ?? []).filter((e) => !known.has(e.uuid) && !/Port$/.test(e.kind));
  const old = new Map((before.elements?.elements ?? []).map((e) => [e.uuid, JSON.stringify(e.attributes) + e.name + e.owner]));
  const changed = (after.elements?.elements ?? []).filter((e) => old.get(e.uuid) !== JSON.stringify(e.attributes) + e.name + e.owner);
  return { created: created.length === 1 ? created[0] : null, changed: changed.length > 50 ? [] : changed };
}

export function pulse(host: HTMLElement, ids: string[]): void {
  for (const id of ids) {
    const node = host.querySelector<HTMLElement>(`[data-id="${CSS.escape(id)}"]`);
    if (!node) continue;
    node.classList.add('wb-flash');
    setTimeout(() => node.classList.remove('wb-flash'), 900);
  }
}

/** The layers of the model, one big button each, the one on screen pressed. */
export function renderLayers(bar: HTMLElement, graphs: ViewerGraph[], current: string | null, show: (id: string) => void, more?: () => void): void {
  clear(bar);
  for (const viewpoint of VIEWPOINTS) {
    const own = graphs.filter((g) => g.arc.view === viewpoint.view);
    if (own.length === 0) continue;
    const active = own.some((g) => g.id === current);
    const item = button('', () => show(own[0].id), { class: `wb-layer${active ? ' is-active' : ''}`, 'aria-pressed': String(active), title: `${viewpoint.label}${own.length > 1 ? ` (${own.length} diagrams)` : ''}` });
    item.append(el('span', { class: 'wb-layer-name', text: viewpoint.short }));
    if (own.length > 1) item.appendChild(el('span', { class: 'wb-layer-count', text: String(own.length) }));
    bar.appendChild(item);
  }
  const extra = graphs.filter((g) => !VIEWPOINTS.some((v) => v.view === g.arc.view));
  if (extra.length > 0) {
    const active = extra.some((g) => g.id === current);
    bar.appendChild(button(`Other (${extra.length})`, () => show(extra[0].id), { class: `wb-layer${active ? ' is-active' : ''}`, 'aria-pressed': String(active) }));
  }
  // nav hook: every diagram of the model, by layer, in a list
  if (more) bar.appendChild(button('Diagrams…', more, { class: 'wb-layer wb-layer-more', title: 'Open a diagram by layer, here or in a tab of its own' }));
}
