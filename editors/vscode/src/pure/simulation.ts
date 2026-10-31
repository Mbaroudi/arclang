// Simulation with OpenModelica, without the editor: what `arclang verify
// --check-tools --json` says, the checklist and the one-click fixes that come
// out of it, the state of the status bar item, and the verdicts of `arclang
// verify --json`. The compiler is the one place that knows how OpenModelica
// is found; nothing here repeats that.

/** `arclang verify --check-tools --json`. */
export interface Diagnosis {
  tool: 'local' | 'docker' | null;
  detail: string;
  docker_installed: boolean;
  docker_running: boolean;
  image: string;
  image_pulled: boolean | null;
  library_volume: string;
  msl_version: string;
  library_installed: boolean | null;
  local_omc: boolean;
}

export const DOCKER_DOWNLOAD_URL = 'https://www.docker.com/products/docker-desktop/';
export const DOCS_URL = 'https://github.com/Mbaroudi/arclang/blob/main/docs/modelica.md';
export const MODEX_ID = 'modexai.modex-vscode';

const isBoolOrNull = (v: unknown): v is boolean | null => v === null || typeof v === 'boolean';

/** The diagnosis in the compiler's JSON, or null when it is anything else. */
export function parseDiagnosis(text: string): Diagnosis | null {
  let value: unknown;
  try {
    value = JSON.parse(text);
  } catch {
    return null;
  }
  if (typeof value !== 'object' || value === null) return null;
  const d = value as Record<string, unknown>;
  const tool = d.tool === 'local' || d.tool === 'docker' ? d.tool : d.tool === null ? null : undefined;
  if (tool === undefined || typeof d.detail !== 'string' || typeof d.image !== 'string') return null;
  if (typeof d.docker_installed !== 'boolean' || typeof d.docker_running !== 'boolean' || typeof d.local_omc !== 'boolean') return null;
  if (!isBoolOrNull(d.image_pulled) || !isBoolOrNull(d.library_installed)) return null;
  return {
    tool,
    detail: d.detail,
    docker_installed: d.docker_installed,
    docker_running: d.docker_running,
    image: d.image,
    image_pulled: d.image_pulled,
    library_volume: typeof d.library_volume === 'string' ? d.library_volume : 'arclang-openmodelica-libraries',
    msl_version: typeof d.msl_version === 'string' ? d.msl_version : '4.1.0',
    library_installed: d.library_installed,
    local_omc: d.local_omc,
  };
}

/** A Docker image name that cannot be taken for an option or a second argument. */
export function safeImageName(image: string): boolean {
  return /^[A-Za-z0-9][A-Za-z0-9._/:@-]{0,200}$/.test(image);
}

// ---- Settings -> environment ---------------------------------------------------

export interface SimulationSettings {
  image: string;
  omc: string;
  timeout: number;
}

/** The environment variables the compiler reads, from the settings that are set. */
export function simulationEnv(settings: SimulationSettings): Record<string, string> {
  const env: Record<string, string> = {};
  if (settings.image.trim()) env.ARCLANG_OMC_DOCKER = settings.image.trim();
  if (settings.omc.trim()) env.ARCLANG_OMC = settings.omc.trim();
  if (Number.isFinite(settings.timeout) && settings.timeout > 0) env.ARCLANG_OMC_TIMEOUT = String(Math.floor(settings.timeout));
  return env;
}

// ---- The checklist and its fixes -----------------------------------------------

export type Fix =
  | { kind: 'installCompiler'; label: string }
  | { kind: 'openUrl'; label: string; url: string }
  | { kind: 'startDocker'; label: string }
  | { kind: 'pullImage'; label: string; image: string }
  | { kind: 'installLibrary'; label: string; image: string; volume: string; msl: string }
  | { kind: 'installExtension'; label: string; id: string }
  | { kind: 'terminal'; label: string; command: string };

/** ok: done; todo: to do; optional: not needed, can help; unknown: cannot be told yet; na: not needed here. */
export type ItemState = 'ok' | 'todo' | 'optional' | 'unknown' | 'na';

export interface ChecklistItem {
  id: string;
  title: string;
  state: ItemState;
  detail: string;
  /** What blocks a simulation if not done. */
  blocking: boolean;
  fixes: Fix[];
}

export interface Checklist {
  /** A simulation can start now. */
  ready: boolean;
  /** The first run installs the Modelica library (minutes). */
  firstRun: boolean;
  headline: string;
  items: ChecklistItem[];
}

