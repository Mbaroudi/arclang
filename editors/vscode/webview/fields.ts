// The controls of the properties pane, one per kind of value: a quantity
// with its unit, an enumeration, a switch, a reference picker with search
// and a jump to the element, a list of chips, a text, a description with
// its preview. Each says what is wrong as the reader types (the page's own
// check, then the compiler's, nothing written), and commits on Enter or on
// leaving the field: one edit the compiler checks, one undo.
import { checkValue, CheckContext, enumValues, listItems, markdownBlocks, quantityOf, referenceCandidates, resolveReference, unitsOf, valueShape } from '../src/pure/properties';
import type { ElementRecord, MetamodelAttribute } from '../src/pure/workbench';
import { button, el } from './dom';
import { iconNode } from './icons';

export interface FieldHost extends CheckContext {
  element: ElementRecord | null;
  attribute: MetamodelAttribute;
  value: unknown;
  /** Write the value (null clears it); the pane builds the request. */
  commit: (raw: string | string[] | null) => void;
  /** The compiler's answer on the value, nothing written; null when it cannot be asked. */
  ask: (raw: string | string[]) => Promise<{ ok: boolean; error?: string } | null>;
  /** Step to an element (a reference's jump link). */
  jump: (element: ElementRecord) => void;
  /** A refusal of the compiler kept on the field. */
  error?: string;
  /** The text the reader typed before a refusal, shown again. */
  draft?: string;
  /** Placeholder for several elements whose values differ. */
  mixed?: boolean;
}

export interface Field {
  node: HTMLElement;
  focus: () => void;
}

const CHECK_DELAY = 350;

/** The message line under a control and the state of its row. */
function feedback(row: () => HTMLElement | null, message: HTMLElement) {
  return (state: 'ok' | 'checking' | 'invalid', text = '') => {
    const r = row();
    r?.classList.toggle('is-invalid', state === 'invalid');
    r?.classList.toggle('is-checking', state === 'checking');
    message.textContent = state === 'invalid' ? text : state === 'checking' ? 'checking with the compiler…' : '';
    message.hidden = state === 'ok';
  };
}

/** Live checking of what a control holds: the page's own check at once, the compiler's after a pause. */
function liveCheck(host: FieldHost, control: HTMLElement, message: HTMLElement) {
  const show = feedback(() => control.closest('tr, .wb-desc-block, .wb-trace-add'), message);
  let timer: ReturnType<typeof setTimeout> | null = null;
  let generation = 0;
  let invalid: string | null = host.error ?? null;
  const run = (raw: string | string[]): string | null => {
    if (timer) clearTimeout(timer);
    const empty = Array.isArray(raw) ? raw.length === 0 : raw.trim() === '';
    const local = empty ? null : checkValue(host.attribute, raw, host);
    invalid = local;
    if (local) {
      show('invalid', local);
      return local;
    }
    show('ok');
    if (empty) return null;
    const mine = ++generation;
    timer = setTimeout(() => {
      show('checking');
      void host.ask(raw).then((answer) => {
        if (mine !== generation) return;
        if (answer && !answer.ok) {
          invalid = answer.error ?? 'refused';
          show('invalid', invalid);
        } else {
          invalid = null;
          show('ok');
        }
      });
    }, CHECK_DELAY);
    return null;
  };
  if (host.error) setTimeout(() => show('invalid', host.error), 0);
  return {
    run,
    /** Commit unless the page already knows the value is wrong (the compiler's own refusal is shown when it answers). */
    commit: (raw: string | string[] | null) => {
      if (raw !== null && run(raw)) return;
      generation++;
      if (timer) clearTimeout(timer);
      host.commit(raw);
    },
    invalid: () => invalid,
  };
}

function messageNode(): HTMLElement {
  return el('div', { class: 'wb-field-msg', role: 'alert', hidden: true });
}

