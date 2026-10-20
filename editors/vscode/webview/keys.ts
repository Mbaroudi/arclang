// The keyboard of the canvas: the keys VS Code resolves and sends back as
// commands, the page's own keys, and what tells VS Code that a field of
// the page has the keyboard. Split from main.ts, behaviour unchanged.
import type { CanvasCommand, EditRequest, ElementRecord, Snapshot } from '../src/pure/workbench';
import type { Editing } from './editing';
import type { ViewerApi } from './globals';
import type { Palette } from './palette';

export interface KeyDeps {
  post(message: unknown): void;
  editing: Editing;
  palette: Palette;
  tools: HTMLElement;
  toast: HTMLElement;
  multi(): ElementRecord[];
  selected(): ElementRecord | null;
  viewer(): ViewerApi | null;
  snapshot(): Snapshot | null;
  deleteMany(elements: ElementRecord[]): void;
  hideMany(elements: ElementRecord[]): void;
  hideFromDiagram(element: ElementRecord): void;
  setFocusMode(on: boolean): void;
  toggleMinimap(): void;
  stepLayer(step: number): void;
  openFind(): void;
  showToast(text: string, kind: 'info' | 'error', actions?: [string, () => void][], ms?: number): void;
  submit(requests: EditRequest[], label: string): void;
  /** ⇧⌘C outside VS Code (VS Code binds it itself): the diagram as an image on the clipboard. */
  copyImage(): void;
}

/** Inside VS Code (not the test page), whose keybindings run the canvas's commands. */
// The browser test page wears VS Code's body class for its themes; it binds no keys (__arclangHarness).
const inVsCode = (): boolean => !(window as { __arclangHarness?: boolean }).__arclangHarness && /\bvscode-(light|dark|high-contrast)/.test(document.body.className);

/**
 * The keys package.json binds for the workbench: VS Code resolves them on
 * the keyboard's layout and sends the command back ('key'); the page lets
 * them through instead of acting on them twice.
 */
function boundInVsCode(event: KeyboardEvent): boolean {
  const command = event.metaKey || event.ctrlKey;
  const key = event.key.toLowerCase();
  if (command && !event.altKey) return ['z', 'y', 'c', 'x', 'v', 'd', 'a', '=', '+', '-', '0'].includes(key) || event.code === 'Digit0';
  if (!event.altKey && (event.key === 'Delete' || event.key === 'Backspace')) return true;
  return event.shiftKey && !event.altKey && event.code === 'Digit1';
}

// A field of the page has the keyboard: VS Code's keys go to it (⌘C copies its text, ⌘Z undoes its typing).
const isField = (node: EventTarget | null): boolean => node instanceof HTMLInputElement || node instanceof HTMLSelectElement || node instanceof HTMLTextAreaElement || (node instanceof HTMLElement && node.isContentEditable);

export function installKeys(d: KeyDeps): { canvasCommand(command: CanvasCommand, text?: string): void; deleteOne(element: ElementRecord): void } {
  /** A command of the canvas, from a key VS Code resolved (or the page's own key outside VS Code). */
  function canvasCommand(command: CanvasCommand, text?: string): void {
    const selection = window.getSelection()?.toString() ?? '';
    const multi = d.multi();
    const selected = d.selected();
    switch (command) {
      case 'copy':
      case 'cut':
        // Text selected on the page (a property, a finding) is copied as text.
        if (selection) return d.post({ type: 'clipboard-text', text: selection });
        return d.editing.copy(command === 'cut');
      case 'paste':
        return d.editing.paste();
      case 'paste-text':
        return text ? d.editing.pasteText(text) : d.editing.paste();
      case 'duplicate':
        return d.editing.duplicate();
      case 'select-all':
        return d.editing.selectAll();
      case 'delete':
        if (multi.length > 1) return d.deleteMany(multi);
        if (selected) return deleteOne(selected);
        return;
      case 'zoom-in':
      case 'zoom-out':
        // The viewer's own buttons: the zoom it animates, around the middle of the frame.
        d.tools.querySelector<HTMLButtonElement>(command === 'zoom-in' ? '#arcviz-zoom-in' : '#arcviz-zoom-out')?.click();
        return;
      case 'fit':
        d.viewer()?.fit?.();
        return;
      default:
        return;
    }
  }

  /** Delete one element; one that owns others says so first. */
  function deleteOne(element: ElementRecord): void {
    const owned = (d.snapshot()?.elements?.elements ?? []).filter((e) => e.owner === element.uuid).length;
    if (owned > 0) {
      d.showToast(`${element.name || element.id} owns ${owned} element${owned > 1 ? 's' : ''}.`, 'info', [
        [`Delete with ${owned === 1 ? 'it' : `all ${owned}`}`, () => {
          d.toast.hidden = true;
          d.submit([{ op: 'remove', element: element.id }], `delete ${element.id} and what it owns`);
        }],
      ], 10000);
      return;
    }
    d.submit([{ op: 'remove', element: element.id }], `delete ${element.id}`);
  }

  let typingReported = false;
  const reportTyping = () => {
    const typing = isField(document.activeElement);
    if (typing === typingReported) return;
    typingReported = typing;
    d.post({ type: 'typing', on: typing });
  };
  document.addEventListener('focusin', reportTyping);
  document.addEventListener('focusout', () => setTimeout(reportTyping, 0));

  document.addEventListener('keydown', (event) => onKey(event));

  function onKey(event: KeyboardEvent): void {
    const typing = event.target instanceof HTMLInputElement || event.target instanceof HTMLSelectElement || event.target instanceof HTMLTextAreaElement;
    const multi = d.multi();
    const selected = d.selected();
    // Esc in a field is the field's (it puts the value back), not the canvas's.
    if (event.key === 'Escape' && !typing) {
      // Esc first cancels what is under way (a tool, a selection); then it leaves full screen.
      const busy = d.palette.isArmed() || !!selected || multi.length > 0;
      if (!busy && document.body.classList.contains('wb-focus')) d.setFocusMode(false);
      d.palette.disarm();
      d.toast.hidden = true;
      return;
    }
    if (typing) return;
    if (inVsCode() && boundInVsCode(event)) return;
    if (d.editing.key(event)) return;
    const plain = !event.metaKey && !event.ctrlKey && !event.altKey;
    const command = event.metaKey || event.ctrlKey;
    const lower = event.key.toLowerCase();
    let done = true;
    if (event.shiftKey && !event.metaKey && !event.ctrlKey && event.key === 'M') d.toggleMinimap();
    else if (plain && (event.key === '[' || event.key === ']')) d.stepLayer(event.key === ']' ? 1 : -1);
    else if (plain && event.key.length === 1 && d.palette.shortcut(event.key)) { /* the palette armed its tool */ }
    else if (command && event.shiftKey && lower === 'c') d.copyImage();
    else if (command && lower === 'f') d.openFind();
    else if (command && lower === 'h' && multi.length > 1) d.hideMany(multi);
    else if (command && lower === 'h' && selected) d.hideFromDiagram(selected);
    else if ((event.key === 'Delete' || event.key === 'Backspace') && multi.length > 1) d.deleteMany(multi);
    else if (event.key === 'F2' && selected) d.editing.rename(selected);
    else if ((event.key === 'Delete' || event.key === 'Backspace') && selected) deleteOne(selected);
    else done = false;
    if (done) event.preventDefault();
  }

  return { canvasCommand, deleteOne };
}
