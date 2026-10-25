// draw.io's Arrange on the canvas: the stacking order (front, back, one step
// forward or backward), groups (a frame kept in the layout file) and "Group
// into a new component" (an edit of the model), same size, locks, clearing
// waypoints, selecting boxes, lines or nothing, and the Layers panel
// (layers.ts). The viewer does the drawing (arcviz-layers.js); this offers
// the commands in the context menu, the floating toolbar, Layout ▾ and the
// keys, and refuses on the canvas what a lock forbids.
import { EditRequest, ElementRecord, Snapshot, ViewerGraph } from '../src/pure/workbench';
import { ArrangeCommand, ARRANGE_KEYS, arrangeCommandOf, groupIntoKind, groupIntoPlan, layerOfTags, layerLabel, moveToLayerEdit, tagsOf } from '../src/pure/arrange';
import { inlinePrompt } from './dom';
import type { ViewerApi } from './globals';
import { ContextMenu, MenuAction, MenuItem } from './menu';
import { LayersPanel } from './layers';

type Done = Promise<{ ok: boolean; reason?: string; id?: string }>;

/** What the viewer offers for Arrange (`viewer.arrange`, arcviz-layers.js). */
export interface ArrangeApi {
  restack(ids: string[], how: 'front' | 'back' | 'forward' | 'backward'): Done;
  group(ids: string[], label?: string): Done;
  ungroup(id: string): Done;
  removeFromGroup(ids: string[]): Done;
  collapseGroup(id: string, on: boolean): Done;
  labelGroup(id: string, label: string): Done;
  enterGroup(id: string): boolean;
  exitGroup(): void;
  enteredGroup(): string | null;
  groupOf(id: string): string | null;
  groupInfo(id: string): { id: string; members: string[]; label: string; collapsed: boolean } | null;
  lock(ids: string[], on: boolean): Done;
  isLocked(id: string, isEdge?: boolean): boolean;
  layerState(): { order: string[]; hidden: string[]; locked: string[] };
  setLayerHidden(name: string, on: boolean): Done;
  setLayerLocked(name: string, on: boolean): Done;
  setLayerOrder(names: string[]): Done;
  resizeMany(changes: { id: string; dw: number; dh: number }[]): Done;
  clearRoutes(edges: string[]): Done;
  drawn(): Promise<{ nodes: string[]; edges: string[] }>;
}

export interface ArrangeDeps {
  host: HTMLElement;
  sheet: HTMLElement;
  viewer: () => ViewerApi | null;
  graph: () => ViewerGraph | null;
  snapshot: () => Snapshot | null;
  selected: () => ElementRecord | null;
  selectedDrawing: () => string | null;
  multi: () => ElementRecord[];
  setMulti: (elements: ElementRecord[]) => void;
  select: (element: ElementRecord | null, inViewer: boolean) => void;
  elementFor: (drawingId: string, isEdge?: boolean) => ElementRecord | null;
  drawingId: (modelId: string) => string;
  submit: (requests: EditRequest[], label: string) => void;
  toast: (text: string, kind: 'info' | 'error') => void;
}

const isGroup = (id: string | null): id is string => !!id && id.startsWith('group:');
const ORDER: ['front' | 'back' | 'forward' | 'backward', string, ArrangeCommand][] = [['front', 'To front', 'bring-front'], ['back', 'To back', 'send-back'], ['forward', 'Forward', 'bring-forward'], ['backward', 'Backward', 'send-backward']];
const ORDER_NAME = { front: 'Bring to front', back: 'Send to back', forward: 'Bring forward', backward: 'Send backward' } as const;
const mac = /mac|iphone|ipad/i.test(navigator.platform || navigator.userAgent);
const keyLabel = (command: ArrangeCommand): string => {
  const keys = ARRANGE_KEYS[command].keys;
  return mac ? keys.replace('Mod+', '⌘').replace('Alt+', '⌥').replace('Shift+', '⇧').replace(/\+/g, '') : keys.replace('Mod', 'Ctrl');
};
const viewerOf = (d: ArrangeDeps) => d.viewer() as unknown as { select?: (id: string | null) => void; arrange?: ArrangeApi } | null;

class Arrange {
  private readonly menu: ContextMenu;
  readonly panel: LayersPanel;
  /** Lines selected by Select lines (the multi-selection holds elements, not lines). */
  private lines: string[] = [];

