// The Suggestions tab beside Properties and Relations: what the compiler
// found the last edit (or a selection) touches and breaks, and the AI's
// proposals to resolve it, each compiled on a copy. Nothing is applied
// without a click; the pane never takes the focus.
import type { SuggestionView, ToPageAi } from '../src/ai/core/protocol';
import { button, clear, el } from './dom';
import type { AiPageHooks, AiStatus } from './ask-ai';
import { renderDiff } from './ai-diff';

type SuggestionsMessage = Extract<ToPageAi, { type: 'ai-suggestions' }>;

export interface SuggestionsPane {
  status(status: AiStatus): void;
  show(message: SuggestionsMessage): void;
  /** Bring the tab forward (an explicit "Analyse impact"). */
  reveal(): void;
}

export function createSuggestions(hooks: AiPageHooks): SuggestionsPane {
  const tabs = document.querySelector<HTMLElement>('.wb-right-tabs');
  const right = document.querySelector<HTMLElement>('.wb-right');
  const pane = el('div', { class: 'wb-pane wb-ai-pane', role: 'tabpanel', 'aria-label': 'AI suggestions' });
  const count = el('span', { class: 'wb-ai-count', hidden: true });
  const tab = el('button', { type: 'button', class: 'wb-right-tab wb-ai-tab', role: 'tab', title: 'AI proposals from the impact of your edits' }, ['Suggestions', count]);
  const pill = button('', () => reveal(), { class: 'wb-ai-pill', hidden: true, title: 'Show the AI suggestions' });
  hooks.sheet.appendChild(pill);
  let status: AiStatus = { configured: false, label: '', suggestOnEdit: 'onRequest' };
  let last: SuggestionsMessage | null = null;

  const isActive = () => tab.classList.contains('is-active');
  const activate = (on: boolean) => {
    tab.classList.toggle('is-active', on);
    tab.setAttribute('aria-selected', String(on));
    pane.classList.toggle('is-active', on);
    if (on) {
      for (const other of right?.querySelectorAll('.wb-pane') ?? []) if (other !== pane) other.classList.remove('is-active');
      for (const other of tabs?.querySelectorAll('.wb-right-tab') ?? []) if (other !== tab) {
        other.classList.remove('is-active');
        other.setAttribute('aria-selected', 'false');
      }
      pill.hidden = true;
    }
  };
  if (tabs && right) {
    tabs.appendChild(tab);
    right.appendChild(pane);
    tab.addEventListener('click', () => activate(true));
    // Another tab clicked: the shell shows its pane, this one steps back.
    tabs.addEventListener('click', (event) => {
      if (!(event.target as HTMLElement).closest('.wb-ai-tab')) activate(false);
    });
  }

  function reveal(): void {
    hooks.openDrawer();
    activate(true);
  }

  function render(): void {
    clear(pane);
    const selection = hooks.selection();
    pane.append(
      el('header', { class: 'wb-ai-pane-head' }, [
        el('strong', { text: '✦ AI suggestions' }),
        button(status.label || 'Configure', () => hooks.post({ type: 'ai-configure' }), { class: 'wb-ai-provider', title: 'The provider and model: click to configure' }),
      ]),
      el('p', { class: 'wb-ai-hint', text: `Proposals of an AI, each compiled on a copy before it is shown. Nothing changes until you apply one. After each edit: ${status.suggestOnEdit === 'afterEachEdit' ? 'the AI proposes' : status.suggestOnEdit === 'off' ? 'off' : 'the consequences are shown, the AI on request'}.` }),
    );
    pane.appendChild(
      button(selection.length ? `Analyse the impact of ${selection.length === 1 ? selection[0].name || selection[0].id : `${selection.length} elements`}` : 'Select an element to analyse its impact', () => {
        const ids = hooks.selection().map((e) => e.id);
        if (ids.length) hooks.post({ type: 'ai-impact', ids });
      }, { class: 'wb-ai-analyse', disabled: selection.length === 0 }),
    );
    const message = last;
    if (!message) {
      pane.appendChild(el('p', { class: 'wb-ai-empty', text: 'Edit the model: what an edit breaks (a trace, a requirement, the gate) shows here.' }));
      return;
    }
    pane.appendChild(el('p', { class: 'wb-ai-trigger', text: message.trigger.replace(/^./, (c) => c.toUpperCase()) }));
    if (message.change?.length) pane.appendChild(details('What changed', message.change));
    if (message.consequences.length) {
      pane.appendChild(el('ul', { class: 'wb-ai-consequences', 'aria-label': 'Consequences' }, message.consequences.map((c) => el('li', { text: c }))));
    }
    if (message.impact?.length) pane.appendChild(details(`arclang impact (${message.impact.length})`, message.impact));
    const state: Record<string, string> = {
      analysing: 'Analysing the impact with the compiler…',
      thinking: `Asking ${message.provider || 'the AI'} for proposals…`,
      none: message.consequences.length ? 'No proposal the compiler accepts.' : 'Nothing to resolve: the compiler finds no consequence.',
      off: message.error ?? 'The AI is off.',
      error: message.error ?? 'The AI failed.',
      idle: '',
      ready: '',
    };
    const line = state[message.state];
    if (line) pane.appendChild(el('p', { class: `wb-ai-state is-${message.state}`, role: 'status' }, [message.state === 'thinking' || message.state === 'analysing' ? el('span', { class: 'wb-ai-spinner', 'aria-hidden': 'true' }) : null, line]));
    if (message.canPropose && message.consequences.length && message.state !== 'thinking') {
      pane.appendChild(button('✦ Propose changes with AI', () => hooks.post({ type: 'ai-propose' }), { class: 'wb-primary wb-ai-propose' }));
    }
    for (const suggestion of message.suggestions) pane.appendChild(card(suggestion));
    if (message.dropped?.length) pane.appendChild(details(`Refused by the compiler (${message.dropped.length})`, message.dropped));
  }

  function card(suggestion: SuggestionView): HTMLElement {
    const p = suggestion.proposal;
    const node = el('article', { class: 'wb-ai-card', 'data-suggestion': suggestion.id }, [
      el('h4', { text: suggestion.title }),
      suggestion.resolves ? el('p', { class: 'wb-ai-resolves', text: `Resolves: ${suggestion.resolves}` }) : null,
      suggestion.rationale ? el('p', { class: 'wb-ai-rationale', text: suggestion.rationale }) : null,
      el('ol', { class: 'wb-ai-edits' }, p.edits.map((e) => el('li', { text: e }))),
      p.newFindings.length ? el('p', { class: 'wb-ai-note is-warning', text: `Would report: ${p.newFindings.slice(0, 3).join('; ')}` }) : null,
      p.ok ? null : el('p', { class: 'wb-ai-note is-error', text: p.error ?? 'refused' }),
    ]);
    const diff = p.diff.length ? renderDiff(p.diff, p.added, p.removed, 24) : null;
    if (diff) {
      diff.hidden = true;
      node.appendChild(diff);
    }
    node.appendChild(
      el('div', { class: 'wb-ai-actions' }, [
        button('Apply', () => hooks.post({ type: 'ai-suggestion', id: suggestion.id, action: 'apply' }), { class: 'wb-primary wb-ai-apply', disabled: !p.ok }),
        button('Preview', () => {
          if (diff) diff.hidden = !diff.hidden;
          hooks.post({ type: 'ai-suggestion', id: suggestion.id, action: 'diff' });
        }, { title: 'The diff here and in VS Code' }),
        button('Dismiss', () => hooks.post({ type: 'ai-suggestion', id: suggestion.id, action: 'dismiss' }), { class: 'wb-ai-reject' }),
      ]),
    );
    return node;
  }

  function details(title: string, lines: string[]): HTMLElement {
    return el('details', { class: 'wb-ai-details' }, [el('summary', { text: title }), el('ul', {}, lines.map((l) => el('li', { text: l })))]);
  }

  // The selection changes the "Analyse" button: re-rendered on clicks, cheaply.
  hooks.host.addEventListener('click', () => setTimeout(() => isActive() && render(), 50));
  render();

  return {
    status(next) {
      status = next;
      render();
    },
    show(message) {
      last = message;
      const waiting = message.suggestions.length + (message.canPropose && message.consequences.length ? 1 : 0);
      count.hidden = waiting === 0;
      count.textContent = String(message.suggestions.length || message.consequences.length);
      // Proposals waiting while the pane is not shown: a quiet pill on the canvas.
      const drawerShown = !!right && !right.hidden;
      pill.hidden = waiting === 0 || (isActive() && drawerShown);
      pill.textContent = message.suggestions.length ? `✦ ${message.suggestions.length} AI suggestion${message.suggestions.length > 1 ? 's' : ''}` : `✦ ${message.consequences.length} consequence${message.consequences.length > 1 ? 's' : ''} of your edit`;
      render();
    },
    reveal,
  };
}
