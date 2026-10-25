// The AI chat page. Everything it shows is built from data (never from
// markup strings): a transcript, a proposal card with the compiler-checked
// diff and Apply, code blocks of Modelica answers with their "not verified"
// label, and a composer. It holds no key and makes no request: the host does.
import type { FromChatPage, ToChatPage } from '../../src/ai/chat/protocol';
import type { ProposalView } from '../../src/ai/core/protocol';
import { QUICK_ACTIONS } from '../../src/ai/core/protocol';
import { el } from '../dom';

const vscode = acquireVsCodeApi() as unknown as { postMessage: (m: FromChatPage) => void };
const post = (message: FromChatPage) => vscode.postMessage(message);

type State = Extract<ToChatPage, { type: 'chat-state' }>;
let state: State | null = null;
let busy = false;
const turns = new Map<string, { body: HTMLElement; text: string; node: HTMLElement }>();

const root = document.getElementById('chat') as HTMLElement;
const header = el('header', { class: 'chat-head' });
const context = el('div', { class: 'chat-context', role: 'status', 'aria-live': 'polite' });
const banner = el('div', { class: 'chat-banner', hidden: true });
const transcript = el('div', { class: 'chat-transcript', role: 'log', 'aria-live': 'polite', 'aria-label': 'Conversation' });
const quick = el('div', { class: 'chat-quick' });
const input = el('textarea', { class: 'chat-input', rows: '3', placeholder: 'Ask about the model, or ask for a change', 'aria-label': 'Message to the assistant' });
const sendButton = el('button', { type: 'button', class: 'chat-send', text: 'Send' });
const stopButton = el('button', { type: 'button', class: 'chat-stop', text: 'Stop', hidden: true });
const hint = el('span', { class: 'chat-hint', text: 'Enter sends, Shift+Enter breaks the line' });
root.append(header, context, banner, transcript, quick, el('div', { class: 'chat-composer' }, [input, el('div', { class: 'chat-actions' }, [hint, stopButton, sendButton])]));

function renderHeader(): void {
  header.textContent = '';
  const provider = el('button', { type: 'button', class: 'chat-provider', title: 'Choose the AI provider and model', text: state ? state.provider : 'AI' });
  provider.addEventListener('click', () => post({ type: 'chat-configure' }));
  const fresh = el('button', { type: 'button', class: 'chat-icon', title: 'New conversation', 'aria-label': 'New conversation', text: '+' });
  fresh.addEventListener('click', () => {
    transcript.textContent = '';
    turns.clear();
    welcome();
    post({ type: 'chat-new' });
  });
  header.append(el('span', { class: 'chat-title', text: 'AI chat' }), provider, fresh);
}

function renderContext(): void {
  context.textContent = '';
  if (!state) return;
  const c = state.context;
  const kindLabel = c.kind === 'arc' ? 'ArcLang' : c.kind === 'modelica' ? 'Modelica' : 'no model file';
  context.append(el('span', { class: `chip kind-${c.kind}`, text: kindLabel }));
  if (c.file) context.append(el('span', { class: 'chat-file', text: c.file }));
  if (c.selectionLines > 0) context.append(el('span', { class: 'chip', text: `${c.selectionLines} line${c.selectionLines > 1 ? 's' : ''} selected${c.ids.length ? `: ${c.ids.slice(0, 3).join(', ')}${c.ids.length > 3 ? '…' : ''}` : ''}` }));
  if (c.kind === 'modelica') context.append(el('span', { class: 'chip warn', title: 'There is no Modelica compiler in the browser: answers are not checked.', text: 'answers not verified' }));
  if (c.kind === 'arc') context.append(el('span', { class: 'chip ok', title: 'The compiler checks every edit on a copy before you see it.', text: 'edits compiler-checked' }));
}

function renderBanner(): void {
  banner.textContent = '';
  if (!state || !state.problem) {
    banner.hidden = true;
    return;
  }
  banner.hidden = false;
  banner.append(el('p', { text: state.problem }));
  const choose = el('button', { type: 'button', text: 'Choose a provider' });
  choose.addEventListener('click', () => post({ type: 'chat-configure' }));
  if (/API key/.test(state.problem)) {
    const key = el('button', { type: 'button', class: 'primary', text: 'Set API key' });
    key.addEventListener('click', () => post({ type: 'chat-setkey' }));
    banner.append(key, ' ');
  }
  banner.append(choose);
}

function renderQuick(): void {
  quick.textContent = '';
  if (!state || state.context.kind !== 'arc') return;
  for (const action of QUICK_ACTIONS) {
    const chip = el('button', { type: 'button', class: 'chat-chip', text: action.label, title: action.prompt });
    chip.addEventListener('click', () => {
      input.value = action.prompt;
      input.focus();
    });
    quick.append(chip);
  }
}

function welcome(): void {
  const note = el('div', { class: 'chat-welcome' }, [
    el('h2', { text: 'Ask about your model' }),
    el('p', { text: 'Select part of an .arc file and ask: edits come back checked by the compiler, as a diff you apply in one click and undo with Ctrl/Cmd+Z.' }),
    el('p', { text: 'On a .mo file the AI explains, writes and fixes Modelica. Those answers are not verified: there is no Modelica compiler in the browser.' }),
  ]);
  transcript.append(note);
}