  constructor(private readonly d: ArrangeDeps) {
    this.menu = new ContextMenu(d.sheet);
    this.panel = new LayersPanel(d.sheet, {
      api: () => this.api(),
      snapshot: d.snapshot,
      moveHere: (layer) => this.moveToLayer(layer),
      toast: d.toast,
      pressed: (on) => document.getElementById('arcviz-layers')?.setAttribute('aria-pressed', String(on)),
    });
    // Esc leaves a group open for editing (the viewer's own key when the sheet has the focus).
    document.addEventListener('keydown', (event) => {
      if (event.key === 'Escape' && this.api()?.enteredGroup()) this.api()?.exitGroup();
    });
    d.host.addEventListener('pointerdown', (event) => {
      if (!event.shiftKey && this.lines.length) this.markLines([]);
    }, true);
    window.addEventListener('message', (event: MessageEvent<{ type?: string }>) => {
      if (event.data?.type === 'snapshot') setTimeout(() => this.panel.refresh(), 0);
    });
  }

  api(): ArrangeApi | null {
    return viewerOf(this.d)?.arrange ?? null;
  }

  // --- the selection as the drawing sees it -------------------------------------------------

  /** The drawing ids of the selection: a member of a group stands for its group (unless the group is open). */
  ids(): string[] {
    const api = this.api();
    const many = this.d.multi();
    const raw = many.length > 1 ? many.map((e) => this.d.drawingId(e.id)) : this.d.selectedDrawing() ? [this.d.selectedDrawing()!] : [];
    const stand = (id: string): string => {
      let at = id;
      for (let guard = 0; guard < 64; guard++) {
        const group = api?.groupOf(at) ?? null;
        if (!group || group.slice('group:'.length) === api?.enteredGroup()) return at;
        at = group;
      }
      return at;
    };
    return [...new Set(raw.map(stand))];
  }

  private edgeIds(): Set<string> {
    return new Set((this.d.graph()?.edges ?? []).map((e) => e.id));
  }

  /** The selected group: the frame selected, or the group the selection stands for. */
  private group(): string | null {
    const ids = this.ids();
    return ids.length === 1 && isGroup(ids[0]) ? ids[0] : null;
  }

  /** May these be changed on the canvas? A locked one says so and nothing is done. */
  allows(elements: ElementRecord[], verb: string): boolean {
    const api = this.api();
    if (!api) return true;
    const selected = this.d.selected();
    const drawing = this.d.selectedDrawing();
    const edges = this.edgeIds();
    const locked = elements.find((e) => {
      const ids = [this.d.drawingId(e.id), ...(selected && drawing && selected.uuid === e.uuid ? [drawing] : [])];
      return ids.some((id) => api.isLocked(id, edges.has(id)));
    });
    if (!locked) return true;
    this.d.toast(`${locked.name || locked.id} is locked: unlock it (${keyLabel('lock')}) or its layer to ${verb} it.`, 'error');
    return false;
  }

  allowsIds(ids: string[], verb: string): boolean {
    const api = this.api();
    const edges = this.edgeIds();
    const locked = api ? ids.find((id) => api.isLocked(id, edges.has(id))) : undefined;
    if (!locked) return true;
    this.d.toast(`${locked} is locked: unlock it (${keyLabel('lock')}) or its layer to ${verb} it.`, 'error');
    return false;
  }

  private async done(result: Done | undefined, quiet = false): Promise<void> {
    const answer = await result;
    if (answer && !answer.ok && answer.reason) this.d.toast(answer.reason, 'info');
    else if (answer?.ok && !quiet) this.panel.refresh();
  }

  // --- the commands -----------------------------------------------------------------

  run(command: ArrangeCommand): void {
    const order = ORDER.find(([, , c]) => c === command);
    if (order) return void this.restack(order[0]);
    if (command === 'group') return void this.groupSelection();
    if (command === 'ungroup') return void this.ungroup();
    if (command === 'lock') return void this.toggleLock();
    if (command === 'layers') return this.panel.toggle();
    if (command === 'select-vertices') return void this.selectBoxes();
    if (command === 'select-edges') return void this.selectLines();
    if (command === 'select-none') return this.selectNone();
  }

