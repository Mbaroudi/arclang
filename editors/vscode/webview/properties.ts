// The properties of the selected element, Capella style: a header with the
// name (edited in place) and the id, then tabs General / Description /
// Resources / All. Set attributes come first; the others are added on
// demand. Each field is one `set` or `unset` request; the identity never
// changes (rename keeps it, remove deletes the declaration).
import { attributeDisplay, attributeLiteral, EditRequest, ElementRecord, Metamodel, MetamodelAttribute } from '../src/pure/workbench';
import { button, clear, el, inlinePrompt } from './dom';

export interface PropertiesHost {
  submit: (requests: EditRequest[], label: string) => void;
  reveal: (id: string) => void;
  ask: (element: ElementRecord) => void;
  diagram?: (op: 'hide' | 'related' | 'new', element: ElementRecord) => void;
  /** The requirements of the model, for "Satisfies…". */
  requirements?: () => ElementRecord[];
  /** The two ends of a relationship (an exchange, a link, a transition), null for an element. */
  ends?: (element: ElementRecord) => { from: ElementRecord; to: ElementRecord } | null;
  /** Turn a relationship around. */
  reverse?: (element: ElementRecord) => void;
}

const HIDDEN = new Set(['id', 'name']);
const DESCRIPTION_KEYS = new Set(['description', 'rationale', 'summary', 'notes', 'comment']);
const RESOURCE_KEYS = new Set(['memory', 'ram', 'flash', 'storage', 'power', 'voltage', 'current', 'mass', 'weight', 'cost', 'bandwidth', 'frequency', 'throughput', 'cpu', 'cores']);
type Tab = 'general' | 'description' | 'resources' | 'all';
const TABS: [Tab, string][] = [
  ['general', 'General'],
  ['description', 'Description'],
  ['resources', 'Resources'],
  ['all', 'All'],
];

/** `safety_level` reads "Safety level". */
export function humanise(key: string): string {
  const words = key.replace(/_/g, ' ').trim();
  return words.charAt(0).toUpperCase() + words.slice(1);
}

export class Properties {
  private readonly root: HTMLElement;
  private element: ElementRecord | null = null;
  private metamodel: Metamodel | null = null;
  private tab: Tab = 'general';
  private adding = false;

  constructor(host: HTMLElement, private readonly owner: PropertiesHost) {
    this.root = el('div', { class: 'wb-properties' });
    host.appendChild(this.root);
    this.show(null, null);
  }

  show(element: ElementRecord | null, metamodel: Metamodel | null): void {
    if (element?.uuid !== this.element?.uuid) this.adding = false;
    this.element = element;
    this.metamodel = metamodel;
    this.render();
  }

  /** Rename the shown element (F2 on the canvas opens the same field). */
  rename(): void {
    const input = this.root.querySelector<HTMLInputElement>('.wb-name-input');
    if (input) {
      input.focus();
      input.select();
      return;
    }
    const element = this.element;
    if (!element) return;
    void inlinePrompt(this.root, `Rename ${element.name || element.id}`, element.name, 'new name').then((name) => {
      if (name === null || name.trim() === '' || name === element.name) return;
      this.owner.submit([{ op: 'rename', element: element.id, name: name.trim() }], `rename to ${name.trim()}`);
    });
  }

  private render(): void {
    clear(this.root);
    const element = this.element;
    if (!element) {
      this.root.appendChild(el('p', { class: 'wb-muted', text: 'Select an element in a diagram, in the tree or in the text to see and edit its properties.' }));
      return;
    }
    const kind = this.metamodel?.kinds.find((k) => k.name === element.kind);
    this.root.append(this.header(element, kind?.doc), this.tabBar(), this.body(element, kind));
  }

