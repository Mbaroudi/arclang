// Suspect traces and verifications, as the compiler reports them (rule `trace.suspect`):
// `suspect: trace 'A' satisfies 'B': 'B' changed since the trace was reviewed; ...`,
// `suspect: verification 'TC' verifies 'REQ': 'REQ' changed since the verification was reviewed; ...`.

export interface SuspectTrace {
  /** A trace, or a test case's `verifies:`. */
  link: 'trace' | 'verification';
  from: string;
  type: string;
  to: string;
}

export const SUSPECT_RULE = 'trace.suspect';

/** The trace a suspect diagnostic is about. */
export function suspectOf(message: string): SuspectTrace | undefined {
  const found = /^suspect: (trace|verification) '([^']+)' (\S+) '([^']+)': /.exec(message);
  return found ? { link: found[1] as SuspectTrace['link'], from: found[2], type: found[3], to: found[4] } : undefined;
}

/** `arclang trace <file> --review [--from A --to B] --print`: the reviewed source on stdout. */
export function reviewArgs(file: string, trace?: SuspectTrace): string[] {
  return ['trace', file, '--review', ...(trace ? ['--from', trace.from, '--to', trace.to] : []), '--print'];
}

export function quickFixTitle(trace: SuspectTrace): string {
  return `Mark the ${trace.link} ${trace.from} ${trace.type} ${trace.to} as reviewed`;
}