  restack(how: 'front' | 'back' | 'forward' | 'backward'): Promise<void> {
    const ids = this.ids().filter((id) => !this.edgeIds().has(id));
    if (!ids.length) return this.done(Promise.resolve({ ok: false, reason: 'Select a box to change its stacking.' }));
    return this.done(this.api()?.restack(ids, how), true);
  }

  async groupSelection(): Promise<void> {
    const api = this.api();
    if (!api) return;
    const answer = await api.group(this.ids());
    if (!answer.ok) return void this.d.toast(answer.reason ?? 'Not grouped.', 'info');
    this.d.setMulti([]);
    this.d.select(null, false);
    setTimeout(() => viewerOf(this.d)?.select?.(answer.id!), 50);
    this.d.toast(`Grouped. ${keyLabel('ungroup')} ungroups, a double-click edits inside.`, 'info');
  }

  ungroup(): Promise<void> {
    const group = this.group();
    if (!group) return this.done(Promise.resolve({ ok: false, reason: 'Select a group to ungroup it.' }));
    return this.done(this.api()?.ungroup(group));
  }

  /** Lock the selection, or unlock it when all of it is locked. */
  toggleLock(): Promise<void> {
    const api = this.api();
    const ids = this.ids();
    if (!api || !ids.length) return this.done(Promise.resolve({ ok: false, reason: 'Select what to lock.' }));
    const all = ids.every((id) => api.isLocked(id, this.edgeIds().has(id)));
    return this.done(api.lock(ids, !all));
  }

  /** Same size, width or height as the box selected last (draw.io's rule). */
  sameSize(how: 'both' | 'width' | 'height'): Promise<void> {
    const many = this.d.multi();
    if (many.length < 2) return this.done(Promise.resolve({ ok: false, reason: 'Select two or more boxes: the last one selected gives its size.' }));
    const boxes = many.flatMap((e) => {
      const id = this.d.drawingId(e.id);
      const shape = this.d.host.querySelector(`.av-node[data-id="${CSS.escape(id)}"] > .av-shape`);
      if (!shape) return [];
      return [{ id, width: Number(shape.getAttribute('width')), height: Number(shape.getAttribute('height')) }];
    });
    const sizing = (window as unknown as { ArcVizLayers: { sameSize: (b: unknown[], r: string, h: string) => { id: string; dw: number; dh: number }[] } }).ArcVizLayers;
    const changes = sizing.sameSize(boxes, this.d.drawingId(many[many.length - 1].id), how);
    if (!changes.length) return this.done(Promise.resolve({ ok: false, reason: 'They have that size already.' }));
    return this.done(this.api()?.resizeMany(changes), true);
  }

  /** The waypoints of the selected lines, or of the lines of the selected boxes, cleared. */
  clearWaypoints(): Promise<void> {
    const edges = this.edgeIds();
    const drawing = this.d.selectedDrawing();
    let chosen = [...this.lines, ...(drawing && edges.has(drawing) ? [drawing] : [])];
    if (!chosen.length) {
      const boxes = new Set(this.ids());
      chosen = (this.d.graph()?.edges ?? []).filter((e) => [...e.sources, ...e.targets].some((end) => boxes.has(end) || boxes.has(end.split('::')[0]))).map((e) => e.id);
    }
    if (!chosen.length) return this.done(Promise.resolve({ ok: false, reason: 'Select lines (or boxes) to clear their waypoints.' }));
    return this.done(this.api()?.clearRoutes([...new Set(chosen)]), true);
  }

  async selectBoxes(): Promise<void> {
    const drawn = (await this.api()?.drawn()) ?? { nodes: [], edges: [] };
    const seen = new Set<string>();
    const elements = drawn.nodes.map((id) => this.d.elementFor(id)).filter((e): e is ElementRecord => !!e && !seen.has(e.uuid) && !!seen.add(e.uuid));
    this.markLines([]);
    this.d.setMulti(elements.length > 1 ? elements : []);
    if (elements.length === 1) this.d.select(elements[0], true);
    this.d.toast(`${elements.length} box${elements.length === 1 ? '' : 'es'} selected.`, 'info');
  }

