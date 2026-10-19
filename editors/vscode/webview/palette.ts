// The palette: one button per gesture of the current view. Create asks a
// name and declares the element where the selection says; connect takes two
// clicks on the diagram then a name. Each gesture is requests for the
// compiler, never a change of the drawing by hand.
import { connectPlan, ConnectEnd, createRequest, EditRequest, Tool, toolsFor } from '../src/pure/workbench';
import { button, clear, el, inlinePrompt } from './dom';

export interface PaletteHost {
  /** The element selected in the views, as the model knows it. */
  selected: () => { id: string; kind: string } | null;
  /** Send a gesture to the compiler. */
  submit: (requests: EditRequest[], label: string) => void;
  /** Tell the user what the palette waits for (the mode chip on the canvas). */
  hint: (text: string) => void;
  /** Where prompts open: on the canvas, near the user's eyes, not in the rail. */
  anchor: () => HTMLElement;
}

export class Palette {
  private readonly root: HTMLElement;
  private view = '';
  private graphContainer: string | null = null;
  private armed: Tool | null = null;
  private firstEnd: ConnectEnd | null = null;
  private compact = false;
  private tools: Tool[] = [];

  constructor(host: HTMLElement, private readonly owner: PaletteHost) {
    this.root = el('div', { class: 'wb-palette' });
    host.appendChild(this.root);
  }

  /** Redraw for a view (oab, sab, lab, pab, cdb...). */
  show(view: string, graphContainer: string | null = null): void {
    this.view = view;
    this.graphContainer = graphContainer;
    this.tools = toolsFor(view);
    this.disarm();
    this.render();
  }

  /** The rail shows icons only when compact, icons and labels otherwise. */
  setCompact(compact: boolean): void {
    this.compact = compact;
    this.root.classList.toggle('is-compact', compact);
    this.render();
  }

  private render(): void {
    clear(this.root);
    if (this.tools.length === 0) {
      if (!this.compact) this.root.appendChild(el('p', { class: 'wb-muted', text: 'This view is drawn from the model; edit it in the source or with the properties.' }));
      return;
    }
    const creates = this.tools.filter((t) => t.kind === 'create');
    const connects = this.tools.filter((t) => t.kind === 'connect');
    if (creates.length) this.root.appendChild(this.section('Create', creates));
    if (connects.length) this.root.appendChild(this.section('Connect', connects));
  }

  /** A key press on the canvas: V selects, letters arm their tool. */
  shortcut(key: string): boolean {
    if (key === 'v' || key === 'V') {
      this.disarm();
      return true;
    }
    const tool = this.tools.find((t) => SHORTCUTS[t.id] === key.toLowerCase());
    if (!tool) return false;
    const node = this.root.querySelector<HTMLButtonElement>(`[data-tool="${tool.id}"]`);
    if (node) this.use(tool, node);
    return true;
  }

  /** A click on the diagram while a connect tool is armed: the first or the second end. */
  pick(end: ConnectEnd | null): boolean {
    if (!this.armed || this.armed.kind !== 'connect') return false;
    if (!end) return true;
    if (!this.firstEnd) {
      this.firstEnd = end;
      this.owner.hint(`${this.armed.label}: now click the target (from ${end.id})`);
      return true;
    }
    const from = this.firstEnd;
    const tool = this.armed;
    this.firstEnd = null;
    const ask = tool.connect === 'transition' ? 'guard (optional)' : tool.connect === 'message' ? 'text of the message' : 'name of the exchange';
    void inlinePrompt(this.owner.anchor(), `${tool.label}`, '', ask).then((name) => {
      if (name === null) return this.disarm();
      const plan = connectPlan(tool, from, end, name, this.view, this.graphContainer);
      if ('error' in plan) {
        this.owner.hint(plan.error);
        return this.disarm();
      }
      this.owner.submit(plan, `${tool.label} ${name}`);
      this.disarm();
    });
    return true;
  }

  disarm(): void {
    this.armed = null;
    this.firstEnd = null;
    for (const node of this.root.querySelectorAll('.is-armed')) {
      node.classList.remove('is-armed');
      node.removeAttribute('aria-pressed');
    }
    this.owner.hint('');
  }

