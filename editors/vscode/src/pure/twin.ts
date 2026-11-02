// A digital twin's outcome, as `arclang twin --json` prints it, and its
// curves drawn as SVG: the measurement, the model driven by the data, the
// tolerance band around the measurement, the first instant beyond it.
// Free of the DOM and of VS Code.
import { escapeXml } from './modelica-drawing';

export interface TwinCurve {
  time: number[];
  measured: number[];
  simulated: number[];
}

export interface TwinComparison {
  target: string;
  variable: string;
  column: string;
  tolerance: string;
  tolerance_si: number;
  unit: string;
  unit_factor: number;
  unit_offset: number;
  samples: number;
  max_deviation: number;
  rms_deviation: number;
  within: number;
  required: number;
  first_exceedance: number | null;
  holds: boolean;
  curve: TwinCurve;
}

export interface TwinValue {
  expect: string;
  value: string;
  holds: boolean;
}

export interface TwinOutcome {
  twin: string;
  name: string;
  verifies: string[];
  verdict: 'pass' | 'fail' | 'error' | 'inconclusive' | string;
  reason: string;
  data: string;
  data_start: string;
  samples: number;
  duration: number;
  comparisons: TwinComparison[];
  values: TwinValue[];
  tool: string;
  omc_version: string;
  seconds: number;
  log: string;
}

export function parseTwinOutcomes(text: string): TwinOutcome[] | null {
  try {
    const parsed = JSON.parse(text) as unknown;
    return Array.isArray(parsed) && parsed.every((o) => typeof o === 'object' && o !== null && 'verdict' in o) ? (parsed as TwinOutcome[]) : null;
  } catch {
    return null;
  }
}

/** A value of the column in its own unit (°C from K: the offset; km/h from m/s: the factor). */
export const inUnit = (si: number, c: Pick<TwinComparison, 'unit_factor' | 'unit_offset'>): number => (si - c.unit_offset) / (c.unit_factor || 1);
/** A difference in the column's unit: the factor only. */
export const differenceInUnit = (si: number, c: Pick<TwinComparison, 'unit_factor'>): number => si / (c.unit_factor || 1);

/** A duration as people read it: 45 s, 12 min, 3.5 h, 2.1 d. */
export function duration(seconds: number): string {
  if (seconds < 120) return `${Math.round(seconds)} s`;
  if (seconds < 7200) return `${Math.round(seconds / 60)} min`;
  if (seconds < 172800) return `${(seconds / 3600).toFixed(1).replace(/\.0$/, '')} h`;
  return `${(seconds / 86400).toFixed(1).replace(/\.0$/, '')} d`;
}

/** Round ticks covering [lo, hi]. */
export function niceTicks(lo: number, hi: number, count = 5): number[] {
  if (!(hi > lo)) return [lo];
  const raw = (hi - lo) / count;
  const magnitude = 10 ** Math.floor(Math.log10(raw));
  const step = [1, 2, 2.5, 5, 10].map((m) => m * magnitude).find((s) => s >= raw) ?? raw;
  const ticks: number[] = [];
  for (let v = Math.ceil(lo / step) * step; v <= hi + step * 1e-9; v += step) ticks.push(Math.round(v / step) * step);
  return ticks;
}

const number = (v: number) => (Math.abs(v) >= 1000 || Number.isInteger(v) ? String(Math.round(v * 100) / 100) : v.toFixed(Math.abs(v) < 1 ? 3 : 2).replace(/0+$/, '').replace(/\.$/, ''));