  private header(element: ElementRecord, doc: string | undefined): HTMLElement {
    const name = el('input', { type: 'text', class: 'wb-name-input', value: element.name || element.id, title: 'Name: Enter renames, the identity is kept', 'aria-label': 'Name' });
    const submitName = () => {
      const value = name.value.trim();
      if (!value || value === element.name) {
        name.value = element.name || element.id;
        return;
      }
      this.owner.submit([{ op: 'rename', element: element.id, name: value }], `rename to ${value}`);
    };
    name.addEventListener('change', submitName);
    name.addEventListener('keydown', (event) => {
      if (event.key === 'Enter') {
        event.preventDefault();
        name.blur();
      }
      if (event.key === 'Escape') {
        name.value = element.name || element.id;
        name.blur();
      }
    });
    const id = el('code', { class: 'wb-id', text: element.id, title: 'Identity (click to copy)' });
    id.addEventListener('click', () => void navigator.clipboard?.writeText(element.id));
    const actions = el('div', { class: 'wb-actions' }, [
      button('Source', () => this.owner.reveal(element.id), { title: 'Show the declaration in the text' }),
      button('Ask @arclang', () => this.owner.ask(element)),
      button('Delete', () => this.remove(), { class: 'wb-danger', title: 'Delete the declaration (Undo takes it back)' }),
    ]);
    const requirements = this.owner.requirements?.() ?? [];
    if (element.kind !== 'Requirement' && requirements.length) {
      actions.appendChild(button('Satisfies…', () => this.satisfies(element, requirements), { title: 'Trace it to a requirement it satisfies' }));
    }
    if (this.owner.diagram) {
      const diagram = this.owner.diagram;
      actions.append(
        button('Hide', () => diagram('hide', element), { title: 'Hide from this diagram (⌘H): out of a named diagram, or a new diagram without it' }),
        button('Show related', () => diagram('related', element), { title: 'Add what it is connected to' }),
        button('New diagram', () => diagram('new', element), { title: 'A new diagram with it and what it is connected to' }),
      );
    }
    const ends = this.owner.ends?.(element) ?? null;
    const linked = ends
      ? el('p', { class: 'wb-ends' }, [
          el('span', { text: endLabel(ends.from), title: ends.from.id }),
          el('span', { class: 'wb-muted', text: ' → ' }),
          el('span', { text: endLabel(ends.to), title: ends.to.id }),
          ' ',
          button('⇄ Reverse', () => this.owner.reverse?.(element), { class: 'wb-link', title: 'Swap the source and the target (one undo)' }),
        ])
      : null;
    if (ends) actions.querySelectorAll('button').forEach((b) => ['Satisfies…', 'Hide', 'Show related', 'New diagram'].includes(b.textContent ?? '') && b.remove());
    return el('header', { class: 'wb-props-head' }, [
      el('div', { class: 'wb-props-title' }, [name, el('span', { class: 'wb-kind', text: humanise(element.kind.replace(/([a-z])([A-Z])/g, '$1 $2')), title: doc ?? '' })]),
      id,
      linked,
      actions,
    ]);
  }

  private tabBar(): HTMLElement {
    const bar = el('div', { class: 'wb-props-tabs', role: 'tablist' });
    for (const [tab, label] of TABS) {
      const node = button(label, () => {
        this.tab = tab;
        this.render();
      }, { class: `wb-props-tab${tab === this.tab ? ' is-active' : ''}`, role: 'tab', 'aria-selected': String(tab === this.tab) });
      bar.appendChild(node);
    }
    return bar;
  }

