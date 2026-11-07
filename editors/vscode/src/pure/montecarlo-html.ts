// The page of *ArcLang: Show Monte Carlo results*. Pure: the cases in, the
// HTML out; every dynamic value is escaped, the charts are inline SVG drawn
// here, and the page script only posts which button was pressed.
import { escapeHtml } from './simulation-html';
import {
  Bin,
  Margin,
  MonteCarloCase,
  Statistics,
  axisStart,
  probabilityText,
  shortNumber,
  shortParameter,
} from './montecarlo';
import { percentText } from './simulation';

export type StatisticsState = { state: 'running'; engine: string } | { state: 'done'; report: Statistics } | { state: 'error'; message: string };

export interface PageInput {
  file: string;
  cases: MonteCarloCase[];
  statistics: Record<string, StatisticsState | undefined>;
  /** The model changed since these results: they are of the last run. */
  stale: boolean;
}

const W = 640;

function gauge(c: MonteCarloCase): string {
  const start = axisStart(c.estimate, c.required.probability);
  const x = (p: number) => 20 + ((p - start) / (1 - start)) * (W - 40);
  const step = 1 - start > 0.3 ? 0.1 : 0.05;
  const ticks: string[] = [];
  for (let p = start; p <= 1 + 1e-9; p += step) {
    const at = x(p);
    ticks.push(`<line x1="${at}" y1="44" x2="${at}" y2="50" class="tick"/><text x="${at}" y="64" class="axis-label" text-anchor="middle">${Math.round(p * 100)} %</text>`);
  }
  const required = x(c.required.probability);
  const e = c.estimate;
  const shows = e ? `${e.held} of ${e.total} samples hold: at ${percentText(c.required.confidence)} confidence, the probability is between ${percentText(e.low)} and ${percentText(e.high)}; ${percentText(c.required.probability)} is required.` : `The samples did not all run; ${percentText(c.required.probability)} is required.`;
  const band = e
    ? `<rect x="${x(e.low)}" y="22" width="${Math.max(2, x(e.high) - x(e.low))}" height="22" rx="3" class="band ${e.low >= c.required.probability ? 'shown' : 'not-shown'}"><title>${escapeHtml(`Between ${percentText(e.low)} and ${percentText(e.high)} at ${percentText(c.required.confidence)} confidence`)}</title></rect>
       <circle cx="${x(e.held / e.total)}" cy="33" r="5" class="observed"><title>${escapeHtml(`Observed: ${e.held} of ${e.total} (${percentText(e.held / e.total)})`)}</title></circle>`
    : '';
  return `<svg class="gauge" viewBox="0 0 ${W} 72" role="img" aria-label="${escapeHtml(shows)}">
  <line x1="20" y1="44" x2="${W - 20}" y2="44" class="axis"/>
  ${ticks.join('\n  ')}
  ${band}
  <line x1="${required}" y1="12" x2="${required}" y2="50" class="required"/>
  <text x="${required}" y="9" class="required-label" text-anchor="${required > W - 90 ? 'end' : 'middle'}">required ${escapeHtml(percentText(c.required.probability))}</text>
</svg>
<p class="shows">${escapeHtml(shows)}</p>`;
}

const HW = 300;
const HH = 120;

