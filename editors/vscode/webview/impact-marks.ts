// The impact analysis on the workbench page: the elements an analysis
// reaches are outlined on the diagram by severity (the seeds apart), the
// layer tabs carry how many each holds, and the context menu and the
// floating toolbar open the analysis of the selection or a what-if.
// The analysis itself lives in its own tab (webview/impact/); this file
// only paints what the host sends (impact hook).
import type { ImpactMark, ImpactMarksMessage, Counts } from '../src/pure/impact';
import { VIEWPOINTS } from '../src/pure/workbench';
import type { ElementRecord } from '../src/pure/workbench';
import type { MenuAction, MenuItem } from './menu';

export interface ImpactPageHooks {
  post: (message: unknown) => void;
  host: HTMLElement;
  layerBar: HTMLElement;
  /** The id an element has on the drawing. */
  drawingId: (id: string) => string;
  selection: () => ElementRecord[];
}

const CLASSES = ['wb-impact-seed', 'wb-impact-blocker', 'wb-impact-major', 'wb-impact-minor'];
const RANK: Record<ImpactMark['severity'], number> = { blocker: 3, major: 2, minor: 1, seed: 4 };

let hooks: ImpactPageHooks | null = null;
let current: ImpactMarksMessage | null = null;
let painting = false;

export function installImpact(h: ImpactPageHooks): void {
  hooks = h;
  loadStyles();
  window.addEventListener('message', (event: MessageEvent) => {
    const message = event.data as ImpactMarksMessage | undefined;
    if (message?.type !== 'impact-marks') return;
    current = message.marks.length ? message : null;
    paint();
  });
  // The drawing and the layer tabs are built again by the page (a new snapshot, another view):
  // the marks are painted again on what was built.
  let timer: ReturnType<typeof setTimeout> | undefined;
  const again = () => {
    if (painting || !current) return;
    clearTimeout(timer);
    timer = setTimeout(paint, 120);
  };
  const observer = new MutationObserver(again);
  observer.observe(h.host, { childList: true, subtree: true });
  observer.observe(h.layerBar, { childList: true });
  h.post({ type: 'impact-hello' });
}

function loadStyles(): void {
  const base = document.querySelector<HTMLLinkElement>('link[href*="workbench.css"]');
  if (!base || document.querySelector('link[data-impact-styles]')) return;
  const link = document.createElement('link');
  link.rel = 'stylesheet';
  link.href = base.href.replace(/workbench\.css(\?.*)?$/, 'impact-marks.css');
  link.dataset.impactStyles = '';
  base.after(link);
}

function nodeOf(h: ImpactPageHooks, mark: ImpactMark): Element | null {
  for (const candidate of [h.drawingId(mark.id), mark.id, mark.name]) {
    if (!candidate) continue;
    const found = h.host.querySelector(`[data-id="${CSS.escape(candidate)}"]`);
    if (found) return found;
  }
  return null;
}

function paint(): void {
  if (!hooks) return;
  painting = true;
  try {
    for (const node of hooks.host.querySelectorAll('.wb-impact')) node.classList.remove('wb-impact', ...CLASSES);
    for (const old of hooks.host.querySelectorAll('title[data-impact]')) old.remove();
    if (current) {
      const best = new Map<Element, ImpactMark>();
      for (const mark of current.marks) {
        const node = nodeOf(hooks, mark);
        if (node && (!best.has(node) || RANK[mark.severity] > RANK[best.get(node)!.severity])) best.set(node, mark);
      }
      for (const [node, mark] of best) {
        node.classList.add('wb-impact', `wb-impact-${mark.severity}`);
        const title = document.createElementNS('http://www.w3.org/2000/svg', 'title');
        title.dataset.impact = '';
        title.textContent = mark.severity === 'seed' ? `Changed: ${mark.name || mark.id}` : `Impact (${mark.severity}): ${mark.name || mark.id}`;
        node.prepend(title);
      }
    }
    badges(hooks.layerBar, current?.counts ?? {});
    hooks.host.classList.toggle('has-impact', !!current);
  } finally {
    // The mutations just made are not a reason to paint again.
    painting = false;
  }
}

/** A count by severity on each layer tab that holds impacted elements. */
function badges(bar: HTMLElement, counts: Record<string, Counts>): void {
  for (const old of bar.querySelectorAll('.wb-impact-count')) old.remove();
  for (const tab of bar.querySelectorAll<HTMLElement>('.wb-layer')) {
    const name = tab.querySelector('.wb-layer-name')?.textContent ?? '';
    const view = VIEWPOINTS.find((v) => v.short === name)?.view;
    const count = view ? counts[view] : undefined;
    if (!count) continue;
    const total = count.blocker + count.major + count.minor;
    const worst = count.blocker ? 'blocker' : count.major ? 'major' : 'minor';
    const badge = document.createElement('span');
    badge.className = `wb-impact-count is-${worst}`;
    badge.textContent = String(total);
    badge.title = `${count.blocker} blocker${count.blocker === 1 ? '' : 's'}, ${count.major} major, ${count.minor} minor in this layer`;
    tab.appendChild(badge);
  }
}

function one(): ElementRecord | null {
  const selected = hooks?.selection() ?? [];
  return selected.length === 1 ? selected[0] : null;
}

/** "Analyse impact" and "What if…" in the context menu of one element (impact hook in editing.ts). */
export function withImpact(items: MenuItem[]): MenuItem[] {
  const element = one();
  if (!hooks || !element || element.kind === 'Model') return items;
  const open = (whatIf: boolean) => hooks?.post({ type: 'impact-open', id: element.id, whatIf });
  return [{ label: 'Analyse impact…', run: () => open(false) }, { label: 'What if…', run: () => open(true) }, 'separator', ...items];
}

/** The floating toolbar's impact button (impact hook in editing.ts). */
export function withImpactMini<T extends MenuAction & { glyph: string }>(actions: T[]): (T | (MenuAction & { glyph: string }))[] {
  const element = one();
  if (!hooks || !element || element.kind === 'Model') return actions;
  return [{ glyph: '⚡', label: 'Analyse impact of the selection', run: () => hooks?.post({ type: 'impact-open', id: element.id }) }, ...actions];
}

/** Open the analysis of an element, or its what-if form (the Properties pane's buttons). */
export function openImpact(id: string, whatIf = false): void {
  hooks?.post({ type: 'impact-open', id, whatIf });
}
