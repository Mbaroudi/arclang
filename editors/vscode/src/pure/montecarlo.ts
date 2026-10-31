// The Monte Carlo results of `arclang verify --json`, as a page draws them:
// for each case, the probability its samples show against the one required,
// the spread of each margin with its bound, what moves each margin, and the
// failing samples. Pure: the compiler's outcomes in, plain numbers out. The
// verdict is the compiler's; nothing here judges again.
import type { Evaluated, Outcome } from './simulation';

export interface Bin {
  /** The margin, in the expectation's unit, from `from` to `to`. */
  from: number;
  to: number;
  count: number;
  /** The bin lies below the bound: its samples fail. */
  failing: boolean;
}

export interface Margin {
  expect: string;
  /** `km`, or '' for a plain number. */
  unit: string;
  /** Samples whose margin is known. */
  samples: number;
  failing: number;
  smallest: number;
  largest: number;
  /** The margin does not vary (the run ends on its bound): no parameter moves it. */
  constant: boolean;
  bins: Bin[];
  influences: { parameter: string; rho: number }[];
}

export interface FailingSample {
  /** The sample's place among the points: what a replay is asked for. */
  index: number;
  label: string;
  failed: string[];
  error: string;
}

export interface MonteCarloCase {
  id: string;
  name: string;
  verdict: string;
  reason: string;
  required: { probability: number; confidence: number };
  estimate: { held: number; total: number; low: number; high: number } | null;
  margins: Margin[];
  failing: FailingSample[];
  parameters: string[];
}

/** Bins a histogram has at most, and at least when the margin varies. */
export const MAX_BINS = 24;
export const MIN_BINS = 6;
/** Failing samples the page lists. */
export const LISTED_FAILURES = 25;

/** The cases of a run that sample their uncertain parameters. */
export function monteCarloCases(outcomes: Outcome[]): MonteCarloCase[] {
  return outcomes.filter((o) => o.probability !== null).map(toCase);
}

function toCase(o: Outcome): MonteCarloCase {
  const ran = o.points.filter((p) => !p.error);
  const expects = ran[0]?.values ?? [];
  return {
    id: o.test_case,
    name: o.name || o.test_case,
    verdict: o.verdict,
    reason: o.reason,
    required: o.probability!,
    estimate: o.estimate,
    margins: expects
      .map((first, e) => margin(first, ran.map((p) => p.values[e]).filter((v): v is Evaluated => v !== undefined && v.expect === first.expect), o.sensitivity))
      .filter((m): m is Margin => m !== null),
    failing: o.points
      .map((p, index) => ({ index, label: p.label, failed: p.values.filter((v) => !v.holds).map((v) => v.value), error: p.error, holds: p.holds }))
      .filter((p) => !p.holds)
      .slice(0, LISTED_FAILURES)
      .map(({ holds: _holds, ...rest }) => rest),
    parameters: o.points[0]?.inputs.map(([name]) => name) ?? [],
  };
}

function margin(first: Evaluated, values: Evaluated[], sensitivity: Outcome['sensitivity']): Margin | null {
  const factor = first.slack_unit?.factor ?? 1;
  const margins = values.map((v) => v.slack).filter((s): s is number => s !== undefined).map((s) => s / factor);
  if (margins.length === 0) return null;
  const smallest = Math.min(...margins);
  const largest = Math.max(...margins);
  const constant = isConstant(values.map((v) => v.slack!).filter((s) => s !== undefined));
  return {
    expect: first.expect,
    unit: first.slack_unit?.symbol ?? '',
    samples: margins.length,
    failing: values.filter((v) => !v.holds).length,
    smallest,
    largest,
    constant,
    bins: histogram(margins, constant),
    influences: constant ? [] : sensitivity.filter((i) => i.expect === first.expect).map((i) => ({ parameter: i.parameter, rho: i.rank_correlation })),
  };
}

/** The compiler's test, on the margins in SI: a spread below a millionth of the largest is numerical noise. */
export function isConstant(margins: number[]): boolean {
  if (margins.length === 0) return true;
  const spread = Math.max(...margins) - Math.min(...margins);
  return spread <= 1e-6 * Math.max(1, ...margins.map(Math.abs));
}

