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
const hint = el('span', { class: 'chat-hint', text: 'Enter sends · Shift+Enter new line · @ attaches a file' });
const attached = el('div', { class: 'chat-attached', 'aria-label': 'Attached files' });
const fileInput = el('input', { type: 'file', multiple: true, hidden: true, accept: '.txt,.md,.csv,.tsv,.json,.yaml,.yml,.xml,.reqif,.reqifz,.docx,.pdf,.arc,.sysml,.mo,.html' }) as HTMLInputElement;
const attachButton = el('button', { type: 'button', class: 'chat-icon chat-attach', title: 'Attach files from this computer: requirements (Word, PDF, ReqIF, CSV, text) or models', 'aria-label': 'Attach files', text: '📎' });
const mentionButton = el('button', { type: 'button', class: 'chat-icon', title: 'Attach files of the project (@)', 'aria-label': 'Attach files of the project', text: '@' });
const composer = el('div', { class: 'chat-composer' }, [attached, input, el('div', { class: 'chat-actions' }, [attachButton, mentionButton, fileInput, hint, stopButton, sendButton])]);
root.append(header, context, banner, transcript, quick, composer);

/** The text of attached files is read here (Word, PDF...) by a module loaded on the first attachment. */
let extractor: Promise<{ extractText: (name: string, bytes: Uint8Array, worker: string) => Promise<string> }> | null = null;
async function attachBytes(name: string, bytes: Uint8Array): Promise<void> {
  const chip = el('span', { class: 'chat-attachment reading', text: `${name} · reading…` });
  attached.append(chip);
  try {
    extractor ??= import(document.body.dataset.extract ?? '') as Promise<{ extractText: (name: string, bytes: Uint8Array, worker: string) => Promise<string> }>;
    const text = await (await extractor).extractText(name, bytes, document.body.dataset.worker ?? '');
    post({ type: 'chat-attach', name, text: text.slice(0, 2_000_000) });
  } catch (error) {
    transcript.append(el('div', { class: 'chat-note', text: `${name} was not attached: ${error instanceof Error ? error.message : String(error)}` }));
    scroll();
  } finally {
    chip.remove();
  }
}
async function attachFiles(files: FileList | File[]): Promise<void> {
  for (const file of Array.from(files)) {
    if (file.size > 25_000_000) {
      transcript.append(el('div', { class: 'chat-note', text: `${file.name} is too large to attach (25 MB at most).` }));
      continue;
    }
    await attachBytes(file.name, new Uint8Array(await file.arrayBuffer()));
  }
}
attachButton.addEventListener('click', () => fileInput.click());
fileInput.addEventListener('change', () => {
  if (fileInput.files) void attachFiles(fileInput.files);
  fileInput.value = '';
});
mentionButton.addEventListener('click', () => post({ type: 'chat-pick-workspace' }));
root.addEventListener('dragover', (event) => {
  if (!event.dataTransfer?.types.includes('Files')) return;
  event.preventDefault();
  root.classList.add('dropping');
});
root.addEventListener('dragleave', (event) => {
  if (event.target === root || !root.contains(event.relatedTarget as Node)) root.classList.remove('dropping');
});
root.addEventListener('drop', (event) => {
  root.classList.remove('dropping');
  if (!event.dataTransfer?.files.length) return;
  event.preventDefault();
  void attachFiles(event.dataTransfer.files);
});

function renderAttachments(): void {
  attached.querySelectorAll('.chat-attachment:not(.reading)').forEach((n) => n.remove());
  for (const file of state?.attachments ?? []) {
    const chip = el('span', { class: 'chat-attachment', title: `${file.chars.toLocaleString()} characters${file.truncated ? ' (the start is sent: the files together are cut to fit one question)' : ''}` }, [
      el('span', { text: `📄 ${file.name}${file.truncated ? ' (start)' : ''}` }),
    ]);
    const remove = el('button', { type: 'button', class: 'chat-attachment-remove', 'aria-label': `Remove ${file.name}`, text: '×' });
    remove.addEventListener('click', () => post({ type: 'chat-detach', name: file.name }));
    chip.append(remove);
    attached.append(chip);
  }
}

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
  if (c.kind === 'modelica') context.append(el('span', { class: 'chip warn', title: 'The chat runs no Modelica: OpenModelica checks the code when a test case simulates it (ArcLang: Run simulations).', text: 'answers not verified' }));
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

/** Where to start, for someone who never modelled a system: each card fills the question. */
const STARTERS: { title: string; prompt: string; attach?: boolean }[] = [
  { title: 'Model a system from my requirements', prompt: 'Build the model of the system described by the attached requirements: keep their ids, add the functions, components, ports and traces.', attach: true },
  { title: 'Model a system from a description', prompt: 'Model this system: ' },
  { title: 'Explain systems modelling simply', prompt: 'Explain simply what a system model is, the Arcadia levels, and how I start, as if I had never done it.' },
  { title: 'Make my model simulable', prompt: 'What does my model need to be simulated with OpenModelica? Add it step by step.' },
  { title: 'Generate embedded code', prompt: 'How do I get embedded C (or VHDL for an FPGA) from my model and test it? Guide me step by step.' },
  { title: 'Apply the CESAM method', prompt: 'Apply the CESAM method to my model: what is missing in each vision? Add it step by step.' },
];

