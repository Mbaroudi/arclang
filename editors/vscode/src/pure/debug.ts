// The model debugger, without VS Code: the message the workbench paints,
// the diagram it shows, the choices of *Run* and *Fire an event*. Pure,
// tested with node:test.

/** What the workbench lights, as the debugger reports it (`arclang.state`). */
export interface DebugStateMessage {
  type: 'debug-state';
  /** `machine`, `scenario`, `chain`; empty when nothing runs. */
  kind: string;
  name: string;
  /** Drawing ids (or references) of the nodes and edges to light. */
  nodes: string[];
  edges: string[];
}

export interface FirePick {
  label: string;
  description: string;
  /** The event, null for a completion step, '?' to type one. */
  event: string | null;
}

/** The events a transition from the active states waits for, then a completion step, then any. */
export function firePicks(events: string[]): FirePick[] {
  return [
    ...events.map((event) => ({ label: event, description: 'a transition from the active states waits for it', event })),
    { label: '(completion step)', description: 'the transitions without trigger', event: null },
    { label: 'Another event…', description: 'type it', event: '?' },
  ];
}

/** The diagram a run is drawn on: a machine's `msm:`, a scenario's `es:`; a chain stays where it is. */
export function debugView(message: DebugStateMessage): string | null {
  if (message.kind === 'machine') return `msm:${message.name}`;
  if (message.kind === 'scenario') return `es:${message.name}`;
  return null;
}

/** What `arclang debug --list` says a model runs. */
export interface Runnables {
  machines: string[];
  scenarios: string[];
  chains: string[];
}

export interface RunPick {
  label: string;
  description: string;
  /** The launch attribute and its value: `machine`, `scenario` or `chain`. */
  key: 'machine' | 'scenario' | 'chain';
  name: string;
}

/** One pick per state machine, scenario and functional chain of the model. */
/** What to say when a model has nothing to run step by step, and what to offer instead. */
export function nothingToRun(simulates: boolean): { text: string; action: string | null } {
  const base = 'ArcLang: this model has no state machine, scenario or functional chain, so there is nothing to run step by step.';
  return simulates
    ? { text: `${base} It is verified by simulation: Run simulations runs its test cases in OpenModelica.`, action: 'Run simulations' }
    : { text: base, action: null };
}

export function runPicks(runnables: Runnables): RunPick[] {
  return [
    ...runnables.machines.map((name) => ({ label: `$(symbol-enum) ${name}`, description: 'state machine: events and guards', key: 'machine' as const, name })),
    ...runnables.scenarios.map((name) => ({ label: `$(list-ordered) ${name}`, description: 'scenario: message by message', key: 'scenario' as const, name })),
    ...runnables.chains.map((name) => ({ label: `$(type-hierarchy) ${name}`, description: 'functional chain: step by step', key: 'chain' as const, name })),
  ];
}
