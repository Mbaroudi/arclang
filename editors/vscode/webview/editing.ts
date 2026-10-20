// Editing what is already on the canvas, draw.io style but on the model:
// rename in place (double-click, F2), reconnect an end of an exchange by
// dragging it, reverse it, bend it, select with a band, connect from the
// arrows on the sides of a box, copy, cut, paste, duplicate, move to,
// align, distribute, nudge with the arrows, and the context menu and the
// floating toolbar that list all of it. Every change of the model is one
// request list for the compiler (one undo); the arrangement goes to the
// layout file through the viewer.
import { alignMoves, Alignment, Box, Clip, distributeMoves, moveTargets, moveToLayerRequest, pasteRequests, quickConnectPlan, reconnectKinds, reconnectPlan, relationshipOfEdge, reversePlan } from '../src/pure/editing';
import { EditRequest, ElementRecord, graphContainerOf, RelationshipRecord, Snapshot, ViewerGraph } from '../src/pure/workbench';
import { el, inlinePrompt } from './dom';
import type { ViewerApi } from './globals';
import { ContextMenu, MenuAction, MenuItem, MiniToolbar } from './menu';

/** What the page lends the editing gestures. */
export interface WorkbenchContext {
  host: HTMLElement;
  sheet: HTMLElement;
  snapshot: () => Snapshot | null;
  viewer: () => ViewerApi | null;
  graph: () => ViewerGraph | null;
  selected: () => ElementRecord | null;
  /** The drawing id of what the viewer has selected (a box, a port, an exchange). */
  selectedDrawing: () => string | null;
  multi: () => ElementRecord[];
  setMulti: (elements: ElementRecord[]) => void;
  elementFor: (drawingId: string, isEdge?: boolean, name?: string) => ElementRecord | null;
  drawingId: (modelId: string) => string;
  outermost: (elements: ElementRecord[]) => ElementRecord[];
  submit: (requests: EditRequest[], label: string) => void;
  toast: (text: string, kind: 'info' | 'error') => void;
  select: (element: ElementRecord | null, inViewer: boolean) => void;
  remove: (elements: ElementRecord[]) => void;
  hide: (elements: ElementRecord[]) => void;
  related: (element: ElementRecord) => void;
  newDiagram: (element: ElementRecord) => void;
  reveal: (element: ElementRecord) => void;
  help: () => void;
}

const NUDGE = 1;
const NUDGE_BIG = 10;

export class Editing {
  private clip: Clip | null = null;
  /** What follows a gesture once the compiler applied it, by label (the route of a reversed line...). */
  private readonly onApplied = new Map<string, () => void>();
  /** What follows once the viewer is mounted on the snapshot of that gesture. */
  private afterMount: (() => void)[] = [];
  /** The field or list open on the canvas, closed when the model changes under it. */
  private open: { model: string; close: () => void } | null = null;
  /** The model the context menu was opened on. */
  private menuModel = '';
  readonly menu: ContextMenu;
  private readonly mini: MiniToolbar;

  constructor(private readonly ctx: WorkbenchContext) {
    this.menu = new ContextMenu(ctx.sheet);
    this.mini = new MiniToolbar(ctx.sheet);
    ctx.host.addEventListener('contextmenu', (event) => this.contextMenu(event));
    // Arrows move the selection before the viewer pans with them.
    ctx.host.addEventListener('keydown', (event) => this.nudge(event), true);
    setInterval(() => this.placeToolbar(), 200);
  }

  /** Submit a gesture; `then` runs once the compiler applied it. */
  private submit(requests: EditRequest[], label: string, then?: () => void): void {
    if (then) this.onApplied.set(label, then);
    this.ctx.submit(requests, label);
  }

  /** The host's answer to a gesture. */
  editResult(label: string, ok: boolean): void {
    const then = this.onApplied.get(label);
    this.onApplied.delete(label);
    if (ok && then) then();
  }

  /** The viewer is mounted on a new snapshot: what waited for it runs. */
  mounted(): void {
    const waiting = this.afterMount;
    this.afterMount = [];
    for (const run of waiting) run();
  }

