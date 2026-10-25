// What the panes add to the page: an element of the side bar's tree
// dropped on the diagram (revealed, or added to a declared diagram), a page
// that shows one diagram only (a diagram's own tab), and "Open in a new
// tab" on the layers of the bar.
import { dropPlan, idOfPayload, PanesToHost, PanesToPage, TREE_MIME } from '../src/pure/panes';
import { EditRequest, ElementRecord, Snapshot, topLevelIds, ViewerGraph, VIEWPOINTS } from '../src/pure/workbench';
import { el } from './dom';
import type { ViewerApi } from './globals';
import { ContextMenu } from './menu';

export interface PanesDeps {
  sheet: HTMLElement;
  layerBar: HTMLElement;
  viewer: () => ViewerApi | null;
  graph: () => ViewerGraph | null;
  snapshot: () => Snapshot | null;
  /** The model id of a box drawn on the diagram. */
  idOf: (drawingId: string) => string | null;
  select: (element: ElementRecord) => void;
  submit: (requests: EditRequest[], label: string) => void;
  toast: (text: string, kind: 'info' | 'error') => void;
  post: (message: PanesToHost) => void;
  showGraph: (id: string) => void;
}

export function installPanes(d: PanesDeps): void {
  const menu = new ContextMenu(d.sheet);

  function drop(id: string): void {
    const snapshot = d.snapshot();
    const element = snapshot?.elements?.elements.find((e) => e.id === id) ?? snapshot?.elements?.elements.find((e) => e.name === id) ?? null;
    const graph = d.graph();
    if (!element || !graph) return d.toast(`Nothing to show: ${id} is not an element of this model.`, 'error');
    const top = topLevelIds(graph).map((node) => d.idOf(node)).filter((x): x is string => !!x);
    const plan = dropPlan(element, graph, top);
    if (plan.kind === 'reveal') {
      d.viewer()?.select?.(plan.drawingId);
      d.select(element);
      d.toast(`${element.name || element.id} is on this diagram: selected.`, 'info');
    } else if (plan.kind === 'edit') d.submit(plan.requests, plan.label);
    else d.toast(plan.reason, 'info');
  }

  // A drag from the tree (Shift held: VS Code hands an editor's webview a drag only so):
  // the tree item's resource URI, as a uri list or VS Code's own list of resources.
  const PAYLOAD_TYPES = [TREE_MIME, 'text/uri-list', 'resourceurls', 'codeeditors', 'text/plain'];
  const accepts = (event: DragEvent) => {
    const types = [...(event.dataTransfer?.types ?? [])].map((type) => type.toLowerCase());
    return types.some((type) => PAYLOAD_TYPES.includes(type) || type.startsWith('application/vnd.code.tree.'));
  };
  d.sheet.addEventListener('dragover', (event) => {
    if (!accepts(event)) return;
    event.preventDefault();
    if (event.dataTransfer) event.dataTransfer.dropEffect = 'copy';
    d.sheet.classList.add('wb-drop-over');
  });
  d.sheet.addEventListener('dragleave', () => d.sheet.classList.remove('wb-drop-over'));
  d.sheet.addEventListener('drop', (event) => {
    d.sheet.classList.remove('wb-drop-over');
    const data = event.dataTransfer;
    if (!data) return;
    const types = [...new Set([...PAYLOAD_TYPES, ...data.types])];
    const text = types.map((type) => data.getData(type)).find((value) => value && idOfPayload(value));
    const id = text ? idOfPayload(text) : null;
    event.preventDefault();
    if (!id) return d.toast('This drag does not name an element of the model: right-click the node in the tree › Show in the diagram.', 'info');
    drop(id);
  });

  window.addEventListener('message', (event: MessageEvent<PanesToPage>) => {
    const message = event.data;
    if (message?.type === 'panes-drop') drop(message.id);
    if (message?.type === 'panes-pin') pin(message.view, message.title);
  });

  /** This page shows one diagram: the bar names it, the layers are left to the main workbench. */
  let pinned: string | null = null;
  const pinTitle = el('div', { class: 'wb-pin-title', hidden: true, role: 'heading', 'aria-level': '1' });
  d.layerBar.before(pinTitle);
  function pin(view: string, title: string): void {
    pinned = view;
    document.body.classList.add('wb-pinned');
    pinTitle.textContent = title;
    pinTitle.hidden = false;
    const show = () => void d.viewer()?.ready.then(() => d.showGraph(view));
    show();
  }
  // A diagram's own tab learns which diagram it shows.
  d.post({ type: 'panes-hello' });

  // A remount on a new snapshot keeps the view on screen; a pinned page puts its own back if it moved.
  setInterval(() => {
    if (!pinned) return;
    const current = d.viewer()?.current?.()?.view;
    if (current && current !== pinned && d.viewer()?.views?.().some((v) => v.id === pinned)) d.showGraph(pinned);
  }, 500);

  // Right-click on a layer: its diagrams, each to open in a tab of its own.
  d.layerBar.addEventListener('contextmenu', (event) => {
    const target = (event.target as HTMLElement).closest<HTMLElement>('.wb-layer');
    if (!target) return;
    event.preventDefault();
    event.stopPropagation();
    const short = target.querySelector('.wb-layer-name')?.textContent?.trim() ?? target.textContent?.trim() ?? '';
    const viewpoint = VIEWPOINTS.find((v) => v.short === short);
    const graphs = (d.snapshot()?.diagrams?.graphs ?? []).filter((g) => (viewpoint ? g.arc.view === viewpoint.view : !VIEWPOINTS.some((v) => v.view === g.arc.view)));
    menu.open(
      event.clientX,
      event.clientY,
      graphs.map((graph) => ({ label: `Open “${graph.arc.title}” in a new tab`, run: () => d.post({ type: 'panes-open-tab', view: graph.id }) })),
    );
  });
}
