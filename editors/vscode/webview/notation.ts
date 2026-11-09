// The notation the model is read in: Arcadia (Capella's viewpoints), SysML v2
// (its views) or CESAM (its grid, then the diagrams of each vision). One
// model, three readings: the switch above the layers changes what the
// workbench shows, never the model. Each notation shows only its own tabs;
// CESAM opens on its grid, drawn from what the compiler computes, and each
// cell opens the diagrams that draw that view.
import type { CesamSnapshot, Notation, Snapshot, ViewerGraph } from '../src/pure/workbench';
import { CESAM_VISIONS, NOTATIONS, NOTATION_VIEWS, VIEWPOINTS, defaultNotation, graphsOfNotation } from '../src/pure/workbench';
import { button, clear, el } from './dom';

export interface NotationDeps {
  /** The bar of the layers: the switch goes before it. */
  layerBar: HTMLElement;
  /** The sheet: the CESAM grid covers the drawing in it. */
  sheet: HTMLElement;
  snapshot: () => Snapshot | null;
  /** The diagram on screen. */
  current: () => string | null;
  show: (id: string) => void;
  /** Every diagram of the model, by layer (the command). */
  more: () => void;
  /** The notation chosen for a file, kept by the page. */
  saved: () => Record<string, Notation>;
  save: (all: Record<string, Notation>) => void;
}

export interface NotationApi {
  /** Draw the switch and the layers again (a new snapshot, another view on screen). */
  render: (file: string | undefined) => void;
  notation: () => Notation;
}

const ASPECTS: { key: string; label: string }[] = [
  { key: 'requirements', label: 'Expected properties' },
  { key: 'states', label: 'States' },
  { key: 'structure', label: 'Static elements' },
  { key: 'interactions', label: 'Interactions' },
  { key: 'scenarios', label: 'Scenarios' },
  { key: 'interfaces', label: 'Interfaces (flows)' },
];