  /** What the model is, for an open field to know it changed. */
  private modelKey(): string {
    return (this.ctx.snapshot()?.elements?.elements ?? []).map((e) => `${e.uuid}:${e.id}:${e.name}`).join('|');
  }

  /** A new snapshot: a field or list opened on another model is closed, it would act on stale ids. */
  snapshotArrived(): void {
    if (this.menu.isOpen && this.menuModel !== this.modelKey()) this.menu.close();
    if (this.open && this.open.model !== this.modelKey()) {
      this.open.close();
      this.open = null;
      this.ctx.toast('The model changed while a field was open: it was closed, nothing was written.', 'info');
    }
  }

  /** A bent line keeps its bends when it turns around (reversed), straightens when it moves (reconnected). */
  private reroute(edge: string, change: (points: { x: number; y: number }[]) => { x: number; y: number }[], to = edge): () => void {
    const points = this.ctx.viewer()?.routeOf?.(edge) ?? [];
    return () => {
      if (points.length === 0) return;
      this.afterMount.push(() => {
        if (to !== edge) void this.ctx.viewer()?.setRoute?.(edge, []);
        void this.ctx.viewer()?.setRoute?.(to, change(points));
      });
    };
  }

  // --- the viewer's edit options --------------------------------------------

  /** What the viewer is mounted with (`options.edit`). */
  viewerOptions(): Record<string, unknown> {
    return {
      bends: true,
      snap: true,
      onEdit: ({ id, isEdge }: { id: string; isEdge: boolean }) => {
        const element = this.ctx.elementFor(id, isEdge);
        if (!element) return false;
        this.ctx.select(element, false);
        this.rename(element, id);
        return true;
      },
      reconnectTargets: ({ edge, end }: { edge: string; end: 'source' | 'target' }) => this.reconnectTargets(edge, end),
      onReconnect: ({ edge, end, to }: { edge: string; end: 'source' | 'target'; to: string }) => {
        const target = this.ctx.elementFor(to);
        if (target) this.reconnect(edge, end, target);
      },
      onMarquee: ({ ids, additive }: { ids: string[]; additive: boolean }) => {
        // A box and what it holds: the box stands for them.
        const picked = this.ctx.outermost(ids.map((id) => this.ctx.elementFor(id)).filter((e): e is ElementRecord => !!e));
        const base = additive ? this.selection() : [];
        const all = [...base, ...picked.filter((e) => !base.some((b) => b.uuid === e.uuid))];
        if (all.length === 1) {
          this.ctx.setMulti([]);
          this.ctx.select(all[0], true);
        } else this.ctx.setMulti(all);
      },
      onQuick: ({ id }: { id: string }) => this.quickCreate(id),
    };
  }

  // --- rename in place --------------------------------------------------------

  /** The name edited over the box, or over the label of the exchange. */
  rename(element: ElementRecord, drawing = this.ctx.selectedDrawing() ?? this.ctx.drawingId(element.id)): void {
    const relationship = this.relationshipOf(element);
    if (relationship && !relationship.name) return this.ctx.toast(`A ${relationship.kind.toLowerCase()} has no name to edit: its guard and trigger are in the properties.`, 'info');
    const node = this.ctx.host.querySelector<SVGGraphicsElement>(`[data-id="${CSS.escape(drawing)}"]`);
    const anchor = node?.querySelector<SVGGraphicsElement>('.av-edge-label') ?? node;
    if (!anchor) return;
    const box = anchor.getBoundingClientRect();
    const frame = this.ctx.sheet.getBoundingClientRect();
    const input = el('input', { type: 'text', class: 'wb-direct-edit', value: element.name || element.id, 'aria-label': `Rename ${element.name || element.id}` });
    input.style.left = `${Math.max(4, box.left - frame.left)}px`;
    input.style.top = `${box.top - frame.top + (relationship ? -4 : 4)}px`;
    input.style.width = `${Math.max(140, box.width)}px`;
    this.ctx.sheet.appendChild(input);
    input.focus();
    input.select();
    let done = false;
    const finish = (commit: boolean) => {
      if (done) return;
      done = true;
      if (this.open?.close === cancel) this.open = null;
      const value = input.value.trim();
      input.remove();
      if (!commit || !value || value === element.name) return;
      // A line is drawn under its name (`ce:Name`): its bends follow it.
      const renamed = relationship && /^[a-z]+:/.test(drawing) ? drawing.replace(/^([a-z]+:).*$/, `$1${value}`) : null;
      this.submit([{ op: 'rename', element: element.id, name: value }], `rename to ${value}`, renamed ? this.reroute(drawing, (p) => p, renamed) : undefined);
    };
    const cancel = () => finish(false);
    this.open = { model: this.modelKey(), close: cancel };
    input.addEventListener('keydown', (event) => {
      event.stopPropagation();
      if (event.key === 'Enter') finish(true);
      if (event.key === 'Escape') finish(false);
    });
    input.addEventListener('blur', () => finish(true));
  }