function histogramSvg(m: Margin): string {
  const bins = m.bins;
  const most = Math.max(1, ...bins.map((b) => b.count));
  const low = bins[0].from;
  const high = bins[bins.length - 1].to;
  const span = high - low || 1;
  const x = (v: number) => 8 + ((v - low) / span) * (HW - 16);
  const unit = m.unit ? ` ${m.unit}` : '';
  const bar = (b: Bin, i: number) => {
    const height = (b.count / most) * (HH - 30);
    const left = bins.length === 1 ? HW / 2 - 20 : x(b.from) + 0.5;
    const width = bins.length === 1 ? 40 : Math.max(1, x(b.to) - x(b.from) - 1);
    const range = bins.length === 1 ? `${shortNumber(b.from)}${unit}` : `${shortNumber(b.from)} to ${shortNumber(b.to)}${unit}`;
    return `<rect data-bin="${i}" x="${left}" y="${HH - 18 - height}" width="${width}" height="${height}" class="bin ${b.failing ? 'failing' : 'holding'}"><title>${escapeHtml(`${b.count} sample${b.count === 1 ? '' : 's'}: margin ${range}`)}</title></rect>`;
  };
  const bound = bins.length > 1 && low <= 0 && high >= 0 ? `<line x1="${x(0)}" y1="4" x2="${x(0)}" y2="${HH - 18}" class="bound"/><text x="${x(0)}" y="${HH - 4}" text-anchor="middle" class="axis-label bound-label">bound</text>` : '';
  // An end of the axis that the bound's label would cover is left out.
  const clear = (at: number) => !bound || Math.abs(at - x(0)) > 56;
  const lowEnd = bins.length > 1 && clear(8) ? `<text x="8" y="${HH - 4}" class="axis-label">${escapeHtml(shortNumber(low) + unit)}</text>` : '';
  const highEnd = bins.length > 1 && clear(HW - 8) ? `<text x="${HW - 8}" y="${HH - 4}" text-anchor="end" class="axis-label">${escapeHtml(shortNumber(high) + unit)}</text>` : '';
  const ends = lowEnd + highEnd;
  const label = `Margin of ${m.expect} over ${m.samples} samples: from ${shortNumber(m.smallest)} to ${shortNumber(m.largest)}${unit}, ${m.failing} below the bound.`;
  return `<svg class="histogram" viewBox="0 0 ${HW} ${HH}" role="img" aria-label="${escapeHtml(label)}">
  <line x1="8" y1="${HH - 18}" x2="${HW - 8}" y2="${HH - 18}" class="axis"/>
  ${bins.map(bar).join('\n  ')}
  ${bound}
  ${ends}
</svg>`;
}

function influences(m: Margin): string {
  if (m.constant) return '<p class="note">Its margin does not vary over the samples (the run ends on its bound): no parameter moves it.</p>';
  if (!m.influences.length) return '';
  const rows = m.influences
    .map((i) => {
      const width = Math.abs(i.rho) * 50;
      const side = i.rho >= 0 ? 'more' : 'less';
      return `<li title="${escapeHtml(i.parameter)}"><span class="parameter">${escapeHtml(shortParameter(i.parameter))}</span>
  <svg class="track" viewBox="0 0 100 10" preserveAspectRatio="none" aria-hidden="true"><rect x="0" y="0" width="100" height="10" class="track-bg"/><rect x="${i.rho >= 0 ? 50 : 50 - width}" y="1" width="${Math.max(0.5, width)}" height="8" class="bar ${i.rho >= 0 ? 'up' : 'down'}"/><line x1="50" y1="0" x2="50" y2="10" class="mid"/></svg>
  <span class="rho">${i.rho >= 0 ? '+' : ''}${i.rho.toFixed(2)}</span><span class="sr">${escapeHtml(`more ${i.parameter}, ${side} to spare`)}</span></li>`;
    })
    .join('\n');
  return `<h4>What moves it <span class="hint">rank correlation, right: more of it, more to spare</span></h4><ul class="influences">${rows}</ul>`;
}

function marginCard(m: Margin): string {
  const unit = m.unit ? ` ${m.unit}` : '';
  const summary = m.constant
    ? Math.abs(m.smallest) < 1e-4 ? 'at the bound on every sample' : `${shortNumber(m.smallest)}${unit} on every sample`
    : `${shortNumber(m.smallest)} to ${shortNumber(m.largest)}${unit}`;
  return `<article class="margin ${m.failing ? 'has-failures' : ''}">
  <h3><code>${escapeHtml(m.expect)}</code></h3>
  <p class="figures"><span>margin ${escapeHtml(summary)}</span><span class="${m.failing ? 'bad' : 'good'}">${m.failing} of ${m.samples} below the bound</span></p>
  ${histogramSvg(m)}
  ${influences(m)}
</article>`;
}

