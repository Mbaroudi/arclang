// Find in the diagram on screen (⌘F): the boxes whose name or id holds the
// text are marked, Enter steps through them and selects each one.
import type { ElementRecord, ViewerGraph } from '../src/pure/workbench';
import { button, el } from './dom';

export interface FindHost {
  graph: () => ViewerGraph | null;
  elementFor: (drawingId: string) => ElementRecord | null;
  /** Select the drawn box in the viewer and the element in the views. */
  select: (drawingId: string, element: ElementRecord | null) => void;
}

export function createFind(sheet: HTMLElement, host: HTMLElement, owner: FindHost): { open: () => void; close: () => void } {
  const box = el('div', { class: 'wb-find', hidden: true, role: 'search' });
  const input = el('input', { type: 'text', placeholder: 'Find in this diagram', 'aria-label': 'Find in this diagram' });
  const count = el('span', { class: 'wb-find-count' });
  box.append(input, count, button('✕', () => close(), { class: 'wb-icon-button', title: 'Close (Esc)' }));
  sheet.appendChild(box);
  let at = 0;

  function open(): void {
    box.hidden = false;
    input.focus();
    input.select();
  }

  function close(): void {
    box.hidden = true;
    for (const node of host.querySelectorAll('.wb-find-hit')) node.classList.remove('wb-find-hit');
  }

  function run(step: number): void {
    const query = input.value.trim().toLowerCase();
    for (const node of host.querySelectorAll('.wb-find-hit')) node.classList.remove('wb-find-hit');
    if (!query) {
      count.textContent = '';
      return;
    }
    const hits: string[] = [];
    const walk = (nodes: NonNullable<ViewerGraph['children']>) => {
      for (const node of nodes) {
        const element = owner.elementFor(node.id);
        const label = `${element?.name ?? ''} ${element?.id ?? node.id}`.toLowerCase();
        if (label.includes(query)) hits.push(node.id);
        walk(node.children ?? []);
      }
    };
    walk(owner.graph()?.children ?? []);
    at = hits.length ? (at + step + hits.length) % hits.length : 0;
    count.textContent = hits.length ? `${at + 1} of ${hits.length}` : 'no match';
    for (const id of hits) host.querySelector(`[data-id="${CSS.escape(id)}"]`)?.classList.add('wb-find-hit');
    const target = hits[at];
    if (target) owner.select(target, owner.elementFor(target));
  }
  input.addEventListener('input', () => {
    at = 0;
    run(0);
  });
  input.addEventListener('keydown', (event) => {
    event.stopPropagation();
    if (event.key === 'Enter') run(event.shiftKey ? -1 : 1);
    if (event.key === 'Escape') close();
  });
  return { open, close };
}
