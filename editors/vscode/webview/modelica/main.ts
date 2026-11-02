// The Modelica drawing's page: the SVG the host computed, zoomed with the
// wheel or + − 0, moved by dragging; a part opens its own class's diagram on
// a click (when it has one), the path at the top goes back.
import type { Drawing, DrawnComponent } from '../../src/pure/modelica-drawing';

declare function acquireVsCodeApi(): { postMessage(message: unknown): void };
const vscode = acquireVsCodeApi();

type FromHost =
  | { type: 'drawing'; title: string; className: string; crumbs: string[]; drawing: Drawing }
  | { type: 'busy'; text: string }
  | { type: 'error'; text: string };

const $ = <T extends HTMLElement>(id: string) => document.getElementById(id) as T;
const stage = $<HTMLElement>('stage');
const tip = $<HTMLElement>('tip');
const status = $<HTMLElement>('status');
let svg: SVGSVGElement | null = null;
let home = { x: 0, y: 0, width: 100, height: 100 };
let view = { ...home };
let parts = new Map<string, DrawnComponent>();

function say(text: string, error = false): void {
  status.textContent = text;
  status.classList.toggle('error', error);
  status.hidden = !text;
}

function setView(next: typeof view): void {
  view = next;
  svg?.setAttribute('viewBox', `${view.x} ${view.y} ${view.width} ${view.height}`);
}

function zoom(factor: number, at?: { x: number; y: number }): void {
  if (!svg) return;
  const width = Math.min(home.width * 20, Math.max(home.width / 40, view.width / factor));
  const height = (width / view.width) * view.height;
  const cx = at?.x ?? view.x + view.width / 2;
  const cy = at?.y ?? view.y + view.height / 2;
  setView({ x: cx - ((cx - view.x) * width) / view.width, y: cy - ((cy - view.y) * height) / view.height, width, height });
}

/** A point of the page in the drawing's coordinates. */
function toDrawing(event: MouseEvent): { x: number; y: number } | undefined {
  const matrix = svg?.getScreenCTM();
  if (!svg || !matrix) return undefined;
  const p = new DOMPoint(event.clientX, event.clientY).matrixTransform(matrix.inverse());
  return { x: p.x, y: p.y };
}

function partAt(target: EventTarget | null): DrawnComponent | undefined {
  const group = target instanceof Element ? target.closest('g.component') : null;
  return group ? parts.get(group.getAttribute('data-name') ?? '') : undefined;
}

function show(message: Extract<FromHost, { type: 'drawing' }>): void {
  tip.hidden = true;
  const parsed = new DOMParser().parseFromString(message.drawing.svg, 'image/svg+xml');
  const root = parsed.documentElement;
  if (root.nodeName !== 'svg') return say('The drawing could not be read.', true);
  svg = document.importNode(root, true) as unknown as SVGSVGElement;
  parts = new Map(message.drawing.components.map((c) => [c.name, c]));
  for (const group of svg.querySelectorAll('g.component')) {
    const part = parts.get(group.getAttribute('data-name') ?? '');
    if (part?.opens) group.classList.add('opens');
  }
  stage.replaceChildren(svg);
  home = { ...message.drawing.viewBox };
  setView({ ...home });
  const crumbs = $<HTMLElement>('crumbs');
  crumbs.replaceChildren(
    ...message.crumbs.flatMap((label, index) => {
      const last = index === message.crumbs.length - 1;
      const item = document.createElement(last ? 'span' : 'button');
      item.textContent = label;
      item.className = last ? 'crumb current' : 'crumb';
      if (last) item.setAttribute('aria-current', 'page');
      else item.addEventListener('click', () => vscode.postMessage({ type: 'crumb', index }));
      return index ? [Object.assign(document.createElement('span'), { className: 'sep', textContent: '›' }), item] : [item];
    }),
  );
  const d = message.drawing;
  $<HTMLElement>('summary').textContent = `${message.className} · ${d.components.length} part${d.components.length === 1 ? '' : 's'}, ${d.connections} connection${d.connections === 1 ? '' : 's'}`;
  say(d.unplaced.length ? `Not placed, so not drawn: ${d.unplaced.join(', ')}` : '');
}

let drag: { x: number; y: number; view: typeof view } | null = null;
let moved = false;
// The part under the pointer when it went down: once the pointer is captured, the events no longer reach the part.
let pressed: DrawnComponent | undefined;
stage.addEventListener('pointerdown', (event) => {
  if (!svg || event.button !== 0) return;
  pressed = partAt(event.target);
  drag = { x: event.clientX, y: event.clientY, view: { ...view } };
  moved = false;
  stage.setPointerCapture(event.pointerId);
});
stage.addEventListener('pointermove', (event) => {
  const part = partAt(event.target);
  if (part && !drag) {
    tip.textContent = `${part.name} : ${part.className}${part.comment ? `\n${part.comment}` : ''}${part.opens ? '\nClick to open its diagram' : ''}`;
    tip.style.left = `${event.clientX + 14}px`;
    tip.style.top = `${event.clientY + 14}px`;
    tip.hidden = false;
  } else tip.hidden = true;
  if (!drag || !svg) return;
  const dx = event.clientX - drag.x;
  const dy = event.clientY - drag.y;
  if (Math.abs(dx) + Math.abs(dy) > 3) moved = true;
  const scale = view.width / stage.clientWidth;
  setView({ ...drag.view, x: drag.view.x - dx * Math.max(scale, view.height / stage.clientHeight), y: drag.view.y - dy * Math.max(scale, view.height / stage.clientHeight) });
});
stage.addEventListener('pointerup', () => {
  const part = moved ? undefined : pressed;
  drag = null;
  pressed = undefined;
  if (part?.opens) vscode.postMessage({ type: 'open', name: part.name });
});
stage.addEventListener('pointerleave', () => (tip.hidden = true));
stage.addEventListener('wheel', (event) => {
  event.preventDefault();
  zoom(event.deltaY < 0 ? 1.15 : 1 / 1.15, toDrawing(event));
}, { passive: false });
window.addEventListener('keydown', (event) => {
  if (event.key === '+' || event.key === '=') zoom(1.25);
  else if (event.key === '-') zoom(1 / 1.25);
  else if (event.key === '0') setView({ ...home });
  else return;
  event.preventDefault();
});
$('in').addEventListener('click', () => zoom(1.25));
$('out').addEventListener('click', () => zoom(1 / 1.25));
$('fit').addEventListener('click', () => setView({ ...home }));
$('source').addEventListener('click', () => vscode.postMessage({ type: 'source' }));

window.addEventListener('message', (event: MessageEvent<FromHost>) => {
  const message = event.data;
  if (message.type === 'drawing') show(message);
  else if (message.type === 'busy') say(message.text);
  else if (message.type === 'error') say(message.text, true);
});
vscode.postMessage({ type: 'ready' });
