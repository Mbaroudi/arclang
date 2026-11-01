// Time travel on the workbench page: a bar along the bottom of the drawing
// with one notch per commit that touched the model (oldest left, "Now" at
// the right end), the snapshots as flags and the merges as diamonds. Moving
// along it draws the model as it was at that commit, the changes since the
// commit before (or since the moment chosen in "Compare with") marked; from
// there an element or the whole model can be restored, a snapshot taken, a
// branch started. What is read from git, and every change, is the host's.
import type { Timeline, TimelineCommit } from '../src/pure/timeline';
import { shortDate } from '../src/pure/timeline';
import { button, el } from './dom';

export interface TimelineHooks {
  host: HTMLElement;
  post: (message: unknown) => void;
  /** The element selected on the drawing, if any. */
  selection: () => { id: string; name?: string; kind?: string } | null;
  toast: (text: string, kind: 'info' | 'error') => void;
}

let hooks: TimelineHooks | null = null;
let bar: HTMLElement | null = null;
let timeline: Timeline | null = null;
/** The notch shown: an index in the commits, or the commits' length for "Now". */
let position = 0;
let base: string | null | undefined;
let pending: ReturnType<typeof setTimeout> | null = null;

const SVG = 'http://www.w3.org/2000/svg';

/** Whether the bar is open. */
export function timelineOpen(): boolean {
  return bar !== null;
}

export function installTimeline(h: TimelineHooks): void {
  hooks = h;
  // Redrawing the diagram empties the host: the bar comes back on top of the new drawing.
  new MutationObserver(() => {
    if (bar && !bar.isConnected) h.host.appendChild(bar);
  }).observe(h.host, { childList: true });
  window.addEventListener('message', (event: MessageEvent) => {
    const message = event.data as { type?: string; timeline?: Timeline; at?: string | null; error?: string | null; hash?: string; base?: string | null; what?: string } | undefined;
    if (!message?.type?.startsWith('timeline')) return;
    if (message.type === 'timeline') {
      if (message.error || !message.timeline) return h.toast(`Timeline: ${message.error ?? 'no history'}`, 'error');
      timeline = message.timeline;
      const at = message.at ? timeline.commits.findIndex((c) => c.hash === message.at) : -1;
      position = at >= 0 ? at : timeline.commits.length;
      render();
    } else if (message.type === 'timeline-at') {
      if (message.error) h.toast(`That moment could not be drawn: ${message.error}`, 'error');
      bar?.classList.remove('is-loading');
    } else if (message.type === 'timeline-restored') {
      h.toast(`Restored ${message.what} as it was at ${message.hash ?? message.at ?? ''}: review it, then save and commit.`, 'info');
      close();
    }
  });
}

/** The selection changed: the "Restore …" button names the element selected. */
export function refreshTimeline(): void {
  if (bar && timeline && position < timeline.commits.length) render();
}

/** Open the bar (the host answers with the timeline). */
export function openTimeline(): void {
  hooks?.post({ type: 'semgit-timeline', op: 'open' });
}

function close(): void {
  if (pending) clearTimeout(pending);
  bar?.remove();
  bar = null;
  if (timeline && position < timeline.commits.length) hooks?.post({ type: 'semgit-timeline', op: 'now' });
  timeline = null;
  document.body.classList.remove('wb-time-travel');
}

const commitAt = (index: number): TimelineCommit | null => (timeline && index < timeline.commits.length ? timeline.commits[index] : null);

/** Draw the moment at `index` (debounced while the slider moves). */
function go(index: number, immediate = false): void {
  if (!timeline) return;
  position = Math.max(0, Math.min(index, timeline.commits.length));
  // A moment compared with must come before the one shown: else, the commit before.
  if (base) {
    const at = timeline.commits.findIndex((c) => c.hash === base);
    if (at < 0 || at >= position) base = undefined;
  }
  render();
  if (pending) clearTimeout(pending);
  const send = () => {
    const commit = commitAt(position);
    bar?.classList.toggle('is-loading', !!commit);
    hooks!.post(commit ? { type: 'semgit-timeline', op: 'show', hash: commit.hash, base } : { type: 'semgit-timeline', op: 'now' });
  };
  if (immediate) send();
  else pending = setTimeout(send, 250);
}