function failingList(c: MonteCarloCase): string {
  if (!c.failing.length) return '';
  const rows = c.failing
    .map((f) => `<tr><td>${escapeHtml(f.label)}</td><td>${escapeHtml(f.error || f.failed.join('; '))}</td><td><button class="secondary" data-replay="${escapeHtml(c.id)}" data-index="${f.index}">Replay</button></td></tr>`)
    .join('\n');
  return `<h3 class="section">Failing samples</h3>
<table class="failing"><thead><tr><th>Sample</th><th>What failed</th><th><span class="sr">Replay</span></th></tr></thead><tbody>
${rows}
</tbody></table>`;
}

/** The analysis of R or Python under a case; sent alone when it changes, so the page keeps its place. */
export function statisticsBlock(c: MonteCarloCase, state: StatisticsState | undefined): string {
  const buttons = `<div class="actions"><button data-statistics="${escapeHtml(c.id)}" data-engine="r" ${state?.state === 'running' ? 'disabled' : ''}>Analyze with R</button><button data-statistics="${escapeHtml(c.id)}" data-engine="python" ${state?.state === 'running' ? 'disabled' : ''}>Analyze with Python</button></div>`;
  const intro = '<p class="note">Reads the same samples again, without simulating: partial rank correlations with intervals, and the failure probability a normal fit of each margin extrapolates. The verdict stays the compiler\'s.</p>';
  let body = '';
  if (state?.state === 'running') body = `<p class="status" role="status">${escapeHtml(state.engine === 'r' ? 'R' : 'Python')} is reading the samples…</p>`;
  else if (state?.state === 'error') body = `<p class="status error" role="alert">${escapeHtml(state.message)}</p>`;
  else if (state?.state === 'done') {
    body = `<p class="status" role="status">${escapeHtml(state.report.engine)}</p>` + state.report.margins
      .map((m) => {
        const head = `<h4><code>${escapeHtml(m.expect)}</code></h4><p class="figures"><span>${m.samples} samples, ${escapeHtml(probabilityText(m.failureFraction))} fail</span><span>normal fit: failure probability ${escapeHtml(probabilityText(m.fittedFailure))}</span></p>`;
        if (m.constant) return `${head}<p class="note">Its margin does not vary: no parameter moves it.</p>`;
        const n = (v: number | null) => (v === null ? 'undefined' : `${v >= 0 ? '+' : ''}${v.toFixed(3)}`);
        const rows = m.influences.map((i) => `<tr><td>${escapeHtml(i.method)}</td><td title="${escapeHtml(i.parameter)}">${escapeHtml(shortParameter(i.parameter))}</td><td class="num">${n(i.value)}</td><td class="num">${n(i.low)} .. ${n(i.high)}</td></tr>`).join('');
        return `${head}<table class="stats"><thead><tr><th>Method</th><th>Parameter</th><th>Value</th><th>Interval</th></tr></thead><tbody>${rows}</tbody></table>`;
      })
      .join('\n');
  }
  return `<details class="statistics" data-case="${escapeHtml(c.id)}" ${state ? 'open' : ''}><summary>Further analysis with R or Python</summary>${intro}${buttons}${body}</details>`;
}

function caseSection(c: MonteCarloCase, state: StatisticsState | undefined): string {
  const verdict = c.verdict === 'pass' || c.verdict === 'fail' ? c.verdict : 'other';
  return `<section class="case verdict-${verdict}" aria-labelledby="case-${escapeHtml(c.id)}">
<header>
  <span class="badge">${escapeHtml(c.verdict.toUpperCase())}</span>
  <div><h2 id="case-${escapeHtml(c.id)}">${escapeHtml(c.name)}</h2><p class="id">${escapeHtml(c.id)}${c.parameters.length ? ` · uncertain: ${escapeHtml(c.parameters.map(shortParameter).join(', '))}` : ''}</p></div>
  <button class="secondary replay-any" data-replay="${escapeHtml(c.id)}" data-index="-1">Replay a sample…</button>
</header>
${gauge(c)}
${c.verdict !== 'pass' && c.reason ? `<p class="reason">${escapeHtml(c.reason)}</p>` : ''}
<div class="margins">${c.margins.map(marginCard).join('\n')}</div>
${failingList(c)}
${statisticsBlock(c, state)}
</section>`;
}

