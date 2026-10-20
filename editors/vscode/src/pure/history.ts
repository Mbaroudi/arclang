// One history for what the workbench changes, draw.io style: an edit of the
// model (the text) and a change of the arrangement (the layout file) are
// steps of the same list, undone and redone in the order they were made.
// No VS Code here: the panel applies the steps.

/** A layout file, or null when there is none. */
export type LayoutState = unknown;

export type HistoryEntry =
  | {
      kind: 'model';
      label: string;
      /** The whole text before and after the edit. */
      before: string;
      after: string;
      /** The arrangement that went with the edit (a pasted copy placed where it was pasted). */
      layout?: { before: LayoutState; after: LayoutState };
      at: number;
    }
  | { kind: 'layout'; label: string; before: LayoutState; after: LayoutState; at: number };

/** Layout changes closer than this are one step (a burst of arrow nudges). */
export const COALESCE_MS = 700;
/** A placement joins the edit it follows for this long (the remount in between). */
export const JOIN_MS = 15000;
const LIMIT = 300;
/** The memory the steps may take together (two bytes a character): past it the oldest go. */
export const MAX_BYTES = 50 * 1024 * 1024;

const layoutBytes = (state: LayoutState): number => (state === undefined || state === null ? 0 : 2 * JSON.stringify(state).length);

const same = (a: LayoutState, b: LayoutState): boolean => JSON.stringify(a ?? null) === JSON.stringify(b ?? null);

export class History {
  private undoable: HistoryEntry[] = [];
  private redoable: HistoryEntry[] = [];
  private readonly sizes = new WeakMap<HistoryEntry, number>();

  constructor(private readonly maxBytes = MAX_BYTES) {}

  /** The bytes a step holds, measured once. */
  private bytes(entry: HistoryEntry): number {
    const known = this.sizes.get(entry);
    if (known !== undefined) return known;
    const own = entry.kind === 'model' ? 2 * (entry.before.length + entry.after.length) : layoutBytes(entry.before) + layoutBytes(entry.after);
    const extra = entry.kind === 'model' && entry.layout ? layoutBytes(entry.layout.before) + layoutBytes(entry.layout.after) : 0;
    this.sizes.set(entry, own + extra);
    return own + extra;
  }

  /** The bytes all the steps hold. */
  get totalBytes(): number {
    return [...this.undoable, ...this.redoable].reduce((sum, entry) => sum + this.bytes(entry), 0);
  }

  /** Drop the oldest steps until the history fits its memory (the newest step is always kept). */
  private fit(): void {
    let total = this.totalBytes;
    let drop = 0;
    while (total > this.maxBytes && drop < this.undoable.length - 1) {
      total -= this.bytes(this.undoable[drop]);
      drop += 1;
    }
    if (drop > 0) this.undoable = this.undoable.slice(drop);
    if (total > this.maxBytes && this.redoable.length > 0) this.redoable = [];
  }

  get canUndo(): boolean {
    return this.undoable.length > 0;
  }

  get canRedo(): boolean {
    return this.redoable.length > 0;
  }

  /** The label of the step ⌘Z would undo, and of the one ⌘⇧Z would redo. */
  labels(): { undo: string | null; redo: string | null } {
    return { undo: this.undoable.at(-1)?.label ?? null, redo: this.redoable.at(-1)?.label ?? null };
  }

  private push(entry: HistoryEntry): void {
    this.undoable = [...this.undoable, entry].slice(-LIMIT);
    // A new step forgets what was undone: redo never brings back an older edit.
    this.redoable = [];
    this.fit();
  }

  recordModel(label: string, before: string, after: string, at: number): void {
    if (before === after) return;
    this.push({ kind: 'model', label, before, after, at });
  }

  /**
   * A change of the arrangement. `join` makes it part of the step before it
   * (a pasted copy placed, the other boxes of a multi-selection following
   * the dragged one); otherwise a change right after another layout change
   * extends it.
   */
  recordLayout(label: string, before: LayoutState, after: LayoutState, at: number, join = false): void {
    if (same(before, after)) return;
    const top = this.undoable.at(-1);
    if (top && join && at - top.at <= JOIN_MS) {
      const merged: HistoryEntry =
        top.kind === 'model'
          ? { ...top, layout: { before: top.layout ? top.layout.before : before, after } }
          : { ...top, after, at };
      this.undoable = [...this.undoable.slice(0, -1), merged];
      this.redoable = [];
      this.fit();
      return;
    }
    if (top && top.kind === 'layout' && at - top.at <= COALESCE_MS && same(top.after, before)) {
      const merged: HistoryEntry = { ...top, after, at };
      this.undoable = [...this.undoable.slice(0, -1), merged];
      this.redoable = [];
      if (same(merged.before, merged.after)) this.undoable = this.undoable.slice(0, -1);
      this.fit();
      return;
    }
    this.push({ kind: 'layout', label, before, after, at });
  }

  /** The step to undo, moved to the redo list (the caller applies it). */
  undo(): HistoryEntry | null {
    const entry = this.undoable.at(-1);
    if (!entry) return null;
    this.undoable = this.undoable.slice(0, -1);
    this.redoable = [...this.redoable, entry];
    return entry;
  }

  redo(): HistoryEntry | null {
    const entry = this.redoable.at(-1);
    if (!entry) return null;
    this.redoable = this.redoable.slice(0, -1);
    this.undoable = [...this.undoable, entry];
    return entry;
  }

  /** The text changed outside the workbench: its steps no longer apply. */
  clear(): void {
    this.undoable = [];
    this.redoable = [];
  }
}
