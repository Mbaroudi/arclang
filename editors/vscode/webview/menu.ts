// The context menu of the canvas and the toolbar that floats by the
// selection. Both only show actions; what each one does is the caller's.
// The menu is a real menu for the keyboard: arrows move, Enter runs, Esc
// closes, and focus goes back where it was.
import { button, clear, el } from './dom';

export interface MenuAction {
  label: string;
  run: () => void;
  shortcut?: string;
  disabled?: boolean;
  danger?: boolean;
}

/** One line of the menu: an action, a row of small actions under a title, or a rule. */
export type MenuItem = MenuAction | { title: string; row: MenuAction[] } | 'separator';

export class ContextMenu {
  private readonly root: HTMLElement;
  private returnFocus: HTMLElement | null = null;

  constructor(private readonly host: HTMLElement) {
    this.root = el('div', { class: 'wb-menu', role: 'menu', hidden: true, tabindex: '-1' });
    host.appendChild(this.root);
    this.root.addEventListener('keydown', (event) => this.key(event));
    document.addEventListener('pointerdown', (event) => {
      if (!this.root.hidden && !this.root.contains(event.target as Node)) this.close();
    }, true);
    window.addEventListener('blur', () => this.close());
  }

  get isOpen(): boolean {
    return !this.root.hidden;
  }

  /** Open at a point of the window, kept inside the host. */
  open(x: number, y: number, items: MenuItem[]): void {
    this.returnFocus = document.activeElement as HTMLElement | null;
    clear(this.root);
    for (const item of items) {
      if (item === 'separator') {
        this.root.appendChild(el('div', { class: 'wb-menu-rule', role: 'separator' }));
      } else if ('row' in item) {
        const row = el('div', { class: 'wb-menu-row', role: 'group', 'aria-label': item.title }, [el('span', { class: 'wb-menu-title', text: item.title })]);
        for (const action of item.row) row.appendChild(this.entry(action, 'wb-menu-chip'));
        this.root.appendChild(row);
      } else {
        this.root.appendChild(this.entry(item, 'wb-menu-item'));
      }
    }
    this.root.hidden = false;
    const frame = this.host.getBoundingClientRect();
    const box = this.root.getBoundingClientRect();
    const left = Math.max(4, Math.min(x - frame.left, frame.width - box.width - 4));
    const top = Math.max(4, Math.min(y - frame.top, frame.height - box.height - 4));
    this.root.style.left = `${left}px`;
    this.root.style.top = `${top}px`;
    this.entries()[0]?.focus();
  }

  close(): void {
    if (this.root.hidden) return;
    this.root.hidden = true;
    this.returnFocus?.focus?.({ preventScroll: true });
  }

  private entry(action: MenuAction, className: string): HTMLButtonElement {
    const node = button('', () => {
      this.close();
      action.run();
    }, { class: `${className}${action.danger ? ' is-danger' : ''}`, role: 'menuitem', disabled: action.disabled, title: action.shortcut ? `${action.label} (${action.shortcut})` : action.label });
    node.appendChild(el('span', { class: 'wb-menu-label', text: action.label }));
    if (action.shortcut && className === 'wb-menu-item') node.appendChild(el('kbd', { class: 'wb-menu-key', text: action.shortcut }));
    return node;
  }

  private entries(): HTMLButtonElement[] {
    return [...this.root.querySelectorAll<HTMLButtonElement>('button:not([disabled])')];
  }

  private key(event: KeyboardEvent): void {
    event.stopPropagation();
    const entries = this.entries();
    const at = entries.indexOf(document.activeElement as HTMLButtonElement);
    const step = { ArrowDown: 1, ArrowRight: 1, ArrowUp: -1, ArrowLeft: -1 }[event.key];
    if (step) {
      event.preventDefault();
      entries[(at + step + entries.length) % entries.length]?.focus();
    } else if (event.key === 'Home' || event.key === 'End') {
      event.preventDefault();
      entries[event.key === 'Home' ? 0 : entries.length - 1]?.focus();
    } else if (event.key === 'Escape' || event.key === 'Tab') {
      event.preventDefault();
      this.close();
    }
  }
}

/** A few actions floating by the selection, draw.io style. */
export class MiniToolbar {
  private readonly root: HTMLElement;

  constructor(private readonly host: HTMLElement) {
    this.root = el('div', { class: 'wb-mini', role: 'toolbar', 'aria-label': 'Selection actions', hidden: true });
    host.appendChild(this.root);
  }

  /** Show `actions` above `around` (window coordinates), or below when there is no room; `extra` (a group of controls) after them. */
  show(around: DOMRect | null, actions: (MenuAction & { glyph: string })[], selection: string, extra?: HTMLElement): void {
    if (!around || actions.length === 0) return this.hide();
    const key = `${selection}:${actions.map((a) => `${a.label}${a.disabled ? '-' : ''}`).join('|')}`;
    if (this.root.dataset.key !== key) {
      clear(this.root);
      for (const action of actions) {
        const node = button(action.glyph, () => action.run(), { class: `wb-mini-button${action.danger ? ' is-danger' : ''}`, title: action.shortcut ? `${action.label} (${action.shortcut})` : action.label, 'aria-label': action.label, disabled: action.disabled });
        this.root.appendChild(node);
      }
      this.root.dataset.key = key;
    }
    if (extra && extra.parentElement !== this.root) this.root.appendChild(extra);
    for (const other of [...this.root.children]) if (other !== extra && !other.classList.contains('wb-mini-button')) other.remove();
    this.root.hidden = false;
    const frame = this.host.getBoundingClientRect();
    const box = this.root.getBoundingClientRect();
    const above = around.top - frame.top - box.height - 8;
    const top = above >= 4 ? above : around.bottom - frame.top + 8;
    const left = Math.max(4, Math.min(around.left - frame.left + around.width / 2 - box.width / 2, frame.width - box.width - 4));
    this.root.style.left = `${left}px`;
    this.root.style.top = `${Math.min(top, frame.height - box.height - 4)}px`;
  }

  hide(): void {
    this.root.hidden = true;
  }

  get element(): HTMLElement {
    return this.root;
  }
}