  async selectLines(): Promise<void> {
    const drawn = (await this.api()?.drawn()) ?? { nodes: [], edges: [] };
    this.d.setMulti([]);
    this.d.select(null, false);
    viewerOf(this.d)?.select?.(null);
    setTimeout(() => this.markLines(drawn.edges), 60);
    this.d.toast(`${drawn.edges.length} line${drawn.edges.length === 1 ? '' : 's'} selected.`, 'info');
  }

  selectNone(): void {
    this.markLines([]);
    this.d.setMulti([]);
    this.d.select(null, false);
    viewerOf(this.d)?.select?.(null);
  }

  private markLines(ids: string[]): void {
    this.lines = ids;
    for (const node of this.d.host.querySelectorAll('.av-edge.wb-multi')) node.classList.remove('wb-multi');
    for (const id of ids) this.d.host.querySelector(`.av-edge[data-id="${CSS.escape(id)}"]`)?.classList.add('wb-multi');
    this.d.host.dataset.selectedLines = String(ids.length);
  }

  /** Put the selection on `layer`: its first tag replaced (an edit of the model, one undo). */
  moveToLayer(layer: string): void {
    const many = this.d.multi();
    const elements = many.length > 1 ? many : this.d.selected() ? [this.d.selected()!] : [];
    if (!elements.length) return this.d.toast('Select what to move to the layer.', 'info');
    const requests = elements.map((e) => moveToLayerEdit(e, layer)).filter((r): r is EditRequest => !!r);
    if (!requests.length) return this.d.toast(`Already on ${layerLabel(layer)}.`, 'info');
    this.d.submit(requests, `move ${requests.length === 1 ? elements[0].id : `${requests.length} elements`} to the ${layerLabel(layer)} layer`);
  }

  /** "Group into a new component…": a container of the model around the selection, one edit. */
  groupInto(): void {
    const graph = this.d.graph();
    const elements = this.d.multi();
    const all = this.d.snapshot()?.elements?.elements ?? [];
    if (!graph) return;
    const kind = groupIntoKind(graph.arc.view, elements);
    if ('error' in kind) return this.d.toast(kind.error, 'info');
    void inlinePrompt(this.d.sheet, `New ${kind.label} holding ${elements.length} elements`, `New ${kind.label}`, 'Its name').then((name) => {
      if (name === null) return;
      const plan = groupIntoPlan(graph.arc.view, elements, all, name);
      if ('error' in plan) return this.d.toast(plan.error, 'error');
      this.d.setMulti([]);
      this.d.submit(plan.requests, `group ${elements.length} elements into ${plan.id}`);
    });
  }

  private renameGroup(group: string): void {
    const info = this.api()?.groupInfo(group);
    void inlinePrompt(this.d.sheet, 'Group label', info?.label ?? '', 'A label, or nothing').then((label) => {
      if (label !== null) void this.done(this.api()?.labelGroup(group, label.trim()), true);
    });
  }

  // --- the menus --------------------------------------------------------------------------