  private body(element: ElementRecord, kind: { attributes: MetamodelAttribute[] } | undefined): HTMLElement {
    const typed = new Map((kind?.attributes ?? []).map((a) => [a.key, a]));
    const keys = new Set<string>([...typed.keys(), ...Object.keys(element.attributes)]);
    const inTab = (key: string): boolean => {
      if (HIDDEN.has(key)) return false;
      const isDescription = DESCRIPTION_KEYS.has(key);
      const isResource = RESOURCE_KEYS.has(key) || typed.get(key)?.type.type === 'Quantity';
      switch (this.tab) {
        case 'description':
          return isDescription;
        case 'resources':
          return isResource;
        case 'general':
          return !isDescription && !isResource;
        default:
          return true;
      }
    };
    const candidates = [...keys].filter(inTab);
    const isSet = (key: string) => element.attributes[key] !== undefined && element.attributes[key] !== null;
    const set = candidates.filter(isSet);
    const unset = candidates.filter((key) => !isSet(key));
    const table = el('table', { class: 'wb-props' });
    for (const key of set) table.appendChild(this.row(typed.get(key) ?? { key, type: { type: 'Text' } }, element.attributes[key]));
    const pane = el('div', { class: 'wb-props-body' });
    if (set.length === 0 && !this.adding) pane.appendChild(el('p', { class: 'wb-muted', text: this.tab === 'all' ? 'No attribute set.' : `No ${this.tab} attribute set.` }));
    pane.appendChild(table);
    if (unset.length > 0) {
      if (this.adding) {
        const picker = el('select', { class: 'wb-field', 'aria-label': 'Attribute to add' });
        picker.appendChild(el('option', { value: '', text: 'Choose an attribute…' }));
        for (const key of unset) {
          const attribute = typed.get(key);
          picker.appendChild(el('option', { value: key, text: `${humanise(key)}${attribute?.type.of ? ` (${attribute.type.of})` : ''}`, title: attribute?.doc ?? '' }));
        }
        picker.addEventListener('change', () => {
          const key = picker.value;
          if (!key) return;
          this.adding = false;
          table.appendChild(this.row(typed.get(key) ?? { key, type: { type: 'Text' } }, undefined, true));
          picker.closest('.wb-add')?.remove();
        });
        pane.appendChild(
          el('div', { class: 'wb-add' }, [
            picker,
            button('Cancel', () => {
              this.adding = false;
              this.render();
            }),
          ]),
        );
        setTimeout(() => picker.focus(), 0);
      } else {
        pane.appendChild(
          button(`+ Add property (${unset.length})`, () => {
            this.adding = true;
            this.render();
          }, { class: 'wb-link' }),
        );
      }
    }
    return pane;
  }

  private row(attribute: MetamodelAttribute, value: unknown, focus = false): HTMLTableRowElement {
    const element = this.element!;
    const current = attributeDisplay(value);
    const type = attribute.type.type;
    const label = el('th', { text: humanise(attribute.key), title: `${attribute.key}${attribute.doc ? `: ${attribute.doc}` : ''}` });
    let field: HTMLInputElement | HTMLSelectElement | HTMLTextAreaElement;
    const submit = (raw: string) => {
      if (raw.trim() === '') {
        if (value !== undefined && value !== null) this.owner.submit([{ op: 'unset', element: element.id, key: attribute.key }], `clear ${humanise(attribute.key)}`);
        return;
      }
      if (raw === current) return;
      const literal = attributeLiteral(attribute.type, raw);
      this.owner.submit([{ op: 'set', element: element.id, key: attribute.key, value: literal }], `${humanise(attribute.key)} = ${raw}`);
    };
    if (type === 'Enum') {
      const values = this.enumValues(attribute.type.of);
      const select = el('select', { class: 'wb-field' });
      select.appendChild(el('option', { value: '', text: '' }));
      for (const option of values) select.appendChild(el('option', { value: option, text: option, selected: option === current }));
      if (current && !values.includes(current)) select.appendChild(el('option', { value: current, text: current, selected: true }));
      select.addEventListener('change', () => submit(select.value));
      field = select;
    } else if (type === 'Boolean') {
      const select = el('select', { class: 'wb-field' });
      for (const option of ['', 'true', 'false']) select.appendChild(el('option', { value: option, text: option, selected: option === current }));
      select.addEventListener('change', () => submit(select.value));
      field = select;
    } else if (DESCRIPTION_KEYS.has(attribute.key)) {
      const area = el('textarea', { class: 'wb-field wb-textarea', rows: '3', placeholder: 'text' });
      area.value = current;
      area.addEventListener('change', () => submit(area.value));
      area.addEventListener('keydown', (event) => {
        if (event.key === 'Enter' && (event.metaKey || event.ctrlKey)) area.blur();
        if (event.key === 'Escape') area.value = current;
      });
      field = area;
    } else {
      const input = el('input', { type: 'text', class: 'wb-field', value: current, placeholder: placeholderFor(attribute) });
      // Enter leaves the field; the change event then submits once.
      input.addEventListener('change', () => submit(input.value));
      input.addEventListener('keydown', (event) => {
        if (event.key === 'Enter') {
          event.preventDefault();
          input.blur();
        }
        if (event.key === 'Escape') input.value = current;
      });
      field = input;
    }
    if (focus) setTimeout(() => field.focus(), 0);
    return el('tr', {}, [label, el('td', {}, [field])]);
  }

