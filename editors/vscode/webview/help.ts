// The keyboard shortcuts of the canvas, one key away (?): what can be done
// to an element without hunting for the menu that does it.
import { button, el } from './dom';
import { ARRANGE_HELP } from '../src/pure/arrange'; // zorder hook

const SHORTCUTS: [string, string][] = [
  ['Double-click, F2', 'Rename the box or exchange in place'],
  ['Right-click', 'Every action on what is under the pointer'],
  ['Delete, Backspace', 'Delete the selection (what it owns is asked first)'],
  ['⌘C  ⌘X  ⌘V', 'Copy, cut, paste into the selected box (a cut then paste moves)'],
  ['⌘D', 'Duplicate, with new ids'],
  ['⌘A', 'Select every box of the diagram'],
  ['Arrows, Shift+arrows', 'Move the selection by 1, by 10'],
  ['Drag on the sheet', 'Select the boxes it encloses; Shift+click adds one'],
  ['Space+drag, two fingers', 'Pan the sheet'],
  ['Drag a box', 'Place it (guides snap it; Alt moves freely); onto another box: move it inside'],
  ['Drag an end of the selected exchange', 'Reconnect it; its middle bends it, a double-click on a bend removes it'],
  ['Arrows on the sides of a box', 'Drag: connect; click: create a connected element'],
  ['⌘H', 'Hide from this diagram'],
  ['⌘F', 'Find in this diagram'],
  ['C', 'Connect from the selected box with the keyboard'],
  ['V, letters', 'The pointer, the palette tools'],
  ['[  ]', 'Previous, next layer'],
  ['Shift+M', 'Minimap'],
  ['+  −  0', 'Zoom in, out, fit'],
  ['⌘Z  ⇧⌘Z', 'Undo, redo (each gesture is one step)'],
  ['Esc', 'Cancel, clear the selection, leave full screen'],
  ...ARRANGE_HELP, // zorder hook: order, groups, locks, layers, select
  ['?', 'This list'],
];

export function createHelp(host: HTMLElement): { toggle: (force?: boolean) => void; isOpen: () => boolean } {
  const list = el('dl', { class: 'wb-help-list' });
  for (const [keys, what] of SHORTCUTS) list.append(el('dt', {}, [el('kbd', { text: keys })]), el('dd', { text: what }));
  const panel = el('section', { class: 'wb-help', role: 'dialog', 'aria-label': 'Keyboard shortcuts', hidden: true, tabindex: '-1' }, [
    el('header', {}, [el('h2', { text: 'Diagram editing' }), button('✕', () => toggle(false), { class: 'wb-icon-button', title: 'Close (Esc)' })]),
    list,
  ]);
  panel.addEventListener('keydown', (event) => {
    if (event.key === 'Escape' || event.key === '?') {
      event.preventDefault();
      event.stopPropagation();
      toggle(false);
    }
  });
  host.appendChild(panel);
  let back: HTMLElement | null = null;
  function toggle(force?: boolean): void {
    const open = force ?? panel.hidden;
    if (open) back = document.activeElement as HTMLElement | null;
    panel.hidden = !open;
    if (open) panel.focus();
    else back?.focus?.({ preventScroll: true });
  }
  return { toggle, isOpen: () => !panel.hidden };
}
