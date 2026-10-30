// The model debugger, without VS Code: the message the workbench paints and
// the choices of *Fire an event*. Pure, tested with node:test.

/** What the workbench lights: the active states and the transitions just fired, by path. */
export interface DebugStateMessage {
  type: 'debug-state';
  machine: string;
  active: string[];
  fired: string[];
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

/** The drawing ids of what to light on a machine's diagram: states `<machine>.<path>`, transitions `tr:<from> -> <to>`. */
export function debugTargets(message: DebugStateMessage): { states: string[]; transitions: string[] } {
  if (!message.machine) return { states: [], transitions: [] };
  const last = (path: string) => path.slice(path.lastIndexOf('.') + 1);
  return {
    states: message.active.map((path) => `${message.machine}.${path}`),
    transitions: message.fired.map((label) => {
      const [from, to] = label.split(' -> ');
      return `tr:${last(from ?? '')} -> ${last(to ?? '')}`;
    }),
  };
}
