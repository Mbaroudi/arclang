// The report of a simulation run, as a PDF or a PowerPoint deck: a summary
// page (every test case, its verdict and the requirements it verifies), then
// one page per test case: its verdict, what it simulated, each expectation
// with its value and margin, and the curve of each quantity checked over
// time with its bound and, for a failure, the first instant it fails. The
// curves are drawn as vectors, in the PDF and as shapes in PowerPoint.
import { assemblePdf, f, PAGE, pdfText, textWidth } from './pdf';
import { buildPptx, xmlText } from './pptx';
import type { Deflate } from './zip';
import type { Evaluated, Outcome } from './simulation';

/** What `arclang verify --replay CASE` prints, as far as the report reads it. */
export interface Replay {
  time: number[];
  elements: { id: string; instance?: string; values: { name: string; unit?: string | null; series: number[] }[] }[];
  expectations: { text: string; holds: boolean; value?: string; margin?: string; element?: string; first_failure?: number }[];
}

export interface Chart {
  title: string;
  unit: string;
  timeUnit: 's' | 'min' | 'h';
  times: number[];
  values: number[];
  bound: number | null;
  holds: boolean;
  /** The first instant the expectation fails, in `timeUnit`. */
  failureAt: number | null;
}

export interface ReportCase {
  id: string;
  name: string;
  verdict: string;
  context: string;
  values: Evaluated[];
  /** A sweep's or a Monte Carlo case's summary lines. */
  extra: string[];
  charts: Chart[];
}

export interface Report {
  model: string;
  date: string;
  tool: string;
  cases: ReportCase[];
}

/** A value in SI shown in the unit an expectation is written in. */
const SHOWN: Record<string, (v: number) => number> = {
  '°C': (v) => v - 273.15,
  km: (v) => v / 1000,
  kW: (v) => v / 1000,
  MW: (v) => v / 1e6,
  kWh: (v) => v / 3.6e6,
  MJ: (v) => v / 1e6,
  '%': (v) => v * 100,
  ms: (v) => v * 1000,
  min: (v) => v / 60,
  h: (v) => v / 3600,
  'km/h': (v) => v * 3.6,
};

