// The graph of the impact set: the changed elements in the centre, one ring
// per distance, each element a disc coloured by severity (its layer inside),
// each link the last step of the path that explains it. Wheel zooms, a drag
// pans, a click selects the row, a double click shows it in its diagram.
import { ImpactedItem, ringLayout, SeedItem, RingNode } from '../../src/pure/impact';
import { el } from '../dom';

const NS = 'http://www.w3.org/2000/svg';

function svg<K extends keyof SVGElementTagNameMap>(tag: K, attributes: Record<string, string | number> = {}, children: (Node | string)[] = []): SVGElementTagNameMap[K] {
  const node = document.createElementNS(NS, tag);
  for (const [key, value] of Object.entries(attributes)) node.setAttribute(key, String(value));
  for (const child of children) node.appendChild(typeof child === 'string' ? document.createTextNode(child) : child);
  return node;
}

export interface GraphHost {
  select: (id: string) => void;
  reveal: (id: string) => void;
}

const short = (text: string, max = 20) => (text.length > max ? `${text.slice(0, max - 1)}…` : text);

export interface GraphView {
  root: HTMLElement;
  /** Mark the node of a selected row (and the path that leads to it). */
  select: (id: string | null) => void;
}

export function renderGraph(seeds: SeedItem[], items: ImpactedItem[], selected: string | null, host: GraphHost): GraphView {
  const { nodes, edges, radius } = ringLayout(seeds, items);
  const byId = new Map(nodes.map((n) => [n.id, n]));
  const pad = 90;
  const full = radius + pad;
  let view = { x: -full, y: -full, w: full * 2, h: full * 2 };
  const canvas = svg('svg', { class: 'ip-graph-svg', role: 'img', 'aria-label': `Impact graph: ${items.length} impacted elements around ${seeds.map((s) => s.name || s.id).join(', ')}`, preserveAspectRatio: 'xMidYMid meet' });
  const apply = () => canvas.setAttribute('viewBox', `${view.x} ${view.y} ${view.w} ${view.h}`);
  apply();

  const rings = new Set(items.map((i) => i.distance));
  const guide = svg('g', { class: 'ip-rings' });
  for (const distance of [...rings].sort((a, b) => a - b)) {
    const r = Math.max(distance * 120, (items.filter((i) => i.distance === distance).length * 46) / (2 * Math.PI));
    guide.append(svg('circle', { cx: 0, cy: 0, r, class: 'ip-ring' }), svg('text', { x: 0, y: -r - 5, class: 'ip-ring-label', 'text-anchor': 'middle' }, [`${distance}`]));
  }
  const lines = svg('g', { class: 'ip-edges' });
  const pathOf = new Map(items.map((i) => [i.id, i.path.map((s) => `${s.from}>${s.to}`)]));
  const edgeEls = new Map<string, SVGElement>();
  for (const edge of edges) {
    const a = byId.get(edge.from);
    const b = byId.get(edge.to);
    if (!a || !b) continue;
    const line = svg('line', { x1: a.x, y1: a.y, x2: b.x, y2: b.y, class: 'ip-edge' });
    line.append(svg('title', {}, [`${edge.from} ${edge.label} ${edge.to}`]));
    lines.appendChild(line);
    edgeEls.set(`${edge.from}>${edge.to}`, line);
  }
  const discs = svg('g', { class: 'ip-nodes' });
  const nodeEls = new Map<string, SVGGElement>();
  const draw = (n: RingNode) => {
    const isSeed = n.severity === 'seed';
    const g = svg('g', { class: `ip-node sev-${n.severity}`, transform: `translate(${n.x} ${n.y})`, tabindex: 0, role: 'button', 'data-id': n.id, 'aria-label': `${n.label}, ${n.severity}, distance ${n.distance}` });
    const item = items.find((i) => i.id === n.id);
    g.append(
      svg('title', {}, [item ? `${n.label} (${n.kind}, ${n.layer})\n${item.reason}\n${item.why}` : `${n.label} (${n.kind}, ${n.layer}): changed`]),
      svg('circle', { r: isSeed ? 17 : 12, class: 'ip-disc' }),
      svg('text', { class: 'ip-layer', y: 3, 'text-anchor': 'middle' }, [n.layer === 'Transverse' ? 'T' : n.layer.slice(0, 2)]),
      svg('text', { class: 'ip-label', y: isSeed ? 31 : 25, 'text-anchor': 'middle' }, [short(n.label, isSeed ? 28 : 18)]),
    );
    g.addEventListener('click', (event) => {
      event.stopPropagation();
      host.select(n.id);
    });
    g.addEventListener('dblclick', () => host.reveal(n.id));
    g.addEventListener('keydown', (event) => {
      if (event.key === 'Enter' || event.key === ' ') {
        event.preventDefault();
        host.select(n.id);
      }
    });
    nodeEls.set(n.id, g);
    discs.appendChild(g);
  };
  nodes.forEach(draw);
  canvas.append(guide, lines, discs);

  // Pan and zoom, by the viewBox.
  let drag: { x: number; y: number; vx: number; vy: number } | null = null;
  canvas.addEventListener('pointerdown', (event) => {
    if ((event.target as Element).closest('.ip-node')) return;
    drag = { x: event.clientX, y: event.clientY, vx: view.x, vy: view.y };
    canvas.setPointerCapture(event.pointerId);
    canvas.classList.add('is-panning');
  });
  canvas.addEventListener('pointermove', (event) => {
    if (!drag) return;
    const scale = view.w / canvas.getBoundingClientRect().width;
    view = { ...view, x: drag.vx - (event.clientX - drag.x) * scale, y: drag.vy - (event.clientY - drag.y) * scale };
    apply();
  });
  const endDrag = () => {
    drag = null;
    canvas.classList.remove('is-panning');
  };
  canvas.addEventListener('pointerup', endDrag);
  canvas.addEventListener('pointercancel', endDrag);
  canvas.addEventListener('wheel', (event) => {
    event.preventDefault();
    const factor = event.deltaY > 0 ? 1.12 : 1 / 1.12;
    const rect = canvas.getBoundingClientRect();
    const px = view.x + ((event.clientX - rect.left) / rect.width) * view.w;
    const py = view.y + ((event.clientY - rect.top) / rect.height) * view.h;
    view = { x: px - (px - view.x) * factor, y: py - (py - view.y) * factor, w: view.w * factor, h: view.h * factor };
    apply();
  }, { passive: false });

  const fit = () => {
    view = { x: -full, y: -full, w: full * 2, h: full * 2 };
    apply();
  };
  const tools = el('div', { class: 'ip-graph-tools' }, [
    el('button', { type: 'button', class: 'ip-tool', title: 'Fit the graph', 'aria-label': 'Fit the graph', text: '⤢' }),
    el('button', { type: 'button', class: 'ip-tool', title: 'Zoom in', 'aria-label': 'Zoom in', text: '+' }),
    el('button', { type: 'button', class: 'ip-tool', title: 'Zoom out', 'aria-label': 'Zoom out', text: '−' }),
  ]);
  const [fitButton, inButton, outButton] = Array.from(tools.children) as HTMLButtonElement[];
  const zoom = (factor: number) => {
    const cx = view.x + view.w / 2;
    const cy = view.y + view.h / 2;
    view = { x: cx - (view.w * factor) / 2, y: cy - (view.h * factor) / 2, w: view.w * factor, h: view.h * factor };
    apply();
  };
  fitButton.addEventListener('click', fit);
  inButton.addEventListener('click', () => zoom(1 / 1.3));
  outButton.addEventListener('click', () => zoom(1.3));

  const legend = el('div', { class: 'ip-legend', 'aria-hidden': 'true' }, [
    el('span', { class: 'ip-key sev-seed', text: 'changed' }),
    el('span', { class: 'ip-key sev-blocker', text: 'blocker' }),
    el('span', { class: 'ip-key sev-major', text: 'major' }),
    el('span', { class: 'ip-key sev-minor', text: 'minor' }),
    el('span', { class: 'ip-key-note', text: 'rings: distance' }),
  ]);
  const root = el('div', { class: 'ip-graph' }, [canvas as unknown as Node, tools, legend]);

  const select = (id: string | null) => {
    for (const node of nodeEls.values()) node.classList.remove('is-selected');
    for (const line of edgeEls.values()) line.classList.remove('is-path');
    if (!id) return;
    nodeEls.get(id)?.classList.add('is-selected');
    for (const step of pathOf.get(id) ?? []) edgeEls.get(step)?.classList.add('is-path');
  };
  select(selected);
  return { root, select };
}