function scroll(): void {
  transcript.scrollTop = transcript.scrollHeight;
}

/** Text with ``` fences: prose paragraphs and code blocks, built node by node. */
function renderText(host: HTMLElement, text: string, withCodeActions: boolean): void {
  host.textContent = '';
  const parts = text.split(/```/);
  parts.forEach((part, index) => {
    if (index % 2 === 0) {
      for (const paragraph of part.split(/\n{2,}/)) {
        const trimmed = paragraph.trim();
        if (!trimmed) continue;
        const p = el('p');
        trimmed.split(/(`[^`\n]+`)/).forEach((piece) => p.append(piece.startsWith('`') && piece.endsWith('`') && piece.length > 2 ? el('code', { text: piece.slice(1, -1) }) : piece));
        host.append(p);
      }
      return;
    }
    const newline = part.indexOf('\n');
    const lang = newline >= 0 ? part.slice(0, newline).trim() : '';
    const code = (newline >= 0 ? part.slice(newline + 1) : part).replace(/\n$/, '');
    const block = el('div', { class: 'chat-code' }, [el('div', { class: 'chat-code-head' }, [el('span', { text: lang || 'code' })]), el('pre', {}, [el('code', { text: code })])]);
    const bar = block.firstElementChild as HTMLElement;
    const copy = el('button', { type: 'button', text: 'Copy' });
    copy.addEventListener('click', () => post({ type: 'chat-copy', code }));
    bar.append(copy);
    if (withCodeActions && /^(modelica|mo)$/i.test(lang)) {
      for (const [label, where] of [['Replace selection', 'selection'], ['Insert at cursor', 'cursor'], ['Replace file', 'file']] as const) {
        const b = el('button', { type: 'button', text: label });
        b.addEventListener('click', () => post({ type: 'chat-insert', code, where }));
        bar.append(b);
      }
    }
    host.append(block);
  });
}

function addTurn(id: string, who: 'you' | 'ai', meta: string): { body: HTMLElement; node: HTMLElement } {
  const body = el('div', { class: 'chat-body' });
  const node = el('article', { class: `chat-turn ${who}`, 'data-turn': id }, [el('div', { class: 'chat-meta', text: meta }), body]);
  transcript.append(node);
  scroll();
  return { body, node };
}

function setBusy(value: boolean): void {
  busy = value;
  sendButton.hidden = value;
  stopButton.hidden = !value;
  input.disabled = false;
}

function proposalCard(id: string, session: string, proposal: ProposalView, provider: string): HTMLElement {
  const card = el('div', { class: 'chat-proposal' });
  card.append(el('div', { class: 'chat-explanation' }));
  renderText(card.firstElementChild as HTMLElement, proposal.explanation, false);
  if (!proposal.ok) {
    card.append(el('div', { class: 'chat-refused', role: 'alert' }, [el('strong', { text: 'The compiler refused the change. ' }), proposal.error ?? '']));
  } else if (proposal.explainOnly) {
    card.append(el('div', { class: 'chat-note', text: 'No change proposed: an explanation only.' }));
  } else {
    card.append(el('div', { class: 'chat-checked', text: `Checked by the compiler on a copy: ${proposal.edits.length} edit${proposal.edits.length === 1 ? '' : 's'}, +${proposal.added} -${proposal.removed} lines${proposal.newFindings.length ? `, ${proposal.newFindings.length} new finding${proposal.newFindings.length > 1 ? 's' : ''}` : ''}.` }));
    if (proposal.edits.length) card.append(el('ul', { class: 'chat-edits' }, proposal.edits.map((edit) => el('li', { text: edit }))));
    if (proposal.newFindings.length) card.append(el('ul', { class: 'chat-findings' }, proposal.newFindings.slice(0, 6).map((f) => el('li', { text: f }))));
    if (proposal.diff.length) {
      const diff = el('pre', { class: 'chat-diff', 'aria-label': 'Diff of the model source' });
      for (const line of proposal.diff.slice(0, 160)) diff.append(el('span', { class: line.startsWith('+') ? 'add' : line.startsWith('-') ? 'del' : line.startsWith('@@') ? 'hunk' : 'ctx', text: `${line}\n` }));
      card.append(diff);
    }
  }
  const actions = el('div', { class: 'chat-proposal-actions' });
  const status = el('span', { class: 'chat-status', role: 'status' });
  if (proposal.ok && !proposal.explainOnly) {
    const apply = el('button', { type: 'button', class: 'primary', text: 'Apply' });
    const undo = el('button', { type: 'button', text: 'Undo', hidden: true });
    const view = el('button', { type: 'button', text: 'Open diff' });
    const reject = el('button', { type: 'button', text: 'Reject' });
    apply.addEventListener('click', () => {
      apply.disabled = true;
      post({ type: 'chat-apply', session });
    });
    undo.addEventListener('click', () => {
      post({ type: 'chat-undo' });
      status.textContent = 'Undone.';
      undo.hidden = true;
    });
    view.addEventListener('click', () => post({ type: 'chat-diff', session }));
    reject.addEventListener('click', () => {
      post({ type: 'chat-reject', session });
      card.classList.add('rejected');
      actions.textContent = '';
      actions.append(el('span', { class: 'chat-status', text: 'Rejected.' }));
    });
    card.dataset.session = session;
    (card as unknown as { _undo: HTMLElement; _status: HTMLElement; _apply: HTMLElement })._undo = undo;
    (card as unknown as { _status: HTMLElement })._status = status;
    (card as unknown as { _apply: HTMLElement })._apply = apply;
    actions.append(apply, undo, view, reject);
  }
  const refine = el('button', { type: 'button', text: 'Refine…' });
  refine.addEventListener('click', () => {
    pendingRefine = session;
    input.placeholder = 'Refine the proposal: what should change?';
    input.focus();
  });
  actions.append(refine, status, el('span', { class: 'chat-provider-note', text: provider }));
  card.append(actions);
  void id;
  return card;
}

