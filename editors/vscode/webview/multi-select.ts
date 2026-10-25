// The multi-selection of the workbench (Shift+click): the elements selected
// together, marked on the drawing, with a bar to delete or hide them all in
// one gesture. The page keeps the primary selection; this keeps the others.
import { EditRequest, ElementRecord } from '../src/pure/workbench';
import { button, el } from './dom';
import { arrangeAllows } from './arrange'; // zorder hook

export interface MultiSelectionDeps {
  host: HTMLElement;
  sheet: HTMLElement;
  /** The drawing id of a model element. */
  drawingId: (id: string) => string;
  byUuid: () => Map<string, ElementRecord>;
  /** The viewer redraws its tools (the selection toolbar) for the new selection. */
  refreshTools: () => void;
  /** The properties pane shows the attributes the elements have in common. */
  showMany: (elements: ElementRecord[]) => void;
  submit: (requests: EditRequest[], label: string) => void;
  /** Hide the elements from the diagram on screen. */
  hide: (elements: ElementRecord[]) => void;
}

export class MultiSelection {
  /** The elements selected together, the primary one included; empty when one or none is selected. */
  elements: ElementRecord[] = [];
  private shiftDown = false;
  private readonly bar: HTMLElement;
  private readonly count: HTMLElement;

  constructor(private readonly d: MultiSelectionDeps) {
    d.host.addEventListener('pointerdown', (event) => (this.shiftDown = event.shiftKey), true);
    this.bar = el('div', { class: 'wb-multibar', hidden: true, role: 'toolbar', 'aria-label': 'Selection' });
    this.count = el('strong', { text: '' });
    this.bar.append(
      this.count,
      button('Delete all', () => this.deleteMany(this.elements), { class: 'wb-danger', title: 'Delete every selected element (one undo)' }),
      button('Hide all', () => this.hideMany(this.elements), { title: 'Hide every selected element from this diagram' }),
      button('Clear', () => {
        this.elements = [];
        this.mark();
      }),
    );
    d.sheet.appendChild(this.bar);
  }

  /** The selection marked on the drawing, the bar and the properties of the elements in common. */
  mark(): void {
    const { host } = this.d;
    for (const node of host.querySelectorAll('.wb-multi')) node.classList.remove('wb-multi');
    for (const element of this.elements) host.querySelector(`[data-id="${CSS.escape(this.d.drawingId(element.id))}"]`)?.classList.add('wb-multi');
    this.bar.hidden = this.elements.length < 2;
    this.d.refreshTools();
    this.count.textContent = `${this.elements.length} selected`;
    if (this.elements.length > 1) this.d.showMany(this.elements);
  }

  /**
   * A click with Shift held on an element (not a line): it joins the selection,
   * or leaves it when it was in it. False when the click was not one.
   */
  shiftClick(element: ElementRecord | null, primary: ElementRecord | null, isEdge: boolean): boolean {
    if (!this.shiftDown || !element || isEdge) return false;
    const base = this.elements.length ? this.elements : primary && primary.uuid !== element.uuid ? [primary] : [];
    this.elements = base.some((e) => e.uuid === element.uuid) ? base.filter((e) => e.uuid !== element.uuid) : [...base, element];
    this.shiftDown = false;
    setTimeout(() => this.mark(), 0);
    return true;
  }

  /** The selected elements without those another selected element owns (removing the owner removes them). */
  outermost(elements: ElementRecord[]): ElementRecord[] {
    const byUuid = this.d.byUuid();
    const uuids = new Set(elements.map((e) => e.uuid));
    const ownedBySelection = (element: ElementRecord): boolean => {
      let owner = element.owner ? byUuid.get(element.owner) : undefined;
      while (owner) {
        if (uuids.has(owner.uuid)) return true;
        owner = owner.owner ? byUuid.get(owner.owner) : undefined;
      }
      return false;
    };
    return elements.filter((e) => !ownedBySelection(e));
  }

  deleteMany(elements: ElementRecord[]): void {
    if (!arrangeAllows(elements, 'delete')) return; // zorder hook
    const targets = this.outermost(elements);
    this.d.submit(targets.map((e) => ({ op: 'remove', element: e.id })), `delete ${targets.length} elements`);
    this.elements = [];
    this.mark();
  }

  hideMany(elements: ElementRecord[]): void {
    this.d.hide(elements);
    this.elements = [];
    this.mark();
  }
}
