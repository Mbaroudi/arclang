// What keeps an edit computed on one text from landing on another: the
// version a text was read at, checked again when the edit is applied, and
// one computation per version of a document.

/** Why an edit computed on an older text is not applied, or null when the text is the one it was computed on. */
export function staleEdit(readAt: number, now: number, what: string): string | null {
  if (readAt === now) return null;
  return `${what} changed while the edit was being computed: applying it would undo those changes, so nothing was written. Ask again.`;
}

/** The path a file is written to: the one the extension chose, never one a page sends back. */
export function chosenTarget(chosen: string | undefined, _fromPage?: unknown): string | undefined {
  return chosen;
}

interface Running<T> {
  version: number;
  work: Promise<T>;
}

/**
 * One computation per key and version: a second ask for the same version
 * shares the first, an ask for a newer version starts its own, and the
 * result of an older version never replaces a newer one.
 */
export class PerVersion<T> {
  private readonly cache = new Map<string, { version: number; value: T }>();
  private readonly pending = new Map<string, Running<T>>();

  /** The value kept for exactly this version, if any. */
  cached(key: string, version: number): T | undefined {
    const kept = this.cache.get(key);
    return kept && kept.version === version ? kept.value : undefined;
  }

  /** The last value kept for the key, whatever its version. */
  last(key: string): T | undefined {
    return this.cache.get(key)?.value;
  }

  /** Whether a computation for the key is under way. */
  busy(key: string): boolean {
    return this.pending.has(key);
  }

  /** Forgets what is kept for the key; true when something was. */
  forget(key: string): boolean {
    return this.cache.delete(key);
  }

  /** The keys something is kept for. */
  keys(): string[] {
    return [...this.cache.keys()];
  }

  clear(): void {
    this.cache.clear();
  }

  /**
   * The computation for this version: the one under way, or a new one.
   * `keep` says, when it ends, whether its value may be kept (false: it is given, not kept).
   * `started` is true when this call began the computation.
   */
  get(key: string, version: number, compute: () => Promise<T>, keep: () => boolean = () => true): { work: Promise<T>; started: boolean } {
    const running = this.pending.get(key);
    if (running && running.version === version) return { work: running.work, started: false };
    const work = (async () => {
      const value = await compute();
      const kept = this.cache.get(key);
      if (keep() && !(kept && kept.version > version)) this.cache.set(key, { version, value });
      return value;
    })();
    const entry: Running<T> = { version, work };
    this.pending.set(key, entry);
    const done = () => {
      if (this.pending.get(key) === entry) this.pending.delete(key);
    };
    work.then(done, done);
    return { work, started: true };
  }
}
