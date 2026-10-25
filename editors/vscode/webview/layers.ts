// The Layers panel, draw.io's: one layer per tag of the model and Default
// (what carries no tag), each with its count, an eye (hidden layers are not
// drawn, nor exported), a padlock (what is on a locked layer cannot be
// moved, resized, reconnected, deleted or renamed on the canvas) and its
// place (drag the rows, or ▲ ▼: a higher layer is drawn over the others).
// Visibility, locks and order are kept in the layout file (one undo step
// each); moving the selection to a layer sets its tag (an edit of the model).
import { DEFAULT_LAYER, dropLayer, layerNameProblem, layerRows, LayerRow, stepLayer } from '../src/pure/arrange';
import type { Snapshot } from '../src/pure/workbench';
import { button, clear, el, inlinePrompt } from './dom';
import type { ArrangeApi } from './arrange';

export interface LayersDeps {
  api: () => ArrangeApi | null;
  snapshot: () => Snapshot | null;
  moveHere: (layer: string) => void;
  toast: (text: string, kind: 'info' | 'error') => void;
  pressed: (on: boolean) => void;
}

export class LayersPanel {
  private readonly root: HTMLElement;
  private readonly list: HTMLElement;
  /** Layers made on the panel, not on any element yet: they exist once an element carries them. */
  extra: string[] = [];
  private dragged: string | null = null;

  constructor(private readonly host: HTMLElement, private readonly d: LayersDeps) {
    loadStyles();
    this.list = el('ul', { class: 'wb-layers-list', role: 'list' });
    this.root = el('section', { class: 'wb-layers', role: 'dialog', 'aria-label': 'Layers', hidden: true }, [
      el('header', { class: 'wb-layers-head' }, [
        el('h2', { text: 'Layers' }),
        button('✕', () => this.toggle(false), { class: 'wb-icon-button', title: 'Close (⇧⌘L)' }),
      ]),
      el('p', { class: 'wb-layers-hint', text: 'A layer is a tag; the first tag of an element is its layer. Drag a row to draw its layer over or under the others.' }),
      this.list,
      el('footer', { class: 'wb-layers-foot' }, [
        button('New layer…', () => this.newLayer(false), { class: 'wb-link', title: 'A new tag: the layer exists once an element carries it' }),
      ]),
    ]);
    this.root.addEventListener('keydown', (event) => {
      if (event.key === 'Escape') {
        event.stopPropagation();
        this.toggle(false);
      }
    });
    host.appendChild(this.root);
  }

  get isOpen(): boolean {
    return !this.root.hidden;
  }

  toggle(force?: boolean): void {
    const open = force ?? this.root.hidden;
    this.root.hidden = !open;
    this.d.pressed(open);
    if (open) {
      this.refresh();
      this.list.querySelector<HTMLElement>('button')?.focus({ preventScroll: true });
    }
  }

  private rows(): LayerRow[] {
    const records = this.d.snapshot()?.elements;
    const order = this.d.api()?.layerState().order ?? [];
    return layerRows([...(records?.elements ?? []), ...(records?.relationships ?? [])], order, this.extra);
  }

  /** The layers as the panel lists them (for the menus). */
  layerRows(): LayerRow[] {
    return this.rows();
  }