/** A histogram whose bins meet at the bound (0), so a bin holds only failing or only holding samples. */
export function histogram(margins: number[], constant = false): Bin[] {
  if (margins.length === 0) return [];
  const low = Math.min(...margins);
  const high = Math.max(...margins);
  if (constant || high - low <= 0) return [{ from: low, to: high, count: margins.length, failing: high < 0 }];
  const count = Math.min(MAX_BINS, Math.max(MIN_BINS, Math.ceil(Math.sqrt(margins.length))));
  const width = niceStep((high - low) / count);
  const start = Math.floor(low / width) * width;
  const bins: Bin[] = [];
  for (let from = start; from <= high && bins.length < MAX_BINS * 2; from += width) {
    bins.push({ from, to: from + width, count: 0, failing: from + width <= 0 });
  }
  for (const m of margins) {
    const at = Math.min(bins.length - 1, Math.max(0, Math.floor((m - start) / width)));
    bins[at].count += 1;
  }
  return bins;
}

/** 1, 2 or 5 times a power of ten, at least `raw`: bins whose edges read well. */
export function niceStep(raw: number): number {
  const power = 10 ** Math.floor(Math.log10(raw));
  const nice = [1, 2, 5, 10].find((m) => m * power >= raw * (1 - 1e-12)) ?? 10;
  return nice * power;
}

/** Four significant digits, without trailing zeros: `0.1411`, `20.5`, `-3e-7`. */
export function shortNumber(value: number): string {
  if (value === 0) return '0';
  const magnitude = Math.abs(value);
  if (magnitude >= 1e6 || magnitude < 1e-4) return value.toExponential(2).replace(/\.?0+e/, 'e');
  return String(Number(value.toPrecision(4)));
}

/** `speedController.target_speed` for `vehicleControlUnit.speedController.target_speed`: the end that names it. */
export function shortParameter(name: string): string {
  const parts = name.split('.');
  return parts.length > 2 ? parts.slice(-2).join('.') : name;
}

/** The left end of the probability axis: low enough to show the bound, the interval and the requirement. */
export function axisStart(estimate: MonteCarloCase['estimate'], required: number): number {
  const lowest = Math.min(required, estimate ? Math.min(estimate.low, estimate.held / estimate.total) : required);
  return Math.max(0, Math.floor((lowest - 0.05) * 20) / 20);
}

// ---- The analysis of R or Python (`--statistics`) ----------------------------------

export interface StatisticsInfluence {
  method: string;
  parameter: string;
  value: number | null;
  low: number | null;
  high: number | null;
}

export interface StatisticsMargin {
  expect: string;
  samples: number;
  failureFraction: number | null;
  fittedFailure: number | null;
  constant: boolean;
  influences: StatisticsInfluence[];
}

export interface Statistics {
  engine: string;
  margins: StatisticsMargin[];
}

const numberOrNull = (v: unknown): number | null => (typeof v === 'number' && Number.isFinite(v) ? v : null);

/** The report `statistics.json` holds, or null when it is anything else. */
export function parseStatistics(text: string): Statistics | null {
  let value: unknown;
  try {
    value = JSON.parse(text);
  } catch {
    return null;
  }
  if (!value || typeof value !== 'object') return null;
  const r = value as Record<string, unknown>;
  if (!Array.isArray(r.margins)) return null;
  return {
    engine: typeof r.engine === 'string' ? r.engine : '?',
    margins: (r.margins as Record<string, unknown>[])
      .filter((m) => m && typeof m.expect === 'string')
      .map((m) => ({
        expect: m.expect as string,
        samples: numberOrNull(m.samples) ?? 0,
        failureFraction: numberOrNull(m.failure_fraction),
        fittedFailure: numberOrNull(m.fitted_failure_probability),
        constant: m.constant === true,
        influences: Array.isArray(m.influences)
          ? (m.influences as Record<string, unknown>[])
              .filter((i) => i && typeof i.parameter === 'string')
              .map((i) => ({ method: typeof i.method === 'string' ? i.method : '?', parameter: i.parameter as string, value: numberOrNull(i.value), low: numberOrNull(i.low), high: numberOrNull(i.high) }))
          : [],
      })),
  };
}

/** A probability as a reader takes it: `0.25 %`, `3.1e-7`, `0 %`. */
export function probabilityText(p: number | null): string {
  if (p === null) return '?';
  if (p > 0 && p < 1e-4) return p.toExponential(1);
  return `${Number((p * 100).toFixed(2))} %`;
}