/** The curves of a comparison as an SVG chart; colors from CSS variables so that the theme decides. */
export function twinChart(c: TwinComparison, size = { width: 720, height: 260 }): string {
  const { time, measured, simulated } = c.curve;
  const pad = { left: 56, right: 16, top: 14, bottom: 34 };
  const w = size.width - pad.left - pad.right;
  const h = size.height - pad.top - pad.bottom;
  if (time.length === 0) return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${size.width} ${size.height}"><text x="${size.width / 2}" y="${size.height / 2}" text-anchor="middle" class="note">No sample of the run matches the data.</text></svg>`;
  const m = measured.map((v) => inUnit(v, c));
  const s = simulated.map((v) => inUnit(v, c));
  const band = differenceInUnit(c.tolerance_si, c);
  const values = [...m.map((v) => v - band), ...m.map((v) => v + band), ...s];
  let lo = Math.min(...values);
  let hi = Math.max(...values);
  if (hi - lo < 1e-9) [lo, hi] = [lo - 1, hi + 1];
  const margin = (hi - lo) * 0.06;
  [lo, hi] = [lo - margin, hi + margin];
  const t0 = time[0];
  const t1 = time[time.length - 1] > t0 ? time[time.length - 1] : t0 + 1;
  const x = (t: number) => pad.left + ((t - t0) / (t1 - t0)) * w;
  const y = (v: number) => pad.top + (1 - (v - lo) / (hi - lo)) * h;
  const line = (vs: number[]) => vs.map((v, i) => `${i ? 'L' : 'M'}${x(time[i]).toFixed(1)} ${y(v).toFixed(1)}`).join('');
  const bandPath = `${m.map((v, i) => `${i ? 'L' : 'M'}${x(time[i]).toFixed(1)} ${y(v + band).toFixed(1)}`).join('')}${[...m].reverse().map((v, j) => `L${x(time[time.length - 1 - j]).toFixed(1)} ${y(v - band).toFixed(1)}`).join('')}Z`;
  const yTicks = niceTicks(lo, hi, 5);
  const xTicks = niceTicks(0, t1 - t0, 6);
  const unit = escapeXml(c.unit);
  const parts = [
    `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${size.width} ${size.height}" role="img" aria-label="${escapeXml(`${c.target} against ${c.column}`)}">`,
    ...yTicks.map((v) => `<line class="grid" x1="${pad.left}" x2="${pad.left + w}" y1="${y(v).toFixed(1)}" y2="${y(v).toFixed(1)}"/><text class="tick" x="${pad.left - 6}" y="${(y(v) + 4).toFixed(1)}" text-anchor="end">${number(v)}</text>`),
    ...xTicks.map((t) => `<text class="tick" x="${x(t0 + t).toFixed(1)}" y="${pad.top + h + 16}" text-anchor="middle">${duration(t)}</text>`),
    `<text class="tick" x="${pad.left - 6}" y="${pad.top - 2}" text-anchor="end">${unit}</text>`,
    `<path class="band" d="${bandPath}"/>`,
    `<path class="measured" d="${line(m)}"/>`,
    `<path class="simulated" d="${line(s)}"/>`,
  ];
  if (c.first_exceedance !== null && c.first_exceedance !== undefined) {
    const fx = x(t0 + c.first_exceedance).toFixed(1);
    parts.push(`<line class="exceed" x1="${fx}" x2="${fx}" y1="${pad.top}" y2="${pad.top + h}"/><text class="exceed-label" x="${fx}" y="${pad.top + 10}" dx="4">first beyond, ${duration(c.first_exceedance)}</text>`);
  }
  parts.push('</svg>');
  return parts.join('');
}

/** One line per outcome, for the notification. */
export function twinSummary(outcomes: TwinOutcome[]): string {
  const count = (v: string) => outcomes.filter((o) => o.verdict === v).length;
  const parts = [['pass', 'hold'], ['fail', 'depart from the data'], ['error', 'could not run'], ['inconclusive', 'inconclusive']]
    .map(([v, label]) => [count(v), label] as const)
    .filter(([n]) => n > 0)
    .map(([n, label]) => `${n} ${label}`);
  return `${outcomes.length} twin${outcomes.length === 1 ? '' : 's'}: ${parts.join(', ')}`;
}
