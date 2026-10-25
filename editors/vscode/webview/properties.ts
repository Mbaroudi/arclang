// The properties of the selected element, in sections that fold: the
// identity (kind, id, layer, owner as a path of links, the source line),
// the typed attributes of the metamodel in its groups with the control each
// type needs, what comes from the element's type (inherited or redefined),
// its relationships, its traces (with the verdicts of the tests), the
// findings on it with the fixes the compiler names, and its description
// with a Markdown preview. Every field explains itself (its metamodel
// documentation), says what is wrong before anything is written, and
// applies on Enter or on leaving it as one compiler-checked edit (one undo).
import { openImpact } from './impact-marks'; // impact hook
import { attributeGroups, commonAttributes, DESCRIPTION_KEYS, displayValue, fieldLiteral, findingsOf, humanise, inheritedOf, kindLabel, quickFixes, relationsOf, RelationRow, sharedValue, tracesOf, valueShape } from '../src/pure/properties';
import { childTools, PanesToHost, PanesToPage } from '../src/pure/panes';
import { EditRequest, ElementRecord, Finding, LAYER_LABELS, layerOfKind, Metamodel, MetamodelAttribute, Snapshot, TRACE_KINDS } from '../src/pure/workbench';
import { button, clear, el, inlinePrompt } from './dom';
import { createField, descriptionField, FieldHost } from './fields';
import { iconNode } from './icons';

export { humanise };

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
  /** panes: the model the pane reads its relationships, types and findings from. */
  snapshot?: () => Snapshot | null;
  /** panes: make another element the selection (a link of the pane). */
  select?: (element: ElementRecord) => void;
  /** panes: a question for the host (validation, the source line). */
  post?: (message: PanesToHost) => void;
}

type Section = { id: string; label: string; count?: string; tone?: 'error' | 'warning'; body: () => HTMLElement };

const VALIDATION_TIMEOUT = 5000;

/**
 * A header action: a glyph and its name. A narrow drawer shows the glyphs
 * alone, in one row, the name in the tooltip and to assistive technology.
 */
function action(glyph: string, label: string, onClick: () => void, attributes: Record<string, string> = {}): HTMLButtonElement {
  const node = button('', onClick, { ...attributes, 'aria-label': label, title: `${label}: ${attributes.title ?? label}` });
  node.append(el('span', { class: 'wb-action-glyph', 'aria-hidden': 'true', text: glyph }), el('span', { class: 'wb-action-label', text: label }));
  return node;
}

export class Properties {
  private readonly root: HTMLElement;
  private element: ElementRecord | null = null;
  private many: ElementRecord[] = [];
  private metamodel: Metamodel | null = null;
  /** Folded sections, kept from one element to the next. */
  private readonly folded = new Set<string>(['relationships']);
  private filter = '';
  /** The compiler's refusals kept on their field (`uuid:key`), with what was typed. */
  private readonly refusals = new Map<string, { error: string; draft: string }>();
  private readonly pending = new Map<string, { field: string; draft: string }>();
  private readonly validations = new Map<number, (answer: { ok: boolean; error?: string } | null) => void>();
  private token = 0;
  private source: { id: string; line: number | null; text?: string } | null = null;
  /** Keys whose inherited value is being redefined (a field opened on it). */
  private readonly redefining = new Set<string>();

  constructor(host: HTMLElement, private readonly owner: PropertiesHost) {
    this.root = el('div', { class: 'wb-properties' });
    host.appendChild(this.root);
    window.addEventListener('message', (event: MessageEvent<PanesToPage | { type: 'edit-result'; ok: boolean; label: string; error?: string }>) => this.message(event.data));
    this.show(null, null);
  }

  private message(message: PanesToPage | { type: 'edit-result'; ok: boolean; label: string; error?: string }): void {
    if (!message || typeof message !== 'object') return;
    if (message.type === 'panes-validated') {
      this.validations.get(message.token)?.({ ok: message.ok, error: message.error });
      this.validations.delete(message.token);
    } else if (message.type === 'panes-located') {
      if (message.id !== this.element?.id) return;
      this.source = message;
      const slot = this.root.querySelector('.wb-source-line');
      if (slot) slot.replaceWith(this.sourceLine());
    } else if (message.type === 'edit-result') {
      const pending = this.pending.get(message.label);
      if (!pending) return;
      this.pending.delete(message.label);
      if (message.ok) {
        this.refusals.delete(pending.field);
        // A redefinition written, or taken back: the row goes back to what the model now says.
        this.redefining.delete(pending.field.slice(pending.field.indexOf(':') + 1));
      }
      else {
        this.refusals.set(pending.field, { error: message.error ?? 'refused', draft: pending.draft });
        this.render();
      }
    }
  }