  private enumValues(name: string | undefined): string[] {
    const enums = this.metamodel?.enums;
    if (!name || !Array.isArray(enums)) return [];
    const found = (enums as { name: string; values: string[] }[]).find((e) => e.name === name);
    return found?.values ?? [];
  }

  private satisfies(element: ElementRecord, requirements: ElementRecord[]): void {
    const picker = el('select', { class: 'wb-field', 'aria-label': 'Requirement satisfied' });
    picker.appendChild(el('option', { value: '', text: 'Requirement it satisfies…' }));
    for (const requirement of requirements) {
      const text = String(requirement.attributes.description ?? requirement.attributes.title ?? requirement.name);
      picker.appendChild(el('option', { value: requirement.id, text: `${requirement.id}: ${text.slice(0, 60)}` }));
    }
    picker.addEventListener('change', () => {
      if (!picker.value) return;
      this.owner.submit([{ op: 'add_trace', from: element.id, kind: 'satisfies', to: picker.value }], `${element.id} satisfies ${picker.value}`);
    });
    this.root.querySelector('.wb-actions')?.after(picker);
    picker.focus();
  }

  /** Several elements: one attribute set on all of them at once. */
  showMany(elements: ElementRecord[], metamodel: Metamodel | null): void {
    this.element = null;
    this.metamodel = metamodel;
    clear(this.root);
    this.root.appendChild(el('p', { class: 'wb-props-title', text: `${elements.length} elements selected` }));
    const keys = new Map<string, MetamodelAttribute>();
    const kinds = new Set(elements.map((e) => e.kind));
    for (const kind of kinds) {
      for (const attribute of metamodel?.kinds.find((k) => k.name === kind)?.attributes ?? []) {
        if (!HIDDEN.has(attribute.key) && [...kinds].every((other) => metamodel?.kinds.find((k) => k.name === other)?.attributes.some((a) => a.key === attribute.key))) keys.set(attribute.key, attribute);
      }
    }
    if (keys.size === 0) {
      this.root.appendChild(el('p', { class: 'wb-muted', text: 'These elements share no attribute.' }));
      return;
    }
    const key = el('select', { class: 'wb-field', 'aria-label': 'Attribute' });
    for (const [name] of keys) key.appendChild(el('option', { value: name, text: humanise(name) }));
    const value = el('input', { type: 'text', class: 'wb-field', placeholder: 'value for all of them' });
    const apply = button('Set on all', () => {
      const attribute = keys.get(key.value);
      if (!attribute || !value.value.trim()) return;
      const literal = attributeLiteral(attribute.type, value.value);
      this.owner.submit(elements.map((e) => ({ op: 'set', element: e.id, key: attribute.key, value: literal })), `${humanise(attribute.key)} = ${value.value} on ${elements.length} elements`);
    }, { class: 'wb-primary' });
    this.root.append(el('table', { class: 'wb-props' }, [el('tr', {}, [el('th', { text: 'Attribute' }), el('td', {}, [key])]), el('tr', {}, [el('th', { text: 'Value' }), el('td', {}, [value])])]), apply);
  }

  private remove(): void {
    const element = this.element;
    if (!element) return;
    this.owner.submit([{ op: 'remove', element: element.id }], `delete ${element.id}`);
  }
}

/** An end as the reader knows it: a port with its owner (`LC-CAM.lane`), anything else by name. */
function endLabel(end: ElementRecord): string {
  return /Port$/.test(end.kind) ? end.id : end.name || end.id;
}

function placeholderFor(attribute: MetamodelAttribute): string {
  switch (attribute.type.type) {
    case 'Quantity':
      return 'value and unit, e.g. 20 ms';
    case 'Number':
      return 'number';
    case 'Reference':
      return attribute.type.of ? `id of a ${attribute.type.of}` : 'id';
    case 'List':
      return 'a, b, c';
    case 'Multiplicity':
      return '1, 0..1, 0..*';
    default:
      return '';
  }
}