export function installNotation(d: NotationDeps): NotationApi {
  const switcher = el('div', { class: 'wb-notation', role: 'radiogroup', 'aria-label': 'Notation' });
  d.layerBar.before(switcher);
  const board = el('section', { class: 'wb-cesam', hidden: true, 'aria-label': 'CESAM grid' });
  const caption = el('div', { class: 'wb-cesam-caption', hidden: true });
  d.sheet.append(board, caption);
  let file: string | undefined;
  let chosen: Notation | null = null;
  let gridOpen = false;

  const notation = (): Notation => chosen ?? d.saved()[file ?? ''] ?? defaultNotation(d.snapshot(), file);
  const graphs = (): ViewerGraph[] => d.snapshot()?.diagrams?.graphs ?? [];
  const cesam = (): CesamSnapshot | undefined => d.snapshot()?.cesam;

  function choose(next: Notation): void {
    if (next === notation()) return;
    chosen = next;
    d.save({ ...d.saved(), [file ?? '']: next });
    const own = graphsOfNotation(next, graphs(), cesam());
    gridOpen = next === 'cesam';
    // The diagram on screen stays when the notation has it; else its first one.
    if (!gridOpen && !own.some((g) => g.id === d.current()) && own[0]) d.show(own[0].id);
    render(file);
  }

  function open(id: string): void {
    gridOpen = false;
    d.show(id);
    render(file);
  }

  function renderSwitch(): void {
    clear(switcher);
    for (const n of NOTATIONS) {
      const on = n.id === notation();
      const item = button(n.label, () => choose(n.id), { class: `wb-notation-item${on ? ' is-active' : ''}`, role: 'radio', 'aria-checked': String(on), title: n.title });
      if (n.id === 'cesam' && !cesam()) item.disabled = true;
      switcher.appendChild(item);
    }
  }

  function renderLayers(): void {
    clear(d.layerBar);
    const current = d.current();
    const n = notation();
    if (n === 'cesam') {
      const tab = (label: string, active: boolean, onClick: () => void, title: string, count?: number) => {
        const item = button('', onClick, { class: `wb-layer${active ? ' is-active' : ''}`, 'aria-pressed': String(active), title });
        item.append(el('span', { class: 'wb-layer-name', text: label }));
        if (count && count > 1) item.appendChild(el('span', { class: 'wb-layer-count', text: String(count) }));
        d.layerBar.appendChild(item);
      };
      tab('Grid', gridOpen, () => { gridOpen = true; render(file); }, 'The CESAM grid: three visions by six aspects, computed by the compiler');
      for (const vision of CESAM_VISIONS) {
        const own = (cesam()?.diagrams ?? []).filter((c) => c.vision === vision.id && graphs().some((g) => g.id === c.id));
        if (!own.length) continue;
        tab(vision.short, !gridOpen && own.some((c) => c.id === current), () => open(own[0].id), `${vision.label}: ${own.map((c) => c.view).join(', ')}`, own.length);
      }
      return;
    }
    const own = graphsOfNotation(n, graphs(), cesam());
    // In the notation's own order (SysML v2: the general view first).
    const ordered = NOTATION_VIEWS[n].map((view) => VIEWPOINTS.find((v) => v.view === view)).filter((v): v is (typeof VIEWPOINTS)[number] => !!v);
    for (const viewpoint of ordered) {
      const diagrams = own.filter((g) => g.arc.view === viewpoint.view);
      if (!diagrams.length) continue;
      const active = diagrams.some((g) => g.id === current);
      const item = button('', () => d.show(diagrams[0].id), { class: `wb-layer${active ? ' is-active' : ''}`, 'aria-pressed': String(active), title: `${viewpoint.label}${diagrams.length > 1 ? ` (${diagrams.length} diagrams)` : ''}` });
      item.append(el('span', { class: 'wb-layer-name', text: viewpoint.short }));
      if (diagrams.length > 1) item.appendChild(el('span', { class: 'wb-layer-count', text: String(diagrams.length) }));
      d.layerBar.appendChild(item);
    }
    d.layerBar.appendChild(button('Diagrams…', d.more, { class: 'wb-layer wb-layer-more', title: 'Open a diagram by layer, here or in a tab of its own' }));
  }

  function renderBoard(): void {
    const grid = cesam();
    board.hidden = !(notation() === 'cesam' && gridOpen && grid);
    d.sheet.classList.toggle('is-cesam-grid', !board.hidden);
    // On the grid, nothing of the drawing's tools: the palette, the zoom and the minimap would act on what is hidden.
    document.body.classList.toggle('wb-on-cesam-grid', !board.hidden);
    if (board.hidden || !grid) return;
    clear(board);
    const rows = grid.table.rows;
    const ok = rows.filter((r) => r.status === 'ok').length;
    board.append(
      el('header', { class: 'wb-cesam-head' }, [
        el('p', { class: 'wb-cesam-eyebrow', text: 'CESAM grid · computed by the compiler' }),
        el('h2', { text: 'One model, read through the three visions of CESAM' }),
        el('p', { class: 'wb-cesam-lede', text: `${ok} of ${rows.length} visions complete. Each cell counts the elements that fill the view, by Arcadia level; click a diagram to open it.${grid.declared ? '' : ' This model does not declare method: "cesam": the grid reads it all the same.'}` }),
      ]),
    );
    const table = el('div', { class: 'wb-cesam-grid', role: 'table' });
    table.appendChild(el('div', { class: 'wb-cesam-corner', role: 'columnheader', text: 'Vision' }));
    for (const aspect of ASPECTS) table.appendChild(el('div', { class: 'wb-cesam-aspect', role: 'columnheader', text: aspect.label }));
    for (const vision of CESAM_VISIONS) {
      const row = rows.find((r) => r.id === vision.id);
      table.appendChild(el('div', { class: `wb-cesam-vision is-${vision.id}`, role: 'rowheader' }, [el('strong', { text: vision.label }), el('span', { text: vision.question })]));
      for (const aspect of ASPECTS) {
        const cell = row?.cells[aspect.key];
        const status = cell?.status ?? 'gap';
        const match = /^(\d+)(?:\s*\(([^)]*)\))?/.exec(cell?.text ?? '');
        const names = (cell?.links ?? []).map((l) => l.name);
        const diagrams = grid.diagrams.filter((c) => c.vision === vision.id && c.aspect === aspect.key && graphs().some((g) => g.id === c.id));
        const box = el('div', { class: `wb-cesam-cell is-${vision.id} is-${status}`, role: 'cell' }, [
          el('div', { class: 'wb-cesam-count' }, [el('span', { text: match ? match[1] : '0' }), el('span', { class: `wb-cesam-badge is-${status}`, text: status === 'ok' ? 'OK' : status === 'warn' ? 'CHECK' : 'EMPTY' })]),
          match?.[2] ? el('div', { class: 'wb-cesam-levels', text: match[2] }) : null,
          names.length ? el('div', { class: 'wb-cesam-names', text: names.slice(0, 3).join(', ') }, names.length > 3 ? [el('span', { class: 'wb-cesam-more', text: ` +${names.length - 3}` })] : []) : el('div', { class: 'wb-cesam-names is-empty', text: 'Nothing modelled yet' }),
        ]);
        for (const c of diagrams) {
          const graph = graphs().find((g) => g.id === c.id);
          box.appendChild(button(graph?.arc.title ?? c.view, () => open(c.id), { class: 'wb-cesam-open', title: `Open ${c.view}: ${graph?.arc.title ?? c.id}` }));
        }
        table.appendChild(box);
      }
    }
    board.appendChild(el('div', { class: 'wb-cesam-scroll' }, [table]));
    board.appendChild(el('p', { class: 'wb-cesam-foot', text: 'OA operational · SA system · LA logical · PA physical. The same grid as a table: "ArcLang: Open a table of the model", kind CESAM grid.' }));
  }

  function renderCaption(): void {
    const c = notation() === 'cesam' && !gridOpen ? cesam()?.diagrams.find((x) => x.id === d.current()) : undefined;
    caption.hidden = !c;
    if (!c) return;
    const vision = CESAM_VISIONS.find((v) => v.id === c.vision);
    caption.textContent = `${c.view} · ${vision?.label ?? c.vision}`;
  }

  function render(next: string | undefined): void {
    if (next !== file) {
      file = next;
      chosen = null;
      gridOpen = notation() === 'cesam';
    }
    // A notation whose diagrams are not drawn yet (a new snapshot): the grid, or the first of its own.
    if (notation() === 'cesam' && !cesam()) chosen = 'arcadia';
    const own = graphsOfNotation(notation(), graphs(), cesam());
    if (notation() !== 'cesam' && own.length && !own.some((g) => g.id === d.current())) d.show(own[0].id);
    renderSwitch();
    renderLayers();
    renderBoard();
    renderCaption();
  }

  return { render, notation };
}