  /** The Arrange section of the context menu, after what the menu has. */
  menuItems(element: ElementRecord | null): MenuItem[] {
    const api = this.api();
    if (!api) return [];
    // The menu opens before the viewer reports its selection: the element under the pointer is the one.
    const ids = this.ids().length || !element ? this.ids() : [this.d.selectedDrawing() ?? this.d.drawingId(element.id)];
    const many = this.d.multi();
    const edges = this.edgeIds();
    const boxes = ids.filter((id) => !edges.has(id));
    if (!ids.length && !element) {
      return [
        { label: 'Select boxes', shortcut: keyLabel('select-vertices'), run: () => void this.selectBoxes() },
        { label: 'Select lines', shortcut: keyLabel('select-edges'), run: () => void this.selectLines() },
        { label: 'Layers…', shortcut: keyLabel('layers'), run: () => this.panel.toggle(true) },
      ];
    }
    const items: MenuItem[] = [];
    if (boxes.length) items.push({ title: 'Order', row: ORDER.map(([how, label, command]) => ({ label, shortcut: keyLabel(command), run: () => void this.restack(how) })) });
    const group = this.group();
    if (boxes.length > 1) {
      items.push({ label: 'Group', shortcut: keyLabel('group'), run: () => void this.groupSelection() });
      const graph = this.d.graph();
      const into = graph ? groupIntoKind(graph.arc.view, many) : { error: '' };
      if (!('error' in into)) items.push({ label: `Group into a new ${into.label}…`, run: () => this.groupInto() });
      items.push({ title: 'Same', row: [{ label: 'Size', run: () => void this.sameSize('both') }, { label: 'Width', run: () => void this.sameSize('width') }, { label: 'Height', run: () => void this.sameSize('height') }] });
    }
    if (group) {
      const info = api.groupInfo(group);
      const entered = api.enteredGroup() === group.slice('group:'.length);
      items.push({ label: 'Ungroup', shortcut: keyLabel('ungroup'), run: () => void this.ungroup() });
      items.push({ label: entered ? 'Exit group' : 'Enter group', shortcut: entered ? 'Esc' : 'Double-click', run: () => (entered ? api.exitGroup() : api.enterGroup(group)) });
      items.push({ label: info?.collapsed ? 'Expand group' : 'Collapse group', run: () => void this.done(api.collapseGroup(group, !info?.collapsed), true) });
      items.push({ label: 'Rename group…', run: () => this.renameGroup(group) });
    }
    if (ids.some((id) => api.groupOf(id))) items.push({ label: 'Remove from group', run: () => void this.done(api.removeFromGroup(ids)) });
    const locked = ids.length > 0 && ids.every((id) => api.isLocked(id, edges.has(id)));
    if (ids.length) items.push({ label: locked ? 'Unlock' : 'Lock', shortcut: keyLabel('lock'), run: () => void this.toggleLock() });
    const lines = ids.filter((id) => edges.has(id));
    if (lines.length || this.lines.length) items.push({ label: 'Clear waypoints', run: () => void this.clearWaypoints() });
    if (element || many.length > 1) {
      const current = element ? layerOfTags(tagsOf(element.attributes)) : null;
      const rows = this.panel.layerRows().filter((r) => r.name !== current);
      items.push({ title: 'Move to layer', row: [...rows.map((r) => ({ label: r.label, run: () => this.moveToLayer(r.name) })), { label: 'New…', run: () => this.panel.newLayer(true) }] });
    }
    return items;
  }

  /** The Arrange entries of Layout ▾ in the toolbar, as the viewer's menus take them. */
  layoutItems(): unknown[] {
    const api = this.api();
    if (!api) return [];
    const ids = this.ids();
    const boxes = ids.filter((id) => !this.edgeIds().has(id));
    const none = !boxes.length;
    const group = this.group();
    const graph = this.d.graph();
    const into = graph ? groupIntoKind(graph.arc.view, this.d.multi()) : { error: 'No diagram' };
    const item = (label: string, command: ArrangeCommand | null, run: () => void, disabled = false, reason = '') => ({ label, action: `arrange-${label.toLowerCase().replace(/[^a-z]+/g, '-').replace(/-$/, '')}`, keys: command ? ARRANGE_KEYS[command].keys : undefined, run, disabled, reason });
    const few = this.d.multi().length < 2;
    return [
      { heading: 'Order' },
      ...ORDER.map(([how, , command]) => item(ORDER_NAME[how], command, () => void this.restack(how), none, 'Select a box first')),
      { heading: 'Group' },
      item('Group', 'group', () => void this.groupSelection(), boxes.length < 2, 'Select two or more boxes (Shift+click)'),
      item('Ungroup', 'ungroup', () => void this.ungroup(), !group, 'Select a group'),
      item('Remove from group', null, () => void this.done(api.removeFromGroup(ids)), !ids.some((id) => api.groupOf(id)), 'The selection is in no group'),
      item(api.enteredGroup() ? 'Exit group' : 'Enter group', null, () => (api.enteredGroup() ? api.exitGroup() : group && api.enterGroup(group)), !api.enteredGroup() && !group, 'Select a group'),
      item(`Group into a new ${'error' in into ? 'component' : into.label}…`, null, () => this.groupInto(), 'error' in into, 'error' in into ? into.error : ''),
      { heading: 'Size' },
      item('Same size', null, () => void this.sameSize('both'), few, 'Select two or more boxes; the last one gives the size'),
      item('Same width', null, () => void this.sameSize('width'), few, 'Select two or more boxes'),
      item('Same height', null, () => void this.sameSize('height'), few, 'Select two or more boxes'),
      'separator',
      item(ids.length && ids.every((id) => api.isLocked(id, this.edgeIds().has(id))) ? 'Unlock' : 'Lock', 'lock', () => void this.toggleLock(), !ids.length, 'Select what to lock'),
      item('Clear waypoints', null, () => void this.clearWaypoints()),
      { heading: 'Select' },
      item('Select boxes', 'select-vertices', () => void this.selectBoxes()),
      item('Select lines', 'select-edges', () => void this.selectLines()),
      item('Select none', 'select-none', () => this.selectNone()),
      'separator',
      item('Layers…', 'layers', () => this.panel.toggle(true)),
    ];
  }

