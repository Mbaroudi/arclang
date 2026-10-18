// The semantic browser: what the selected element is connected to, by
// relationship and by trace, each entry a step to that element.
import { ElementRecord, neighbours, Snapshot } from '../src/pure/workbench';
import { button, clear, el } from './dom';

export class SemanticBrowser {
  private readonly root: HTMLElement;

  constructor(host: HTMLElement, private readonly onStep: (element: ElementRecord) => void) {
    this.root = el('div', { class: 'wb-browser' });
    host.appendChild(this.root);
    this.show(null, null);
  }

  show(element: ElementRecord | null, snapshot: Snapshot | null): void {
    clear(this.root);
    if (!element || !snapshot) {
      this.root.appendChild(el('p', { class: 'wb-muted', text: 'The relationships and traces of the selected element appear here.' }));
      return;
    }
    const { incoming, outgoing } = neighbours(element, snapshot);
    const owner = snapshot.elements?.elements.find((e) => e.uuid === element.owner) ?? null;
    const owned = (snapshot.elements?.elements ?? []).filter((e) => e.owner === element.uuid);
    this.root.appendChild(this.column('Owner', owner ? [{ kind: 'owns', name: '', other: owner }] : []));
    this.root.appendChild(this.column('Incoming', incoming));
    this.root.appendChild(this.column('Outgoing', outgoing));
    this.root.appendChild(this.column('Owned', owned.map((other) => ({ kind: other.kind, name: '', other }))));
  }

  private column(title: string, entries: { kind: string; name: string; other: ElementRecord | null }[]): HTMLElement {
    const list = el('ul', { class: 'wb-browser-list' });
    if (entries.length === 0) list.appendChild(el('li', { class: 'wb-muted', text: 'none' }));
    for (const entry of entries) {
      const item = el('li', {}, [el('span', { class: 'wb-kind', text: entry.name ? `${entry.kind} ${entry.name}` : entry.kind })]);
      if (entry.other) {
        const other = entry.other;
        item.appendChild(button(other.name || other.id, () => this.onStep(other), { class: 'wb-link', title: `${other.kind} ${other.id}` }));
      }
      list.appendChild(item);
    }
    return el('section', { class: 'wb-browser-column' }, [el('h3', { text: title }), list]);
  }
}
