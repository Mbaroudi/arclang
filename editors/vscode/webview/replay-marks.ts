// A simulation replayed on the workbench page: a bar moves through the
// time of the run (play, a slider, the first failure), each part shows its
// values under its box at that instant, and a part whose expectation has
// failed by then is outlined in red, with what failed in its tooltip.
import { failingAt, firstFailure, Replay, ReplayMessage, tagsAt, timeLabel } from '../src/pure/replay';

const SVG = 'http://www.w3.org/2000/svg';
/** Seconds of the page a whole run plays in. */
const PLAY_SECONDS = 12;
const LINE = 12;

let replay: Replay | null = null;
let title = '';
let instant = 0;
let playing = false;
let bar: HTMLElement | null = null;

function svg(tag: string, attributes: Record<string, string | number>): SVGElement {
  const node = document.createElementNS(SVG, tag);
  for (const [key, value] of Object.entries(attributes)) node.setAttribute(key, String(value));
  return node;
}

function paint(host: HTMLElement): void {
  for (const tag of host.querySelectorAll('.wb-replay-tag')) tag.remove();
  for (const node of host.querySelectorAll('.wb-replay-fail')) node.classList.remove('wb-replay-fail');
  const stage = host.querySelector('.arcviz-stage');
  if (!replay || !stage) return;
  const tags = tagsAt(replay, instant);
  const failing = failingAt(replay, instant);
  for (const [id, lines] of tags) {
    const node = stage.querySelector(`.av-node[data-id="${CSS.escape(id)}"]`);
    const shape = node?.querySelector(':scope > .av-shape');
    if (!node || !shape) continue;
    const x = Number(shape.getAttribute('x'));
    const y = Number(shape.getAttribute('y')) + Number(shape.getAttribute('height')) + 3;
    const width = Math.max(Number(shape.getAttribute('width')), ...lines.map((l) => l.length * 5.6 + 10));
    const group = svg('g', { class: 'wb-replay-tag', 'pointer-events': 'none' });
    group.appendChild(svg('rect', { x, y, width, height: lines.length * LINE + 6, rx: 3, class: 'wb-replay-tag-box' }));
    lines.forEach((line, index) => {
      const text = svg('text', { x: x + 5, y: y + 12 + index * LINE, class: 'wb-replay-tag-text' });
      text.textContent = line;
      group.appendChild(text);
    });
    node.appendChild(group);
  }
  for (const [id, expectations] of failing) {
    const node = stage.querySelector(`.av-node[data-id="${CSS.escape(id)}"]`);
    if (!node) continue;
    node.classList.add('wb-replay-fail');
    const tip = svg('title', { class: 'wb-replay-tag' });
    tip.textContent = expectations.map((e) => `${e.requirement ? e.requirement + ': ' : ''}${e.text} fails (${e.value}, ${e.margin})`).join('\n');
    node.appendChild(tip);
  }
}

function el<K extends keyof HTMLElementTagNameMap>(tag: K, attributes: Record<string, string>, text?: string): HTMLElementTagNameMap[K] {
  const node = document.createElement(tag);
  for (const [key, value] of Object.entries(attributes)) node.setAttribute(key, value);
  if (text !== undefined) node.textContent = text;
  return node;
}

