// Ask AI on the selection, as a programmer selects code: ⌘I (Ctrl+I), the
// context menu or the floating toolbar open a prompt by the selected boxes;
// the answer comes back as a proposal already checked by the compiler (the
// explanation, the edits, the diff of the source, the changed boxes marked
// on the diagram) with Apply, Refine and Reject. Only Apply changes the
// model, as one undoable edit. The text of the answer is shown as text.
import type { ElementRecord, Snapshot } from '../src/pure/workbench';
import type { FromPageAi, ProposalView, QuickAction, ToPageAi } from '../src/ai/core/protocol';
import { QUICK_ACTIONS } from '../src/ai/core/protocol';
import { button, clear, el } from './dom';
import type { MenuAction, MenuItem } from './menu';
import { renderDiff } from './ai-diff';
import { createSuggestions, SuggestionsPane } from './suggestions';

export interface AiPageHooks {
  post: (message: FromPageAi) => void;
  sheet: HTMLElement;
  host: HTMLElement;
  /** The selected elements: the multi-selection, else the one selected. */
  selection: () => ElementRecord[];
  drawingId: (id: string) => string;
  toast: (text: string, kind: 'info' | 'error') => void;
  openDrawer: () => void;
  snapshot: () => Snapshot | null;
}

export interface AiStatus {
  configured: boolean;
  label: string;
  suggestOnEdit: string;
  problem?: string;
}

let controller: AskController | null = null;

/** The AI side of the page; called once by main.ts (ai-design hook). */
export function installAi(hooks: AiPageHooks): void {
  loadStyles();
  const suggestions = createSuggestions(hooks);
  controller = new AskController(hooks, suggestions);
  hooks.post({ type: 'ai-hello' });
}

/** The context menu with "Ask AI…" first when something is selected (ai-design hook in editing.ts). */
export function withAi(items: MenuItem[]): MenuItem[] {
  if (!controller || controller.selection().length === 0) return items;
  return [...controller.menuActions(), 'separator', ...items];
}

/** The floating toolbar with the Ask AI button (ai-design hook in editing.ts). */
export function withAiMini<T extends MenuAction & { glyph: string }>(actions: T[]): (T | (MenuAction & { glyph: string }))[] {
  if (!controller || controller.selection().length === 0) return actions;
  return [{ glyph: '✦', label: 'Ask AI about the selection', shortcut: '⌘I', run: () => controller?.open() }, ...actions];
}

/** The page's AI stylesheet, next to the workbench's (no change to the page's HTML). */
function loadStyles(): void {
  const base = document.querySelector<HTMLLinkElement>('link[href*="workbench.css"]');
  if (!base || document.querySelector('link[data-ai-styles]')) return;
  const link = document.createElement('link');
  link.rel = 'stylesheet';
  link.href = base.href.replace(/workbench\.css(\?.*)?$/, 'ai.css');
  link.dataset.aiStyles = '';
  base.after(link);
}

let sessionCounter = 0;

class AskController {
  private status: AiStatus = { configured: false, label: '', suggestOnEdit: 'onRequest' };
  private panel: HTMLElement | null = null;
  private session: string | null = null;
  private targets: ElementRecord[] = [];
  private streamed = '';
  private marked: string[] = [];

  constructor(
    private readonly hooks: AiPageHooks,
    private readonly suggestions: SuggestionsPane,
  ) {
    window.addEventListener('message', (event: MessageEvent<ToPageAi>) => this.receive(event.data));
    // ⌘I / Ctrl+I as VS Code's inline chat: before the canvas's own keys.
    document.addEventListener(
      'keydown',
      (event) => {
        const typing = event.target instanceof HTMLInputElement || event.target instanceof HTMLTextAreaElement || event.target instanceof HTMLSelectElement;
        if (typing || !(event.metaKey || event.ctrlKey) || event.altKey || event.shiftKey) return;
        if (event.key.toLowerCase() !== 'i' && event.code !== 'KeyI') return;
        event.preventDefault();
        event.stopPropagation();
        this.open();
      },
      true,
    );
  }

  selection(): ElementRecord[] {
    return this.hooks.selection();
  }

  menuActions(): MenuAction[] {
    return [
      { label: 'Ask AI…', shortcut: '⌘I', run: () => this.open() },
      { label: 'Analyse impact with AI', run: () => this.analyse() },
    ];
  }