export interface Environment {
  platform: string;
  modexInstalled: boolean;
}

/** `compilerMissing`: the `arclang` binary itself was not found (no diagnosis can be had). */
export function checklist(diagnosis: Diagnosis | null, env: Environment): Checklist {
  const items: ChecklistItem[] = [];
  if (!diagnosis) {
    items.push({
      id: 'compiler',
      title: 'ArcLang compiler',
      state: 'todo',
      detail: 'The arclang program was not found, or it is too old to check the simulation tools.',
      blocking: true,
      fixes: [{ kind: 'installCompiler', label: 'Install the ArcLang compiler' }],
    });
    return { ready: false, firstRun: false, headline: 'Simulation needs the ArcLang compiler first.', items: [...items, modexItem(env)] };
  }
  const localOk = diagnosis.tool === 'local';
  const dockerOk = diagnosis.tool === 'docker';
  const usesDocker = !localOk;

  items.push({
    id: 'omc',
    title: 'OpenModelica on this computer (optional)',
    state: diagnosis.local_omc ? 'ok' : 'optional',
    detail: diagnosis.local_omc ? 'A local omc runs; simulations use it.' : 'Not found. You do not need it when Docker is set up below.',
    blocking: false,
    fixes: [],
  });

  const dockerNote = (text: string) => (usesDocker ? text : 'Not needed: a local OpenModelica is used.');
  const na = (state: ItemState): ItemState => (usesDocker ? state : 'na');
  items.push({
    id: 'docker',
    title: 'Docker is installed',
    state: diagnosis.docker_installed ? 'ok' : na('todo'),
    detail: diagnosis.docker_installed ? 'Docker was found.' : dockerNote('Docker runs OpenModelica for you; nothing else to install by hand.'),
    blocking: usesDocker && !diagnosis.docker_installed,
    fixes: diagnosis.docker_installed || !usesDocker ? [] : [{ kind: 'openUrl', label: 'Download Docker Desktop', url: DOCKER_DOWNLOAD_URL }],
  });
  items.push({
    id: 'daemon',
    title: 'Docker is running',
    state: diagnosis.docker_running ? 'ok' : !diagnosis.docker_installed ? na('unknown') : na('todo'),
    detail: diagnosis.docker_running ? 'Docker answers.' : !diagnosis.docker_installed ? 'Install Docker first.' : dockerNote('Docker is installed but not started.'),
    blocking: usesDocker && diagnosis.docker_installed && !diagnosis.docker_running,
    fixes: diagnosis.docker_running || !diagnosis.docker_installed || !usesDocker ? [] : startDockerFixes(env.platform),
  });
  const pulled = diagnosis.image_pulled;
  items.push({
    id: 'image',
    title: `OpenModelica image (${diagnosis.image})`,
    state: pulled === true ? 'ok' : pulled === false ? na('todo') : na('unknown'),
    detail: pulled === true ? 'The image is on this computer.' : pulled === false ? dockerNote('Downloaded once, about 1 GB.') : dockerNote('Start Docker to tell.'),
    blocking: usesDocker && pulled === false,
    fixes: pulled === false && usesDocker && safeImageName(diagnosis.image)
      ? [
          { kind: 'pullImage', label: 'Download the image', image: diagnosis.image },
          { kind: 'terminal', label: 'Show the command in a terminal', command: `docker pull ${diagnosis.image}` },
        ]
      : [],
  });
  const lib = diagnosis.library_installed;
  items.push({
    id: 'library',
    title: `Modelica library ${diagnosis.msl_version}`,
    state: lib === true ? 'ok' : lib === false ? na('todo') : na('unknown'),
    detail: lib === true ? 'Installed: simulations start at once.' : lib === false ? dockerNote('The first simulation downloads it; this takes a few minutes, once. Later runs are cached.') : dockerNote('Known once the image is on this computer.'),
    blocking: false,
    fixes: lib === false && usesDocker && pulled === true && safeImageName(diagnosis.image)
      ? [{ kind: 'installLibrary', label: 'Install it now', image: diagnosis.image, volume: diagnosis.library_volume, msl: diagnosis.msl_version }]
      : [],
  });
  items.push(modexItem(env));

  const ready = diagnosis.tool !== null;
  const firstRun = dockerOk && lib !== true;
  const headline = !ready
    ? 'Simulation needs setup: OpenModelica is not available yet.'
    : localOk
      ? 'Ready: simulations run with the OpenModelica on this computer.'
      : firstRun
        ? 'Ready. The first simulation installs the Modelica library (a few minutes); later runs are cached.'
        : 'Ready: simulations run in Docker.';
  return { ready, firstRun, headline, items };
}