export function monteCarloHtml(input: PageInput, nonce: string, cspSource: string): string {
  const body = input.cases.length
    ? input.cases.map((c) => caseSection(c, input.statistics[c.id])).join('\n')
    : '<p class="empty">No test case of this model samples uncertain parameters. Add <code>uncertain:</code>, <code>samples:</code> and <code>probability:</code> to a <code>simulate:</code> block.</p>';
  return `<!doctype html>
<html lang="en"><head><meta charset="utf-8">
<meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src ${cspSource} 'nonce-${nonce}'; script-src 'nonce-${nonce}';">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Monte Carlo results</title>
<style nonce="${nonce}">
  :root { --good: var(--vscode-charts-green); --bad: var(--vscode-charts-red); --info: var(--vscode-charts-blue); --muted: var(--vscode-descriptionForeground); --line: var(--vscode-widget-border, var(--vscode-panel-border)); }
  body { font-family: var(--vscode-font-family); color: var(--vscode-foreground); background: var(--vscode-editor-background); padding: 20px 28px 40px; max-width: 1080px; line-height: 1.45; }
  h1 { font-size: 1.6em; font-weight: 600; margin: 0; letter-spacing: -0.01em; }
  .file { color: var(--muted); margin: 2px 0 14px; font-family: var(--vscode-editor-font-family); font-size: 0.92em; }
  .stale { padding: 8px 12px; border-left: 3px solid var(--vscode-charts-orange); background: var(--vscode-textBlockQuote-background); margin-bottom: 14px; }
  .case { border-top: 1px solid var(--line); padding: 22px 0 10px; }
  .case header { display: flex; align-items: center; gap: 14px; }
  .case header > div { flex: 1; min-width: 0; }
  h2 { font-size: 1.3em; margin: 0; font-weight: 600; }
  .id { margin: 0; color: var(--muted); font-size: 0.9em; overflow-wrap: anywhere; }
  .badge { font-weight: 700; font-size: 0.85em; letter-spacing: 0.06em; padding: 5px 10px; border-radius: 3px; color: var(--vscode-editor-background); background: var(--muted); }
  .verdict-pass .badge { background: var(--good); }
  .verdict-fail .badge { background: var(--bad); }
  .gauge { width: 100%; max-width: ${W}px; display: block; margin: 16px 0 0; overflow: visible; }
  .axis { stroke: var(--muted); stroke-width: 1; }
  .tick { stroke: var(--muted); }
  .axis-label { fill: var(--muted); font-size: 13px; }
  .histogram .axis-label { font-size: 12px; }
  .band.shown { fill: var(--good); fill-opacity: 0.35; stroke: var(--good); }
  .band.not-shown { fill: var(--bad); fill-opacity: 0.3; stroke: var(--bad); }
  .observed { fill: var(--vscode-foreground); }
  .required { stroke: var(--vscode-foreground); stroke-width: 2; stroke-dasharray: 3 2; }
  .required-label { fill: var(--vscode-foreground); font-size: 13px; font-weight: 600; }
  .shows { margin: 6px 0 4px; font-size: 1.02em; }
  .reason { color: var(--muted); margin: 4px 0; }
  .margins { display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); gap: 14px; margin-top: 16px; }
  .margin { border: 1px solid var(--line); border-radius: 4px; padding: 12px 14px; background: var(--vscode-sideBar-background, transparent); }
  .margin.has-failures { border-color: var(--bad); }
  h3 { font-size: 1em; margin: 0 0 4px; font-weight: 600; overflow-wrap: anywhere; }
  h3.section { margin: 22px 0 8px; }
  h4 { font-size: 0.92em; margin: 12px 0 6px; font-weight: 600; }
  .hint { font-weight: 400; color: var(--muted); }
  .figures { display: flex; flex-wrap: wrap; justify-content: space-between; gap: 4px 12px; margin: 0 0 6px; color: var(--muted); font-size: 0.92em; }
  .good { color: var(--good); } .bad { color: var(--bad); font-weight: 600; }
  .histogram { width: 100%; display: block; }
  .bin.holding { fill: var(--info); fill-opacity: 0.75; }
  .bin.failing { fill: var(--bad); }
  .bin:hover { fill-opacity: 1; }
  .bound { stroke: var(--bad); stroke-width: 1.5; stroke-dasharray: 4 3; }
  .bound-label { fill: var(--bad); }
  .influences { list-style: none; padding: 0; margin: 0; display: grid; gap: 4px; }
  .influences li { display: grid; grid-template-columns: minmax(80px, 38%) 1fr 3.4em; align-items: center; gap: 8px; font-size: 0.92em; }
  .parameter { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-family: var(--vscode-editor-font-family); }
  .track { width: 100%; height: 12px; display: block; }
  .track-bg { fill: var(--vscode-editorWidget-background, transparent); }
  .mid { stroke: var(--muted); stroke-width: 0.6; vector-effect: non-scaling-stroke; }
  .bar.up { fill: var(--good); } .bar.down { fill: var(--vscode-charts-orange); }
  .rho { text-align: right; font-variant-numeric: tabular-nums; }
  .note { color: var(--muted); font-size: 0.92em; margin: 8px 0; }
  table { border-collapse: collapse; width: 100%; font-size: 0.92em; }
  th, td { text-align: left; padding: 5px 8px; border-bottom: 1px solid var(--line); vertical-align: top; }
  th { color: var(--muted); font-weight: 600; }
  td.num { font-variant-numeric: tabular-nums; white-space: nowrap; }
  .statistics { margin-top: 18px; border: 1px solid var(--line); border-radius: 4px; padding: 8px 14px; }
  .statistics summary { cursor: pointer; font-weight: 600; }
  .status { margin: 10px 0 4px; } .status.error { color: var(--vscode-errorForeground); }
  .actions { display: flex; gap: 8px; flex-wrap: wrap; margin: 8px 0; }
  button { font: inherit; padding: 4px 12px; border: 1px solid transparent; border-radius: 2px; cursor: pointer; color: var(--vscode-button-foreground); background: var(--vscode-button-background); white-space: nowrap; }
  button:hover { background: var(--vscode-button-hoverBackground); }
  button.secondary { color: var(--vscode-button-secondaryForeground); background: var(--vscode-button-secondaryBackground); }
  button.secondary:hover { background: var(--vscode-button-secondaryHoverBackground); }
  button:disabled { opacity: 0.5; cursor: default; }
  button:focus-visible, summary:focus-visible { outline: 1px solid var(--vscode-focusBorder); outline-offset: 2px; }
  .sr { position: absolute; width: 1px; height: 1px; overflow: hidden; clip: rect(0 0 0 0); }
  .empty { color: var(--muted); }
  @media (max-width: 520px) { body { padding: 14px 16px; } .case header { flex-wrap: wrap; } }
</style></head>
<body>
<h1>Monte Carlo results</h1>
<p class="file">${escapeHtml(input.file)}</p>
${input.stale ? '<p class="stale" role="status">The model changed since these results. Run the simulations again to see where it stands now.</p>' : ''}
${body}
<script nonce="${nonce}">
  const vscode = acquireVsCodeApi();
  document.addEventListener('click', (event) => {
    const target = event.target instanceof Element ? event.target.closest('button') : null;
    if (!target || target.disabled) return;
    if (target.dataset.replay !== undefined) vscode.postMessage({ type: 'replay', testCase: target.dataset.replay, index: Number(target.dataset.index) });
    else if (target.dataset.statistics !== undefined) vscode.postMessage({ type: 'statistics', testCase: target.dataset.statistics, engine: target.dataset.engine });
  });
  // The analysis of one case, drawn by the extension (escaped there): only that block changes.
  window.addEventListener('message', (event) => {
    const message = event.data;
    if (!message || message.type !== 'statistics' || typeof message.html !== 'string') return;
    const block = [...document.querySelectorAll('details.statistics')].find((d) => d.dataset.case === message.testCase);
    if (!block) return;
    const template = document.createElement('template');
    template.innerHTML = message.html;
    const fresh = template.content.firstElementChild;
    if (fresh) block.replaceWith(fresh);
  });
</script>
</body></html>`;
}
