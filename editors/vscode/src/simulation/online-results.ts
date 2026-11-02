// The results of the last online run of each model (the browser build on
// arclang.fr): what the desktop reads back from `.arclang/sim-results` with
// `arclang verify --replay` and `--statistics`, computed by the server in
// the same job, kept here so that the replay on the diagram, the Monte Carlo
// page's statistics and the report work the same in the browser. The
// desktop never fills it.

export interface OnlineStatistics {
  report?: unknown;
  error?: string;
}

export interface OnlineRun {
  cases: { id: string; name: string; replayable: boolean }[];
  /** By test case, then by point: the replay as `arclang verify --replay --json` prints it. */
  replays: Record<string, Record<string, unknown>>;
  /** By test case, then by engine (r, python). */
  statistics: Record<string, Record<string, OnlineStatistics>>;
  /** The verdicts kept by the run (`.arclang/sim-cache/<case>.json` on the desktop): sent with the compiler online's commands. */
  cache?: Record<string, string>;
}

const runs = new Map<string, OnlineRun>();

export function keepOnlineRun(file: string, run: OnlineRun): void {
  runs.set(file, run);
}

export function onlineRun(file: string): OnlineRun | undefined {
  return runs.get(file);
}

/** The replay of a point as JSON text, or why there is none; undefined when the model has no online run. */
export function onlineReplay(file: string, testCase: string, point: number): { text: string } | { error: string } | undefined {
  const run = runs.get(file);
  if (!run) return undefined;
  const value = run.replays[testCase]?.[String(point)];
  if (value) return { text: JSON.stringify(value) };
  return { error: run.replays[testCase] ? `the online run keeps the first and the failing points only; point ${point + 1} is not one of them` : `'${testCase}' has no results of the online run` };
}

/** The statistics of a Monte Carlo case with an engine; undefined when the model has no online run. */
export function onlineStatistics(file: string, testCase: string, engine: string): OnlineStatistics | undefined {
  const run = runs.get(file);
  if (!run) return undefined;
  return run.statistics[testCase]?.[engine] ?? { error: 'the online run did not analyze this case' };
}