function startDockerFixes(platform: string): Fix[] {
  if (platform === 'darwin') return [{ kind: 'startDocker', label: 'Start Docker Desktop' }];
  if (platform === 'win32') return [{ kind: 'startDocker', label: 'Start Docker Desktop' }];
  return [{ kind: 'terminal', label: 'Show the command in a terminal', command: 'sudo systemctl start docker' }];
}

function modexItem(env: Environment): ChecklistItem {
  return {
    id: 'modex',
    title: 'Modelica drawing (Modex extension, optional)',
    state: env.modexInstalled ? 'ok' : 'optional',
    detail: env.modexInstalled ? 'Modex is installed: .mo files open as drawings.' : 'Lets you draw and read .mo files as diagrams.',
    blocking: false,
    fixes: env.modexInstalled ? [] : [{ kind: 'installExtension', label: 'Install Modex', id: MODEX_ID }],
  };
}

/** The first blocking item with a fix, then the first todo with a fix: the next step of the guided setup. */
export function nextStep(list: Checklist): { item: ChecklistItem; fix: Fix } | null {
  for (const wantBlocking of [true, false]) {
    for (const item of list.items) {
      if (item.state === 'todo' && item.blocking === wantBlocking && item.fixes.length) return { item, fix: item.fixes[0] };
    }
  }
  return null;
}

// ---- The status bar item -------------------------------------------------------

export type SimState = 'hidden' | 'checking' | 'ready' | 'setup' | 'running' | 'result';

export interface Counts {
  pass: number;
  fail: number;
  other: number;
}

export interface SimModel {
  /** There is something to simulate or draw in the workspace, in a trusted desktop window. */
  relevant: boolean;
  checking: boolean;
  running: boolean;
  /** The checklist of the last diagnosis, null before the first. */
  list: Checklist | null;
  toolLabel: string | null;
  last: Counts | null;
}

export interface SimView {
  state: SimState;
  text: string;
  tooltip: string;
  warning: boolean;
  error: boolean;
}

export function toolLabel(diagnosis: Diagnosis | null): string | null {
  if (!diagnosis || !diagnosis.tool) return null;
  return diagnosis.tool === 'docker' ? 'OpenModelica (Docker)' : 'OpenModelica (local)';
}

export function countsText(counts: Counts): string {
  const parts: string[] = [];
  if (counts.fail) parts.push(`${counts.fail} fail`);
  if (counts.other) parts.push(`${counts.other} inconclusive`);
  if (counts.pass) parts.push(`${counts.pass} pass`);
  return parts.join(', ') || 'no case';
}

export function simView(model: SimModel): SimView {
  const none = { warning: false, error: false };
  if (!model.relevant) return { state: 'hidden', text: '', tooltip: '', ...none };
  if (model.running) return { state: 'running', text: '$(sync~spin) Simulating…', tooltip: 'ArcLang is running the simulations. Click for the menu.', ...none };
  if (model.checking && !model.list) return { state: 'checking', text: '$(sync~spin) Simulation', tooltip: 'Looking for OpenModelica…', ...none };
  if (model.list && !model.list.ready) {
    return { state: 'setup', text: '$(warning) Simulation: needs setup', tooltip: `${model.list.headline} Click to set it up.`, warning: true, error: false };
  }
  if (model.last) {
    const bad = model.last.fail > 0 || model.last.other > 0;
    return {
      state: 'result',
      text: `${bad ? '$(error)' : '$(pass)'} Simulation: ${countsText(model.last)}`,
      tooltip: `Last run: ${countsText(model.last)}. Click for the menu.`,
      warning: false,
      error: model.last.fail > 0,
    };
  }
  const label = model.toolLabel ?? 'OpenModelica';
  return { state: 'ready', text: `$(play-circle) Simulation: ${label}`, tooltip: `${model.list?.headline ?? ''} Click for the simulation menu.`.trim(), ...none };
}

// ---- Which files call for the item ---------------------------------------------

/** Whether the text of a model simulates (`simulate:` in a test case) or has a physics port. */
export function simulates(text: string): boolean {
  const code = text.replace(/\/\/[^\n]*/g, '');
  return /\bsimulate\s*:/.test(code) || /\bphysics\s*:/.test(code);
}

