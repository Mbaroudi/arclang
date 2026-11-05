// The realization ladder as the compiler reports it (the `realization`
// field of the workbench snapshot, `arclang check --json`): per
// requirement, what each stage says — the simulation, the C (SIL), the C on
// the processor (PIL), the logic (HDL), the SoC, the board, the twin — and
// what to run next. Pure: no VS Code, no DOM; the panel and the page use it.

export const STAGES = ['simulation', 'sil', 'pil', 'hdl', 'soc', 'board', 'twin'] as const;
export type StageName = (typeof STAGES)[number];
export type StageState = 'pass' | 'fail' | 'error' | 'stale' | 'missing' | 'n/a';

export interface StageCase {
  id: string;
  state: string;
  reason?: string;
  evidence?: string;
}

export interface Stage {
  stage: string;
  state: StageState;
  cases?: StageCase[];
  target?: string;
}

export interface Ladder {
  requirement: string;
  name: string;
  stages: Stage[];
  next?: string;
}

const LABELS: Record<string, string> = { simulation: 'simulation', sil: 'SIL', pil: 'PIL', hdl: 'HDL', soc: 'SoC', board: 'board', twin: 'twin' };
const LONG: Record<string, string> = {
  simulation: 'the architecture simulated in OpenModelica (arclang verify)',
  sil: 'the generated C run back to back with the model (arclang sil)',
  pil: 'the C on the target’s processor (arclang pil)',
  hdl: 'the machines allocated to logic, in VHDL (arclang hdl)',
  soc: 'the processor and the logic together, as one SoC in simulation (arclang soc)',
  board: 'the bitstream on the board (arclang soc --port)',
  twin: 'the twin on measurements from the field (arclang twin)',
};

export function stageLabel(stage: string): string {
  return LABELS[stage] ?? stage;
}

export function stageDescription(stage: string): string {
  return LONG[stage] ?? stage;
}

export function stageIndex(stage: string): number {
  const index = (STAGES as readonly string[]).indexOf(stage);
  return index < 0 ? STAGES.length : index;
}

/** The mark `arclang check` prints for a state. */
export function markOf(state: string): string {
  switch (state) {
    case 'pass':
      return '✓';
    case 'fail':
    case 'error':
      return '✗';
    case 'stale':
      return '~';
    case 'missing':
      return '○';
    default:
      return '–';
  }
}

/** What a state means, in a sentence. */
export function stateText(state: string): string {
  switch (state) {
    case 'pass':
      return 'passes, and the verdict still holds for the model';
    case 'fail':
      return 'fails';
    case 'error':
      return 'is in error';
    case 'stale':
      return 'ran, but the model, its sources, the case or the target changed since';
    case 'missing':
      return 'never ran';
    default:
      return 'is not a stage of this requirement';
  }
}

/** Whether any stage applies to the requirement. */
export function applies(ladder: Ladder): boolean {
  return ladder.stages.some((s) => s.state !== 'n/a');
}

/** The highest stage with fresh passing evidence, if any. */
export function reached(ladder: Ladder): string | null {
  const passing = ladder.stages.filter((s) => s.state === 'pass');
  return passing.length ? passing[passing.length - 1].stage : null;
}

/** Whether every stage of the requirement passes. */
export function complete(ladder: Ladder): boolean {
  const applicable = ladder.stages.filter((s) => s.state !== 'n/a');
  return applicable.length > 0 && applicable.every((s) => s.state === 'pass');
}

export interface Summary {
  requirements: number;
  /** Requirements with at least one stage. */
  climbing: number;
  /** Requirements whose every stage passes. */
  complete: number;
  failing: number;
  stale: number;
  missing: number;
  /** Per stage: how many requirements are in each state. */
  byStage: Record<string, Record<string, number>>;
}