  analyse(): void {
    const ids = this.selection().map((e) => e.id);
    if (ids.length === 0) return;
    this.hooks.post({ type: 'ai-impact', ids });
    this.suggestions.reveal();
  }

  private receive(message: ToPageAi | undefined): void {
    if (!message || typeof message.type !== 'string' || !message.type.startsWith('ai-')) return;
    switch (message.type) {
      case 'ai-status':
        this.status = { configured: message.configured, label: message.label, suggestOnEdit: message.suggestOnEdit, problem: message.problem };
        this.suggestions.status(this.status);
        if (this.panel) this.renderHeader();
        return;
      case 'ai-progress':
        if (message.session !== this.session) return;
        this.streamed += message.text;
        this.renderProgress(message.chars);
        return;
      case 'ai-proposal':
        if (message.session !== this.session) return;
        this.renderProposal(message.proposal, message.provider);
        return;
      case 'ai-error':
        if (message.session !== this.session) return;
        this.renderError(message.error);
        return;
      case 'ai-applied':
        if (message.session !== this.session) return;
        if (message.ok) {
          this.hooks.toast('Applied: the AI proposal, as one change. Undo with ⌘Z in the editor.', 'info');
          this.close(false);
        } else this.renderError(`Not applied: ${message.error ?? 'refused'}`);
        return;
      case 'ai-suggestions':
        this.suggestions.show(message);
        return;
      default:
        return;
    }
  }

  // --- the panel -------------------------------------------------------------

  open(): void {
    const selected = this.selection();
    if (selected.length === 0) {
      this.hooks.toast('Ask AI: select one or more boxes or lines first (Shift+click or a band adds to the selection).', 'info');
      return;
    }
    this.close(true);
    this.targets = selected;
    this.panel = el('section', { class: 'wb-ai', role: 'dialog', 'aria-label': 'Ask AI about the selection' });
    this.panel.addEventListener('keydown', (event) => {
      if (event.key !== 'Escape' || event.target instanceof HTMLTextAreaElement) return;
      event.preventDefault();
      event.stopPropagation();
      this.reject();
    });
    this.hooks.sheet.appendChild(this.panel);
    this.renderAsk();
    this.place();
  }

  /** By the selection: below it when there is room, else above; inside the sheet. */
  private place(): void {
    const panel = this.panel;
    if (!panel) return;
    const rects = this.targets
      .map((e) => this.hooks.host.querySelector(`[data-id="${CSS.escape(this.hooks.drawingId(e.id))}"]`))
      .map((node) => node?.querySelector(':scope > .av-shape, :scope > .av-line') ?? node)
      .filter((n): n is Element => !!n)
      .map((n) => n.getBoundingClientRect())
      .filter((r) => r.width > 0 || r.height > 0);
    const frame = this.hooks.sheet.getBoundingClientRect();
    const box = panel.getBoundingClientRect();
    if (rects.length === 0) {
      panel.style.left = `${Math.max(8, frame.width - box.width - 12)}px`;
      panel.style.top = '44px';
      return;
    }
    const left = Math.min(...rects.map((r) => r.left));
    const bottom = Math.max(...rects.map((r) => r.bottom));
    const top = Math.min(...rects.map((r) => r.top));
    let y = bottom - frame.top + 10;
    if (y + box.height > frame.height - 8) y = Math.max(8, top - frame.top - box.height - 10);
    const x = Math.max(8, Math.min(left - frame.left, frame.width - box.width - 8));
    panel.style.left = `${x}px`;
    panel.style.top = `${Math.max(8, Math.min(y, frame.height - box.height - 8))}px`;
  }

  close(cancel: boolean): void {
    if (cancel && this.session) this.hooks.post({ type: 'ai-cancel', session: this.session });
    this.panel?.remove();
    this.panel = null;
    this.session = null;
    this.unmark();
  }

  private names(): string {
    const names = this.targets.map((e) => e.name || e.id);
    return names.length > 3 ? `${names.slice(0, 3).join(', ')} and ${names.length - 3} more` : names.join(', ');
  }

