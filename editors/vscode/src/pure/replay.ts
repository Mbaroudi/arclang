// Replaying a simulation on the diagram: the results `arclang verify
// --replay` reads back (the time, each part's values, each expectation and
// the first instant it fails), and what the diagram shows at one instant.

export interface ReplayVariable {
  name: string;
  unit?: string;
  series: number[];
}

export interface ReplayElement {
  id: string;
  instance: string;
  values: ReplayVariable[];
}

export interface ReplayExpectation {
  text: string;
  requirement?: string;
  holds: boolean;
  value: string;
  margin: string;
  first_failure?: number;
  element?: string;
}

export interface Replay {
  case: string;
  point: number;
  points: string[];
  stale: boolean;
  time: number[];
  elements: ReplayElement[];
  expectations: ReplayExpectation[];
}

/** The message the workbench page reads: a replay to show, or null to stop. */
export interface ReplayMessage {
  type: 'replay-load';
  replay: Replay | null;
  /** The case's name, as the bar shows it. */
  title: string;
}

/** The replay in the compiler's JSON, or null when it is anything else. */
export function parseReplay(text: string): Replay | null {
  try {
    const value = JSON.parse(text) as Replay;
    if (!value || !Array.isArray(value.time) || !Array.isArray(value.elements) || !Array.isArray(value.expectations)) return null;
    return value;
  } catch {
    return null;
  }
}

/** The step at or just before `t`. */
export function stepAt(time: number[], t: number): number {
  let low = 0;
  let high = time.length - 1;
  if (high < 0) return 0;
  if (t <= time[0]) return 0;
  if (t >= time[high]) return high;
  while (low < high) {
    const middle = Math.ceil((low + high) / 2);
    if (time[middle] <= t) low = middle;
    else high = middle - 1;
  }
  return low;
}

/** A variable's value at `t`, linearly interpolated between two steps. */
export function valueAt(time: number[], series: number[], t: number): number {
  const i = stepAt(time, t);
  const next = i + 1;
  if (next >= time.length || time[next] <= time[i]) return series[i];
  return series[i] + ((series[next] - series[i]) * (t - time[i])) / (time[next] - time[i]);
}

/** `2246.4` reads `37 min 26 s`, `3.2` reads `3.2 s`, `7200` reads `2 h 0 min`. */
export function timeLabel(seconds: number): string {
  if (seconds < 60) return `${Math.round(seconds * 10) / 10} s`;
  if (seconds < 3600) return `${Math.floor(seconds / 60)} min ${Math.round(seconds % 60)} s`;
  return `${Math.floor(seconds / 3600)} h ${Math.round((seconds % 3600) / 60)} min`;
}

/** A value for a box: four significant digits and its unit; a flag (0 or 1, no unit) reads yes or no. */
export function valueLabel(value: number, unit?: string): string {
  if (!unit && (value === 0 || value === 1)) return value === 1 ? 'yes' : 'no';
  const shown = Math.abs(value) >= 1e4 || (Math.abs(value) > 0 && Math.abs(value) < 1e-3) ? value.toExponential(3) : String(Number(value.toPrecision(4)));
  return unit && unit !== '1' ? `${shown} ${unit}` : shown;
}

/** The lines each box shows at `t`: `voltage 23.73 V`. */
export function tagsAt(replay: Replay, t: number): Map<string, string[]> {
  const out = new Map<string, string[]>();
  for (const element of replay.elements) {
    out.set(element.id, element.values.map((v) => `${v.name} ${valueLabel(valueAt(replay.time, v.series, t), v.unit)}`));
  }
  return out;
}

/** The elements whose expectation has failed by `t`, with what failed. */
export function failingAt(replay: Replay, t: number): Map<string, ReplayExpectation[]> {
  const out = new Map<string, ReplayExpectation[]>();
  for (const expectation of replay.expectations) {
    if (expectation.first_failure === undefined || expectation.first_failure > t || !expectation.element) continue;
    out.set(expectation.element, [...(out.get(expectation.element) ?? []), expectation]);
  }
  return out;
}

/** The first instant an expectation fails, or null when every one holds. */
export function firstFailure(replay: Replay): number | null {
  const times = replay.expectations.map((e) => e.first_failure).filter((t): t is number => t !== undefined);
  return times.length ? Math.min(...times) : null;
}