/** Enter commits, Escape puts the value back. */
function keys(input: HTMLInputElement | HTMLTextAreaElement, original: string, commit: () => void, multiline = false): void {
  let reverted = false;
  input.addEventListener('keydown', (event) => {
    const ke = event as KeyboardEvent;
    if (ke.key === 'Enter' && (!multiline || ke.metaKey || ke.ctrlKey)) {
      event.preventDefault();
      input.blur();
    }
    if (ke.key === 'Escape') {
      reverted = true;
      input.value = original;
      input.dispatchEvent(new Event('input'));
      input.blur();
      ke.stopPropagation();
    }
  });
  input.addEventListener('change', () => {
    if (reverted) {
      reverted = false;
      return;
    }
    commit();
  });
}

export function createField(host: FieldHost): Field {
  const shape = valueShape(host.attribute.type);
  if (shape.list && shape.item === 'Reference') return listField(host, true);
  if (shape.list) return listField(host, false);
  switch (shape.item) {
    case 'Quantity':
      return quantityField(host);
    case 'Enum':
    case 'Enumeration':
      return enumField(host);
    case 'Boolean':
      return booleanField(host);
    case 'Reference':
      return referenceField(host);
    default:
      return textField(host);
  }
}

function textField(host: FieldHost): Field {
  const current = host.value === undefined || host.value === null ? '' : String(host.value);
  const input = el('input', { type: 'text', class: 'wb-field', value: host.draft ?? current, placeholder: host.mixed ? 'several values' : placeholderOf(host), 'aria-label': host.attribute.key });
  const message = messageNode();
  const check = liveCheck(host, input, message);
  input.addEventListener('input', () => check.run(input.value));
  keys(input, current, () => {
    const value = input.value;
    if (value === current && !host.mixed) return;
    check.commit(value.trim() === '' ? null : value);
  });
  return { node: el('div', { class: 'wb-control' }, [input, message]), focus: () => input.focus() };
}

function placeholderOf(host: FieldHost): string {
  switch (valueShape(host.attribute.type).item) {
    case 'Number':
      return 'number';
    case 'Multiplicity':
      return '1, 0..1, 1..*';
    default:
      return '—';
  }
}

function quantityField(host: FieldHost): Field {
  const shape = valueShape(host.attribute.type);
  const quantity = quantityOf(host.value);
  const units = unitsOf(host.metamodel, shape.of);
  const draft = host.draft ? quantityOf(host.draft) : null;
  const number = el('input', { type: 'text', inputmode: 'decimal', class: 'wb-field wb-quantity-value', value: draft?.value ?? (quantity ? String(Number(quantity.value)) : ''), placeholder: host.mixed ? 'several' : '—', 'aria-label': `${host.attribute.key} value` });
  const unit = el('select', { class: 'wb-field wb-unit', 'aria-label': `${host.attribute.key} unit`, title: `Unit (${shape.of ?? 'quantity'})` });
  const preferred = draft?.unit || quantity?.unit || preferredUnit(units);
  for (const u of units) unit.appendChild(el('option', { value: u.symbol, text: u.symbol, selected: u.symbol === preferred }));
  if (preferred && !units.some((u) => u.symbol === preferred)) unit.appendChild(el('option', { value: preferred, text: preferred, selected: true }));
  const message = messageNode();
  const control = el('div', { class: 'wb-control wb-quantity' }, [el('div', { class: 'wb-quantity-row' }, [number, unit]), message]);
  const check = liveCheck(host, control, message);
  // A unit typed with the number (`5 kg`) wins over the picker: it is what the reader wrote.
  const raw = (): string => {
    const typed = quantityOf(number.value);
    if (!number.value.trim()) return '';
    return typed && typed.unit ? `${typed.value} ${typed.unit}` : `${number.value.trim()} ${unit.value}`.trim();
  };
  const original = quantity ? `${Number(quantity.value)} ${quantity.unit}` : '';
  number.addEventListener('input', () => check.run(raw()));
  keys(number, number.value, () => {
    const value = raw();
    if (value === original && !host.mixed) return;
    check.commit(value === '' ? null : value);
  });
  unit.addEventListener('change', () => {
    const value = raw();
    if (!value || value === original) return;
    check.commit(value);
  });
  return { node: control, focus: () => number.focus() };
}

