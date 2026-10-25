// The model's file, at the head of the tool row: Save (with ● while the
// model has unsaved changes, "Saved" when it has none or when VS Code saves
// on its own), and File ▾: Save, Save as SysML v2, Capella, ReqIF or a copy
// as ArcLang, Import a model. A Capella, SysML v2 or ReqIF file dropped on
// the sheet (Shift held, as VS Code wants for a drop on a page) is offered
// for import. What each does is the host's.
import { droppedModelFiles } from '../src/pure/importing';
import type { FromWebview, ToWebview } from '../src/pure/workbench';
import { el } from './dom';
import { ContextMenu, MenuItem } from './menu';

const SVG = 'http://www.w3.org/2000/svg';

/** 16 × 16 outlines in the toolbar's style. */
const PATHS: Record<string, string[]> = {
  save: ['M3 2.5h8l2.5 2.5v8.5h-11z', 'M5.5 2.5v3.5h5v-3.5', 'M5 13.5v-4h6v4'],
  file: ['M4 1.5h5.5l3 3v10h-8.5z', 'M9.5 1.5v3h3'],
};

function glyph(name: string): SVGSVGElement {
  const svg = document.createElementNS(SVG, 'svg');
  for (const [key, value] of Object.entries({ viewBox: '0 0 16 16', width: '16', height: '16', fill: 'none', stroke: 'currentColor', 'stroke-width': '1.5', 'stroke-linecap': 'round', 'stroke-linejoin': 'round', 'aria-hidden': 'true', focusable: 'false', class: `arcviz-icon wb-file-icon-${name}` })) svg.setAttribute(key, value);
  for (const d of PATHS[name] ?? []) {
    const path = document.createElementNS(SVG, 'path');
    path.setAttribute('d', d);
    svg.appendChild(path);
  }
  return svg;
}

export interface FileMenuDeps {
  slot: HTMLElement;
  sheet: HTMLElement;
  post: (message: FromWebview) => void;
  toast: (text: string, kind: 'info' | 'error') => void;
}

export function installFileMenu(d: FileMenuDeps): void {
  const mac = window.ArcVizToolbar.isMac();
  const saveKeys = mac ? '⌘S' : 'Ctrl+S';
  const label = el('span', { class: 'wb-file-label', text: 'Saved' });
  const dot = el('span', { class: 'wb-dirty-dot', 'aria-hidden': 'true', text: '●', hidden: true });
  const save = el('button', { type: 'button', class: 'wb-icon-button wb-bar-toggle wb-save', id: 'wb-save', 'aria-label': 'Saved', title: 'No unsaved change', disabled: true }, [glyph('save'), label, dot]);
  const fileButton = el('button', { type: 'button', class: 'wb-icon-button wb-bar-toggle wb-file-button', id: 'wb-file-menu', 'aria-haspopup': 'menu', 'aria-label': 'File: save as, import', title: 'File: save as another format, import a model' }, [glyph('file'), el('span', { class: 'wb-toggle-label', text: 'File' })]);
  d.slot.append(save, fileButton);
  const menu = new ContextMenu(d.sheet);

  let state: { dirty: boolean; autoSave: boolean; language: 'arclang' | 'sysml' } = { dirty: false, autoSave: false, language: 'arclang' };
  const sysml = () => state.language === 'sysml';
  function render(): void {
    const unsaved = state.dirty && !state.autoSave;
    label.textContent = unsaved ? 'Save' : 'Saved';
    dot.hidden = !unsaved;
    save.disabled = !unsaved;
    save.classList.toggle('is-dirty', unsaved);
    const what = sysml() ? 'the SysML v2 file' : 'the model';
    save.setAttribute('aria-label', unsaved ? `Save ${what} (unsaved changes)` : state.autoSave ? 'Saved (auto save is on)' : 'Saved');
    save.title = unsaved ? `Save ${what} (${saveKeys})${sysml() ? ': written back as SysML v2' : ''}` : state.autoSave ? 'Auto save is on (files.autoSave): every change is saved' : 'No unsaved change';
    document.body.classList.toggle('wb-unsaved', unsaved);
  }
  render();
  save.addEventListener('click', () => d.post({ type: 'model-file', op: 'save' }));

  const saveAs = (format: string) => () => d.post({ type: 'model-file', op: 'save-as', format });
  fileButton.addEventListener('click', () => {
    if (menu.isOpen) return menu.close();
    const box = fileButton.getBoundingClientRect();
    const items: MenuItem[] = [
      { label: state.dirty && !state.autoSave ? `Save${sysml() ? ' (as SysML v2)' : ''}` : 'Save (nothing to save)', shortcut: saveKeys, disabled: !state.dirty || state.autoSave, run: () => d.post({ type: 'model-file', op: 'save' }) },
      'separator',
      { label: 'Save as SysML v2…', run: saveAs('sysml') },
      { label: 'Save as Capella…', run: saveAs('capella') },
      { label: 'Save as ReqIF (requirements)…', run: saveAs('reqif') },
      { label: 'Save a copy as ArcLang…', run: saveAs('arc') },
      'separator',
      { label: 'Import a model (Capella, SysML v2, ReqIF)…', run: () => d.post({ type: 'model-file', op: 'import' }) },
    ];
    menu.open(box.left, box.bottom + 2, items);
  });

  window.addEventListener('message', (event: MessageEvent<ToWebview>) => {
    const message = event.data;
    if (message?.type !== 'file-state') return;
    state = { dirty: message.dirty, autoSave: message.autoSave, language: message.language };
    render();
  });

  // A model file dropped on the sheet: offered for import (the tree's nodes are the panes' drop).
  d.sheet.addEventListener('drop', (event) => {
    const data = event.dataTransfer;
    if (!data) return;
    const files = droppedModelFiles([...new Set(['text/uri-list', 'resourceurls', 'codeeditors', 'text/plain', ...data.types])].map((type) => data.getData(type)));
    if (files.length === 0) return;
    event.preventDefault();
    event.stopImmediatePropagation();
    d.sheet.classList.remove('wb-drop-over');
    d.toast(`Import ${files.length > 1 ? `${files.length} files` : decodeURIComponent(files[0].split('/').pop() ?? '')}?`, 'info');
    d.post({ type: 'import-drop', uris: files });
  }, true);
}
