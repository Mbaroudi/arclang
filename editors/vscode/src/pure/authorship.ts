// Who wrote the last change of a model: the engineer, or the assistant of
// the chat. The review "after your edit" is for the engineer's own edits:
// a change the assistant wrote was already checked and explained in the
// chat, and reviewing it again, as if the engineer had typed it, misleads.

/** A change the assistant writes is recognised for this long after it says so. */
export const HOLD_MS = 20_000;

export class Authorship {
  private readonly marks = new Map<string, number>();

  /** The assistant is about to change this file. */
  assistantWrites(file: string, now = Date.now()): void {
    this.marks.set(file, now);
  }

  /** Whether the change just seen in this file is the assistant's; asked once per change. */
  takeAssistantChange(file: string, now = Date.now()): boolean {
    const at = this.marks.get(file);
    if (at === undefined) return false;
    this.marks.delete(file);
    return now - at <= HOLD_MS;
  }
}

/** The one record of the extension. */
export const authorship = new Authorship();