/** The unit a new quantity starts with: the one engineers write most for its dimension. */
const PREFERRED_UNITS = ['ms', 'MB', 'Hz', 'Mbps', 'kg', 'm', 'W', 'V', 'A', 'N', 'J', 'kPa', 'km/h', '°C', '%'];
function preferredUnit(units: { symbol: string; factor?: number }[]): string {
  return units.find((u) => PREFERRED_UNITS.includes(u.symbol))?.symbol ?? units.find((u) => u.factor === 1)?.symbol ?? units[0]?.symbol ?? '';
}

function enumField(host: FieldHost): Field {
  const shape = valueShape(host.attribute.type);
  const values = enumValues(host.metamodel, shape.of);
  const current = host.value === undefined || host.value === null ? '' : String(host.value);
  const select = el('select', { class: 'wb-field', 'aria-label': host.attribute.key });
  select.appendChild(el('option', { value: '', text: host.mixed ? 'several values' : '—' }));
  for (const value of values) select.appendChild(el('option', { value, text: value, selected: value === current }));
  if (current && !values.includes(current)) select.appendChild(el('option', { value: current, text: `${current} (not a ${shape.of})`, selected: true }));
  const message = messageNode();
  const check = liveCheck(host, select, message);
  select.addEventListener('change', () => check.commit(select.value === '' ? null : select.value));
  if (current && values.length && !values.includes(current)) setTimeout(() => check.run(current), 0);
  return { node: el('div', { class: 'wb-control' }, [select, message]), focus: () => select.focus() };
}

function booleanField(host: FieldHost): Field {
  const on = host.value === true || host.value === 'true';
  const set = host.value !== undefined && host.value !== null;
  const message = messageNode();
  const toggle = button('', () => check.commit(on ? 'false' : 'true'), { class: `wb-switch${on ? ' is-on' : ''}${set ? '' : ' is-unset'}`, role: 'switch', 'aria-checked': String(on), 'aria-label': host.attribute.key });
  toggle.append(el('span', { class: 'wb-switch-knob' }));
  const label = el('span', { class: 'wb-switch-label', text: set ? (on ? 'true' : 'false') : 'not set' });
  const node = el('div', { class: 'wb-control' }, [el('div', { class: 'wb-switch-row' }, [toggle, label]), message]);
  const check = liveCheck(host, node, message);
  return { node, focus: () => toggle.focus() };
}