// ---- The verdicts of `arclang verify --json` -----------------------------------

export interface Outcome {
  test_case: string;
  /** The test case's name, the requirements it verifies, its configuration's name (4.8.1). */
  name: string;
  verifies: string[];
  configuration_name: string | null;
  model: string;
  verdict: string;
  reason: string;
  configuration?: string | null;
  values: Evaluated[];
  tool: string;
  omc_version: string;
  /** What OpenModelica printed, and how long the run took. */
  log: string;
  seconds: number;
  /** Every point of a parameter sweep (4.8.1), or every sample of a Monte Carlo case; empty when the case sweeps nothing. */
  points: Point[];
  /** What a Monte Carlo case had to show, what its samples show, and what moves each margin; null and empty otherwise. */
  probability: { probability: number; confidence: number } | null;
  estimate: { held: number; total: number; low: number; high: number } | null;
  sensitivity: { expect: string; parameter: string; rank_correlation: number }[];
}

export interface Evaluated {
  value: string;
  expect: string;
  holds: boolean;
  margin?: string;
  /** The margin in SI, positive on the side that holds; absent for an equality. */
  slack?: number;
  /** The unit the margin reads in: `slack / factor` is the margin in it. */
  slack_unit?: { symbol: string; factor: number };
}

export interface Point {
  label: string;
  values: Evaluated[];
  holds: boolean;
  error: string;
  /** The uncertain parameters at a sample, in SI. */
  inputs: [string, number][];
}

const finite = (v: unknown): v is number => typeof v === 'number' && Number.isFinite(v);

function evaluated(raw: unknown): Evaluated | null {
  if (!raw || typeof raw !== 'object') return null;
  const v = raw as Record<string, unknown>;
  if (typeof v.value !== 'string') return null;
  const unit = v.slack_unit as Record<string, unknown> | undefined;
  return {
    value: v.value,
    expect: typeof v.expect === 'string' ? v.expect : '',
    holds: v.holds === true,
    ...(typeof v.margin === 'string' ? { margin: v.margin } : {}),
    ...(finite(v.slack) ? { slack: v.slack } : {}),
    ...(unit && typeof unit.symbol === 'string' && finite(unit.factor) && unit.factor > 0 ? { slack_unit: { symbol: unit.symbol, factor: unit.factor } } : {}),
  };
}

const evaluatedList = (raw: unknown): Evaluated[] => (Array.isArray(raw) ? raw.map(evaluated).filter((v): v is Evaluated => v !== null) : []);

function probabilityOf(raw: unknown): Outcome['probability'] {
  const p = raw as Record<string, unknown> | null | undefined;
  return p && finite(p.probability) && finite(p.confidence) ? { probability: p.probability, confidence: p.confidence } : null;
}

function estimateOf(raw: unknown): Outcome['estimate'] {
  const e = raw as Record<string, unknown> | null | undefined;
  return e && finite(e.held) && finite(e.total) && finite(e.low) && finite(e.high) && e.total > 0 ? { held: e.held, total: e.total, low: e.low, high: e.high } : null;
}

export function parseOutcomes(text: string): Outcome[] | null {
  let value: unknown;
  try {
    value = JSON.parse(text);
  } catch {
    return null;
  }
  if (!Array.isArray(value)) return null;
  const out: Outcome[] = [];
  for (const raw of value) {
    if (typeof raw !== 'object' || raw === null) return null;
    const o = raw as Record<string, unknown>;
    if (typeof o.test_case !== 'string' || typeof o.verdict !== 'string') return null;
    const text = (key: string) => (typeof o[key] === 'string' ? (o[key] as string) : '');
    out.push({
      test_case: o.test_case,
      name: text('name'),
      verifies: Array.isArray(o.verifies) ? o.verifies.filter((v): v is string => typeof v === 'string') : [],
      configuration_name: typeof o.configuration_name === 'string' ? o.configuration_name : null,
      model: text('model'),
      verdict: o.verdict,
      reason: text('reason'),
      configuration: typeof o.configuration === 'string' ? o.configuration : null,
      values: evaluatedList(o.values),
      tool: text('tool'),
      omc_version: text('omc_version'),
      log: text('log'),
      seconds: typeof o.seconds === 'number' && Number.isFinite(o.seconds) ? o.seconds : 0,
      points: Array.isArray(o.points)
        ? (o.points as Record<string, unknown>[])
            .filter((p) => p && typeof p.label === 'string')
            .map((p) => ({
              label: p.label as string,
              values: evaluatedList(p.values),
              holds: p.holds === true,
              error: typeof p.error === 'string' ? p.error : '',
              inputs: Array.isArray(p.inputs)
                ? (p.inputs as unknown[]).filter((i): i is [string, number] => Array.isArray(i) && typeof i[0] === 'string' && finite(i[1])).map(([n, v]) => [n, v] as [string, number])
                : [],
            }))
        : [],
      probability: probabilityOf(o.probability),
      estimate: estimateOf(o.estimate),
      sensitivity: Array.isArray(o.sensitivity)
        ? (o.sensitivity as Record<string, unknown>[])
            .filter((i) => i && typeof i.expect === 'string' && typeof i.parameter === 'string' && finite(i.rank_correlation))
            .map((i) => ({ expect: i.expect as string, parameter: i.parameter as string, rank_correlation: i.rank_correlation as number }))
        : [],
    });
  }
  return out;
}

