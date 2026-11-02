// The impact set as a table: severity, element, layer, category, distance,
// and the "why" path whose elements are links (a click shows the element in
// the text and its diagram). Sorted by severity then distance; any column
// sorts. A row selects its node in the graph.
import { ImpactedItem, pathSegments, Severity } from '../../src/pure/impact';
import { el } from '../dom';

export interface TableHost {
  select: (id: string) => void;
  reveal: (id: string, open: boolean) => void;
  analyze: (id: string) => void;
}

type SortKey = 'severity' | 'name' | 'layer' | 'category' | 'distance';
export interface Sort {
  key: SortKey;
  descending: boolean;
}

const RANK: Record<Severity, number> = { blocker: 0, major: 1, minor: 2 };
const LAYER_ORDER = ['OA', 'SA', 'LA', 'PA', 'EPBS', 'Data', 'Transverse'];

export function sorted(items: ImpactedItem[], sort: Sort): ImpactedItem[] {
  const compare = (a: ImpactedItem, b: ImpactedItem): number => {
    switch (sort.key) {
      case 'severity':
        return RANK[a.severity] - RANK[b.severity] || a.distance - b.distance || a.id.localeCompare(b.id);
      case 'name':
        return (a.name || a.id).localeCompare(b.name || b.id);
      case 'layer':
        return LAYER_ORDER.indexOf(a.layer) - LAYER_ORDER.indexOf(b.layer) || a.id.localeCompare(b.id);
      case 'category':
        return a.category.localeCompare(b.category) || a.id.localeCompare(b.id);
      case 'distance':
        return a.distance - b.distance || RANK[a.severity] - RANK[b.severity] || a.id.localeCompare(b.id);
    }
  };
  const out = [...items].sort(compare);
  return sort.descending ? out.reverse() : out;
}

const COLUMNS: { key: SortKey | null; label: string; cls: string }[] = [
  { key: 'severity', label: 'Severity', cls: 'c-sev' },
  { key: 'name', label: 'Element', cls: 'c-el' },
  { key: 'layer', label: 'Layer', cls: 'c-layer' },
  { key: 'category', label: 'Category', cls: 'c-cat' },
  { key: 'distance', label: 'Dist.', cls: 'c-dist' },
  { key: null, label: 'Why', cls: 'c-why' },
  { key: null, label: '', cls: 'c-act' },
];

function pathCell(item: ImpactedItem, host: TableHost): HTMLElement {
  const cell = el('td', { class: 'c-why' });
  const reason = el('div', { class: 'ip-reason', text: item.removed ? `Removed by the change. ${item.reason}` : item.reason });
  const trail = el('div', { class: 'ip-path' });
  for (const segment of pathSegments(item)) {
    if (segment.element !== undefined) {
      const link = el('button', { type: 'button', class: 'ip-seg', text: segment.element, title: `Show ${segment.element} in its diagram` });
      link.addEventListener('click', (event) => {
        event.stopPropagation();
        host.reveal(segment.element!, false);
      });
      trail.appendChild(link);
    } else {
      trail.appendChild(el('span', { class: 'ip-rel', text: `› ${segment.relation} ›` }));
    }
  }
  cell.append(reason, trail);
  return cell;
}

export function renderTable(items: ImpactedItem[], sort: Sort, selected: string | null, host: TableHost, onSort: (sort: Sort) => void): HTMLElement {
  const head = el('tr');
  for (const column of COLUMNS) {
    const th = el('th', { scope: 'col', class: column.cls });
    if (column.key) {
      const key = column.key;
      const active = sort.key === key;
      const button = el('button', { type: 'button', class: 'ip-sort', 'aria-label': `Sort by ${column.label}`, text: `${column.label}${active ? (sort.descending ? ' ▾' : ' ▴') : ''}` });
      button.addEventListener('click', () => onSort({ key, descending: active ? !sort.descending : false }));
      th.appendChild(button);
      th.setAttribute('aria-sort', active ? (sort.descending ? 'descending' : 'ascending') : 'none');
    } else {
      th.textContent = column.label;
    }
    head.appendChild(th);
  }
  const body = el('tbody');
  for (const item of sorted(items, sort)) {
    const row = el('tr', { class: `ip-row sev-${item.severity}${item.id === selected ? ' is-selected' : ''}${item.removed ? ' is-removed' : ''}`, 'data-id': item.id, tabindex: '0' });
    const pill = el('span', { class: `ip-pill sev-${item.severity}`, text: item.severity });
    row.append(
      el('td', { class: 'c-sev' }, [pill]),
      el('td', { class: 'c-el' }, [el('div', { class: 'ip-name', text: item.name || item.id, title: item.id }), el('div', { class: 'ip-id', text: item.name && item.name !== item.id ? `${item.id} · ${item.kind}` : item.kind })]),
      el('td', { class: 'c-layer', text: item.layer }),
      el('td', { class: 'c-cat', text: item.category.replace('_', ' ') }),
      el('td', { class: 'c-dist', text: String(item.distance) }),
      pathCell(item, host),
    );
    const actions = el('td', { class: 'c-act' });
    const show = el('button', { type: 'button', class: 'ip-act', title: 'Show in the text and its diagram', 'aria-label': `Show ${item.id}`, text: '↗' });
    show.addEventListener('click', (event) => {
      event.stopPropagation();
      host.reveal(item.id, true);
    });
    const again = el('button', { type: 'button', class: 'ip-act', title: 'Analyze the impact from here', 'aria-label': `Analyze impact of ${item.id}`, text: '⚡' });
    again.addEventListener('click', (event) => {
      event.stopPropagation();
      host.analyze(item.id);
    });
    actions.append(show, again);
    row.appendChild(actions);
    row.addEventListener('click', () => host.select(item.id));
    row.addEventListener('keydown', (event) => {
      if (event.key === 'Enter') host.reveal(item.id, true);
      if (event.key === ' ') {
        event.preventDefault();
        host.select(item.id);
      }
    });
    body.appendChild(row);
  }
  const table = el('table', { class: 'ip-table' }, [el('thead', {}, [head]), body]);
  if (items.length === 0) return el('div', { class: 'ip-empty', text: 'Nothing reached with these filters.' });
  return el('div', { class: 'ip-table-wrap' }, [table]);
}
