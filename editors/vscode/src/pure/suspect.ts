// Suspect traces, as the compiler reports them (rule `trace.suspect`):
// `suspect: trace 'A' satisfies 'B': 'B' changed since the trace was reviewed; ...`.

export interface SuspectTrace {
  from: string;
  type: string;
  to: string;
}

export const SUSPECT_RULE = 'trace.suspect';

/** The trace a suspect diagnostic is about. */
export function suspectOf(message: string): SuspectTrace | undefined {
  const found = /^suspect: trace '([^']+)' (\S+) '([^']+)': /.exec(message);
  return found ? { from: found[1], type: found[2], to: found[3] } : undefined;
}

/** `arclang trace <file> --review [--from A --to B] --print`: the reviewed source on stdout. */
export function reviewArgs(file: string, trace?: SuspectTrace): string[] {
  return ['trace', file, '--review', ...(trace ? ['--from', trace.from, '--to', trace.to] : []), '--print'];
}

export function quickFixTitle(trace: SuspectTrace): string {
  return `Mark the trace ${trace.from} ${trace.type} ${trace.to} as reviewed`;
}