export function countOutcomes(outcomes: Outcome[]): Counts {
  const counts: Counts = { pass: 0, fail: 0, other: 0 };
  for (const o of outcomes) {
    if (o.verdict === 'pass') counts.pass += 1;
    else if (o.verdict === 'fail') counts.fail += 1;
    else counts.other += 1;
  }
  return counts;
}

/** Whether the reason of a run says that OpenModelica itself was not there. */
export function toolMissing(outcomes: Outcome[], stderr: string): boolean {
  return /OpenModelica not available/i.test(stderr) || outcomes.some((o) => /OpenModelica not available/i.test(o.reason));
}

/** `verifies REQ-RANGE · configuration City (CFG-CITY) · model EVPhysics.CityRange`. */
export function contextLine(o: Outcome): string {
  const parts: string[] = [];
  if (o.verifies.length) parts.push(`verifies ${o.verifies.join(', ')}`);
  if (o.configuration) parts.push(o.configuration_name && o.configuration_name !== o.configuration ? `configuration ${o.configuration_name} (${o.configuration})` : `configuration ${o.configuration}`);
  if (o.model) parts.push(`model ${o.model}`);
  return parts.join(' · ');
}

/** The lines OpenModelica printed, without the blank ones. */
export function toolLogLines(log: string): string[] {
  return log.split(/\r?\n/).filter((l) => l.trim() !== '');
}

/** One case of a run: its verdict, what it checks, each value and how far it is from its bound. */
export function outcomesLog(outcomes: Outcome[], options: { toolLog?: boolean } = {}): string {
  const lines: string[] = [];
  for (const o of outcomes) {
    const mark = o.verdict === 'pass' ? '✓ PASS' : o.verdict === 'fail' ? '✗ FAIL' : `? ${o.verdict.toUpperCase()}`;
    const name = o.name && o.name !== o.test_case ? ` "${o.name}"` : '';
    lines.push(`${mark}  ${o.test_case}${name}`);
    const context = contextLine(o);
    if (context) lines.push(`      ${context}`);
    const value = (indent: string, v: Evaluated) => `${indent}${v.holds ? 'ok  ' : 'FAIL'} ${v.value}   (${v.expect}${v.margin ? `, ${v.margin}` : ''})`;
    if (o.probability) lines.push(...monteCarloLines(o, value));
    else if (o.points.length === 0) for (const v of o.values) lines.push(value('      ', v));
    else {
      const passing = o.points.filter((p) => p.holds).length;
      lines.push(`      sweep: ${o.points.length} points, ${passing} pass, ${o.points.length - passing} do not`);
      for (const p of o.points) {
        lines.push(`      ${p.holds ? 'ok  ' : p.error ? 'ERR ' : 'FAIL'} ${p.label}`);
        for (const v of p.values) lines.push(value('          ', v));
        if (p.error) lines.push(`          ${p.error}`);
      }
    }
    if (o.reason && (o.verdict !== 'fail' || o.probability)) lines.push(`      ${o.reason}`);
    const log = toolLogLines(o.log);
    if (options.toolLog && log.length) {
      lines.push(`      OpenModelica log (${log.length} line${log.length > 1 ? 's' : ''}):`);
      for (const l of log) lines.push(`        | ${l}`);
    }
    lines.push('');
  }
  return lines.join('\n').trimEnd();
}

