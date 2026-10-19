// The properties of the selected element: a form typed by the metamodel
// (texts, numbers, quantities, enumerations, references, lists), each field
// one `set` or `unset` request; the identity and the name have their own
// gestures (rename keeps the identity, remove deletes the declaration).
import { attributeDisplay, attributeLiteral, EditRequest, ElementRecord, Metamodel, MetamodelAttribute } from '../src/pure/workbench';
import { button, clear, el, inlinePrompt } from './dom';

export interface PropertiesHost {
  submit: (requests: EditRequest[], label: string) => void;
  reveal: (id: string) => void;
  ask: (element: ElementRecord) => void;
}

const HIDDEN = new Set(['id', 'name']);

export class Properties {
  private readonly root: HTMLElement;
  private element: ElementRecord | null = null;
  private metamodel: Metamodel | null = null;

  constructor(host: HTMLElement, private readonly owner: PropertiesHost) {
    this.root = el('div', { class: 'wb-properties' });
    host.appendChild(this.root);
    this.show(null, null);
  }

  show(element: ElementRecord | null, metamodel: Metamodel | null): void {
    this.element = element;
    this.metamodel = metamodel;
    clear(this.root);
    if (!element) {
      this.root.appendChild(el('p', { class: 'wb-muted', text: 'Select an element in a diagram to see and edit its properties.' }));
      return;
    }
    const kind = metamodel?.kinds.find((k) => k.name === element.kind);
    const head = el('header', { class: 'wb-props-head' }, [
      el('div', { class: 'wb-props-title' }, [el('strong', { text: element.name || element.id }), el('span', { class: 'wb-kind', text: element.kind })]),
      el('code', { class: 'wb-id', text: element.id }),
    ]);
    const actions = el('div', { class: 'wb-actions' }, [
      button('Go to source', () => this.owner.reveal(element.id)),
      button('Rename', () => this.rename()),
      button('Ask @arclang', () => this.owner.ask(element)),
      button('Delete', () => this.remove(), { class: 'wb-danger' }),
    ]);
    this.root.append(head, actions);
    if (kind?.doc) this.root.appendChild(el('p', { class: 'wb-muted', text: kind.doc }));

    const table = el('table', { class: 'wb-props' });
    const known = kind?.attributes.filter((a) => !HIDDEN.has(a.key)) ?? [];
    const seen = new Set<string>();
    for (const attribute of known) {
      seen.add(attribute.key);
      table.appendChild(this.row(attribute, element.attributes[attribute.key]));
    }
    for (const [key, value] of Object.entries(element.attributes)) {
      if (seen.has(key) || HIDDEN.has(key)) continue;
      table.appendChild(this.row({ key, type: { type: 'Text' } }, value));
    }
    this.root.appendChild(table);
  }

  private row(attribute: MetamodelAttribute, value: unknown): HTMLTableRowElement {
    const element = this.element!;
    const current = attributeDisplay(value);
    const type = attribute.type.type;
    const label = el('th', { text: attribute.key, title: attribute.doc ?? '' });
    let field: HTMLInputElement | HTMLSelectElement;
    const submit = (raw: string) => {
      if (raw.trim() === '') {
        if (value !== undefined && value !== null) this.owner.submit([{ op: 'unset', element: element.id, key: attribute.key }], `clear ${attribute.key}`);
        return;
      }
      if (raw === current) return;
      const literal = attributeLiteral(attribute.type, raw);
      this.owner.submit([{ op: 'set', element: element.id, key: attribute.key, value: literal }], `${attribute.key} = ${raw}`);
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
    return el('tr', {}, [label, el('td', {}, [field])]);
  }

  private enumValues(name: string | undefined): string[] {
    const enums = this.metamodel?.enums;
    if (!name || !Array.isArray(enums)) return [];
    const found = (enums as { name: string; values: string[] }[]).find((e) => e.name === name);
    return found?.values ?? [];
  }

  /** Rename the shown element (F2 on the canvas opens the same field). */
  rename(): void {
    const element = this.element;
    if (!element) return;
    void inlinePrompt(this.root, `Rename ${element.name || element.id}`, element.name, 'new name').then((name) => {
      if (name === null || name.trim() === '' || name === element.name) return;
      this.owner.submit([{ op: 'rename', element: element.id, name: name.trim() }], `rename to ${name.trim()}`);
    });
  }

  private remove(): void {
    const element = this.element;
    if (!element) return;
    void inlinePrompt(this.root, `Delete ${element.id}? Type its id to confirm`, '', element.id).then((typed) => {
      if (typed === null) return;
      if (typed.trim() !== element.id) return;
      this.owner.submit([{ op: 'remove', element: element.id }], `delete ${element.id}`);
    });
  }
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