function welcome(): void {
  const cards = el('div', { class: 'chat-starters' });
  for (const starter of STARTERS) {
    const card = el('button', { type: 'button', class: 'chat-starter', text: starter.title });
    card.addEventListener('click', () => {
      input.value = starter.prompt;
      input.focus();
      input.setSelectionRange(input.value.length, input.value.length);
      if (starter.attach && !(state?.attachments.length)) fileInput.click();
    });
    cards.append(card);
  }
  const note = el('div', { class: 'chat-welcome' }, [
    el('h2', { text: 'What do you want to build?' }),
    el('p', { text: 'Describe your system or attach its requirements (Word, PDF, ReqIF, Excel as CSV, text): the assistant writes the ArcLang model, checks it with the compiler, and writes the change in the file for you to Keep or Undo. No modelling experience needed: ask, it guides you step by step.' }),
    cards,
  ]);
  transcript.append(note);
}

function scroll(): void {
  transcript.scrollTop = transcript.scrollHeight;
}

/** Text with ``` fences: prose paragraphs and code blocks, built node by node. */
function renderText(host: HTMLElement, text: string, withCodeActions: boolean, createsModels = false, compiled?: boolean): void {
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
    if (createsModels && /^(arc|arclang)$/i.test(lang) && /^\s*(?:\/\/[^\n]*\n\s*)*model\b/.test(code)) {
      if (compiled !== undefined) bar.append(el('span', { class: `chat-compiled ${compiled ? 'ok' : 'ko'}`, text: compiled ? '✓ compiles' : '⚠ does not compile yet', title: compiled ? 'Checked by the arclang compiler' : 'The compiler still finds errors: create it and press "Fix the findings", or ask again' }));
      const create = el('button', { type: 'button', class: 'primary', text: 'Create model' });
      create.title = 'Name the .arc file, open it beside its diagrams; the compiler checks it';
      create.addEventListener('click', () => post({ type: 'chat-create-model', code }));
      bar.append(create);
    }
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

function proposalCard(id: string, session: string, proposal: ProposalView, provider: string, written = false): HTMLElement {
  const card = el('div', { class: 'chat-proposal' });
  const actions = el('div', { class: 'chat-proposal-actions' });
  const status = el('span', { class: 'chat-status', role: 'status' });
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
  if (proposal.ok && !proposal.explainOnly && written) {
    card.classList.add('written');
    card.dataset.session = session;
    status.textContent = 'Written in the file: the highlighted lines are the change.';
    const keep = el('button', { type: 'button', class: 'primary', text: '✓ Keep' });
    const undo = el('button', { type: 'button', text: '↩ Undo' });
    const view = el('button', { type: 'button', text: 'Open diff' });
    keep.addEventListener('click', () => post({ type: 'chat-keep', session }));
    undo.addEventListener('click', () => post({ type: 'chat-undo-change', session }));
    view.addEventListener('click', () => post({ type: 'chat-diff', session }));
    (card as unknown as { _settle: (kept: boolean, error?: string) => void })._settle = (kept, error) => {
      keep.remove();
      undo.remove();
      view.remove();
      status.textContent = error ? `Kept: ${error}.` : kept ? 'Kept. Ctrl/Cmd+Z still undoes it in the file.' : 'Undone: the file is as it was.';
      card.classList.toggle('rejected', !kept);
    };
    actions.append(keep, undo, view);
  } else if (proposal.ok && !proposal.explainOnly) {
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
input.addEventListener('input', () => {
  const at = input.selectionStart ?? 0;
  if (input.value[at - 1] === '@' && (at === 1 || /\s/.test(input.value[at - 2]))) {
    input.value = input.value.slice(0, at - 1) + input.value.slice(at);
    post({ type: 'chat-pick-workspace' });
  }
});
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
      renderAttachments();
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
      turn.body.append(proposalCard(message.id, message.session, message.proposal, message.provider, message.written));
      setBusy(false);
      scroll();
      return;
    }
    case 'chat-done': {
      const turn = turns.get(message.id);
      if (!turn) return;
      turn.body.textContent = '';
      renderText(turn.body, message.text, message.kind === 'modelica', message.kind === 'plain', message.compiled);
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
    case 'chat-settled': {
      const card = transcript.querySelector<HTMLElement>(`.chat-proposal[data-session="${CSS.escape(message.session)}"]`) as (HTMLElement & { _settle?: (kept: boolean, error?: string) => void }) | null;
      card?._settle?.(message.kept, message.error);
      return;
    }
    case 'chat-extract': {
      const bytes = Uint8Array.from(atob(message.data), (c) => c.charCodeAt(0));
      void attachBytes(message.name, bytes);
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
