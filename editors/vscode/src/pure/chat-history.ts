// The conversation of the AI chat, kept across a reload of the window or of
// the browser: what was asked and what was answered, as text. A change the
// assistant proposed is kept as its explanation; its buttons are not, since
// the proposal itself is gone with the session.

export interface KeptTurn {
  who: 'you' | 'ai';
  /** Who answered (the provider and the file), or "You". */
  meta: string;
  text: string;
}

/** At most this many turns, each cut to this many characters. */
export const KEPT_LIMITS = { turns: 60, characters: 8000 };

const cut = (text: string): string => (text.length > KEPT_LIMITS.characters ? `${text.slice(0, KEPT_LIMITS.characters)}…` : text);

/** The turns with one more, the oldest dropped beyond the limit. */
export function keepTurn(kept: KeptTurn[], turn: KeptTurn): KeptTurn[] {
  if (!turn.text.trim()) return kept;
  return [...kept, { who: turn.who, meta: turn.meta.slice(0, 200), text: cut(turn.text) }].slice(-KEPT_LIMITS.turns);
}

/** The turns read back from a stored state: whatever is not a turn is left out. */
export function readKept(state: unknown): KeptTurn[] {
  const turns = (state as { turns?: unknown } | null | undefined)?.turns;
  if (!Array.isArray(turns)) return [];
  const out: KeptTurn[] = [];
  for (const turn of turns) {
    const t = turn as Partial<KeptTurn> | null;
    if (!t || (t.who !== 'you' && t.who !== 'ai') || typeof t.meta !== 'string' || typeof t.text !== 'string' || !t.text.trim()) continue;
    out.push({ who: t.who, meta: t.meta.slice(0, 200), text: cut(t.text) });
  }
  return out.slice(-KEPT_LIMITS.turns);
}