/** A search over the elements a reference may point at, as a combobox: arrows, Enter, Escape. */
function picker(host: FieldHost, initial: string, onPick: (element: ElementRecord) => void, onText: (text: string) => void, label: string) {
  const shape = valueShape(host.attribute.type);
  const candidates = referenceCandidates(shape.of, host.records, host.element ?? undefined);
  const listId = `wb-pick-${Math.random().toString(36).slice(2)}`;
  const input = el('input', { type: 'text', class: 'wb-field wb-picker-input', value: initial, placeholder: shape.of && shape.of !== 'Element' ? `${shape.of.toLowerCase()}…` : 'element…', role: 'combobox', 'aria-autocomplete': 'list', 'aria-expanded': 'false', 'aria-controls': listId, 'aria-label': label, autocomplete: 'off' });
  const list = el('ul', { class: 'wb-picker-list', role: 'listbox', id: listId, hidden: true });
  let active = -1;
  let shown: ElementRecord[] = [];
  const open = (query: string) => {
    const words = query.toLowerCase().split(/\s+/).filter(Boolean);
    const q = query.trim().toLowerCase();
    // Exact ids first, then what starts with the query, then the rest.
    const rank = (c: ElementRecord) => (c.id.toLowerCase() === q || c.name.toLowerCase() === q ? 0 : c.id.toLowerCase().startsWith(q) || c.name.toLowerCase().startsWith(q) ? 1 : 2);
    shown = candidates.filter((c) => words.every((w) => `${c.name} ${c.id} ${c.kind}`.toLowerCase().includes(w))).sort((a, b) => rank(a) - rank(b)).slice(0, 40);
    list.textContent = '';
    shown.forEach((candidate, index) => {
      const option = el('li', { role: 'option', id: `${listId}-${index}`, class: 'wb-picker-option', 'aria-selected': 'false' }, [iconNode(candidate.kind, 14), el('span', { class: 'wb-picker-name', text: candidate.name || candidate.id }), el('span', { class: 'wb-picker-id', text: candidate.id !== candidate.name ? candidate.id : candidate.kind })]);
      option.addEventListener('mousedown', (event) => {
        event.preventDefault();
        pick(candidate);
      });
      list.appendChild(option);
    });
    if (shown.length === 0) list.appendChild(el('li', { class: 'wb-picker-empty', text: 'No element matches' }));
    list.hidden = false;
    input.setAttribute('aria-expanded', 'true');
    highlight(shown.length ? 0 : -1);
  };
  const close = () => {
    list.hidden = true;
    input.setAttribute('aria-expanded', 'false');
    input.removeAttribute('aria-activedescendant');
  };
  const highlight = (index: number) => {
    active = index;
    [...list.children].forEach((node, i) => node.setAttribute('aria-selected', String(i === index)));
    if (index >= 0) {
      input.setAttribute('aria-activedescendant', `${listId}-${index}`);
      list.children[index]?.scrollIntoView?.({ block: 'nearest' });
    }
  };
  const pick = (candidate: ElementRecord) => {
    close();
    onPick(candidate);
  };
  input.addEventListener('focus', () => open(''));
  input.addEventListener('input', () => {
    open(input.value);
    onText(input.value);
  });
  input.addEventListener('blur', () => setTimeout(close, 120));
  input.addEventListener('keydown', (event) => {
    if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
      event.preventDefault();
      if (list.hidden) open(input.value);
      const step = event.key === 'ArrowDown' ? 1 : -1;
      highlight(Math.max(0, Math.min(shown.length - 1, active + step)));
    } else if (event.key === 'Enter' && !list.hidden && active >= 0 && shown[active]) {
      event.preventDefault();
      event.stopPropagation();
      pick(shown[active]);
    } else if (event.key === 'Escape' && !list.hidden) {
      event.stopPropagation();
      close();
    }
  });
  return { input, list };
}

function jumpButton(host: FieldHost, target: ElementRecord | null): HTMLButtonElement {
  const jump = button('↗', () => target && host.jump(target), { class: 'wb-icon-button wb-jump', title: target ? `Go to ${target.name || target.id}` : 'Not an element of the model', 'aria-label': 'Go to the element', disabled: !target });
  return jump;
}

function referenceField(host: FieldHost): Field {
  const current = host.value === undefined || host.value === null ? '' : String(host.value);
  const target = current ? resolveReference(current, host.records) : null;
  const message = messageNode();
  const wrap = el('div', { class: 'wb-control wb-reference' });
  const check = liveCheck(host, wrap, message);
  const { input, list } = picker(
    host,
    host.draft ?? (target ? target.name || target.id : current),
    (candidate) => {
      input.value = candidate.name || candidate.id;
      if (candidate.id !== current) check.commit(candidate.id);
    },
    (text) => check.run(text),
    host.attribute.key,
  );
  input.addEventListener('keydown', (event) => {
    if (event.key === 'Enter' && list.hidden) {
      event.preventDefault();
      const typed = input.value.trim();
      const resolved = typed ? resolveReference(typed, host.records) : null;
      if (!typed && current) check.commit(null);
      else if (resolved && resolved.id !== current) check.commit(resolved.id);
      else if (!resolved && typed) check.run(typed);
    }
  });
  wrap.append(el('div', { class: 'wb-reference-row' }, [input, jumpButton(host, target)]), list, message);
  return { node: wrap, focus: () => input.focus() };
}