  // --- reconnect, reverse -------------------------------------------------------

  /** The relationship a selected edge stands for (the page lists it as an element). */
  relationshipOf(element: ElementRecord | null): RelationshipRecord | null {
    if (!element) return null;
    return (this.ctx.snapshot()?.elements?.relationships ?? []).find((r) => r.uuid === element.uuid) ?? null;
  }

  private relationshipOfEdge(edge: string): RelationshipRecord | null {
    const graph = this.ctx.graph();
    const snapshot = this.ctx.snapshot();
    return graph && snapshot ? relationshipOfEdge(edge, graph, snapshot) : null;
  }

  /** The drawn boxes and ports an end of the edge may move to (null: any). */
  private reconnectTargets(edge: string, end: 'source' | 'target'): string[] | null {
    const relationship = this.relationshipOfEdge(edge);
    const graph = this.ctx.graph();
    if (!relationship || !graph) return [];
    const kinds = reconnectKinds(relationship.kind, graph.arc.view);
    if (kinds === null) return null;
    // The other end stays: an exchange does not join it to itself.
    const otherUuid = end === 'source' ? relationship.target : relationship.source;
    const wanted = new Set(kinds);
    const ids: string[] = [];
    const walk = (nodes: NonNullable<ViewerGraph['children']>) => {
      for (const node of nodes) {
        const element = this.ctx.elementFor(node.id);
        if (element && wanted.has(element.kind) && element.uuid !== otherUuid) ids.push(node.id);
        for (const port of node.ports ?? []) {
          const owned = this.ctx.elementFor(port.id);
          // A port keeps its direction: the source of an exchange leaves by an out port.
          const direction = port.arc?.direction;
          const fits = !direction || direction === 'inout' || direction === 'undirected' || direction === (end === 'source' ? 'out' : 'in');
          if (owned && wanted.has(owned.kind) && fits && owned.uuid !== otherUuid) ids.push(port.id);
        }
        walk(node.children ?? []);
      }
    };
    walk(graph.children ?? []);
    return ids;
  }

  private reconnect(edge: string, end: 'source' | 'target', target: ElementRecord): void {
    const relationship = this.relationshipOfEdge(edge);
    const elements = this.ctx.snapshot()?.elements?.elements ?? [];
    if (!relationship) return this.ctx.toast('This line stands for no relationship the page can reconnect.', 'error');
    const plan = reconnectPlan(relationship, end, target, elements);
    if ('error' in plan) return this.ctx.toast(plan.error, 'info');
    // Its bends were for the old ends: the line is drawn afresh.
    this.submit(plan, `reconnect the ${end} of ${relationship.name || relationship.kind} to ${target.name || target.id}`, this.reroute(edge, () => []));
  }

  /** From the menu: the end to reconnect, then the element, from a list. */
  private reconnectFromList(edge: string, end: 'source' | 'target'): void {
    const ids = this.reconnectTargets(edge, end);
    const graph = this.ctx.graph();
    const all = ids ?? (graph?.children ?? []).map((n) => n.id);
    const choices = [...new Set(all)].map((id) => ({ id, element: this.ctx.elementFor(id) })).filter((c): c is { id: string; element: ElementRecord } => !!c.element);
    this.pick(`Reconnect the ${end} to…`, choices.map((c) => [c.element.id, c.element.name && c.element.name !== c.element.id ? `${c.element.name} (${c.element.id})` : c.element.id]), (id) => {
      const choice = choices.find((c) => c.element.id === id);
      if (choice) this.reconnect(edge, end, choice.element);
    });
  }

