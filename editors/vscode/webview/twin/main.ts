// The digital twins' page: per twin its verdict, its data, each
// comparison's numbers and curves (drawn by the host, escaped), and the
// requirements it checks on the run.
import type { TwinOutcome } from '../../src/pure/twin';
import { differenceInUnit, duration, twinChart } from '../../src/pure/twin';

declare function acquireVsCodeApi(): { postMessage(message: unknown): void };
const vscode = acquireVsCodeApi();

type Shown = TwinOutcome;
type FromHost = { type: 'twins'; model: string; twins: Shown[] } | { type: 'busy'; text: string } | { type: 'live'; running: boolean; status: string; detail: string };

const $ = <T extends HTMLElement>(id: string) => document.getElementById(id) as T;
const status = $<HTMLElement>('status');

function el<K extends keyof HTMLElementTagNameMap>(tag: K, className?: string, text?: string): HTMLElementTagNameMap[K] {
  const node = document.createElement(tag);
  if (className) node.className = className;
  if (text !== undefined) node.textContent = text;
  return node;
}

function svg(text: string): Node {
  const parsed = new DOMParser().parseFromString(text, 'image/svg+xml').documentElement;
  return parsed.nodeName === 'svg' ? document.importNode(parsed, true) : el('p', 'note', 'The chart could not be drawn.');
}

const VERDICT: Record<string, string> = { pass: 'holds', fail: 'departs', error: 'error', inconclusive: 'inconclusive' };

function twinCard(t: Shown): HTMLElement {
  const card = el('section', `twin ${t.verdict}`);
  card.setAttribute('aria-label', t.name);
  const head = el('header', 'twin-head');
  head.append(el('h2', '', t.name), el('span', `badge ${t.verdict}`, VERDICT[t.verdict] ?? t.verdict));
  card.append(head);
  const meta = el('p', 'meta');
  if (t.data) {
    // A file opens; a live source (mqtt://…, opc.tcp://…) is only named.
    const live = t.data.includes('://');
    const open = el(live ? 'span' : 'button', live ? 'source' : 'link', t.data);
    if (!live) open.addEventListener('click', () => vscode.postMessage({ type: 'data', file: t.data }));
    meta.append(open, document.createTextNode(` · ${t.samples} samples over ${duration(t.duration)}${t.data_start ? ` from ${t.data_start}` : ''}`));
  }
  if (t.verifies.length) meta.append(document.createTextNode(` · verifies ${t.verifies.join(', ')}`));
  if (t.omc_version) meta.append(document.createTextNode(` · ${t.omc_version}, ${t.seconds.toFixed(1)} s`));
  card.append(meta);
  if (t.reason && t.verdict !== 'pass') card.append(el('p', `reason ${t.verdict}`, t.reason));
  t.comparisons.forEach((c) => {
    const block = el('div', `comparison ${c.holds ? 'ok' : 'off'}`);
    const unit = c.unit ? ` ${c.unit}` : '';
    const title = el('h3', '');
    title.append(el('code', '', c.target), document.createTextNode(' ~ '), el('code', '', c.column));
    const numbers = el('dl', 'numbers');
    const add = (label: string, value: string, strong = false) => {
      const pair = el('div', 'pair');
      pair.append(el('dt', '', label), el('dd', strong ? 'strong' : '', value));
      numbers.append(pair);
    };
    add('within', `${(c.within * 100).toFixed(1)} % (${(c.required * 100).toFixed(0)} % required)`, !c.holds);
    add('tolerance', c.tolerance);
    add('max deviation', `${differenceInUnit(c.max_deviation, c).toFixed(3)}${unit}`);
    add('RMS', `${differenceInUnit(c.rms_deviation, c).toFixed(3)}${unit}`);
    if (c.first_exceedance !== null) add('first beyond', duration(c.first_exceedance));
    const figure = el('figure', 'chart');
    // Drawn at the width it has, so that its text keeps its size; again when the view is resized.
    charts.push({ figure, comparison: c });
    const legend = el('figcaption', 'legend');
    legend.append(el('span', 'key measured', 'measured'), el('span', 'key simulated', 'model'), el('span', 'key band', 'tolerance'));
    figure.append(legend);
    block.append(title, numbers, figure);
    card.append(block);
  });
  if (t.values.length) {
    const list = el('ul', 'expects');
    for (const v of t.values) {
      const item = el('li', v.holds ? 'ok' : 'off');
      item.append(el('span', 'mark', v.holds ? '✓' : '✗'), el('code', '', v.expect), document.createTextNode(`  ${v.value}`));
      list.append(item);
    }
    card.append(el('h3', '', 'Requirements on the run'), list);
  }
  return card;
}

let charts: { figure: HTMLElement; comparison: TwinOutcome['comparisons'][number] }[] = [];

function drawCharts(): void {
  for (const { figure, comparison } of charts) {
    const width = Math.max(320, Math.floor(figure.clientWidth));
    const drawn = svg(twinChart(comparison, { width, height: Math.round(Math.min(300, Math.max(200, width * 0.42))) }));
    const old = figure.querySelector('svg');
    if (old) old.replaceWith(drawn);
    else figure.prepend(drawn);
  }
}
new ResizeObserver(() => drawCharts()).observe(document.body);

window.addEventListener('message', (event: MessageEvent<FromHost>) => {
  const message = event.data;
  if (message.type === 'live') {
    const banner = $<HTMLElement>('live');
    banner.hidden = false;
    banner.className = `live ${message.running ? message.status : 'stopped'}`;
    $<HTMLElement>('live-text').textContent = message.detail;
    $<HTMLButtonElement>('stop').hidden = !message.running;
    $<HTMLButtonElement>('rerun').hidden = message.running;
    return;
  }
  if (message.type === 'busy') {
    status.textContent = message.text;
    status.hidden = false;
    return;
  }
  status.hidden = true;
  if (message.twins.length === 0) return;
  $('title').textContent = `Digital twins · ${message.model}`;
  charts = [];
  $('twins').replaceChildren(...message.twins.map(twinCard));
  drawCharts();
});
$('rerun').addEventListener('click', () => vscode.postMessage({ type: 'rerun' }));
$('log').addEventListener('click', () => vscode.postMessage({ type: 'log' }));
$('stop').addEventListener('click', () => vscode.postMessage({ type: 'stop' }));
vscode.postMessage({ type: 'ready' });