function track(t: Timeline): SVGSVGElement {
  const n = t.commits.length;
  const svg = document.createElementNS(SVG, 'svg');
  svg.setAttribute('class', 'wb-time-track');
  svg.setAttribute('viewBox', '0 0 1000 34');
  svg.setAttribute('preserveAspectRatio', 'none');
  svg.setAttribute('aria-hidden', 'true');
  const x = (i: number) => 10 + (980 * i) / Math.max(1, n);
  const line = document.createElementNS(SVG, 'line');
  line.setAttribute('x1', '10'); line.setAttribute('x2', '990'); line.setAttribute('y1', '24'); line.setAttribute('y2', '24');
  line.setAttribute('class', 'wb-time-line');
  svg.appendChild(line);
  t.commits.forEach((c, i) => {
    const mark = document.createElementNS(SVG, c.merge ? 'rect' : 'circle');
    if (c.merge) {
      mark.setAttribute('x', String(x(i) - 4)); mark.setAttribute('y', '20'); mark.setAttribute('width', '8'); mark.setAttribute('height', '8');
      mark.setAttribute('transform', `rotate(45 ${x(i)} 24)`);
    } else {
      mark.setAttribute('cx', String(x(i))); mark.setAttribute('cy', '24'); mark.setAttribute('r', '4');
    }
    mark.setAttribute('class', `wb-time-dot${i === position ? ' is-here' : ''}${c.merge ? ' is-merge' : ''}`);
    svg.appendChild(mark);
    if (c.tags.length) {
      const flag = document.createElementNS(SVG, 'path');
      flag.setAttribute('d', `M${x(i)} 19 V4 h12 l-3 4 l3 4 h-12`);
      flag.setAttribute('class', 'wb-time-flag');
      svg.appendChild(flag);
    }
  });
  const now = document.createElementNS(SVG, 'circle');
  now.setAttribute('cx', String(x(n))); now.setAttribute('cy', '24'); now.setAttribute('r', '5');
  now.setAttribute('class', `wb-time-now${position === n ? ' is-here' : ''}`);
  svg.appendChild(now);
  return svg;
}