  private renderHeader(): void {
    const panel = this.panel;
    if (!panel) return;
    const old = panel.querySelector('.wb-ai-head');
    const head = el('header', { class: 'wb-ai-head' }, [
      el('span', { class: 'wb-ai-glyph', 'aria-hidden': 'true', text: '✦' }),
      el('div', { class: 'wb-ai-title' }, [el('strong', { text: 'Ask AI' }), el('span', { class: 'wb-ai-about', text: this.names() })]),
      button(this.status.label || 'AI', () => this.hooks.post({ type: 'ai-configure' }), { class: 'wb-ai-provider', title: 'The provider and model that answer: click to configure' }),
      button('✕', () => this.close(true), { class: 'wb-icon-button', title: 'Close (Esc)', 'aria-label': 'Close' }),
    ]);
    if (old) old.replaceWith(head);
    else panel.prepend(head);
  }

  private renderAsk(prefill = ''): void {
    const panel = this.panel!;
    clear(panel);
    this.renderHeader();
    if (!this.status.configured) {
      panel.append(
        el('p', { class: 'wb-ai-note is-warning', text: this.status.problem ? `Not configured: ${this.status.problem}.` : 'No AI provider is configured.' }),
        el('div', { class: 'wb-ai-actions' }, [button('Configure AI', () => this.hooks.post({ type: 'ai-configure' }), { class: 'wb-primary' })]),
      );
    }
    const field = el('textarea', { class: 'wb-ai-input', rows: '2', placeholder: 'Ask to modify, correct, complete… (Enter sends, Shift+Enter breaks the line)', 'aria-label': 'Your request' });
    field.value = prefill;
    const send = (action?: QuickAction) => {
      const prompt = field.value.trim();
      if (!prompt && !action) return field.focus();
      this.send(prompt, action);
    };
    field.addEventListener('keydown', (event) => {
      if (event.key === 'Enter' && !event.shiftKey && !event.isComposing) {
        event.preventDefault();
        send();
      } else if (event.key === 'Escape') {
        event.preventDefault();
        this.close(true);
      }
      event.stopPropagation();
    });
    const chips = el('div', { class: 'wb-ai-chips', role: 'group', 'aria-label': 'Quick actions' }, QUICK_ACTIONS.map((a) => button(a.label, () => send(a.id), { class: 'wb-ai-chip', title: a.prompt })));
    panel.append(field, el('div', { class: 'wb-ai-row' }, [chips, button('Send', () => send(), { class: 'wb-primary wb-ai-send', title: 'Send (Enter)' })]));
    field.focus();
  }

  private send(prompt: string, action?: QuickAction, refine = false): void {
    if (!refine) this.session = `w${Date.now().toString(36)}${++sessionCounter}`;
    this.streamed = '';
    const session = this.session!;
    if (refine) this.hooks.post({ type: 'ai-refine', session, prompt });
    else this.hooks.post({ type: 'ai-ask', session, ids: this.targets.map((e) => e.id), prompt, action });
    const panel = this.panel!;
    clear(panel);
    this.renderHeader();
    const asked = action ? QUICK_ACTIONS.find((a) => a.id === action)?.label ?? action : '';
    panel.append(
      el('p', { class: 'wb-ai-asked', text: [asked, prompt].filter(Boolean).join(': ') }),
      el('div', { class: 'wb-ai-progress', role: 'status', 'aria-live': 'polite' }, [el('span', { class: 'wb-ai-spinner', 'aria-hidden': 'true' }), el('span', { class: 'wb-ai-progress-text', text: `Asking ${this.status.label || 'the AI'}…` })]),
      el('pre', { class: 'wb-ai-stream', hidden: true }),
      el('div', { class: 'wb-ai-actions' }, [button('Stop', () => this.close(true))]),
    );
    this.place();
  }

