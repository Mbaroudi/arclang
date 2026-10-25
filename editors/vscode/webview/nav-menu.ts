// New ▾ at the head of the tool row: a new model, a new layer, a new diagram
// of the open model, and a diagram or a model to open. Each entry is a VS Code
// command (src/workbench/create.ts); the page only asks. Entries that belong
// with the File menu (webview/file-menu.ts, when it exists) can move there.
import type { FromWebview } from '../src/pure/workbench';
import { el } from './dom';
import { ContextMenu, MenuItem } from './menu';

export interface NavMenuDeps {
  /** The tool row the button goes into, after the palette toggle. */
  toolRow: HTMLElement | null;
  sheet: HTMLElement;
  post: (message: FromWebview) => void;
}

/** The entries of the menu, as commands. */
export const NAV_MENU_COMMANDS: { label: string; command: string }[] = [
  { label: 'New model…', command: 'arclang.newModel' },
  { label: 'New layer…', command: 'arclang.newLayer' },
  { label: 'New diagram…', command: 'arclang.newDiagram' },
  { label: 'Open diagram…', command: 'arclang.pickDiagram' },
  { label: 'Open model…', command: 'arclang.openModel' },
];

export function installNavMenu(d: NavMenuDeps): { open: () => void } {
  const button = el('button', { type: 'button', class: 'wb-icon-button wb-bar-toggle wb-nav-button', id: 'wb-nav-menu', 'aria-haspopup': 'menu', 'aria-label': 'New: model, layer, diagram; open a diagram', title: 'New model, layer or diagram; open a diagram' }, [
    el('span', { class: 'wb-nav-plus', 'aria-hidden': 'true', text: '＋' }),
    el('span', { class: 'wb-toggle-label', text: 'New' }),
  ]);
  const palette = d.toolRow?.querySelector('.wb-palette-toggle');
  if (palette) palette.after(button);
  else d.toolRow?.append(button);
  const menu = new ContextMenu(d.sheet);
  const ask = (command: string) => () => d.post({ type: 'command', command });
  const open = (): void => {
    if (menu.isOpen) return menu.close();
    const box = button.getBoundingClientRect();
    const [model, layer, diagram, pick, file] = NAV_MENU_COMMANDS;
    const items: MenuItem[] = [
      { label: model.label, run: ask(model.command) },
      { label: layer.label, run: ask(layer.command) },
      { label: diagram.label, run: ask(diagram.command) },
      'separator',
      { label: pick.label, run: ask(pick.command) },
      { label: file.label, run: ask(file.command) },
    ];
    menu.open(box.left, box.bottom + 2, items);
  };
  button.addEventListener('click', open);
  return { open };
}