function render(): void {
  const h = hooks;
  const t = timeline;
  if (!h || !t) return;
  const n = t.commits.length;
  const commit = commitAt(position);
  document.body.classList.toggle('wb-time-travel', !!commit);
  const fresh = el('section', { class: `wb-timeline${commit ? ' is-past' : ''}`, role: 'group', 'aria-label': 'Timeline of the model' });

  const head = el('div', { class: 'wb-time-head' }, [
    el('strong', { class: 'wb-time-title', text: 'Timeline' }),
    el('span', { class: 'wb-time-branch', title: 'The branch checked out', text: t.branch ? `⎇ ${t.branch}` : 'detached' }),
    el('span', { class: 'wb-time-count', text: `${n} commit${n === 1 ? '' : 's'}` }),
  ]);
  const closeButton = button('×', close, { class: 'wb-time-close', title: 'Close the timeline and come back to now', 'aria-label': 'Close the timeline' });
  head.appendChild(closeButton);
  fresh.appendChild(head);

  // The slider (keyboard and screen readers) over the drawn track.
  const slider = el('input', { type: 'range', class: 'wb-time-slider', min: '0', max: String(n), step: '1', value: String(position), 'aria-label': 'Moment of the model', 'aria-valuetext': commit ? `${commit.short} ${commit.subject}` : 'Now' }) as HTMLInputElement;
  slider.addEventListener('input', () => go(Number(slider.value)));
  const rail = el('div', { class: 'wb-time-rail' }, [track(t), slider]);
  const flags = el('div', { class: 'wb-time-flags', 'aria-hidden': 'true' });
  t.commits.forEach((c, i) => {
    if (!c.tags.length) return;
    const tag = el('span', { class: 'wb-time-tag', text: c.tags[0], title: c.tags.join(', ') });
    tag.style.left = `${1 + (98 * i) / Math.max(1, n)}%`;
    flags.appendChild(tag);
  });
  fresh.appendChild(el('div', { class: 'wb-time-row' }, [
    button('◀', () => go(position - 1, true), { class: 'wb-time-step', title: 'The commit before', 'aria-label': 'The commit before', disabled: position === 0 ? 'true' : undefined }),
    el('div', { class: 'wb-time-rail-wrap' }, [flags, rail]),
    button('▶', () => go(position + 1, true), { class: 'wb-time-step', title: 'The commit after', 'aria-label': 'The commit after', disabled: position === n ? 'true' : undefined }),
  ]));

  const info = el('div', { class: 'wb-time-info', 'aria-live': 'polite' });
  if (commit) {
    info.appendChild(el('code', { class: 'wb-time-hash', text: commit.short }));
    for (const tag of commit.tags) info.appendChild(el('span', { class: 'wb-time-chip is-tag', text: `⚑ ${tag}` }));
    for (const head of commit.heads) info.appendChild(el('span', { class: 'wb-time-chip', text: `⎇ ${head}` }));
    if (commit.merge) info.appendChild(el('span', { class: 'wb-time-chip is-merge', text: 'merge' }));
    info.appendChild(el('span', { class: 'wb-time-subject', text: commit.subject }));
    info.appendChild(el('span', { class: 'wb-time-meta', text: `${commit.author} · ${shortDate(commit.date)}` }));
  } else {
    info.appendChild(el('span', { class: 'wb-time-subject', text: 'Now: the model as it is in the editor' }));
  }
  fresh.appendChild(info);

  if (commit) {
    const choice = el('select', { class: 'wb-time-base', 'aria-label': 'Compare with' }) as HTMLSelectElement;
    const option = (value: string, text: string, selected: boolean) => {
      const o = el('option', { value, text }) as HTMLOptionElement;
      o.selected = selected;
      return o;
    };
    choice.appendChild(option('', 'the commit before', base === undefined));
    for (let i = position - 1; i >= 0; i--) {
      const c = t.commits[i];
      choice.appendChild(option(c.hash, `${c.tags[0] ? `⚑ ${c.tags[0]} · ` : ''}${c.short} ${c.subject}`.slice(0, 70), base === c.hash));
    }
    choice.addEventListener('change', () => {
      base = choice.value === '' ? undefined : choice.value;
      go(position, true);
    });
    const selected = h.selection();
    const restorable = selected && selected.kind !== 'Model' ? selected : null;
    fresh.appendChild(el('div', { class: 'wb-time-actions' }, [
      el('label', { class: 'wb-time-base-label' }, [el('span', { text: 'Changes since' }), choice]),
      el('span', { class: 'wb-time-gap' }),
      button(restorable ? `Restore ${restorable.name || restorable.id} as here` : 'Restore an element', () => restorable && h.post({ type: 'semgit-timeline', op: 'restore', hash: commit.hash, element: restorable.id }), { title: restorable ? 'Put this element back as it was at this commit (an edit of the file, undoable)' : 'Select an element on the drawing first', disabled: restorable ? undefined : 'true' }),
      button('Restore the model as here', () => h.post({ type: 'semgit-timeline', op: 'restore', hash: commit.hash }), { title: 'The whole model as it was at this commit (an edit of the file, undoable)' }),
      button('⚑ Snapshot here', () => h.post({ type: 'semgit-timeline', op: 'snapshot', hash: commit.hash }), { title: 'Name this moment (an annotated git tag, never moved)' }),
      button('⎇ Branch from here', () => h.post({ type: 'semgit-timeline', op: 'branch', hash: commit.hash }), { title: 'Start a branch at this commit' }),
      button('Back to now', () => go(n, true), { class: 'is-primary', title: 'The model as it is in the editor' }),
    ]));
  }

  if (bar) bar.replaceWith(fresh);
  else h.host.appendChild(fresh);
  bar = fresh;
  if (document.activeElement?.classList.contains('wb-time-slider') || !commit) {
    (fresh.querySelector('.wb-time-slider') as HTMLElement | null)?.focus({ preventScroll: true });
  }
}