  reverse(element: ElementRecord): void {
    const relationship = this.relationshipOf(element);
    if (!relationship) return;
    const plan = reversePlan(relationship, this.ctx.snapshot()?.elements?.elements ?? [], this.ctx.snapshot()?.elements?.relationships ?? []);
    if ('error' in plan) return this.ctx.toast(plan.error, 'error');
    // The bends are kept from source to target: turned around with the line.
    const edge = this.ctx.selectedDrawing();
    const drawnHere = edge && this.relationshipOf(this.ctx.selected())?.uuid === relationship.uuid ? edge : null;
    this.submit(plan, `reverse ${relationship.name || relationship.kind}`, drawnHere ? this.reroute(drawnHere, (p) => [...p].reverse()) : undefined);
  }

  /** Move a trace's other end to another element of the same kind, chosen from a list. */
  retargetTrace(element: ElementRecord, other: ElementRecord, outgoing: boolean): void {
    const elements = this.ctx.snapshot()?.elements?.elements ?? [];
    const choices: [string, string][] = elements.filter((e) => e.kind === other.kind && e.uuid !== other.uuid && e.uuid !== element.uuid).map((e) => [e.id, e.name && e.name !== e.id ? `${e.name} (${e.id})` : e.id]);
    if (choices.length === 0) return this.ctx.toast(`No other ${other.kind} to trace to.`, 'info');
    this.pick(`Trace ${outgoing ? 'to' : 'from'}…`, choices, (id) => {
      const from = outgoing ? element.id : other.id;
      const to = outgoing ? other.id : element.id;
      this.submit([{ op: 'reconnect', kind: 'Trace', from, to, ...(outgoing ? { new_to: id } : { new_from: id }) }], `trace ${outgoing ? `${from} to ${id}` : `${id} to ${to}`}`);
    });
  }

  /** The ends of a relationship, for the properties view. */
  endsOf(element: ElementRecord): { from: ElementRecord; to: ElementRecord } | null {
    const relationship = this.relationshipOf(element);
    const elements = this.ctx.snapshot()?.elements?.elements ?? [];
    const from = relationship && elements.find((e) => e.uuid === relationship.source);
    const to = relationship && elements.find((e) => e.uuid === relationship.target);
    return from && to ? { from, to } : null;
  }

  // --- quick connect --------------------------------------------------------------

  private quickCreate(drawing: string): void {
    const from = this.ctx.elementFor(drawing);
    const graph = this.ctx.graph();
    const snapshot = this.ctx.snapshot();
    if (!from || !graph || !snapshot?.elements) return;
    const container = graphContainerOf(graph.id, graph.arc.view);
    void inlinePrompt(this.ctx.sheet, `New element connected to ${from.name || from.id}`, `New ${from.kind.replace(/([a-z])([A-Z])/g, '$1 $2').toLowerCase()}`, 'name').then((name) => {
      if (name === null) return;
      const plan = quickConnectPlan(graph.arc.view, from, name, snapshot.elements!.elements, container);
      if ('error' in plan) return this.ctx.toast(plan.error, 'info');
      this.submit(plan.requests, `${name.trim()} connected to ${from.name || from.id}`);
    });
  }

  // --- copy, cut, paste, duplicate, move to ---------------------------------------

  /** The selection as the gestures take it: the multi-selection, else the one selected element. */
  selection(): ElementRecord[] {
    const multi = this.ctx.multi();
    if (multi.length > 0) return multi;
    const one = this.ctx.selected();
    return one && !this.relationshipOf(one) ? [one] : [];
  }

  /** What copy, cut and duplicate take: the selection without its ports (they move with Move to…). */
  private copyable(): ElementRecord[] {
    return this.ctx.outermost(this.selection()).filter((e) => !/Port$/.test(e.kind));
  }