  show(element: ElementRecord | null, metamodel: Metamodel | null): void {
    if (element?.uuid !== this.element?.uuid) {
      this.source = null;
      this.redefining.clear();
      if (element) this.owner.post?.({ type: 'panes-locate', id: element.id });
    }
    this.element = element;
    this.many = [];
    this.metamodel = metamodel;
    this.render();
  }

  /** Several elements: what they have in common, set on all of them at once. */
  showMany(elements: ElementRecord[], metamodel: Metamodel | null): void {
    this.element = null;
    this.many = elements;
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

  private records(): ElementRecord[] {
    return this.owner.snapshot?.()?.elements?.elements ?? [];
  }

  private render(): void {
    // The field that had the keyboard gets it back once the pane is drawn again.
    const focused = document.activeElement instanceof HTMLElement && this.root.contains(document.activeElement) ? document.activeElement.closest('[data-key]')?.getAttribute('data-key') : null;
    clear(this.root);
    if (this.many.length > 1) return this.renderMany();
    const element = this.element;
    if (!element) {
      this.root.appendChild(el('p', { class: 'wb-muted wb-props-empty', text: 'Select an element in a diagram, in the tree or in the text to see and edit its properties.' }));
      return;
    }
    this.root.append(this.header(element), this.filterBar());
    for (const section of this.sections(element)) this.root.appendChild(this.section(section));
    if (focused) this.root.querySelector<HTMLElement>(`[data-key="${CSS.escape(focused)}"] input, [data-key="${CSS.escape(focused)}"] select, [data-key="${CSS.escape(focused)}"] textarea`)?.focus();
  }

  // --- Header -----------------------------------------------------------------------

  private header(element: ElementRecord): HTMLElement {
    const kind = this.metamodel?.kinds.find((k) => k.name === element.kind);
    const name = el('input', { type: 'text', class: 'wb-name-input', value: element.name || element.id, title: 'Name: Enter renames, the identity is kept', 'aria-label': 'Name' });
    name.addEventListener('change', () => {
      const value = name.value.trim();
      if (!value || value === element.name) {
        name.value = element.name || element.id;
        return;
      }
      this.owner.submit([{ op: 'rename', element: element.id, name: value }], `rename to ${value}`);
    });
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
    const chip = el('span', { class: 'wb-kind', text: kindLabel(element.kind), title: kind?.doc ?? element.kind });
    const actions = el('div', { class: 'wb-actions', role: 'toolbar', 'aria-label': 'Element actions' }, [
      action('✦', 'Ask @arclang', () => this.owner.ask(element), { title: 'Ask the assistant about this element' }),
    ]);
    const ends = this.owner.ends?.(element) ?? null;
    const requirements = this.owner.requirements?.() ?? [];
    if (!ends && element.kind !== 'Requirement' && requirements.length) actions.appendChild(action('⊨', 'Satisfies…', () => this.satisfies(element, requirements), { title: 'Trace it to a requirement it satisfies' }));
    if (!ends && this.owner.diagram) {
      const diagram = this.owner.diagram;
      actions.append(
        action('◌', 'Hide', () => diagram('hide', element), { title: 'Hide from this diagram (⌘H)' }),
        action('⇆', 'Show related', () => diagram('related', element), { title: 'Add what it is connected to' }),
        action('⊞', 'New diagram', () => diagram('new', element), { title: 'A new diagram with it and what it is connected to' }),
      );
    }
    actions.appendChild(action('⚡', 'Impact', () => openImpact(element.id), { title: 'Analyse the impact of a change to it, or ask what if…' })); // impact hook
    actions.appendChild(action('⌫', 'Delete', () => this.owner.submit([{ op: 'remove', element: element.id }], `delete ${element.id}`), { class: 'wb-danger', title: 'Delete the declaration (⌘Z takes it back)' }));
    const linked = ends
      ? el('p', { class: 'wb-ends' }, [
          this.link(ends.from),
          el('span', { class: 'wb-muted', text: ' → ' }),
          this.link(ends.to),
          ' ',
          button('⇄ Reverse', () => this.owner.reverse?.(element), { class: 'wb-link', title: 'Swap the source and the target (one undo)' }),
        ])
      : null;
    return el('header', { class: 'wb-props-head' }, [el('div', { class: 'wb-props-title' }, [iconNode(element.kind, 22, kindLabel(element.kind)), el('div', { class: 'wb-props-name' }, [name, el('div', { class: 'wb-props-sub' }, [chip, kind?.sysml ? el('span', { class: 'wb-props-sub-id', text: `SysML v2: ${kind.sysml}`, title: 'What it is in SysML v2' }) : null])])]), linked, actions]);
  }

  private filterBar(): HTMLElement {
    const input = el('input', { type: 'search', class: 'wb-field wb-props-filter', placeholder: 'Filter properties', value: this.filter, 'aria-label': 'Filter properties' });
    input.addEventListener('input', () => {
      this.filter = input.value;
      for (const row of this.root.querySelectorAll<HTMLElement>('tr[data-key]')) {
        const text = `${row.dataset.key} ${row.textContent}`.toLowerCase();
        row.hidden = !!this.filter && !text.includes(this.filter.toLowerCase());
      }
      if (this.filter) for (const section of this.root.querySelectorAll<HTMLElement>('.wb-sec')) this.setFolded(section, false, false);
    });
    return el('div', { class: 'wb-props-filterbar' }, [input]);
  }

  // --- Sections -----------------------------------------------------------------------

  private sections(element: ElementRecord): Section[] {
    const snapshot = this.owner.snapshot?.() ?? null;
    const records = this.records();
    const inherited = inheritedOf(element, records);
    const inheritedKeys = new Set(inherited.map((i) => i.key));
    const groups = attributeGroups(element, this.metamodel);
    const sections: Section[] = [{ id: 'identity', label: 'Identity', body: () => this.identity(element, groups.find((g) => g.id === 'identity')?.attributes ?? []) }];
    for (const group of groups) {
      if (group.id === 'identity' || group.id === 'description') continue;
      const attributes = group.attributes.filter((a) => !inheritedKeys.has(a.key));
      if (attributes.length === 0) continue;
      const set = attributes.filter((a) => isSet(element.attributes[a.key])).length;
      sections.push({ id: group.id, label: group.label, count: `${set}/${attributes.length}`, body: () => this.table(element, attributes) });
    }
    if (inherited.length) {
      const types = [...new Set(inherited.map((i) => i.from.name || i.from.id))].join(', ');
      sections.push({ id: 'inherited', label: `Inherited from ${types}`, count: String(inherited.length), body: () => this.inherited(element, inherited) });
    }
    if (snapshot) {
      const relations = relationsOf(element, snapshot);
      const traces = tracesOf(element, snapshot);
      sections.push({ id: 'relationships', label: 'Relationships', count: String(relations.owned.length + relations.incoming.length + relations.outgoing.length), body: () => this.relationships(element, relations) });
      sections.push({ id: 'traces', label: 'Traces', count: String(traces.reduce((n, g) => n + g.rows.length, 0)), body: () => this.traces(element, traces) });
      const findings = findingsOf(element, snapshot.findings ?? []);
      if (findings.length) {
        const errors = findings.some((f) => f.severity.toLowerCase() === 'error');
        sections.push({ id: 'findings', label: 'Findings', count: String(findings.length), tone: errors ? 'error' : 'warning', body: () => this.findings(element, findings) });
      }
    }
    const descriptions = groups.find((g) => g.id === 'description')?.attributes ?? [];
    if (descriptions.length) sections.push({ id: 'description', label: 'Description', body: () => this.descriptions(element, descriptions) });
    return sections;
  }

  private section(section: Section): HTMLElement {
    const node = el('section', { class: `wb-sec${section.tone ? ` is-${section.tone}` : ''}`, 'data-section': section.id });
    const bodyId = `wb-sec-${section.id}`;
    const head = button('', () => this.setFolded(node, !node.classList.contains('is-folded'), true), { class: 'wb-sec-head', 'aria-expanded': 'true', 'aria-controls': bodyId });
    head.append(el('span', { class: 'wb-chevron', 'aria-hidden': 'true' }), el('span', { class: 'wb-sec-label', text: section.label }), ...(section.count ? [el('span', { class: `wb-sec-count${section.tone ? ` is-${section.tone}` : ''}`, text: section.count })] : []));
    head.addEventListener('keydown', (event) => {
      if (event.key !== 'ArrowDown' && event.key !== 'ArrowUp') return;
      const heads = [...this.root.querySelectorAll<HTMLElement>('.wb-sec-head')];
      const at = heads.indexOf(head);
      heads[at + (event.key === 'ArrowDown' ? 1 : -1)]?.focus();
      event.preventDefault();
    });
    const body = el('div', { class: 'wb-sec-body', id: bodyId }, [section.body()]);
    node.append(el('h3', { class: 'wb-sec-title' }, [head]), body);
    this.setFolded(node, this.folded.has(section.id) && !(section.id === 'findings'), false);
    return node;
  }

  private setFolded(section: HTMLElement, folded: boolean, remember: boolean): void {
    section.classList.toggle('is-folded', folded);
    section.querySelector('.wb-sec-head')?.setAttribute('aria-expanded', String(!folded));
    const body = section.querySelector<HTMLElement>('.wb-sec-body');
    if (body) body.hidden = folded;
    const id = section.dataset.section ?? '';
    if (remember) {
      if (folded) this.folded.add(id);
      else this.folded.delete(id);
    }
  }

  private identity(element: ElementRecord, attributes: MetamodelAttribute[]): HTMLElement {
    const table = propsTable('wb-props');
    const records = this.records();
    const byUuid = new Map(records.map((r) => [r.uuid, r]));
    const chain: ElementRecord[] = [];
    let at = element.owner ? byUuid.get(element.owner) : undefined;
    while (at && chain.length < 12) {
      chain.unshift(at);
      at = at.owner ? byUuid.get(at.owner) : undefined;
    }
    const layer = layerOfKind(element.kind, this.metamodel);
    const crumbs = el('nav', { class: 'wb-crumbs', 'aria-label': 'Owner' });
    chain.forEach((owner, index) => {
      if (index > 0) crumbs.appendChild(el('span', { class: 'wb-crumb-sep', text: '›', 'aria-hidden': 'true' }));
      crumbs.appendChild(this.link(owner));
    });
    if (chain.length === 0) crumbs.appendChild(el('span', { class: 'wb-muted', text: 'top of the file' }));
    const id = el('code', { class: 'wb-id', text: element.id, title: 'Identity (click to copy)', tabindex: '0' });
    id.addEventListener('click', () => void navigator.clipboard?.writeText(element.id));
    table.append(
      this.staticRow('Id', [id], 'Stable identity; drives the UUID. Rename keeps it.'),
      this.staticRow('Layer', [el('span', { text: LAYER_LABELS[layer] ?? layer })], 'The Arcadia layer of the kind.'),
      this.staticRow('Owner', [crumbs], 'Where it is declared: each step is a link.'),
      this.staticRow('Source', [this.sourceLine()], 'The line that declares it.'),
    );
    for (const attribute of attributes.filter((a) => a.key !== 'id' && a.key !== 'name')) table.appendChild(this.row(element, attribute));
    return table;
  }

  private sourceLine(): HTMLElement {
    const element = this.element;
    const node = el('span', { class: 'wb-source-line' });
    if (!element) return node;
    const line = this.source?.id === element.id ? this.source.line : undefined;
    const text = line === undefined ? 'locating…' : line === null ? 'not in this file' : `line ${line}`;
    node.append(el('span', { class: 'wb-muted', text, title: this.source?.text ?? '' }), button('Go to source', () => this.owner.reveal(element.id), { class: 'wb-link', title: 'Show the declaration in the text' }));
    return node;
  }

  private staticRow(label: string, content: (Node | string)[], doc: string, extra = ''): HTMLTableRowElement {
    return el('tr', { class: 'is-static', 'data-key': `:${label.toLowerCase()}` }, [this.label(label, [doc, extra].filter(Boolean).join('\n')), el('td', { colspan: '2' }, content)]);
  }

  private label(text: string, doc: string): HTMLTableCellElement {
    // The full name on hover: a narrow drawer cuts a long label with an ellipsis.
    const th = el('th', { scope: 'row' }, [el('span', { class: 'wb-label-text', text, title: text })]);
    if (doc) {
      const tipId = `wb-tip-${Math.random().toString(36).slice(2)}`;
      const info = el('button', { type: 'button', class: 'wb-info', 'aria-label': `About ${text}`, 'aria-describedby': tipId, text: 'i' });
      const tip = el('span', { class: 'wb-tip', role: 'tooltip', id: tipId, text: doc });
      th.append(info, tip);
    }
    return th;
  }

  private table(element: ElementRecord, attributes: MetamodelAttribute[]): HTMLElement {
    const table = propsTable('wb-props');
    // Set values first, then what can be added.
    const sorted = [...attributes].sort((a, b) => Number(isSet(element.attributes[b.key])) - Number(isSet(element.attributes[a.key])));
    for (const attribute of sorted) table.appendChild(this.row(element, attribute));
    return table;
  }

  private row(element: ElementRecord, attribute: MetamodelAttribute, inheritedFrom?: string): HTMLTableRowElement {
    const value = element.attributes[attribute.key];
    const field = `${element.uuid}:${attribute.key}`;
    const refusal = this.refusals.get(field);
    const host: FieldHost = {
      element,
      attribute,
      value,
      metamodel: this.metamodel,
      records: this.records(),
      error: refusal?.error,
      draft: refusal?.draft,
      commit: (raw) => this.commit(element, attribute, raw),
      ask: (raw) => this.validate([{ op: 'set', element: element.id, key: attribute.key, value: fieldLiteral(attribute.type, raw) }]),
      jump: (target) => this.owner.select?.(target),
    };
    const control = createField(host);
    const tools = el('td', { class: 'wb-row-tools' });
    if (inheritedFrom) tools.appendChild(button('↺', () => this.commit(element, attribute, null), { class: 'wb-icon-button', title: `Reset to inherited from ${inheritedFrom}`, 'aria-label': 'Reset to inherited' }));
    else if (isSet(value) && valueShape(attribute.type).item !== 'Boolean') tools.appendChild(button('×', () => this.commit(element, attribute, null), { class: 'wb-icon-button wb-clear', title: `Clear ${humanise(attribute.key)}`, 'aria-label': `Clear ${humanise(attribute.key)}` }));
    else if (valueShape(attribute.type).item === 'Boolean' && isSet(value)) tools.appendChild(button('×', () => this.commit(element, attribute, null), { class: 'wb-icon-button wb-clear', title: 'Not set', 'aria-label': 'Clear' }));
    const doc = [attribute.doc ?? '', typeLine(attribute)].filter(Boolean).join('\n');
    const row = el('tr', { 'data-key': attribute.key, class: `${isSet(value) ? 'is-set' : 'is-unset'}${inheritedFrom ? ' is-redefined' : ''}` }, [this.label(humanise(attribute.key), doc), el('td', { class: 'wb-value' }, [control.node, inheritedFrom ? el('div', { class: 'wb-redefines', text: `redefines ${inheritedFrom}` }) : null]), tools]);
    row.addEventListener('wb-focus', () => control.focus());
    return row;
  }

  private commit(element: ElementRecord, attribute: MetamodelAttribute, raw: string | string[] | null): void {
    const field = `${element.uuid}:${attribute.key}`;
    const draft = raw === null ? '' : Array.isArray(raw) ? raw.join(', ') : raw;
    if (raw === null) {
      if (!isSet(element.attributes[attribute.key])) return;
      const label = `clear ${humanise(attribute.key)} of ${element.id}`;
      this.pending.set(label, { field, draft });
      this.owner.submit([{ op: 'unset', element: element.id, key: attribute.key }], label);
      return;
    }
    const value = fieldLiteral(attribute.type, raw);
    const label = `${humanise(attribute.key)} = ${displayValue(Array.isArray(raw) ? raw : raw.trim())}`;
    this.pending.set(label, { field, draft });
    this.owner.submit([{ op: 'set', element: element.id, key: attribute.key, value }], label);
  }

  /** The compiler's answer on edits, nothing written: null when the host does not answer. */
  private validate(requests: EditRequest[]): Promise<{ ok: boolean; error?: string } | null> {
    if (!this.owner.post) return Promise.resolve(null);
    const token = ++this.token;
    return new Promise((resolve) => {
      this.validations.set(token, resolve);
      this.owner.post!({ type: 'panes-validate', token, requests });
      setTimeout(() => {
        if (!this.validations.has(token)) return;
        this.validations.delete(token);
        resolve(null);
      }, VALIDATION_TIMEOUT);
    });
  }

  private inherited(element: ElementRecord, inherited: ReturnType<typeof inheritedOf>): HTMLElement {
    const table = propsTable('wb-props wb-inherited');
    const declared = new Map((this.metamodel?.kinds.find((k) => k.name === element.kind)?.attributes ?? []).map((a) => [a.key, a]));
    for (const item of inherited) {
      const attribute = declared.get(item.key) ?? { key: item.key, type: { type: 'Text' } };
      const from = item.from.name || item.from.id;
      if (item.redefined || this.redefining.has(item.key)) {
        table.appendChild(this.row(element, attribute, `${from}: ${displayValue(item.value)}`));
        continue;
      }
      const redefine = button('Redefine', () => {
        this.redefining.add(item.key);
        this.render();
        this.root.querySelector<HTMLElement>(`tr[data-key="${CSS.escape(item.key)}"]`)?.dispatchEvent(new Event('wb-focus'));
      }, { class: 'wb-link', title: `Write its own ${humanise(item.key)} on ${element.name || element.id}` });
      table.appendChild(
        el('tr', { 'data-key': item.key, class: 'is-inherited' }, [
          this.label(humanise(item.key), [attribute.doc ?? '', `Inherited from ${from}`].filter(Boolean).join('\n')),
          el('td', { class: 'wb-value' }, [el('span', { class: 'wb-inherited-value', text: displayValue(item.value) }), el('span', { class: 'wb-from', text: ` from ${from}`, title: `from ${from}` })]),
          el('td', { class: 'wb-row-tools' }, [redefine]),
        ]),
      );
    }
    return table;
  }

  private link(target: ElementRecord): HTMLElement {
    const node = button(target.name || target.id, () => this.owner.select?.(target), { class: 'wb-link wb-elem-link', title: `${kindLabel(target.kind)} ${target.id}` });
    node.prepend(iconNode(target.kind, 12));
    return node;
  }

  private relationRows(rows: RelationRow[], empty: string, byKind = false): HTMLElement {
    const list = el('ul', { class: 'wb-rel-list' });
    if (rows.length === 0) list.appendChild(el('li', { class: 'wb-muted', text: empty }));
    for (const row of rows) {
      const item = el('li', { class: 'wb-rel' }, [row.other ? this.link(row.other) : el('span', { class: 'wb-muted', text: '?' }), el('span', { class: 'wb-rel-kind', text: byKind && row.other ? kindLabel(row.other.kind) : row.label })]);
      if (row.verdict) item.appendChild(el('span', { class: `wb-verdict is-${row.verdict}`, text: row.verdict }));
      if (row.remove) {
        const remove = row.remove;
        item.appendChild(button('×', () => this.owner.submit(remove, `remove ${row.label} ${row.other?.id ?? ''}`.trim()), { class: 'wb-icon-button wb-rel-remove', title: `Remove ${row.label}`, 'aria-label': `Remove ${row.label} ${row.other?.name ?? ''}` }));
      }
      list.appendChild(item);
    }
    return list;
  }

  private relationships(element: ElementRecord, relations: ReturnType<typeof relationsOf>): HTMLElement {
    const box = el('div', { class: 'wb-rels' });
    const children = childTools(element.kind);
    const add = children.length ? el('select', { class: 'wb-field wb-add-child', 'aria-label': 'New child' }, [el('option', { value: '', text: '+ New child…' }), ...children.map((c) => el('option', { value: c.keyword, text: c.label }))]) : null;
    add?.addEventListener('change', () => {
      const keyword = add.value;
      if (!keyword) return;
      add.value = '';
      void inlinePrompt(this.root, `New ${keyword} in ${element.name || element.id}`, '', 'name').then((name) => {
        if (name?.trim()) this.owner.submit([{ op: 'add', keyword, name: name.trim(), container: element.id }], `new ${keyword} ${name.trim()}`);
      });
    });
    box.append(
      el('h4', { class: 'wb-rel-head' }, [`Owned (${relations.owned.length})`, add]),
      this.relationRows(relations.owned, 'owns nothing'),
      el('h4', { class: 'wb-rel-head', text: `Incoming (${relations.incoming.length})` }),
      this.relationRows(relations.incoming, 'none'),
      el('h4', { class: 'wb-rel-head', text: `Outgoing (${relations.outgoing.length})` }),
      this.relationRows(relations.outgoing, 'none'),
    );
    return box;
  }

  private traces(element: ElementRecord, groups: ReturnType<typeof tracesOf>): HTMLElement {
    const box = el('div', { class: 'wb-rels' });
    if (groups.length === 0) box.appendChild(el('p', { class: 'wb-muted', text: 'No trace: nothing says what it satisfies, realizes or is verified by.' }));
    for (const group of groups) box.append(el('h4', { class: 'wb-rel-head', text: `${humanise(group.label)} (${group.rows.length})` }), this.relationRows(group.rows, '', true));
    box.appendChild(this.traceAdder(element));
    return box;
  }

  private traceAdder(element: ElementRecord): HTMLElement {
    const kind = el('select', { class: 'wb-field wb-trace-kind', 'aria-label': 'Kind of trace' }, TRACE_KINDS.map((k) => el('option', { value: k, text: k })));
    const attribute: MetamodelAttribute = { key: 'trace', type: { type: 'Reference', of: 'Element' } };
    const field = createField({
      element,
      attribute,
      value: '',
      metamodel: this.metamodel,
      records: this.records(),
      commit: (raw) => {
        if (raw === null || Array.isArray(raw)) return;
        this.owner.submit([{ op: 'add_trace', from: element.id, kind: kind.value, to: raw }], `${element.id} ${kind.value} ${raw}`);
      },
      ask: (raw) => this.validate([{ op: 'add_trace', from: element.id, kind: kind.value, to: String(raw) }]),
      jump: (target) => this.owner.select?.(target),
    });
    return el('div', { class: 'wb-trace-add', 'data-key': ':trace' }, [el('span', { class: 'wb-muted', text: '+ Trace' }), kind, field.node]);
  }

  private findings(element: ElementRecord, findings: Finding[]): HTMLElement {
    const list = el('ul', { class: 'wb-findings' });
    for (const finding of findings) {
      const severity = finding.severity.toLowerCase();
      const item = el('li', { class: `wb-finding is-${severity}` }, [el('span', { class: 'wb-finding-mark', text: severity === 'error' ? '●' : severity === 'warning' ? '▲' : 'i', 'aria-label': severity }), el('span', { class: 'wb-finding-text' }, [el('code', { text: finding.rule }), ` ${finding.message}`])]);
      for (const fix of quickFixes(finding)) {
        item.appendChild(
          button(fix.label, () => {
            if (fix.action === 'focus') this.focusField(fix.key);
            else if (fix.action === 'trace') this.focusTrace();
            else
              void inlinePrompt(this.root, `An explicit id for ${element.name || element.id}`, element.id, 'unique id').then((id) => {
                if (id?.trim() && id.trim() !== element.id) this.owner.submit([{ op: 'set', element: element.id, key: 'id', value: JSON.stringify(id.trim()) }], `id ${id.trim()}`);
              });
          }, { class: 'wb-link wb-fix' }),
        );
      }
      list.appendChild(item);
    }
    return list;
  }

  /** Unfold the section that holds a field and give it the keyboard (a quick fix, a redefinition). */
  focusField(key: string): void {
    const row = this.root.querySelector<HTMLElement>(`tr[data-key="${CSS.escape(key)}"]`);
    if (!row) return;
    const section = row.closest<HTMLElement>('.wb-sec');
    if (section) this.setFolded(section, false, true);
    row.scrollIntoView({ block: 'center' });
    row.dispatchEvent(new Event('wb-focus'));
  }

  private focusTrace(): void {
    const section = this.root.querySelector<HTMLElement>('.wb-sec[data-section="traces"]');
    if (section) this.setFolded(section, false, true);
    this.root.querySelector<HTMLInputElement>('.wb-trace-add input')?.focus();
  }

  private descriptions(element: ElementRecord, attributes: MetamodelAttribute[]): HTMLElement {
    const box = el('div', { class: 'wb-descriptions' });
    const sorted = [...attributes].sort((a, b) => Number(a.key !== 'description') - Number(b.key !== 'description'));
    for (const attribute of sorted) {
      const value = element.attributes[attribute.key];
      if (attribute.key !== 'description' && !isSet(value) && !this.redefining.has(`text:${attribute.key}`)) continue;
      const field = `${element.uuid}:${attribute.key}`;
      const refusal = this.refusals.get(field);
      const control = descriptionField(
        {
          element,
          attribute,
          value,
          metamodel: this.metamodel,
          records: this.records(),
          error: refusal?.error,
          draft: refusal?.draft,
          commit: (raw) => this.commit(element, attribute, raw),
          ask: (raw) => this.validate([{ op: 'set', element: element.id, key: attribute.key, value: fieldLiteral(attribute.type, raw) }]),
          jump: (target) => this.owner.select?.(target),
        },
        true,
        humanise(attribute.key),
        attribute.doc ?? '',
      );
      // A block, not a table row: the text takes the whole width of the pane.
      const block = el('div', { 'data-key': attribute.key, class: `wb-desc-block ${isSet(value) ? 'is-set' : 'is-unset'}` }, [control.node]);
      block.addEventListener('wb-focus', () => control.focus());
      box.appendChild(block);
    }
    const others = attributes.filter((a) => a.key !== 'description' && !isSet(element.attributes[a.key]) && !this.redefining.has(`text:${a.key}`));
    if (others.length) {
      const add = el('select', { class: 'wb-field wb-add-text', 'aria-label': 'Add a text' }, [el('option', { value: '', text: '+ Rationale, notes…' }), ...others.map((a) => el('option', { value: a.key, text: humanise(a.key) }))]);
      add.addEventListener('change', () => {
        if (!add.value) return;
        const key = add.value;
        this.redefining.add(`text:${key}`);
        this.render();
        this.focusField(key);
      });
      box.appendChild(add);
    }
    return box;
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

  // --- Several elements -----------------------------------------------------------------

  private renderMany(): void {
    const elements = this.many;
    const kinds = [...new Set(elements.map((e) => e.kind))];
    const chips = el('ul', { class: 'wb-chips wb-many' }, elements.slice(0, 12).map((e) => el('li', { class: 'wb-chip' }, [iconNode(e.kind, 12), this.link(e)])));
    if (elements.length > 12) chips.appendChild(el('li', { class: 'wb-chip wb-muted', text: `+${elements.length - 12}` }));
    this.root.append(
      el('header', { class: 'wb-props-head' }, [el('p', { class: 'wb-props-title', text: `${elements.length} elements selected` }), el('p', { class: 'wb-muted', text: kinds.map(kindLabel).join(', ') }), chips]),
    );
    const common = commonAttributes(elements, this.metamodel).filter((a) => !DESCRIPTION_KEYS.has(a.key) || a.key === 'description');
    if (common.length === 0) {
      this.root.appendChild(el('p', { class: 'wb-muted', text: 'These elements share no attribute.' }));
      return;
    }
    const table = propsTable('wb-props wb-bulk');
    for (const attribute of common) {
      const shared = sharedValue(elements, attribute.key);
      const field = createField({
        element: null,
        attribute,
        value: shared.mixed ? '' : (elements[0].attributes[attribute.key] ?? ''),
        mixed: shared.mixed,
        metamodel: this.metamodel,
        records: this.records(),
        commit: (raw) => {
          const label = `${humanise(attribute.key)} on ${elements.length} elements`;
          if (raw === null) {
            const set = elements.filter((e) => isSet(e.attributes[attribute.key]));
            if (set.length) this.owner.submit(set.map((e) => ({ op: 'unset', element: e.id, key: attribute.key })), `clear ${label}`);
            return;
          }
          const value = fieldLiteral(attribute.type, raw);
          this.owner.submit(elements.map((e) => ({ op: 'set', element: e.id, key: attribute.key, value })), `${label} = ${displayValue(raw)}`);
        },
        ask: (raw) => this.validate(elements.map((e) => ({ op: 'set', element: e.id, key: attribute.key, value: fieldLiteral(attribute.type, raw) }))),
        jump: (target) => this.owner.select?.(target),
      });
      table.appendChild(el('tr', { 'data-key': attribute.key, class: shared.mixed ? 'is-mixed' : '' }, [this.label(humanise(attribute.key), [attribute.doc ?? '', typeLine(attribute), shared.mixed ? 'The selected elements have different values.' : ''].filter(Boolean).join('\n')), el('td', { class: 'wb-value' }, [field.node]), el('td', { class: 'wb-row-tools' }, [shared.mixed ? el('span', { class: 'wb-mixed', text: 'mixed', title: 'Different values' }) : null])]));
    }
    this.root.append(el('section', { class: 'wb-sec', 'data-section': 'common' }, [el('h3', { class: 'wb-sec-title' }, [el('span', { class: 'wb-sec-head is-static', text: `Common attributes (${common.length})` })]), el('div', { class: 'wb-sec-body' }, [table])]));
  }
}

/** A table of rows, its columns fixed: the label, the control, the row's tools. */
function propsTable(className: string): HTMLTableElement {
  return el('table', { class: className }, [el('colgroup', {}, [el('col', { class: 'wb-col-label' }), el('col', { class: 'wb-col-value' }), el('col', { class: 'wb-col-tools' })])]);
}

function isSet(value: unknown): boolean {
  return value !== undefined && value !== null && !(Array.isArray(value) && value.length === 0) && value !== '';
}

/** The type of an attribute as the tooltip says it: `Quantity (Time)`, `List of Reference (Requirement)`. */
function typeLine(attribute: MetamodelAttribute): string {
  const shape = valueShape(attribute.type);
  const of = shape.of ? ` (${shape.of})` : '';
  return `${shape.list ? 'List of ' : ''}${shape.item}${of} · ${attribute.key}`;
}