/** The quantity an expectation checks, its bound and its unit: `max("PN-RACKS".gpu_temperature) <= 85 °C`. */
export function parseExpectation(text: string): { element: string | null; name: string; bound: number | null; unit: string } | null {
  const body = text.replace(/^[^:]*:\s(?=[a-z]+\()/i, ''); // "REQ-X: min(...)" -> "min(...)"
  const quantity = /"([^"]+)"\.([A-Za-z_][\w.]*)/.exec(body) ?? /\(\s*([A-Za-z_][\w.]*)\s*\)/.exec(body);
  if (!quantity) return null;
  const [element, name] = quantity.length === 3 && quantity[2] ? [quantity[1], quantity[2]] : [null, quantity[1]];
  const limit = /(?:<=|>=|<|>|==)\s*(-?[\d.]+(?:e-?\d+)?)\s*([^\s"]*)\s*$/.exec(body);
  return { element, name, bound: limit ? Number(limit[1]) : null, unit: limit ? limit[2] : '' };
}

/** At most `n` points of a curve, the extremes kept. */
function thin(times: number[], values: number[], n = 400): { times: number[]; values: number[] } {
  if (times.length <= n) return { times, values };
  const step = times.length / n;
  const t: number[] = [];
  const v: number[] = [];
  for (let i = 0; i < n; i++) {
    const from = Math.floor(i * step);
    const to = Math.min(times.length, Math.floor((i + 1) * step));
    let lo = from;
    let hi = from;
    for (let k = from; k < to; k++) {
      if (values[k] < values[lo]) lo = k;
      if (values[k] > values[hi]) hi = k;
    }
    for (const k of lo < hi ? [lo, hi] : [hi, lo]) {
      t.push(times[k]);
      v.push(values[k]);
    }
  }
  return { times: t, values: v };
}

/** The curves of a replayed case: one per expectation whose quantity it holds. */
export function chartsOf(replay: Replay): Chart[] {
  const end = replay.time[replay.time.length - 1] ?? 0;
  const timeUnit: Chart['timeUnit'] = end > 7200 ? 'h' : end > 120 ? 'min' : 's';
  const toTime = (t: number) => (timeUnit === 'h' ? t / 3600 : timeUnit === 'min' ? t / 60 : t);
  const charts: Chart[] = [];
  for (const expectation of replay.expectations) {
    const parsed = parseExpectation(expectation.text);
    if (!parsed) continue;
    // "PN-RACKS".gpu names the element by id; vehicleBody.distance by its Modelica instance.
    const dot = parsed.name.indexOf('.');
    const byInstance = parsed.element === null && dot > 0 ? replay.elements.find((e) => e.instance === parsed.name.slice(0, dot)) : undefined;
    const element = byInstance ?? replay.elements.find((e) => e.id === (parsed.element ?? expectation.element));
    const name = byInstance ? parsed.name.slice(dot + 1) : parsed.name;
    const series = element?.values.find((v) => v.name === name)?.series;
    if (!series || series.length !== replay.time.length) continue;
    const shown = SHOWN[parsed.unit] ?? ((v: number) => v);
    const curve = thin(replay.time.map(toTime), series.map(shown));
    charts.push({
      title: expectation.text,
      unit: parsed.unit,
      timeUnit,
      times: curve.times,
      values: curve.values,
      bound: parsed.bound,
      holds: expectation.holds,
      failureAt: typeof expectation.first_failure === 'number' ? toTime(expectation.first_failure) : null,
    });
  }
  return charts;
}

/** The report of a run: the outcomes, and the replay of each case when one was kept. */
export function buildReport(model: string, outcomes: Outcome[], replays: Map<string, Replay>, date: Date): Report {
  const cases = outcomes.map((o) => {
    const extra: string[] = [];
    if (o.estimate && o.probability) {
      const pct = (x: number) => `${(x * 100).toFixed(1)} %`;
      extra.push(`Monte Carlo: ${o.estimate.held} of ${o.estimate.total} samples hold; the probability is between ${pct(o.estimate.low)} and ${pct(o.estimate.high)} at ${pct(o.probability.confidence)} confidence, ${pct(o.probability.probability)} required.`);
    } else if (o.points.length) {
      extra.push(`Sweep: ${o.points.length} points, ${o.points.filter((p) => p.holds).length} hold.`);
      for (const p of o.points.filter((p) => !p.holds).slice(0, 6)) extra.push(`does not hold at ${p.label}`);
    }
    if (o.reason && o.verdict !== 'pass') extra.push(o.reason);
    const replay = replays.get(o.test_case);
    return {
      id: o.test_case,
      name: o.name,
      verdict: o.verdict,
      context: [o.verifies.length ? `verifies ${o.verifies.join(', ')}` : '', o.configuration_name ? `configuration ${o.configuration_name}` : '', o.model ? `model ${o.model}` : ''].filter(Boolean).join(' · '),
      values: o.values,
      extra,
      charts: replay && !o.points.length ? chartsOf(replay).slice(0, 2) : [],
    };
  });
  const tool = outcomes.find((o) => o.omc_version)?.omc_version ?? outcomes[0]?.tool ?? '';
  return { model, date: date.toISOString().slice(0, 16).replace('T', ' '), tool, cases };
}

/** Round axis ticks covering [lo, hi]. */
export function ticks(lo: number, hi: number, count = 5): number[] {
  if (!(hi > lo)) return [lo];
  const raw = (hi - lo) / count;
  const magnitude = 10 ** Math.floor(Math.log10(raw));
  const step = [1, 2, 2.5, 5, 10].map((m) => m * magnitude).find((s) => s >= raw) ?? raw;
  const out: number[] = [];
  const decimals = Math.max(0, -Math.floor(Math.log10(step)) + 1);
  for (let v = Math.ceil(lo / step) * step; v <= hi + step * 1e-9; v += step) out.push(Number((Math.round(v / step) * step).toFixed(decimals)));
  return out;
}

const label = (v: number) => (Math.abs(v) >= 1000 || Number.isInteger(v) ? String(Math.round(v)) : String(Number(v.toPrecision(3))));
const VERDICT_COLOR: Record<string, [number, number, number]> = { pass: [0.14, 0.55, 0.3], fail: [0.75, 0.2, 0.15] };
const verdictColor = (v: string) => VERDICT_COLOR[v] ?? [0.4, 0.42, 0.45];

/** A chart in a box: its axes, ticks, the curve, the bound and the first failure, as geometry both renderers draw. */
interface Plot {
  frame: { x: number; y: number; w: number; h: number };
  polyline: [number, number][];
  bound: number | null;
  failure: number | null;
  xTicks: { at: number; text: string }[];
  yTicks: { at: number; text: string }[];
}

/** The geometry of `chart` in a box (origin at the bottom left, y up). */
export function plot(chart: Chart, box: { x: number; y: number; w: number; h: number }): Plot {
  const frame = { x: box.x + 40, y: box.y + 26, w: box.w - 48, h: box.h - 46 };
  const t0 = chart.times[0] ?? 0;
  const t1 = chart.times[chart.times.length - 1] ?? 1;
  let lo = Math.min(...chart.values, ...(chart.bound === null ? [] : [chart.bound]));
  let hi = Math.max(...chart.values, ...(chart.bound === null ? [] : [chart.bound]));
  if (!(hi > lo)) {
    lo -= 1;
    hi += 1;
  }
  const pad = (hi - lo) * 0.08;
  lo -= pad;
  hi += pad;
  const sx = (t: number) => frame.x + ((t - t0) / Math.max(1e-9, t1 - t0)) * frame.w;
  const sy = (v: number) => frame.y + ((v - lo) / (hi - lo)) * frame.h;
  return {
    frame,
    polyline: chart.times.map((t, i) => [sx(t), sy(chart.values[i])]),
    bound: chart.bound === null ? null : sy(chart.bound),
    failure: chart.failureAt === null ? null : sx(chart.failureAt),
    xTicks: ticks(t0, t1).map((t) => ({ at: sx(t), text: label(t) })),
    yTicks: ticks(lo, hi, 4).map((v) => ({ at: sy(v), text: label(v) })),
  };
}

// ---- PDF -----------------------------------------------------------------------------

const MARGIN = 36;

function pdfLine(color: [number, number, number], width: number, points: [number, number][], dash = ''): string {
  if (points.length < 2) return '';
  const path = points.map(([x, y], i) => `${f(x)} ${f(y)} ${i ? 'l' : 'm'}`).join(' ');
  return `q ${color.map(f).join(' ')} RG ${width} w ${dash || '[] 0 d'} 1 j ${path} S Q`;
}

const textAt = (font: 'F1' | 'F2', size: number, color: [number, number, number], x: number, y: number, text: string) =>
  `BT /${font} ${size} Tf ${color.map(f).join(' ')} rg ${f(x)} ${f(y)} Td ${pdfText(text)} Tj ET`;

/** `text` cut to fit `width` points at `size`, with an ellipsis. */
function fit(text: string, size: number, width: number): string {
  if (textWidth(text, size) <= width) return text;
  let cut = text;
  while (cut.length > 1 && textWidth(`${cut}…`, size) > width) cut = cut.slice(0, -1);
  return `${cut}…`;
}

function pdfChart(chart: Chart, box: { x: number; y: number; w: number; h: number }): string {
  const p = plot(chart, box);
  const ink: [number, number, number] = [0.35, 0.37, 0.4];
  const ops = [
    textAt('F2', 9, [0.15, 0.16, 0.2], box.x, box.y + box.h - 10, fit(chart.title, 9, box.w)),
    pdfLine([0.75, 0.75, 0.75], 0.6, [[p.frame.x, p.frame.y], [p.frame.x + p.frame.w, p.frame.y]]),
    pdfLine([0.75, 0.75, 0.75], 0.6, [[p.frame.x, p.frame.y], [p.frame.x, p.frame.y + p.frame.h]]),
  ];
  for (const t of p.xTicks) ops.push(textAt('F1', 7, ink, t.at - textWidth(t.text, 7) / 2, p.frame.y - 10, t.text));
  for (const t of p.yTicks) {
    ops.push(pdfLine([0.92, 0.92, 0.92], 0.4, [[p.frame.x, t.at], [p.frame.x + p.frame.w, t.at]]));
    ops.push(textAt('F1', 7, ink, p.frame.x - 4 - textWidth(t.text, 7), t.at - 2.5, t.text));
  }
  ops.push(textAt('F1', 7, ink, p.frame.x + p.frame.w - textWidth(chart.timeUnit, 7), p.frame.y - 19, chart.timeUnit));
  if (p.bound !== null) ops.push(pdfLine([0.8, 0.25, 0.2], 1, [[p.frame.x, p.bound], [p.frame.x + p.frame.w, p.bound]], '[4 3] 0 d'));
  if (p.failure !== null) ops.push(pdfLine([0.8, 0.25, 0.2], 0.8, [[p.failure, p.frame.y], [p.failure, p.frame.y + p.frame.h]], '[1.5 2] 0 d'));
  ops.push(pdfLine([0.18, 0.37, 0.66], 1.4, p.polyline));
  return ops.join('\n');
}

function pdfFooter(report: Report, index: number, total: number): string {
  const n = `${index + 1} / ${total}`;
  return [textAt('F1', 8, [0.5, 0.5, 0.5], MARGIN, MARGIN - 12, `${report.model} · simulation report · ${report.date}`), textAt('F1', 8, [0.5, 0.5, 0.5], PAGE.width - MARGIN - textWidth(n, 8), MARGIN - 12, n)].join('\n');
}

function pdfTitle(text: string): string {
  const y = PAGE.height - MARGIN - 20;
  return [textAt('F2', 20, [0.13, 0.14, 0.17], MARGIN, y, fit(text, 20, PAGE.width - 2 * MARGIN - 80)), pdfLine([0.8, 0.8, 0.8], 0.75, [[MARGIN, y - 8], [PAGE.width - MARGIN, y - 8]])].join('\n');
}

/** The report as PDF bytes. */
export function buildReportPdf(report: Report): Uint8Array {
  const total = report.cases.length + 1;
  const summary: string[] = [pdfTitle(`Simulation report — ${report.model}`)];
  let y = PAGE.height - MARGIN - 48;
  const passed = report.cases.filter((c) => c.verdict === 'pass').length;
  summary.push(textAt('F1', 10, [0.35, 0.37, 0.4], MARGIN, y, `${report.date} · ${report.tool || 'OpenModelica'} · ${passed} of ${report.cases.length} test case${report.cases.length === 1 ? '' : 's'} pass`));
  y -= 28;
  const columns = [MARGIN, MARGIN + 110, MARGIN + 470, MARGIN + 540];
  ['Test case', 'Name', 'Verdict', 'Verifies'].forEach((h, i) => summary.push(textAt('F2', 9, [0.4, 0.42, 0.45], columns[i], y, h)));
  y -= 6;
  summary.push(pdfLine([0.8, 0.8, 0.8], 0.6, [[MARGIN, y], [PAGE.width - MARGIN, y]]));
  for (const c of report.cases) {
    y -= 18;
    if (y < MARGIN + 20) break;
    summary.push(textAt('F1', 9, [0.15, 0.16, 0.2], columns[0], y, fit(c.id, 9, 104)));
    summary.push(textAt('F1', 9, [0.15, 0.16, 0.2], columns[1], y, fit(c.name, 9, 350)));
    summary.push(textAt('F2', 9, verdictColor(c.verdict), columns[2], y, c.verdict.toUpperCase()));
    summary.push(textAt('F1', 9, [0.35, 0.37, 0.4], columns[3], y, fit(c.context.replace(/ ·.*/, '').replace(/^verifies /, ''), 9, PAGE.width - MARGIN - columns[3])));
  }
  summary.push(pdfFooter(report, 0, total));
  const pages = [{ content: summary.join('\n'), images: [] as Uint8Array[] }];

  report.cases.forEach((c, i) => {
    const ops = [pdfTitle(`${c.id} — ${c.name}`)];
    const badge = c.verdict.toUpperCase();
    const bx = PAGE.width - MARGIN - textWidth(badge, 12) - 16;
    const color = verdictColor(c.verdict);
    ops.push(`q ${color.map(f).join(' ')} rg ${f(bx)} ${f(PAGE.height - MARGIN - 25)} ${f(textWidth(badge, 12) + 16)} 22 re f Q`);
    ops.push(textAt('F2', 12, [1, 1, 1], bx + 8, PAGE.height - MARGIN - 18, badge));
    let y = PAGE.height - MARGIN - 48;
    ops.push(textAt('F1', 9, [0.35, 0.37, 0.4], MARGIN, y, fit(c.context, 9, PAGE.width - 2 * MARGIN)));
    y -= 22;
    for (const v of c.values) {
      ops.push(textAt('F2', 9, v.holds ? VERDICT_COLOR.pass : VERDICT_COLOR.fail, MARGIN, y, v.holds ? 'OK' : 'FAIL'));
      ops.push(textAt('F1', 9, [0.15, 0.16, 0.2], MARGIN + 36, y, fit(`${v.value}   (${v.expect})`, 9, 560)));
      if (v.margin) ops.push(textAt('F1', 9, [0.35, 0.37, 0.4], MARGIN + 610, y, fit(v.margin, 9, PAGE.width - MARGIN - 610 - MARGIN)));
      y -= 15;
    }
    for (const line of c.extra) {
      ops.push(textAt('F1', 9, [0.35, 0.37, 0.4], MARGIN, y, fit(line, 9, PAGE.width - 2 * MARGIN)));
      y -= 14;
    }
    const top = y - 8;
    const bottom = MARGIN + 12;
    if (c.charts.length && top - bottom > 120) {
      const w = (PAGE.width - 2 * MARGIN - (c.charts.length - 1) * 24) / c.charts.length;
      c.charts.forEach((chart, k) => ops.push(pdfChart(chart, { x: MARGIN + k * (w + 24), y: bottom, w, h: top - bottom })));
    }
    ops.push(pdfFooter(report, i + 1, total));
    pages.push({ content: ops.join('\n'), images: [] });
  });
  return assemblePdf(`Simulation report — ${report.model}`, pages);
}

// ---- PowerPoint --------------------------------------------------------------------------

/** A point in EMU; slides are 960 x 540 points, as the PDF pages, so the geometry is shared (y flipped). */
const PT = 12700;
const emu = (v: number) => Math.round(v * PT);
const flip = (y: number) => PAGE.height - y;
const hex = (c: [number, number, number]) => c.map((v) => Math.round(v * 255).toString(16).padStart(2, '0')).join('').toUpperCase();

class Shapes {
  private id = 100;
  readonly xml: string[] = [];

  text(x: number, yTop: number, w: number, text: string, size: number, color: [number, number, number], bold = false): void {
    this.xml.push(`<p:sp><p:nvSpPr><p:cNvPr id="${this.id++}" name="Text"/><p:cNvSpPr txBox="1"/><p:nvPr/></p:nvSpPr><p:spPr><a:xfrm><a:off x="${emu(x)}" y="${emu(yTop)}"/><a:ext cx="${emu(w)}" cy="${emu(size * 1.6)}"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom><a:noFill/></p:spPr>` +
      `<p:txBody><a:bodyPr wrap="none" lIns="0" tIns="0" rIns="0" bIns="0"/><a:lstStyle/><a:p><a:r><a:rPr lang="en-US" sz="${Math.round(size * 100)}"${bold ? ' b="1"' : ''}><a:solidFill><a:srgbClr val="${hex(color)}"/></a:solidFill></a:rPr><a:t>${xmlText(text)}</a:t></a:r></a:p></p:txBody></p:sp>`);
  }

  rect(x: number, yTop: number, w: number, h: number, fill: [number, number, number]): void {
    this.xml.push(`<p:sp><p:nvSpPr><p:cNvPr id="${this.id++}" name="Badge"/><p:cNvSpPr/><p:nvPr/></p:nvSpPr><p:spPr><a:xfrm><a:off x="${emu(x)}" y="${emu(yTop)}"/><a:ext cx="${emu(w)}" cy="${emu(h)}"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom><a:solidFill><a:srgbClr val="${hex(fill)}"/></a:solidFill><a:ln><a:noFill/></a:ln></p:spPr></p:sp>`);
  }

  /** A polyline (points in PDF coordinates, y up), as one freeform shape. */
  line(points: [number, number][], color: [number, number, number], width: number, dash = false): void {
    if (points.length < 2) return;
    const xs = points.map((p) => p[0]);
    const ys = points.map((p) => flip(p[1]));
    const x0 = Math.min(...xs);
    const y0 = Math.min(...ys);
    const w = Math.max(1, Math.max(...xs) - x0);
    const h = Math.max(1, Math.max(...ys) - y0);
    const path = points.map(([x, y], i) => `<a:${i ? 'lnTo' : 'moveTo'}><a:pt x="${emu(x - x0)}" y="${emu(flip(y) - y0)}"/></a:${i ? 'lnTo' : 'moveTo'}>`).join('');
    this.xml.push(`<p:sp><p:nvSpPr><p:cNvPr id="${this.id++}" name="Curve"/><p:cNvSpPr/><p:nvPr/></p:nvSpPr><p:spPr><a:xfrm><a:off x="${emu(x0)}" y="${emu(y0)}"/><a:ext cx="${emu(w)}" cy="${emu(h)}"/></a:xfrm>` +
      `<a:custGeom><a:avLst/><a:gdLst/><a:ahLst/><a:cxnLst/><a:rect l="0" t="0" r="r" b="b"/><a:pathLst><a:path w="${emu(w)}" h="${emu(h)}" fill="none">${path}</a:path></a:pathLst></a:custGeom>` +
      `<a:noFill/><a:ln w="${Math.round(width * PT)}"><a:solidFill><a:srgbClr val="${hex(color)}"/></a:solidFill>${dash ? '<a:prstDash val="dash"/>' : ''}<a:round/></a:ln></p:spPr></p:sp>`);
  }
}

function pptxChart(s: Shapes, chart: Chart, box: { x: number; y: number; w: number; h: number }): void {
  const p = plot(chart, box);
  const ink: [number, number, number] = [0.35, 0.37, 0.4];
  s.text(box.x, flip(box.y + box.h - 2), box.w, chart.title, 9, [0.15, 0.16, 0.2], true);
  s.line([[p.frame.x, p.frame.y], [p.frame.x + p.frame.w, p.frame.y]], [0.75, 0.75, 0.75], 0.6);
  s.line([[p.frame.x, p.frame.y], [p.frame.x, p.frame.y + p.frame.h]], [0.75, 0.75, 0.75], 0.6);
  for (const t of p.xTicks) s.text(t.at - 12, flip(p.frame.y - 3), 30, t.text, 7, ink);
  for (const t of p.yTicks) {
    s.line([[p.frame.x, t.at], [p.frame.x + p.frame.w, t.at]], [0.92, 0.92, 0.92], 0.4);
    s.text(box.x, flip(t.at + 4), 36, t.text, 7, ink);
  }
  s.text(p.frame.x + p.frame.w - 10, flip(p.frame.y - 12), 20, chart.timeUnit, 7, ink);
  if (p.bound !== null) s.line([[p.frame.x, p.bound], [p.frame.x + p.frame.w, p.bound]], [0.8, 0.25, 0.2], 1, true);
  if (p.failure !== null) s.line([[p.failure, p.frame.y], [p.failure, p.frame.y + p.frame.h]], [0.8, 0.25, 0.2], 0.8, true);
  s.line(p.polyline, [0.18, 0.37, 0.66], 1.4);
}

/** The report as .pptx bytes: the same pages as the PDF, as slides of editable shapes. */
export function buildReportPptx(report: Report, deflate?: Deflate): Uint8Array {
  const passed = report.cases.filter((c) => c.verdict === 'pass').length;
  const summary = new Shapes();
  summary.text(MARGIN, 84, 800, `${report.date} · ${report.tool || 'OpenModelica'} · ${passed} of ${report.cases.length} test case${report.cases.length === 1 ? '' : 's'} pass`, 10, [0.35, 0.37, 0.4]);
  const columns = [MARGIN, MARGIN + 110, MARGIN + 470, MARGIN + 540];
  ['Test case', 'Name', 'Verdict', 'Verifies'].forEach((h, i) => summary.text(columns[i], 110, 200, h, 9, [0.4, 0.42, 0.45], true));
  report.cases.slice(0, 22).forEach((c, k) => {
    const y = 130 + k * 17;
    summary.text(columns[0], y, 104, c.id, 9, [0.15, 0.16, 0.2]);
    summary.text(columns[1], y, 350, c.name, 9, [0.15, 0.16, 0.2]);
    summary.text(columns[2], y, 60, c.verdict.toUpperCase(), 9, verdictColor(c.verdict), true);
    summary.text(columns[3], y, 360, c.context.replace(/ ·.*/, '').replace(/^verifies /, ''), 9, [0.35, 0.37, 0.4]);
  });
  const slides = [{ title: `Simulation report — ${report.model}`, width: 1, height: 1, notes: report.cases.map((c) => `${c.id} ${c.verdict}`), shapes: summary.xml.join('') }];
  for (const c of report.cases) {
    const s = new Shapes();
    const badge = c.verdict.toUpperCase();
    s.rect(PAGE.width - MARGIN - textWidth(badge, 12) - 16, 26, textWidth(badge, 12) + 16, 22, verdictColor(c.verdict));
    s.text(PAGE.width - MARGIN - textWidth(badge, 12) - 8, 30, 80, badge, 12, [1, 1, 1], true);
    s.text(MARGIN, 84, 880, c.context, 9, [0.35, 0.37, 0.4]);
    let y = 104;
    for (const v of c.values) {
      s.text(MARGIN, y, 30, v.holds ? 'OK' : 'FAIL', 9, v.holds ? VERDICT_COLOR.pass : VERDICT_COLOR.fail, true);
      s.text(MARGIN + 36, y, 560, `${v.value}   (${v.expect})`, 9, [0.15, 0.16, 0.2]);
      if (v.margin) s.text(MARGIN + 610, y, 300, v.margin, 9, [0.35, 0.37, 0.4]);
      y += 15;
    }
    for (const line of c.extra) {
      s.text(MARGIN, y, 880, line, 9, [0.35, 0.37, 0.4]);
      y += 14;
    }
    const top = flip(y + 8);
    const bottom = MARGIN + 12;
    if (c.charts.length && top - bottom > 120) {
      const w = (PAGE.width - 2 * MARGIN - (c.charts.length - 1) * 24) / c.charts.length;
      c.charts.forEach((chart, k) => pptxChart(s, chart, { x: MARGIN + k * (w + 24), y: bottom, w, h: top - bottom }));
    }
    slides.push({ title: `${c.id} — ${c.name}`, width: 1, height: 1, notes: [c.context, ...c.values.map((v) => `${v.holds ? 'OK' : 'FAIL'} ${v.value} (${v.expect}) ${v.margin ?? ''}`)], shapes: s.xml.join('') });
  }
  return buildPptx({ title: `Simulation report — ${report.model}`, slides }, deflate);
}
