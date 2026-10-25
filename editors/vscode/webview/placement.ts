// Where boxes go on the sheet, draw.io style: a pasted or duplicated copy
// lands where it was asked (at the mouse, or beside its original), several
// selected boxes move together when one of them is dragged, ⌘+drag
// (Ctrl+drag elsewhere) drops a copy where it is let go, and the box under the
// pointer is outlined. Positions are in sheet units (the drawing's own),
// so they hold across a redraw, a pan or a zoom.
import type { ElementRecord, Snapshot } from '../src/pure/workbench';
import { el } from './dom';
import type { ViewerApi } from './globals';

export interface Point {
  x: number;
  y: number;
}

export interface PlacementContext {
  host: HTMLElement;
  sheet: HTMLElement;
  viewer: () => ViewerApi | null;
  snapshot: () => Snapshot | null;
  drawingId: (modelId: string) => string;
  elementFor: (drawingId: string) => ElementRecord | null;
  multi: () => ElementRecord[];
  outermost: (elements: ElementRecord[]) => ElementRecord[];
  /** Run a change of the arrangement as part of the step before it (one undo). */
  joinLayout: (run: () => Promise<void> | undefined) => Promise<void>;
  /** Copies of `elements` dropped this far from them (⌘/Ctrl+drag). */
  copyBy: (elements: ElementRecord[], by: Point) => void;
  /** Select a box as a click would. */
  select: (drawingId: string) => void;
  /** The selection marks are drawn again (after a redraw). */
  remark: () => void;
  /** A click (no drag) on a box of the selection: that box alone is selected. */
  selectOnly: (drawingId: string) => void;
}

/** A copy is placed this far from its original when nothing says where. */
export const PASTE_OFFSET = 20;
/** A press that moves less than this is a click, not a drag. */
const DRAG_THRESHOLD = 4;
const isMac = /Mac/.test(navigator.platform);

/** The drawing on screen (the sheet's SVG). */
function drawing(host: HTMLElement): SVGSVGElement | null {
  return host.querySelector<SVGSVGElement>('.arcviz-stage svg.av-svg') ?? host.querySelector<SVGSVGElement>('.arcviz-stage svg');
}

/** A point of the window in sheet units. */
export function sheetPoint(host: HTMLElement, clientX: number, clientY: number): Point | null {
  const svg = drawing(host);
  const matrix = svg?.getScreenCTM();
  if (!svg || !matrix) return null;
  const p = new DOMPoint(clientX, clientY).matrixTransform(matrix.inverse());
  return { x: p.x, y: p.y };
}

/**
 * The top-left corner of a drawn box, in sheet units: read from the drawing
 * itself, so a pan or a zoom under way (the sheet animates) changes nothing.
 */
export function sheetCorner(host: HTMLElement, drawingId: string): Point | null {
  const node = host.querySelector(`.av-node[data-id="${CSS.escape(drawingId)}"]`);
  const shape = (node?.querySelector(':scope > .av-shape') ?? node) as SVGGraphicsElement | null;
  if (!shape || typeof shape.getBBox !== 'function') return null;
  const svg = shape.ownerSVGElement;
  const box = shape.getBBox();
  const toScreen = shape.getScreenCTM();
  const sheetToScreen = svg?.getScreenCTM();
  if (!toScreen || !sheetToScreen || (box.width === 0 && box.height === 0)) return null;
  // The sheet's own transform on screen cancels out: what is left is the box's place on the sheet.
  const p = new DOMPoint(box.x, box.y).matrixTransform(sheetToScreen.inverse().multiply(toScreen));
  return { x: p.x, y: p.y };
}

/** The top-left corner of several points. */
export function cornerOf(points: Point[]): Point | null {
  if (points.length === 0) return null;
  return { x: Math.min(...points.map((p) => p.x)), y: Math.min(...points.map((p) => p.y)) };
}

/** Where copies go: their group's corner to a point, or moved by an offset from the originals. */
export type PlaceTarget = { to: Point } | { by: Point };

/** An original and where it was when it was copied. */
export interface Original {
  element: ElementRecord;
  at: Point | null;
}

/** The copy of `original` among the new elements: same kind, named after it ("Camera 2"). */
export function copyOf(original: ElementRecord, created: ElementRecord[], used: Set<string>): ElementRecord | null {
  const named = new RegExp(`^${original.name.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')} \\d+$`);
  return created.find((c) => !used.has(c.uuid) && c.kind === original.kind && named.test(c.name)) ?? created.find((c) => !used.has(c.uuid) && c.kind === original.kind) ?? null;
}