  refresh(): void {
    if (this.root.hidden) return;
    const api = this.d.api();
    clear(this.list);
    if (!api) {
      this.list.appendChild(el('li', { class: 'wb-layers-empty', text: 'No diagram on screen.' }));
      return;
    }
    const state = api.layerState();
    const rows = this.rows();
    const names = rows.map((r) => r.name);
    rows.forEach((row, index) => {
      const hidden = state.hidden.includes(row.name);
      const locked = state.locked.includes(row.name);
      const item = el('li', { class: `wb-layer-row${hidden ? ' is-hidden' : ''}${locked ? ' is-locked' : ''}`, draggable: 'true', 'data-layer': row.name, title: row.name === DEFAULT_LAYER ? 'The elements that carry no tag' : `The elements whose first tag is "${row.name}"` }, [
        button(hidden ? '◌' : '◉', () => void this.act(api.setLayerHidden(row.name, !hidden)), { class: 'wb-layer-eye', 'aria-pressed': String(!hidden), 'aria-label': `${hidden ? 'Show' : 'Hide'} ${row.label}`, title: hidden ? 'Hidden: not drawn, not exported. Click to show.' : 'Shown. Click to hide (and leave out of exports).' }),
        button(locked ? '🔒' : '🔓', () => void this.act(api.setLayerLocked(row.name, !locked)), { class: 'wb-layer-lock', 'aria-pressed': String(locked), 'aria-label': `${locked ? 'Unlock' : 'Lock'} ${row.label}`, title: locked ? 'Locked on the canvas. Click to unlock.' : 'Click to lock: no move, resize, reconnect, delete or rename on the canvas.' }),
        el('span', { class: 'wb-layer-name', text: row.label }),
        el('span', { class: 'wb-layer-count', text: String(row.count), title: `${row.count} element${row.count === 1 ? '' : 's'} on it` }),
        button('▲', () => void this.order(stepLayer(names, row.name, -1)), { class: 'wb-layer-step', 'aria-label': `Move ${row.label} up`, disabled: index === 0 }),
        button('▼', () => void this.order(stepLayer(names, row.name, 1)), { class: 'wb-layer-step', 'aria-label': `Move ${row.label} down`, disabled: index === rows.length - 1 }),
        button('⇥', () => this.d.moveHere(row.name), { class: 'wb-layer-move', 'aria-label': `Move the selection to ${row.label}`, title: 'Move the selection to this layer (sets its tag)' }),
      ]);
      item.addEventListener('dragstart', (event) => {
        this.dragged = row.name;
        event.dataTransfer?.setData('text/plain', row.name);
        item.classList.add('is-dragged');
      });
      item.addEventListener('dragend', () => item.classList.remove('is-dragged'));
      item.addEventListener('dragover', (event) => {
        if (this.dragged === null) return;
        event.preventDefault();
        item.classList.add('is-drop');
      });
      item.addEventListener('dragleave', () => item.classList.remove('is-drop'));
      item.addEventListener('drop', (event) => {
        event.preventDefault();
        item.classList.remove('is-drop');
        const moved = this.dragged;
        this.dragged = null;
        if (moved === null || moved === row.name) return;
        void this.order(dropLayer(names, moved, row.name));
      });
      this.list.appendChild(item);
    });
  }

  private async act(done: ReturnType<ArrangeApi['setLayerHidden']>): Promise<void> {
    const answer = await done;
    if (!answer.ok && answer.reason) this.d.toast(answer.reason, 'info');
    this.refresh();
  }

  /** The order of every layer shown, top first, written as the reader's. */
  private order(names: string[]): Promise<void> {
    const api = this.d.api();
    if (!api) return Promise.resolve();
    return this.act(api.setLayerOrder(names));
  }

  /** A new layer by name; `move`: the selection goes on it at once. */
  newLayer(move: boolean): void {
    void inlinePrompt(this.host, 'New layer (a tag)', '', 'Safety, Draft, Comms…').then((name) => {
      if (name === null) return;
      const problem = layerNameProblem(name, this.rows().map((r) => r.name));
      if (problem) return this.d.toast(problem, 'error');
      this.extra = [...this.extra, name.trim()];
      if (move) this.d.moveHere(name.trim());
      this.toggle(true);
    });
  }
}

/** The panel's stylesheet, next to the workbench's (no change to the page's HTML). */
function loadStyles(): void {
  const base = document.querySelector<HTMLLinkElement>('link[href*="workbench.css"]');
  if (!base || document.querySelector('link[data-arrange-styles]')) return;
  const link = document.createElement('link');
  link.rel = 'stylesheet';
  link.href = base.href.replace(/workbench\.css(\?.*)?$/, 'arrange.css');
  link.dataset.arrangeStyles = '';
  base.after(link);
}