function listField(host: FieldHost, references: boolean): Field {
  const items = listItems(host.value);
  const message = messageNode();
  const wrap = el('div', { class: 'wb-control wb-list' });
  const check = liveCheck(host, wrap, message);
  const chips = el('ul', { class: 'wb-chips', 'aria-label': `${host.attribute.key} items` });
  items.forEach((item, index) => {
    const target = references ? resolveReference(item, host.records) : null;
    const label = references && target ? button(target.name || target.id, () => host.jump(target), { class: 'wb-chip-label wb-link', title: `${target.kind} ${target.id}: go to it` }) : el('span', { class: 'wb-chip-label', text: item });
    const remove = button('×', () => check.commit(items.filter((_, i) => i !== index)), { class: 'wb-chip-remove', 'aria-label': `Remove ${item}`, title: `Remove ${item}` });
    chips.appendChild(el('li', { class: `wb-chip${references && !target ? ' is-unresolved' : ''}` }, [references && target ? iconNode(target.kind, 12) : null, label, remove]));
  });
  let adder: HTMLInputElement;
  if (references) {
    const { input, list } = picker(
      host,
      '',
      (candidate) => {
        if (!items.includes(candidate.id)) check.commit([...items, candidate.id]);
      },
      () => undefined,
      `Add to ${host.attribute.key}`,
    );
    input.placeholder = '+ add…';
    adder = input;
    wrap.append(chips, el('div', { class: 'wb-reference-row' }, [input]), list, message);
  } else {
    adder = el('input', { type: 'text', class: 'wb-field wb-chip-input', placeholder: '+ add', 'aria-label': `Add to ${host.attribute.key}` });
    adder.addEventListener('keydown', (event) => {
      if (event.key !== 'Enter') return;
      event.preventDefault();
      const value = adder.value.trim();
      if (value && !items.includes(value)) check.commit([...items, value]);
    });
    wrap.append(chips, adder, message);
  }
  return { node: wrap, focus: () => adder.focus() };
}

/** A description: the text, multi-line, and its Markdown preview. */
export function descriptionField(host: FieldHost, previewFirst: boolean, title = 'Description', doc = ''): Field {
  const current = host.value === undefined || host.value === null ? '' : String(host.value);
  const area = el('textarea', { class: 'wb-field wb-textarea', rows: '4', placeholder: 'Markdown: **bold**, *italic*, `code`, - lists. ⌘Enter or leaving the field applies.', 'aria-label': host.attribute.key });
  area.value = host.draft ?? current;
  const preview = el('div', { class: 'wb-markdown', tabindex: '0', 'aria-label': `${host.attribute.key} preview` });
  const message = messageNode();
  const check = liveCheck(host, area, message);
  const showPreview = (on: boolean) => {
    area.hidden = on;
    preview.hidden = !on;
    toggle.textContent = on ? 'Edit' : 'Preview';
    toggle.setAttribute('aria-pressed', String(on));
    if (on) renderMarkdown(preview, area.value);
  };
  const toggle = button('Preview', () => showPreview(area.hidden === false), { class: 'wb-link wb-preview-toggle' });
  preview.addEventListener('dblclick', () => {
    showPreview(false);
    area.focus();
  });
  keys(area, current, () => {
    if (area.value === current) return;
    check.commit(area.value.trim() === '' ? null : area.value);
  }, true);
  const heading = el('span', { class: 'wb-description-title', text: title, title: doc });
  const node = el('div', { class: 'wb-control wb-description' }, [el('div', { class: 'wb-description-bar' }, [heading, toggle]), area, preview, message]);
  showPreview(previewFirst && current.trim() !== '' && !host.draft);
  return {
    node,
    focus: () => {
      showPreview(false);
      area.focus();
    },
  };
}

export function renderMarkdown(target: HTMLElement, text: string): void {
  target.textContent = '';
  if (!text.trim()) {
    target.appendChild(el('p', { class: 'wb-muted', text: 'No description yet. Double-click to write one.' }));
    return;
  }
  let list: HTMLUListElement | null = null;
  for (const block of markdownBlocks(text)) {
    const runs = block.inline.map((run) => {
      const node = run.code ? el('code', { text: run.text }) : run.bold ? el('strong', { text: run.text }) : run.italic ? el('em', { text: run.text }) : document.createTextNode(run.text);
      return node;
    });
    if (block.type === 'li') {
      if (!list) {
        list = el('ul');
        target.appendChild(list);
      }
      list.appendChild(el('li', {}, runs));
      continue;
    }
    list = null;
    if (block.type === 'h') target.appendChild(el('h4', {}, runs));
    else if (block.type === 'code') target.appendChild(el('pre', {}, runs));
    else target.appendChild(el('p', {}, runs));
  }
}