  copy(cut: boolean): void {
    const elements = this.copyable();
    if (elements.length === 0) return;
    this.clip = { ids: elements.map((e) => e.id), cut };
    for (const node of this.ctx.host.querySelectorAll('.wb-cut')) node.classList.remove('wb-cut');
    if (cut) for (const e of elements) this.ctx.host.querySelector(`[data-id="${CSS.escape(this.ctx.drawingId(e.id))}"]`)?.classList.add('wb-cut');
    this.ctx.toast(`${cut ? 'Cut' : 'Copied'}: ${elements.map((e) => e.name || e.id).join(', ')}. Select where it goes, then ⌘V.`, 'info');
  }

  paste(): void {
    if (!this.clip) return this.ctx.toast('Nothing copied yet: select elements, then ⌘C or ⌘X.', 'info');
    const plan = pasteRequests(this.clip, this.ctx.selected());
    if ('error' in plan) return this.ctx.toast(plan.error, 'info');
    const into = this.ctx.selected();
    const clip = this.clip;
    // A cut is used up only when the move is applied: refused, it can be pasted elsewhere.
    this.submit(plan, `${clip.cut ? 'move' : 'paste'} ${clip.ids.join(', ')}${into && !clip.ids.includes(into.id) ? ` into ${into.name || into.id}` : ''}`, () => {
      if (!clip.cut || this.clip !== clip) return;
      this.clip = null;
      for (const node of this.ctx.host.querySelectorAll('.wb-cut')) node.classList.remove('wb-cut');
    });
  }

  duplicate(): void {
    const elements = this.copyable();
    if (elements.length === 0) return;
    this.submit(elements.map((e) => ({ op: 'duplicate', element: e.id })), `duplicate ${elements.map((e) => e.name || e.id).join(', ')}`);
  }

  moveTo(element: ElementRecord): void {
    const elements = this.ctx.snapshot()?.elements?.elements ?? [];
    const view = this.ctx.graph()?.arc.view ?? '';
    const targets = moveTargets(element, elements);
    const layer = /Port$/.test(element.kind) ? null : moveToLayerRequest(element, view);
    const choices: [string, string][] = targets.map((t) => [t.id, t.name && t.name !== t.id ? `${t.name} (${t.id})` : t.id]);
    if (layer && !('error' in layer) && element.owner && elements.find((e) => e.uuid === element.owner)?.kind !== 'Model') choices.unshift(['', 'Out of its container, into the layer']);
    if (choices.length === 0) return this.ctx.toast(`Nothing on this view can hold ${element.name || element.id}.`, 'info');
    this.pick(`Move ${element.name || element.id} to…`, choices, (id) => {
      if (id === '' && layer && !('error' in layer)) return this.submit([layer], `move ${element.id} to the layer`);
      this.submit([{ op: 'move', element: element.id, container: id }], `move ${element.id} into ${id}`);
    });
  }

  /** A list to choose one entry from, on the canvas. */
  private pick(label: string, choices: [string, string][], chosen: (id: string) => void): void {
    this.ctx.sheet.querySelector('.wb-pick')?.remove();
    const select = el('select', { class: 'wb-field', 'aria-label': label });
    select.appendChild(el('option', { value: '\u0000', text: label }));
    for (const [id, text] of choices) select.appendChild(el('option', { value: id, text }));
    const form = el('div', { class: 'wb-prompt wb-pick' }, [el('label', { text: label }), select]);
    const done = () => {
      form.remove();
      if (this.open?.close === done) this.open = null;
    };
    this.open = { model: this.modelKey(), close: done };
    select.addEventListener('change', () => {
      const value = select.value;
      done();
      if (value !== '\u0000') chosen(value);
    });
    select.addEventListener('keydown', (event) => {
      event.stopPropagation();
      if (event.key === 'Escape') done();
    });
    this.ctx.sheet.prepend(form);
    select.focus();
  }

  // --- align, distribute, nudge ----------------------------------------------------

