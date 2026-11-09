// OpenModelica online (the browser build on arclang.fr): which files of the
// model's folder are sent to the sandboxed OpenModelica, and how the answer
// is read. The names follow the server's rule: a name it would refuse (a
// dot file, the compiler's own `.arclang` folder) is never sent.

/** The files a simulation may read: models, Modelica sources, scripts and data (a twin's measurements too). */
const SENT = /\.(arc|mo|mos|csv|txt|json|tsv|mat|parquet)$/i;
/** The files sent as bytes (base64), not as text. */
const BINARY = /\.(mat|parquet)$/i;

/** Whether a file goes as bytes (base64) rather than text. */
export function sentAsBytes(rel: string): boolean {
  return BINARY.test(rel);
}
const SEGMENT = /^[A-Za-z0-9_][A-Za-z0-9._ -]{0,99}$/;
const SKIPPED_DIRS = new Set(['node_modules', 'target', 'dist', 'out']);

export const ONLINE_LIMITS = { files: 200, bytes: 2_000_000, depth: 8 };

/** Whether a path relative to the model's folder can be sent. */
export function sendablePath(rel: string): boolean {
  const parts = rel.split('/');
  return (
    parts.length <= ONLINE_LIMITS.depth &&
    parts.every((p) => SEGMENT.test(p) && !p.endsWith('.') && !p.endsWith(' ')) &&
    !parts.slice(0, -1).some((p) => SKIPPED_DIRS.has(p)) &&
    SENT.test(parts[parts.length - 1])
  );
}

/** Whether a folder is worth walking into. */
export function walkable(name: string): boolean {
  return SEGMENT.test(name) && !SKIPPED_DIRS.has(name);
}

/**
 * The files sent with the model: the model first, then the others by path,
 * within the limits. `error` says why the model itself cannot go.
 */
export function pickFiles(model: string, found: { path: string; size: number }[]): { take: string[]; left: number; error?: string } {
  const modelFile = found.find((f) => f.path === model);
  if (!sendablePath(model) || !modelFile) return { take: [], left: 0, error: `the model's file name (${model}) cannot be sent: use letters, digits, spaces, dots, dashes and underscores` };
  if (modelFile.size > ONLINE_LIMITS.bytes) return { take: [], left: 0, error: `the model is larger than ${ONLINE_LIMITS.bytes / 1e6} MB` };
  const take = [model];
  let bytes = modelFile.size;
  const others = found.filter((f) => f.path !== model && sendablePath(f.path)).sort((a, b) => a.path.localeCompare(b.path));
  let left = 0;
  for (const f of others) {
    if (take.length >= ONLINE_LIMITS.files || bytes + f.size > ONLINE_LIMITS.bytes) {
      left += 1;
      continue;
    }
    take.push(f.path);
    bytes += f.size;
  }
  return { take, left };
}

/** `· 19 online runs left today`, or nothing when the server did not say. */
export function leftText(left: unknown): string {
  if (typeof left !== 'number' || !Number.isFinite(left)) return '';
  return ` · ${left} online run${left === 1 ? '' : 's'} left today`;
}

// ---- Co-simulations online: the test cases that execute:, in the same job ----

/** Whether a model has test cases that execute: (co-simulated online with its simulations). */
export function executes(text: string): boolean {
  return /\bexecute\s*:\s*\{/.test(text.replace(/\/\/[^\n]*/g, ''));
}

/** One co-simulated case, as `arclang cosim --json` gives it. */
export interface CosimCase {
  test_case: string;
  name: string;
  verifies: string[];
  verdict: string;
  reason: string;
}

/** The co-simulated cases of an online answer; [] when it has none. */
export function cosimCases(value: unknown): CosimCase[] {
  if (!Array.isArray(value)) return [];
  return value
    .filter((c): c is Record<string, unknown> => !!c && typeof c === 'object' && typeof (c as { test_case?: unknown }).test_case === 'string')
    .map((c) => ({
      test_case: String(c.test_case),
      name: typeof c.name === 'string' ? c.name : String(c.test_case),
      verifies: Array.isArray(c.verifies) ? c.verifies.filter((v): v is string => typeof v === 'string') : [],
      verdict: typeof c.verdict === 'string' ? c.verdict : 'error',
      reason: typeof c.reason === 'string' ? c.reason : '',
    }))
    .filter((c) => c.verdict !== 'skipped');
}

/**
 * "1 co-simulation (OMSimulator) passes." / "2 co-simulations: 1 pass, 1 fail."
 * With `stage`, the SIL cases the server ran the same way: "All 3 SIL cases pass."
 */
export function cosimText(cases: CosimCase[], stage: { one: string; many: string } = { one: 'co-simulation (OMSimulator)', many: 'co-simulations (OMSimulator)' }): string {
  if (!cases.length) return '';
  const pass = cases.filter((c) => c.verdict === 'pass').length;
  const fail = cases.filter((c) => c.verdict === 'fail').length;
  const other = cases.length - pass - fail;
  if (pass === cases.length) return `${cases.length === 1 ? `1 ${stage.one} passes` : `All ${cases.length} ${stage.many} pass`}.`;
  return `${cases.length} ${cases.length > 1 ? stage.many : stage.one}: ${[pass && `${pass} pass`, fail && `${fail} fail`, other && `${other} error`].filter(Boolean).join(', ')}.`;
}

/** The SIL cases of an online answer (the C compiled and run on the server), as the co-simulated ones. */
export const SIL_STAGE = { one: 'SIL case', many: 'SIL cases' };

/** The log of the co-simulated cases: one line each, with its reason when it does not pass. */
export function cosimLog(cases: CosimCase[]): string {
  return cases.map((c) => `   ${c.verdict.toUpperCase().padEnd(5)} ${c.test_case}  ${c.name}${c.verifies.length ? `  (verifies ${c.verifies.join(', ')})` : ''}${c.verdict !== 'pass' && c.reason ? `\n         ${c.reason}` : ''}`).join('\n');
}