/** Failing samples a log lists before it says how many more there are. */
export const LOGGED_FAILURES = 10;
/** An influence below this (|Spearman's rho|) is not worth a line. */
export const NOTABLE_INFLUENCE = 0.1;

/** A percentage as the compiler writes it: `92.78 %`, `100 %`. */
export function percentText(p: number): string {
  return `${(Math.round(p * 10000) / 100).toString()} %`;
}

/** A Monte Carlo case in a log: how many samples held, the failing ones, what moves each margin; never every sample. */
function monteCarloLines(o: Outcome, value: (indent: string, v: Evaluated) => string): string[] {
  const lines: string[] = [];
  const held = o.points.filter((p) => p.holds).length;
  const required = o.probability!;
  lines.push(`      Monte Carlo: ${o.points.length} samples, ${held} hold; required: ${percentText(required.probability)} at ${percentText(required.confidence)} confidence`);
  const failing = o.points.filter((p) => !p.holds);
  for (const p of failing.slice(0, LOGGED_FAILURES)) {
    lines.push(`      ${p.error ? 'ERR ' : 'FAIL'} ${p.label}`);
    for (const v of p.values.filter((v) => !v.holds)) lines.push(value('          ', v));
    if (p.error) lines.push(`          ${p.error}`);
  }
  if (failing.length > LOGGED_FAILURES) lines.push(`      … and ${failing.length - LOGGED_FAILURES} more failing samples`);
  const notable = o.sensitivity.filter((i) => Math.abs(i.rank_correlation) >= NOTABLE_INFLUENCE);
  if (notable.length) {
    lines.push('      What moves the margins (rank correlation; positive: more of it, more to spare):');
    for (const i of notable) lines.push(`          ${i.rank_correlation >= 0 ? '+' : ''}${i.rank_correlation.toFixed(2)}  ${i.parameter}  on  ${i.expect}`);
  }
  return lines;
}

/** The whole log of one run: when, which file, which tool, the cases, the verdicts and the time. */
export function runLog(file: string, outcomes: Outcome[], at: Date): string {
  const time = `${String(at.getHours()).padStart(2, '0')}:${String(at.getMinutes()).padStart(2, '0')}:${String(at.getSeconds()).padStart(2, '0')}`;
  const lines = [`── ${time}  arclang verify ${file}`];
  const ran = outcomes.find((o) => o.tool);
  if (ran) lines.push(`   Tool: ${ran.tool}${ran.omc_version ? `, ${ran.omc_version}` : ''}`);
  lines.push('', outcomesLog(outcomes, { toolLog: true }), '');
  const seconds = outcomes.reduce((sum, o) => sum + o.seconds, 0);
  lines.push(`${summaryText(countOutcomes(outcomes))}${seconds > 0 ? ` Simulated in ${seconds.toFixed(1)} s.` : ''}`, '');
  return lines.join('\n');
}

// ---- The Modelica drawing ---------------------------------------------------------

/** The system models of a split export, one per configuration: what Modex draws whole. */
export function systemDrawings(files: string[]): { label: string; file: string }[] {
  return files
    .map((file) => ({ file, match: /[\\/]Systems[\\/]([A-Za-z_][A-Za-z0-9_]*)\.mo$/.exec(file) }))
    .filter((f): f is { file: string; match: RegExpExecArray } => f.match !== null && f.match[1] !== 'package')
    .map((f) => ({ label: f.match[1], file: f.file }))
    .sort((a, b) => a.label.localeCompare(b.label) || a.file.localeCompare(b.file));
}

/** The folder Modex must know as a library root for a split export: the one holding `<Package>/`. */
export function libraryRootOf(systemFile: string): string {
  // <root>/<Package>/Systems/<Configuration>.mo
  return systemFile.replace(/[\\/][^\\/]+[\\/]Systems[\\/][^\\/]+\.mo$/, '');
}

/** `modex.libraryRoots` with a root added, or null when it is there already. */
export function withLibraryRoot(current: unknown, root: string): string[] | null {
  const roots = Array.isArray(current) ? current.filter((r): r is string => typeof r === 'string') : [];
  return roots.includes(root) ? null : [...roots, root];
}

/** The sentence a run ends on. */
export function summaryText(counts: Counts): string {
  const total = counts.pass + counts.fail + counts.other;
  if (total === 0) return 'No test case in this model simulates.';
  if (counts.fail === 0 && counts.other === 0) return `All ${total} simulated case${total > 1 ? 's' : ''} pass.`;
  return `${countsText(counts)}.`;
}