export class Placement {
  private group: { primary: string; before: Point; others: { id: string; node: SVGGElement }[]; x: number; y: number; moved: boolean; held: Set<string> } | null = null;
  private hovered: Element | null = null;

  constructor(private readonly ctx: PlacementContext) {
    ctx.host.addEventListener('pointerdown', (event) => this.pointerDown(event), true);
    window.addEventListener('pointermove', (event) => this.pointerMove(event));
    window.addEventListener('pointerup', () => void this.pointerUp());
    ctx.host.addEventListener('pointermove', (event) => this.hover(event), { passive: true });
    ctx.host.addEventListener('pointerleave', () => this.hover(null));
  }

  /** Whether an element is part of a group being dragged: the selection stays as it is. */
  holds(element: ElementRecord | null): boolean {
    return !!element && !!this.group && this.group.held.has(element.uuid);
  }

  /**
   * Put the copies made by the last edit where they were asked: the corner
   * of the group of originals moved `to` a point, or `by` an offset from
   * where the originals are drawn now (the drawing may have been laid out
   * again around the new boxes); each copy keeps its place in the group.
   * `known` are the elements before the edit. A cut moves the originals.
   */
  async place(originals: Original[], known: Set<string>, target: PlaceTarget, cut: boolean): Promise<ElementRecord[]> {
    // The view on screen is drawn again after the remount: the copies are measured on it.
    await this.ctx.viewer()?.settled?.();
    const elements = this.ctx.snapshot()?.elements?.elements ?? [];
    const created = elements.filter((e) => !known.has(e.uuid) && !/Port$/.test(e.kind) && (!e.owner || known.has(e.owner)));
    const used = new Set<string>();
    const pairs = originals.flatMap((original) => {
      const copy = cut ? (elements.find((e) => e.uuid === original.element.uuid) ?? null) : copyOf(original.element, created, used);
      if (!copy) return [];
      used.add(copy.uuid);
      // Where the original is drawn now; a cut one, where it was.
      const now = cut ? null : sheetCorner(this.ctx.host, this.ctx.drawingId(original.element.id));
      return [{ at: now ?? original.at, copy }];
    });
    // Text pasted from elsewhere has no original here: the copies keep their arrangement.
    const placed = pairs.length ? pairs : created.map((copy) => ({ at: sheetCorner(this.ctx.host, this.ctx.drawingId(copy.id)), copy }));
    const corner = cornerOf(placed.map((p) => p.at).filter((p): p is Point => !!p));
    if (!corner) return placed.map((p) => p.copy);
    const to = 'to' in target ? target.to : { x: corner.x + target.by.x, y: corner.y + target.by.y };
    const moves = placed.flatMap(({ at, copy }) => {
      const now = sheetCorner(this.ctx.host, this.ctx.drawingId(copy.id));
      if (!now || !at) return [];
      const dx = at.x - corner.x + to.x - now.x;
      const dy = at.y - corner.y + to.y - now.y;
      return Math.abs(dx) < 0.5 && Math.abs(dy) < 0.5 ? [] : [{ id: this.ctx.drawingId(copy.id), dx, dy }];
    });
    if (moves.length) await this.ctx.joinLayout(() => this.ctx.viewer()?.moveMany?.(moves));
    return placed.map((p) => p.copy);
  }

  // --- several boxes dragged together, a copy dragged off ------------------------

  private nodeAt(event: PointerEvent): SVGGElement | null {
    const target = event.target as Element | null;
    if (!target || target.closest('.arcviz-grip, .arcviz-end, .arcviz-quick, .arcviz-bend, .arcviz-handle, .av-port, .av-edge')) return null;
    return target.closest<SVGGElement>('.av-node');
  }

  private pointerDown(event: PointerEvent): void {
    if (event.button !== 0) return;
    const node = this.nodeAt(event);
    const element = node ? this.ctx.elementFor(node.dataset.id ?? '') : null;
    if (!node || !element) return;
    // As in draw.io: ⌘+drag (Ctrl+drag elsewhere) copies; Alt+drag moves without the grid (the viewer's).
    const copying = isMac ? event.metaKey : event.ctrlKey;
    if (copying && !event.shiftKey) return this.copyDrag(event, node, element);
    if (event.shiftKey || event.metaKey || event.ctrlKey || event.altKey) return;
    const multi = this.ctx.multi();
    if (multi.length < 2 || !multi.some((e) => e.uuid === element.uuid)) return;
    const before = sheetCorner(this.ctx.host, node.dataset.id ?? '');
    if (!before) return;
    const others = this.ctx
      .outermost(multi)
      .filter((e) => e.uuid !== element.uuid)
      .map((e) => this.ctx.drawingId(e.id))
      .flatMap((id) => {
        const other = this.ctx.host.querySelector<SVGGElement>(`.av-node[data-id="${CSS.escape(id)}"]`);
        return other ? [{ id, node: other }] : [];
      });
    this.group = { primary: node.dataset.id ?? '', before, others, x: event.clientX, y: event.clientY, moved: false, held: new Set(multi.map((e) => e.uuid)) };
  }