export function summarize(ladders: Ladder[]): Summary {
  const byStage: Record<string, Record<string, number>> = {};
  for (const stage of STAGES) byStage[stage] = {};
  let climbing = 0;
  let done = 0;
  let failing = 0;
  let stale = 0;
  let missing = 0;
  for (const ladder of ladders) {
    if (!applies(ladder)) continue;
    climbing += 1;
    if (complete(ladder)) done += 1;
    const states = ladder.stages.map((s) => s.state);
    if (states.includes('fail') || states.includes('error')) failing += 1;
    else if (states.includes('stale')) stale += 1;
    else if (states.includes('missing')) missing += 1;
    for (const s of ladder.stages) {
      const counts = (byStage[s.stage] ??= {});
      counts[s.state] = (counts[s.state] ?? 0) + 1;
    }
  }
  return { requirements: ladders.length, climbing, complete: done, failing, stale, missing, byStage };
}

/** The commands the ladders name as next, each once, with the requirements waiting on it, in the order of the stages. */
export function nextCommands(ladders: Ladder[]): { command: string; stage: string; requirements: string[] }[] {
  const out = new Map<string, { command: string; stage: string; requirements: string[] }>();
  for (const ladder of ladders) {
    const command = ladder.next?.trim();
    if (!command || !command.startsWith('arclang ')) continue;
    const stage = ladder.stages.find((s) => !['pass', 'n/a'].includes(s.state))?.stage ?? 'twin';
    const entry = out.get(command) ?? { command, stage, requirements: [] };
    entry.requirements.push(ladder.requirement);
    out.set(command, entry);
  }
  return [...out.values()].sort((a, b) => stageIndex(a.stage) - stageIndex(b.stage) || a.command.localeCompare(b.command));
}

/** The command with the configured compiler binary in place of the bare `arclang`. */
export function withBinary(command: string, binary: string): string {
  const trimmed = binary.trim();
  if (!trimmed || trimmed === 'arclang' || !command.startsWith('arclang ')) return command;
  const quoted = /[\s"']/.test(trimmed) ? `"${trimmed.replace(/"/g, '\\"')}"` : trimmed;
  return `${quoted}${command.slice('arclang'.length)}`;
}

/** The command with the model file it names replaced by `path`: the compiler
 * computed the ladder on a copy (the editor's buffer in a temporary file), the
 * command must run on the real one. */
export function withModel(command: string, path: string): string {
  if (!command.startsWith('arclang ')) return command;
  const quoted = /[\s"']/.test(path) ? `"${path.replace(/"/g, '\\"')}"` : path;
  return command
    .split(' ')
    .map((token) => (/\.(arc|sysml)$/i.test(token) || token === '<model>' ? quoted : token))
    .join(' ');
}

/** The ladder with every next command on the real model file. */
export function onModel(ladders: Ladder[], path: string): Ladder[] {
  return ladders.map((l) => (l.next ? { ...l, next: withModel(l.next, path) } : l));
}

/** The tooltip of a stage: each case with its state and reason. */
export function stageTitle(stage: Stage): string {
  const head = `${stageLabel(stage.stage)}: ${stateText(stage.state)}${stage.target ? ` · ${stage.target}` : ''}`;
  const cases = (stage.cases ?? []).map((c) => `${c.id}: ${c.state}${c.reason ? ` — ${c.reason}` : ''}${c.evidence ? ` (${c.evidence})` : ''}`);
  return [head, ...cases].join('\n');
}

/** The ladder on one line, as `arclang check` prints it. */
export function lineOf(ladder: Ladder): string {
  const stages = ladder.stages.map((s) => `${stageLabel(s.stage)} ${markOf(s.state)}`).join(' · ');
  const name = ladder.name && ladder.name !== ladder.requirement ? ` "${ladder.name}"` : '';
  return `${ladder.requirement}${name}  ${stages}`;
}

export type ToRealizationView =
  | { type: 'ladders'; ladders: Ladder[]; file: string; binary: string; computedMs: number }
  | { type: 'busy' }
  | { type: 'error'; error: string; file: string }
  | { type: 'result'; ok: boolean; message: string };

export type FromRealizationView = { type: 'ready' } | { type: 'refresh' } | { type: 'run'; command: string } | { type: 'realize'; requirement: string | null } | { type: 'reveal'; id: string } | { type: 'table' };