  private renderProgress(chars: number): void {
    const text = this.panel?.querySelector('.wb-ai-progress-text');
    if (text) text.textContent = `${this.status.label || 'The AI'} is answering… ${chars} characters`;
    const stream = this.panel?.querySelector<HTMLElement>('.wb-ai-stream');
    // The prose as it comes; a JSON or code block is not shown raw.
    const prose = this.streamed.replace(/```[\s\S]*?(```|$)/g, '').trim();
    if (stream && prose) {
      stream.hidden = false;
      stream.textContent = prose.slice(-600);
    }
  }

  private renderError(error: string): void {
    const panel = this.panel;
    if (!panel) return;
    clear(panel);
    this.renderHeader();
    panel.append(
      el('p', { class: 'wb-ai-note is-error', text: error }),
      el('div', { class: 'wb-ai-actions' }, [button('Try again', () => this.renderAsk()), button('Configure AI', () => this.hooks.post({ type: 'ai-configure' })), button('Close', () => this.close(true))]),
    );
  }

  private renderProposal(proposal: ProposalView, provider: string): void {
    const panel = this.panel;
    if (!panel) return;
    clear(panel);
    this.renderHeader();
    panel.append(el('p', { class: 'wb-ai-badge', text: `AI proposal from ${provider}: ${proposal.explainOnly ? 'an answer, nothing to apply' : proposal.ok ? 'compiled on a copy, not applied' : 'refused by the compiler'}` }));
    if (proposal.explanation) panel.append(el('p', { class: 'wb-ai-explanation', text: proposal.explanation }));
    if (!proposal.ok) panel.append(el('p', { class: 'wb-ai-note is-error', text: proposal.error ?? 'refused' }));
    if (proposal.edits.length) panel.append(el('ol', { class: 'wb-ai-edits' }, proposal.edits.map((e) => el('li', { text: e }))));
    if (proposal.created.length) panel.append(el('p', { class: 'wb-ai-created', text: `New: ${proposal.created.join(', ')}` }));
    if (proposal.newFindings.length) panel.append(el('div', { class: 'wb-ai-note is-warning' }, [el('strong', { text: 'The compiler would report: ' }), ...proposal.newFindings.slice(0, 4).map((f) => el('div', { text: f }))]));
    if (proposal.diff.length) panel.append(renderDiff(proposal.diff, proposal.added, proposal.removed));
    const actions = el('div', { class: 'wb-ai-actions' });
    if (proposal.ok && !proposal.explainOnly && proposal.diff.length) {
      actions.append(button('Apply', () => this.apply(), { class: 'wb-primary wb-ai-apply', title: 'Apply as one undoable change' }), button('Open diff', () => this.hooks.post({ type: 'ai-diff', session: this.session! }), { title: 'The whole diff in VS Code' }));
    }
    actions.append(button('Refine…', () => this.refineField(), { class: 'wb-ai-refine' }), button(proposal.explainOnly ? 'Close' : 'Reject', () => this.reject(), { class: 'wb-ai-reject' }));
    panel.append(actions);
    this.mark(proposal);
    this.place();
    panel.querySelector<HTMLButtonElement>('.wb-ai-apply, .wb-ai-refine')?.focus();
  }

  private refineField(): void {
    const panel = this.panel;
    if (!panel || panel.querySelector('.wb-ai-refine-field')) return;
    const field = el('textarea', { class: 'wb-ai-input wb-ai-refine-field', rows: '2', placeholder: 'A follow-up: what to change in the proposal (Enter sends)', 'aria-label': 'Refine the proposal' });
    field.addEventListener('keydown', (event) => {
      event.stopPropagation();
      if (event.key === 'Enter' && !event.shiftKey && field.value.trim()) {
        event.preventDefault();
        this.unmark();
        this.send(field.value.trim(), undefined, true);
      }
      if (event.key === 'Escape') field.remove();
    });
    panel.querySelector('.wb-ai-actions')?.before(field);
    field.focus();
  }

  private apply(): void {
    if (!this.session) return;
    const button = this.panel?.querySelector<HTMLButtonElement>('.wb-ai-apply');
    if (button) {
      button.disabled = true;
      button.textContent = 'Applying…';
    }
    this.hooks.post({ type: 'ai-apply', session: this.session });
  }

  private reject(): void {
    // Esc while the AI is still answering cancels the question.
    if (this.session) this.hooks.post({ type: this.panel?.querySelector('.wb-ai-progress') ? 'ai-cancel' : 'ai-reject', session: this.session });
    this.session = null;
    this.close(false);
  }

  /** The boxes the proposal changes or removes, marked on the drawing. */
  private mark(proposal: ProposalView): void {
    this.unmark();
    const mark = (id: string, cls: string) => {
      const node = this.hooks.host.querySelector(`[data-id="${CSS.escape(this.hooks.drawingId(id))}"]`);
      if (!node) return;
      node.classList.add(cls);
      this.marked.push(id);
    };
    for (const id of proposal.changed) mark(id, 'wb-ai-mark');
    for (const id of proposal.removedIds) mark(id, 'wb-ai-mark-removed');
  }

  private unmark(): void {
    for (const node of this.hooks.host.querySelectorAll('.wb-ai-mark, .wb-ai-mark-removed')) node.classList.remove('wb-ai-mark', 'wb-ai-mark-removed');
    this.marked = [];
  }
}