  private pointerMove(event: PointerEvent): void {
    const group = this.group;
    if (!group || !(event.buttons & 1)) return;
    const dx = event.clientX - group.x;
    const dy = event.clientY - group.y;
    if (!group.moved && Math.hypot(dx, dy) < DRAG_THRESHOLD) return;
    group.moved = true;
    // The others follow the pointer while the viewer moves the one under it.
    const scale = this.ctx.viewer()?.scale?.() ?? 1;
    for (const other of group.others) other.node.style.translate = `${dx / scale}px ${dy / scale}px`;
  }

  private async pointerUp(): Promise<void> {
    const group = this.group;
    if (!group) return;
    if (!group.moved) {
      this.group = null;
      this.ctx.selectOnly(group.primary);
      return;
    }
    // The viewer redraws the dragged box where it was let go: the others follow by as much.
    let after: Point | null = null;
    for (let waited = 0; waited < 2500; waited += 50) {
      await new Promise((resolve) => setTimeout(resolve, 50));
      after = sheetCorner(this.ctx.host, group.primary);
      if (after && (Math.abs(after.x - group.before.x) > 0.5 || Math.abs(after.y - group.before.y) > 0.5)) break;
    }
    for (const other of group.others) other.node.style.translate = '';
    const dx = after ? after.x - group.before.x : 0;
    const dy = after ? after.y - group.before.y : 0;
    if (Math.abs(dx) > 0.5 || Math.abs(dy) > 0.5) {
      await this.ctx.joinLayout(() => this.ctx.viewer()?.moveMany?.(group.others.map((o) => ({ id: o.id, dx, dy }))));
    }
    this.group = null;
    this.ctx.remark();
  }

  /** ⌘+drag on a Mac, Ctrl+drag elsewhere: a copy follows the pointer and is made where it is let go. */
  private copyDrag(down: PointerEvent, node: SVGGElement, element: ElementRecord): void {
    down.preventDefault();
    down.stopImmediatePropagation();
    const multi = this.ctx.multi();
    const elements = multi.length > 1 && multi.some((e) => e.uuid === element.uuid) ? this.ctx.outermost(multi) : [element];
    const rects = elements.flatMap((e) => {
      const shape = this.ctx.host.querySelector(`.av-node[data-id="${CSS.escape(this.ctx.drawingId(e.id))}"] > .av-shape`);
      return shape ? [shape.getBoundingClientRect()] : [];
    });
    const frame = this.ctx.sheet.getBoundingClientRect();
    const ghosts = rects.map((r) => {
      const ghost = el('div', { class: 'wb-ghost', 'aria-hidden': 'true' });
      Object.assign(ghost.style, { left: `${r.left - frame.left}px`, top: `${r.top - frame.top}px`, width: `${r.width}px`, height: `${r.height}px` });
      this.ctx.sheet.appendChild(ghost);
      return ghost;
    });
    let moved = false;
    const move = (event: PointerEvent) => {
      const dx = event.clientX - down.clientX;
      const dy = event.clientY - down.clientY;
      moved = moved || Math.hypot(dx, dy) >= DRAG_THRESHOLD;
      for (const ghost of ghosts) ghost.style.translate = `${dx}px ${dy}px`;
    };
    const up = (event: PointerEvent) => {
      window.removeEventListener('pointermove', move, true);
      window.removeEventListener('pointerup', up, true);
      for (const ghost of ghosts) ghost.remove();
      if (!moved) return this.ctx.select(node.dataset.id ?? '');
      const from = sheetPoint(this.ctx.host, down.clientX, down.clientY);
      const to = sheetPoint(this.ctx.host, event.clientX, event.clientY);
      if (from && to) this.ctx.copyBy(elements, { x: to.x - from.x, y: to.y - from.y });
    };
    window.addEventListener('pointermove', move, true);
    window.addEventListener('pointerup', up, true);
  }

  // --- the box under the pointer ----------------------------------------------------

  private hover(event: PointerEvent | null): void {
    const target = event?.target as Element | null | undefined;
    const node = target && !(event && event.buttons) ? target.closest('.av-node') : null;
    if (node === this.hovered) return;
    this.hovered?.classList.remove('wb-hover');
    node?.classList.add('wb-hover');
    this.hovered = node;
  }
}