  private section(title: string, tools: Tool[]): HTMLElement {
    const list = el('div', { class: 'wb-tools' });
    for (const tool of tools) {
      const key = SHORTCUTS[tool.id];
      const node = button('', () => this.use(tool, node), { class: `wb-tool is-${tool.kind}`, title: `${tool.label}${key ? ` (${key.toUpperCase()})` : ''}: ${toolTitle(tool)}`, 'aria-label': tool.label, 'data-tool': tool.id });
      node.append(el('span', { class: 'wb-tool-glyph', text: glyphOf(tool) }), el('span', { class: 'wb-tool-label', text: tool.label }));
      if (key) node.appendChild(el('kbd', { class: 'wb-tool-key', text: key.toUpperCase() }));
      list.appendChild(node);
    }
    return el('section', { class: 'wb-palette-section' }, [el('h3', { text: title }), list]);
  }

  private use(tool: Tool, node: HTMLButtonElement): void {
    if (tool.kind === 'connect') {
      if (this.armed === tool) return this.disarm();
      this.disarm();
      this.armed = tool;
      node.classList.add('is-armed');
      node.setAttribute('aria-pressed', 'true');
      this.owner.hint(`${tool.label}: click the source element, then the target. Esc cancels.`);
      return;
    }
    const selected = this.owner.selected();
    const where =
      selected && tool.containerKinds?.includes(selected.kind) ? `inside ${selected.id}` : tool.inGraph ? `in ${this.graphContainer ?? 'this diagram'}` : tool.topLevel ? 'at the top level' : 'in the layer';
    void inlinePrompt(this.owner.anchor(), `New ${tool.label.toLowerCase()} ${where}`, '', 'name').then((name) => {
      if (name === null) return;
      const request = createRequest(tool, { view: this.view, name, selected, graphContainer: this.graphContainer });
      if ('error' in request) return this.owner.hint(request.error);
      this.owner.submit([request], `${tool.label} ${name}`);
    });
  }
}

function toolTitle(tool: Tool): string {
  if (tool.kind === 'connect') return `Connect ${(tool.endKinds ?? []).join(' or ')}: click the source, then the target`;
  if (tool.needsContainer) return `Declared inside the selected ${(tool.containerKinds ?? []).join(' or ')}`;
  if (tool.containerKinds?.length) return `Declared inside the selected ${tool.containerKinds.join(' or ')}, else in the layer`;
  return tool.topLevel ? 'Declared at the top level of the file' : 'Declared in the layer of this view';
}

/** A two-letter glyph per tool, until the viewer's own icons are reused. */
function glyphOf(tool: Tool): string {
  const glyphs: Record<string, string> = {
    actor: '☺',
    entity: '▣',
    activity: '⚙',
    operational_capability: '★',
    function: 'ƒ',
    component: '▭',
    node: '▤',
    behavior_component: '▦',
    hardware_component: '▧',
    port_in: '▸',
    port_out: '▹',
    interface: '◇',
    mission: '◎',
    capability: '★',
    functional_chain: '⛓',
    state: '◯',
    mode: '◉',
    class: '▥',
    enumeration: '≡',
    data_type: '#',
    exchange_item: '✉',
    interaction: '→',
    functional_exchange: '⇢',
    component_exchange: '⇒',
    physical_link: '═',
    physical_exchange: '⇉',
    transition: '↷',
    message: '✉',
    association: '—',
  };
  return glyphs[tool.id] ?? tool.label.slice(0, 1);
}

/** One letter per tool, pressed with the canvas focused (V goes back to the pointer). */
const SHORTCUTS: Record<string, string> = {
  actor: 'a', entity: 'y', activity: 'f', function: 'f', component: 'c', node: 'n', behavior_component: 'b', hardware_component: 'h',
  port_in: 'p', port_out: 'o', interface: 'i', mission: 'm', capability: 'k', functional_chain: 'l', state: 's', mode: 'd',
  class: 'c', enumeration: 'u', data_type: 'd', exchange_item: 'x', interaction: 'e', functional_exchange: 'x', component_exchange: 'e',
  physical_link: 'e', physical_exchange: 'x', transition: 't', message: 'e', association: 'e', operational_capability: 'k',
};