  private boxes(elements: ElementRecord[]): Box[] {
    const scale = this.ctx.viewer()?.scale?.() ?? 1;
    return elements.flatMap((e) => {
      const id = this.ctx.drawingId(e.id);
      const node = this.ctx.host.querySelector(`.av-node[data-id="${CSS.escape(id)}"]`);
      const shape = node?.querySelector(':scope > .av-shape') ?? node;
      if (!shape) return [];
      const r = shape.getBoundingClientRect();
      return [{ id, x: r.left / scale, y: r.top / scale, width: r.width / scale, height: r.height / scale }];
    });
  }

  align(how: Alignment): void {
    const moves = alignMoves(this.boxes(this.ctx.outermost(this.selection())), how);
    if (moves.length) void this.ctx.viewer()?.moveMany?.(moves);
  }

  distribute(axis: 'horizontal' | 'vertical'): void {
    const moves = distributeMoves(this.boxes(this.ctx.outermost(this.selection())), axis);
    if (moves.length) void this.ctx.viewer()?.moveMany?.(moves);
  }

  private nudge(event: KeyboardEvent): void {
    const step = { ArrowLeft: [-1, 0], ArrowRight: [1, 0], ArrowUp: [0, -1], ArrowDown: [0, 1] }[event.key];
    if (!step || event.metaKey || event.ctrlKey || event.altKey) return;
    if (event.target instanceof HTMLInputElement || event.target instanceof HTMLSelectElement || event.target instanceof HTMLTextAreaElement) return;
    if ((event.target as HTMLElement).closest?.('.wb-menu, .wb-help, .arcviz-handle')) return;
    if (this.ctx.host.querySelector('.arcviz-stage.is-connecting')) return;
    const elements = this.ctx.outermost(this.selection());
    if (elements.length === 0) return;
    event.preventDefault();
    event.stopPropagation();
    const by = event.shiftKey ? NUDGE_BIG : NUDGE;
    void this.ctx.viewer()?.moveMany?.(elements.map((e) => ({ id: this.ctx.drawingId(e.id), dx: step[0] * by, dy: step[1] * by })));
  }

  selectAll(): void {
    const all = (this.ctx.graph()?.children ?? []).map((node) => this.ctx.elementFor(node.id)).filter((e): e is ElementRecord => !!e);
    this.ctx.setMulti(all);
  }

  // --- the keys of the canvas -------------------------------------------------------

  /** ⌘C ⌘X ⌘V ⌘D ⌘A and ?: true when the key was taken. */
  key(event: KeyboardEvent): boolean {
    // The canvas's keys only: text selected or focused elsewhere keeps the browser's (and VS Code's) own.
    const target = event.target as Node;
    const onCanvas = target === document.body || this.ctx.host.contains(target);
    if (!onCanvas || (window.getSelection()?.toString() ?? '') !== '') return false;
    const command = event.metaKey || event.ctrlKey;
    const letter = event.key.toLowerCase();
    if (!command && event.key === '?') return this.run(event, () => this.ctx.help());
    if (!command || event.altKey || event.shiftKey) return false;
    const some = this.copyable().length > 0;
    if (letter === 'c' && some) return this.run(event, () => this.copy(false));
    if (letter === 'x' && some) return this.run(event, () => this.copy(true));
    if (letter === 'v' && this.clip) return this.run(event, () => this.paste());
    if (letter === 'd' && some) return this.run(event, () => this.duplicate());
    if (letter === 'a' && this.ctx.graph()) return this.run(event, () => this.selectAll());
    return false;
  }

  private run(event: KeyboardEvent, action: () => void): boolean {
    event.preventDefault();
    action();
    return true;
  }

  // --- the context menu and the floating toolbar --------------------------------------

  private contextMenu(event: MouseEvent): void {
    event.preventDefault();
    // Under the handles that float over the sheet, what is drawn there.
    const target = document.elementsFromPoint(event.clientX, event.clientY).map((hit) => hit.closest<HTMLElement>('.av-port, .av-edge, .av-node')).find((hit) => !!hit) ?? null;
    const isEdge = !!target?.classList.contains('av-edge');
    const element = target ? this.ctx.elementFor(target.dataset.id ?? '', isEdge) : null;
    const inMulti = element && this.ctx.multi().some((e) => e.uuid === element.uuid);
    if (element && !inMulti) {
      this.ctx.setMulti([]);
      this.ctx.viewer()?.select?.(target!.dataset.id ?? null);
      this.ctx.select(element, false);
    }
    this.menuModel = this.modelKey();
    this.menu.open(event.clientX, event.clientY, this.menuItems(element, target?.dataset.id ?? null));
  }

