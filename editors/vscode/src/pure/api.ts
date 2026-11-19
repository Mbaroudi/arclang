// The shared part of the extension's public API: reading what `arclang workbench` printed.
// (No `vscode` here: the node tests load this file.)

/** What a compiler answer is: the model's workbench document, or why there is none. */
export type WorkbenchAnswer =
  | { ok: true; workbench: Record<string, unknown> }
  | { ok: false; errors: string[] };

/**
 * The answer of `arclang workbench`: a JSON document, or `{"success": false, "errors": [...]}` for a model that
 * does not compile. Anything else (a crash, a missing compiler) is an error with its text, never a throw.
 */
export function parseWorkbenchAnswer(code: number, stdout: string, stderr: string): WorkbenchAnswer {
  let parsed: unknown;
  try {
    parsed = JSON.parse(stdout);
  } catch {
    return { ok: false, errors: [stderr.trim() || `the compiler answered nothing readable (exit ${code})`] };
  }
  if (!parsed || typeof parsed !== 'object' || Array.isArray(parsed)) return { ok: false, errors: ['the compiler answered with something other than a document'] };
  const document = parsed as Record<string, unknown>;
  if (document.success === false) {
    const errors = Array.isArray(document.errors) ? document.errors.map((e) => (typeof e === 'string' ? e : JSON.stringify(e))) : [];
    return { ok: false, errors: errors.length ? errors : ['the model does not compile'] };
  }
  return { ok: true, workbench: document };
}

/** Debounces a burst of changes of one file into one call. Returns what the caller cancels with. */
export function debounced<T>(wait: number, call: (value: T) => void, timers: { set: typeof setTimeout; clear: typeof clearTimeout } = { set: setTimeout, clear: clearTimeout }): { push(key: string, value: T): void; dispose(): void } {
  const waiting = new Map<string, ReturnType<typeof setTimeout>>();
  return {
    push(key, value) {
      const earlier = waiting.get(key);
      if (earlier !== undefined) timers.clear(earlier);
      waiting.set(key, timers.set(() => { waiting.delete(key); call(value); }, wait));
    },
    dispose() {
      for (const timer of waiting.values()) timers.clear(timer);
      waiting.clear();
    },
  };
}