let pendingRefine: string | null = null;

function send(): void {
  const text = input.value.trim();
  if (!text || busy) return;
  input.value = '';
  const turn = addTurn(`u${Date.now()}`, 'you', 'You');
  renderText(turn.body, text, false);
  if (pendingRefine) {
    post({ type: 'chat-refine', session: pendingRefine, text });
    pendingRefine = null;
    input.placeholder = 'Ask about the model, or ask for a change';
  } else {
    post({ type: 'chat-send', text });
  }
  setBusy(true);
}

sendButton.addEventListener('click', send);
stopButton.addEventListener('click', () => post({ type: 'chat-cancel' }));
input.addEventListener('keydown', (event) => {
  if (event.key === 'Enter' && !event.shiftKey && !event.isComposing) {
    event.preventDefault();
    send();
  }
});

window.addEventListener('message', (event: MessageEvent<ToChatPage>) => {
  const message = event.data;
  switch (message.type) {
    case 'chat-state':
      state = message;
      renderHeader();
      renderContext();
      renderBanner();
      renderQuick();
      return;
    case 'chat-start': {
      transcript.querySelector('.chat-welcome')?.remove();
      const meta = `${message.label}${message.file ? ` · ${message.file}` : ''}`;
      const turn = addTurn(message.id, 'ai', meta);
      turn.body.append(el('p', { class: 'chat-wait', text: 'Thinking…' }));
      turns.set(message.id, { ...turn, text: '' });
      setBusy(true);
      return;
    }
    case 'chat-delta': {
      const turn = turns.get(message.id);
      if (!turn) return;
      turn.text += message.text;
      renderText(turn.body, turn.text, false);
      scroll();
      return;
    }
    case 'chat-proposal': {
      const turn = turns.get(message.id);
      if (!turn) return;
      turn.body.textContent = '';
      turn.body.append(proposalCard(message.id, message.session, message.proposal, message.provider));
      setBusy(false);
      scroll();
      return;
    }
    case 'chat-done': {
      const turn = turns.get(message.id);
      if (!turn) return;
      turn.body.textContent = '';
      renderText(turn.body, message.text, message.kind === 'modelica');
      if (message.notVerified) {
        turn.node.classList.add('unverified');
        turn.body.prepend(el('div', { class: 'chat-unverified', role: 'note' }, [el('strong', { text: message.notVerified.label }), ' ', message.notVerified.where]));
      }
      setBusy(false);
      scroll();
      return;
    }
    case 'chat-error': {
      const turn = turns.get(message.id);
      const host = turn?.body ?? addTurn(message.id, 'ai', 'Assistant').body;
      const failure = el('div', { class: 'chat-error', role: 'alert' }, [el('strong', { text: 'Not answered. ' }), message.error]);
      if (message.fix) {
        const fix = el('button', { type: 'button', text: message.fix === 'key' ? 'Set API key' : 'Choose a provider' });
        fix.addEventListener('click', () => post({ type: message.fix === 'key' ? 'chat-setkey' : 'chat-configure' }));
        failure.append(' ', fix);
      }
      if (turn && turn.text) host.append(failure);
      else {
        host.textContent = '';
        host.append(failure);
      }
      setBusy(false);
      scroll();
      return;
    }
    case 'chat-applied': {
      const card = transcript.querySelector<HTMLElement>(`.chat-proposal[data-session="${CSS.escape(message.session)}"]`) as (HTMLElement & { _undo?: HTMLElement; _status?: HTMLElement; _apply?: HTMLElement }) | null;
      if (!card) return;
      if (card._status) card._status.textContent = message.ok ? 'Applied as one change. Ctrl/Cmd+Z undoes it.' : `Not applied: ${message.error ?? 'refused'}`;
      if (card._undo) card._undo.hidden = !message.ok;
      if (card._apply) (card._apply as HTMLButtonElement).disabled = message.ok;
      return;
    }
    case 'chat-note':
      transcript.append(el('div', { class: 'chat-note', text: message.text }));
      scroll();
      return;
  }
});

renderHeader();
welcome();
post({ type: 'chat-hello' });