  /** The actions on an element (or on the sheet when none), in the order a modeller looks for them. */
  menuItems(element: ElementRecord | null, drawing: string | null): MenuItem[] {
    const many = this.ctx.multi().length > 1 ? this.ctx.multi() : [];
    const paste: MenuAction = { label: this.clip?.cut ? 'Paste (move here)' : 'Paste', shortcut: '⌘V', run: () => this.paste(), disabled: !this.clip };
    if (many.length) {
      return [
        { title: 'Align', row: [['Left', 'left'], ['Centre', 'center'], ['Right', 'right'], ['Top', 'top'], ['Middle', 'middle'], ['Bottom', 'bottom']].map(([label, how]) => ({ label, run: () => this.align(how as Alignment) })) },
        { title: 'Distribute', row: [{ label: 'Horizontally', run: () => this.distribute('horizontal'), disabled: many.length < 3 }, { label: 'Vertically', run: () => this.distribute('vertical'), disabled: many.length < 3 }] },
        'separator',
        { label: 'Copy', shortcut: '⌘C', run: () => this.copy(false) },
        { label: 'Cut', shortcut: '⌘X', run: () => this.copy(true) },
        { label: 'Duplicate', shortcut: '⌘D', run: () => this.duplicate() },
        'separator',
        { label: `Hide ${many.length} from this diagram`, shortcut: '⌘H', run: () => this.ctx.hide(many) },
        { label: `Delete ${many.length} elements`, shortcut: '⌫', danger: true, run: () => this.ctx.remove(many) },
      ];
    }
    if (!element) {
      return [paste, { label: 'Select all', shortcut: '⌘A', run: () => this.selectAll() }, 'separator', { label: 'Fit the diagram', shortcut: '0', run: () => this.ctx.viewer()?.fit?.() }, { label: 'Keyboard shortcuts', shortcut: '?', run: () => this.ctx.help() }];
    }
    const relationship = this.relationshipOf(element);
    const source: MenuAction = { label: 'Go to source', run: () => this.ctx.reveal(element) };
    if (relationship && drawing) {
      const bent = (this.ctx.viewer()?.routeOf?.(drawing) ?? []).length > 0;
      return [
        { label: 'Rename', shortcut: 'F2', run: () => this.rename(element, drawing), disabled: !relationship.name },
        { label: 'Reconnect the source…', run: () => this.reconnectFromList(drawing, 'source') },
        { label: 'Reconnect the target…', run: () => this.reconnectFromList(drawing, 'target') },
        { label: 'Reverse', run: () => this.reverse(element) },
        ...(bent ? [{ label: 'Straighten', run: () => this.ctx.viewer()?.setRoute?.(drawing, []) } as MenuAction] : []),
        'separator',
        source,
        { label: 'Delete', shortcut: '⌫', danger: true, run: () => this.ctx.remove([element]), disabled: !relationship.name },
      ];
    }
    const isPort = /Port$/.test(element.kind);
    return [
      { label: 'Rename', shortcut: 'F2', run: () => this.rename(element, drawing ?? undefined) },
      { label: 'Move to…', run: () => this.moveTo(element) },
      ...(isPort ? [] : [{ label: 'Hide from this diagram', shortcut: '⌘H', run: () => this.ctx.hide([element]) } as MenuAction]),
      'separator',
      { label: 'Copy', shortcut: '⌘C', run: () => this.copy(false), disabled: isPort },
      { label: 'Cut', shortcut: '⌘X', run: () => this.copy(true), disabled: isPort },
      { ...paste, label: this.clip ? `${paste.label} into ${element.name || element.id}` : 'Paste into' },
      { label: 'Duplicate', shortcut: '⌘D', run: () => this.duplicate(), disabled: isPort },
      'separator',
      { label: 'Show related', run: () => this.ctx.related(element), disabled: isPort },
      { label: 'New diagram around', run: () => this.ctx.newDiagram(element), disabled: isPort },
      source,
      'separator',
      { label: 'Delete', shortcut: '⌫', danger: true, run: () => this.ctx.remove([element]) },
    ];
  }