  /** The Order button of the floating toolbar: one button, a small menu. */
  miniAction(): MenuAction & { glyph: string } {
    return {
      glyph: '⧉',
      label: 'Order: to front, to back, forward, backward',
      run: () => {
        const button = this.d.sheet.querySelector<HTMLElement>('.wb-mini-button[aria-label^="Order"]');
        const r = button?.getBoundingClientRect();
        this.menu.open(r ? r.left : 40, r ? r.bottom + 4 : 40, ORDER.map(([how, , command]) => ({ label: ORDER_NAME[how], shortcut: keyLabel(command), run: () => void this.restack(how) })));
      },
    };
  }
}

let controller: Arrange | null = null;

/** Install Arrange on the page (main.ts, zorder hook). */
export function installArrange(deps: ArrangeDeps): void {
  controller = new Arrange(deps);
}

/** What `ArcViz.mount` takes for Arrange: the tags of what is drawn (its layer). */
export function arrangeOptions(elementFor: (id: string, isEdge?: boolean) => ElementRecord | null): { layers: { tagsOf: (view: string, id: string, isEdge: boolean) => string[] } } {
  return { layers: { tagsOf: (_view, id, isEdge) => tagsOf(elementFor(id, isEdge)?.attributes) } };
}

/** Layout ▾ entries and the Layers button, as the toolbar's options take them. */
export function arrangeToolbar(): { layoutItems: () => unknown[]; tools: unknown[] } {
  return {
    layoutItems: () => controller?.layoutItems() ?? [],
    tools: [{ id: 'arcviz-layers', icon: 'layers', label: 'Layers', keys: 'Mod+Shift+L', hint: 'Layers: show, hide, lock and order the tags', pressed: false, rank: 6, run: () => controller?.panel.toggle() }],
  };
}

/** The context menu with its Arrange section (zorder hook in editing.ts). */
export function withArrange(items: MenuItem[], element: ElementRecord | null): MenuItem[] {
  const more = controller?.menuItems(element) ?? [];
  return more.length ? [...items, 'separator', ...more] : items;
}

/** The floating toolbar with its Order button (zorder hook in editing.ts). */
export function withArrangeMini<T extends MenuAction & { glyph: string }>(actions: T[]): (T | (MenuAction & { glyph: string }))[] {
  if (!controller || actions.length === 0) return actions;
  const at = Math.max(0, actions.length - 2);
  return [...actions.slice(0, at), controller.miniAction(), ...actions.slice(at)];
}

/** A command of the canvas from VS Code's keybindings (keys.ts); false when it is not one of Arrange's. */
export function runArrange(command: string): boolean {
  if (!controller || !(command in ARRANGE_KEYS)) return false;
  controller.run(command as ArrangeCommand);
  return true;
}

/** A key the page takes itself outside VS Code; true when it was an Arrange key. */
export function arrangeKey(event: KeyboardEvent): boolean {
  const command = arrangeCommandOf(event);
  if (!command || !controller) return false;
  event.preventDefault();
  controller.run(command);
  return true;
}

/** Whether a canvas gesture on these elements is allowed (a lock refuses it, saying why). */
export function arrangeAllows(elements: ElementRecord[], verb: string): boolean {
  return controller ? controller.allows(elements, verb) : true;
}

export function arrangeAllowsIds(ids: string[], verb: string): boolean {
  return controller ? controller.allowsIds(ids, verb) : true;
}

/** The export entries say that hidden layers are left out. */
export function withLayersNote<T extends { title?: string }>(items: T[]): T[] {
  const hidden = controller?.api()?.layerState().hidden ?? [];
  if (!hidden.length) return items;
  const note = ` Hidden layers are left out (${hidden.map(layerLabel).join(', ')}).`;
  return items.map((item) => ({ ...item, title: `${item.title ?? ''}${note}` }));
}
