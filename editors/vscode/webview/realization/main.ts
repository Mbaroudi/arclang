// The realization ladder page: a header (the file, how many requirements
// climb, how many are complete, what to run next), then one row per
// requirement with a chip per stage, the stage reached and the next
// command with its Run button. The page only shows: the compiler computes
// the ladder, the host runs the commands.
import {
  applies,
  complete,
  FromRealizationView,
  Ladder,
  markOf,
  nextCommands,
  reached,
  STAGES,
  stageDescription,
  stageLabel,
  stageTitle,
  summarize,
  ToRealizationView,
} from '../../src/pure/realization';
import { button, clear, el } from '../dom';

const vscode = acquireVsCodeApi() as unknown as { postMessage: (m: FromRealizationView) => void };
const post = (message: FromRealizationView) => vscode.postMessage(message);

let ladders: Ladder[] = [];
let file = '';
let binary = 'arclang';
let computedMs = 0;
let busy = true;
let error: string | null = null;
let notice: { text: string; error: boolean } | null = null;
let onlyClimbing = true;

const root = document.getElementById('realization') as HTMLElement;

function say(text: string, isError: boolean): void {
  notice = { text, error: isError };
  render();
  setTimeout(() => {
    if (notice?.text === text) {
      notice = null;
      render();
    }
  }, 6000);
}

function header(): HTMLElement {
  const summary = summarize(ladders);
  const counts = [
    el('span', { class: 'rl-count', text: `${summary.climbing} requirement${summary.climbing === 1 ? '' : 's'} on the ladder` }),
    el('span', { class: 'rl-count rl-ok', text: `${summary.complete} complete` }),
    summary.failing ? el('span', { class: 'rl-count rl-fail', text: `${summary.failing} failing` }) : null,
    summary.stale ? el('span', { class: 'rl-count rl-stale', text: `${summary.stale} stale` }) : null,
    summary.missing ? el('span', { class: 'rl-count rl-missing', text: `${summary.missing} with a stage never run` }) : null,
  ];
  const next = nextCommands(ladders);
  const actions = el('div', { class: 'rl-actions' }, [
    button('Refresh', () => post({ type: 'refresh' }), { title: 'Read the cached verdicts again' }),
    button('As a table', () => post({ type: 'table' }), { title: 'The same ladder as a table: Markdown, Excel' }),
    el('label', { class: 'rl-toggle' }, [
      (() => {
        const box = el('input', { type: 'checkbox' }) as HTMLInputElement;
        box.checked = onlyClimbing;
        box.addEventListener('change', () => {
          onlyClimbing = box.checked;
          render();
        });
        return box;
      })(),
      'only requirements with a stage',
    ]),
  ]);
  const nextList = next.length
    ? el('div', { class: 'rl-next' }, [
        el('h2', { text: 'Next' }),
        ...next.map((n) =>
          el('div', { class: 'rl-next-row' }, [
            button('Run', () => post({ type: 'run', command: n.command }), { class: 'rl-run', title: `In a terminal beside the model, as ${binary === 'arclang' ? 'arclang' : binary}` }),
            el('code', { text: n.command }),
            el('span', { class: 'rl-muted', text: `→ ${stageLabel(n.stage)} of ${n.requirements.join(', ')}` }),
          ]),
        ),
      ])
    : el('div', { class: 'rl-next rl-muted', text: summary.climbing ? 'Every stage that applies has fresh evidence.' : 'No requirement has a case that simulates or executes, nor a twin: nothing to climb yet.' });
  return el('header', { class: 'rl-header' }, [
    el('div', { class: 'rl-title' }, [el('h1', { text: 'Realization ladder' }), el('span', { class: 'rl-file', text: file }), busy ? el('span', { class: 'rl-busy', text: 'computing…' }) : el('span', { class: 'rl-muted', text: `${computedMs} ms` })]),
    el('div', { class: 'rl-counts' }, counts),
    actions,
    nextList,
  ]);
}

function legend(): HTMLElement {
  return el(
    'div',
    { class: 'rl-legend' },
    STAGES.map((s) => el('span', { title: stageDescription(s), text: stageLabel(s) })).concat([
      el('span', { class: 'rl-muted', text: '✓ fresh pass · ✗ fail or error · ~ stale · ○ never run · – not a stage of this requirement' }),
    ]),
  );
}

function row(ladder: Ladder): HTMLElement {
  const name = el('div', { class: 'rl-req' }, [
    button(ladder.requirement, () => post({ type: 'reveal', id: ladder.requirement }), { class: 'rl-link', title: 'Show in the text' }),
    ladder.name && ladder.name !== ladder.requirement ? el('span', { class: 'rl-name', text: ladder.name }) : null,
  ]);
  const chips = el(
    'div',
    { class: 'rl-chips' },
    ladder.stages.map((s) => el('span', { class: `rl-chip rl-${s.state === 'n/a' ? 'na' : s.state}`, title: stageTitle(s) }, [el('b', { text: markOf(s.state) }), ` ${stageLabel(s.stage)}`])),
  );
  const top = reached(ladder);
  const state = el('div', { class: 'rl-reached' }, [complete(ladder) ? el('span', { class: 'rl-ok', text: 'complete' }) : top ? el('span', { text: `reached ${stageLabel(top)}` }) : el('span', { class: 'rl-muted', text: 'no evidence yet' })]);
  const next = ladder.next
    ? el('div', { class: 'rl-next-cell' }, [
        ladder.next.startsWith('arclang ') ? button('Run', () => post({ type: 'run', command: ladder.next! }), { class: 'rl-run rl-small' }) : null,
        el('code', { text: ladder.next }),
      ])
    : el('div', { class: 'rl-next-cell rl-muted', text: '' });
  return el('div', { class: `rl-row${applies(ladder) ? '' : ' rl-idle'}` }, [name, chips, state, next]);
}

function render(): void {
  clear(root);
  root.appendChild(header());
  if (notice) root.appendChild(el('div', { class: `rl-notice${notice.error ? ' rl-notice-error' : ''}`, text: notice.text }));
  if (error) {
    root.appendChild(el('div', { class: 'rl-error', text: error }));
    return;
  }
  root.appendChild(legend());
  const shown = ladders.filter((l) => !onlyClimbing || applies(l));
  if (!shown.length) {
    root.appendChild(el('p', { class: 'rl-muted', text: ladders.length ? 'No requirement has a stage: untick the filter to see them all.' : 'No requirement in the model.' }));
    return;
  }
  root.appendChild(el('section', { class: 'rl-rows', 'aria-label': 'Requirements' }, shown.map(row)));
}

window.addEventListener('message', (event: MessageEvent<ToRealizationView>) => {
  const message = event.data;
  switch (message.type) {
    case 'busy':
      busy = true;
      render();
      return;
    case 'ladders':
      ladders = message.ladders;
      file = message.file;
      binary = message.binary;
      computedMs = message.computedMs;
      busy = false;
      error = null;
      render();
      return;
    case 'error':
      error = message.error;
      file = message.file;
      ladders = [];
      busy = false;
      render();
      return;
    case 'result':
      say(message.message, !message.ok);
      return;
  }
});

render();
post({ type: 'ready' });