function drawBar(host: HTMLElement): void {
  bar?.remove();
  bar = null;
  if (!replay) return;
  const end = replay.time[replay.time.length - 1] ?? 0;
  const start = replay.time[0] ?? 0;
  const failure = firstFailure(replay);
  const root = el('div', { class: 'wb-replay', role: 'group', 'aria-label': `Replay of ${title}` });
  const play = el('button', { class: 'wb-replay-play', type: 'button', title: 'Play or pause the run (Space)', 'aria-label': 'Play' }, '▶');
  const slider = el('input', { class: 'wb-replay-slider', type: 'range', min: String(start), max: String(end), step: String((end - start) / 1000 || 1), 'aria-label': 'Instant of the run' });
  slider.value = String(instant);
  const clock = el('span', { class: 'wb-replay-time' });
  const jump = el('button', { class: 'wb-replay-jump', type: 'button', title: failure === null ? 'Every expectation holds' : 'Go to the first instant an expectation fails' }, failure === null ? 'Every expectation holds' : `First failure at ${timeLabel(failure)}`);
  if (failure === null) jump.setAttribute('disabled', 'true');
  const status = el('span', { class: 'wb-replay-status' });
  const close = el('button', { class: 'wb-replay-close', type: 'button', title: 'Stop the replay', 'aria-label': 'Stop the replay' }, '×');
  const name = el('span', { class: 'wb-replay-title', title: replay.stale ? 'The model changed since this run: run the simulations again to replay it as it is now' : title }, `${title}${replay.stale ? ' · changed since the run' : ''}`);
  root.append(name, play, slider, clock, jump, status, close);
  const update = () => {
    if (!replay) return;
    slider.value = String(instant);
    clock.textContent = `${timeLabel(instant)} / ${timeLabel(end)}`;
    const failing = [...failingAt(replay, instant).values()].flat();
    status.textContent = failing.length ? `✗ ${failing.map((e) => e.requirement ?? e.text).join(', ')}` : '';
    status.classList.toggle('is-failing', failing.length > 0);
    play.textContent = playing ? '⏸' : '▶';
    play.setAttribute('aria-label', playing ? 'Pause' : 'Play');
    paint(host);
  };
  let last = 0;
  const step = (now: number) => {
    if (!playing || !replay) return;
    const dt = last ? (now - last) / 1000 : 0;
    last = now;
    instant = Math.min(end, instant + (dt * (end - start)) / PLAY_SECONDS);
    if (instant >= end) playing = false;
    update();
    if (playing) requestAnimationFrame(step);
  };
  play.addEventListener('click', () => {
    if (instant >= end) instant = start;
    playing = !playing;
    last = 0;
    update();
    if (playing) requestAnimationFrame(step);
  });
  slider.addEventListener('input', () => {
    playing = false;
    instant = Number(slider.value);
    update();
  });
  jump.addEventListener('click', () => {
    if (failure === null) return;
    playing = false;
    instant = failure;
    update();
  });
  close.addEventListener('click', () => {
    replay = null;
    playing = false;
    drawBar(host);
    paint(host);
  });
  if (getComputedStyle(host).position === 'static') host.style.position = 'relative';
  host.appendChild(root);
  bar = root;
  update();
}

/** `show` brings a view on screen; `drawn` tells whether the current view draws an element. */
export function installReplayMarks(host: HTMLElement, show: (view: string) => void, drawn: (id: string) => boolean): void {
  window.addEventListener('message', (event: MessageEvent) => {
    const message = event.data as ReplayMessage | undefined;
    if (message?.type !== 'replay-load') return;
    try {
      load(message);
    } catch (error) {
      console.error('replay: the run cannot be shown', error);
    }
  });
  const load = (message: ReplayMessage) => {
    const same = replay && message.replay && replay.case === message.replay.case && replay.point === message.replay.point;
    if (same) return;
    replay = message.replay;
    title = message.title;
    playing = false;
    instant = replay?.time[0] ?? 0;
    drawBar(host);
    placing = !!replay;
    place();
  };
  // The parts are drawn on the physical view: show it, once a drawing is on the page, when it draws none of them.
  let placing = false;
  const place = () => {
    if (!placing || !replay || !host.querySelector('.arcviz-stage .av-node')) return;
    placing = false;
    if (replay.elements.some((e) => drawn(e.id))) return;
    try {
      show('pab');
    } catch (error) {
      console.error('replay: the physical view cannot be shown', error);
    }
  };
  // The drawing is built again (another view, a new snapshot): paint again.
  let timer: ReturnType<typeof setTimeout> | undefined;
  new MutationObserver((records) => {
    // The page rebuilt the part holding the bar: put it back.
    if (replay && bar && !bar.isConnected) host.appendChild(bar);
    place();
    if (!replay || records.every((r) => [...r.addedNodes, ...r.removedNodes].every((n) => n instanceof Element && (n.classList.contains('wb-replay-tag') || n.closest?.('.wb-replay'))))) return;
    clearTimeout(timer);
    timer = setTimeout(() => paint(host), 120);
  }).observe(host, { childList: true, subtree: true });
}