  /** The floating toolbar follows the selection (hidden while something is dragged). */
  private placeToolbar(): void {
    const stage = this.ctx.host.querySelector('.arcviz-stage');
    if (!stage || stage.matches('.is-moving, .is-connecting, .is-reconnecting, .is-resizing, .is-panning') || this.menu.isOpen || this.ctx.sheet.querySelector('.wb-direct-edit, .wb-prompt')) return this.mini.hide();
    const many = this.ctx.multi().length > 1 ? this.ctx.multi() : [];
    const one = this.ctx.selected();
    const drawing = this.ctx.selectedDrawing();
    const rectOf = (id: string) => {
      const node = this.ctx.host.querySelector(`[data-id="${CSS.escape(id)}"]`);
      const shape = node?.querySelector(':scope > .av-shape, :scope > .av-line') ?? node;
      return shape ? shape.getBoundingClientRect() : null;
    };
    if (many.length) {
      const rects = many.map((e) => rectOf(this.ctx.drawingId(e.id))).filter((r): r is DOMRect => !!r && r.width > 0);
      if (rects.length === 0) return this.mini.hide();
      const around = new DOMRect(Math.min(...rects.map((r) => r.left)), Math.min(...rects.map((r) => r.top)), 0, 0);
      const union = new DOMRect(around.x, around.y, Math.max(...rects.map((r) => r.right)) - around.x, Math.max(...rects.map((r) => r.bottom)) - around.y);
      return this.mini.show(union, [
        { glyph: '⫷', label: 'Align left', run: () => this.align('left') },
        { glyph: '⫠', label: 'Align top', run: () => this.align('top') },
        { glyph: '⋯', label: 'More: align, distribute, copy…', run: () => this.menu.open(union.left, union.top, this.menuItems(null, null)) },
        { glyph: '⌫', label: 'Delete all', shortcut: '⌫', danger: true, run: () => this.ctx.remove(many) },
      ], `multi:${many.map((e) => e.uuid).join(',')}`);
    }
    if (!one || !drawing) return this.mini.hide();
    const relationship = this.relationshipOf(one);
    const drawn = rectOf(drawing);
    if (!drawn || (drawn.width === 0 && drawn.height === 0)) return this.mini.hide();
    // Above a line, the toolbar clears its name.
    const around = relationship ? new DOMRect(drawn.x, drawn.y - 18, drawn.width, drawn.height + 18) : drawn;
    const more = { glyph: '⋯', label: 'More actions', run: () => this.menu.open(around.right, around.top, this.menuItems(one, drawing)) };
    const actions = relationship
      ? [
          { glyph: '✎', label: 'Rename', shortcut: 'F2', run: () => this.rename(one, drawing), disabled: !relationship.name },
          { glyph: '⇄', label: 'Reverse', run: () => this.reverse(one) },
          { glyph: '⤳', label: 'Reconnect: drag an end of the line, or choose the target', run: () => this.reconnectFromList(drawing, 'target') },
          more,
          { glyph: '⌫', label: 'Delete', shortcut: '⌫', danger: true, run: () => this.ctx.remove([one]), disabled: !relationship.name },
        ]
      : [
          { glyph: '✎', label: 'Rename', shortcut: 'F2', run: () => this.rename(one, drawing) },
          { glyph: '⧉', label: 'Duplicate', shortcut: '⌘D', run: () => this.duplicate(), disabled: /Port$/.test(one.kind) },
          { glyph: '↦', label: 'Move to…', run: () => this.moveTo(one) },
          more,
          { glyph: '⌫', label: 'Delete', shortcut: '⌫', danger: true, run: () => this.ctx.remove([one]) },
        ];
    this.mini.show(around, actions, `${one.uuid}:${drawing}`);
  }
}
