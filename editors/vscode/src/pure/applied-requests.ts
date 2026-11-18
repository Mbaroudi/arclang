// The requests of the AI chat that changed the model, for whoever keeps
// versions of the work (the playground's account): each with the start of
// the assistant's explanation. A change that is undone is forgotten; the
// list is bounded and handed over on request.

export interface AppliedRequest {
  /** The change it led to (the chat's session). */
  session: string;
  question: string;
  answer: string;
}

export const APPLIED_LIMITS = { requests: 20, question: 2000, answer: 1000 };

const cut = (text: string, limit: number): string => (text.length > limit ? `${text.slice(0, limit)}…` : text);

/** The list with one more request; a second request on the same change (a refinement) joins the first. */
export function withApplied(list: AppliedRequest[], session: string, question: string, answer: string): AppliedRequest[] {
  if (!question.trim()) return list;
  const known = list.find((r) => r.session === session);
  const entry: AppliedRequest = known
    ? { session, question: cut(known.question === question ? question : `${known.question}\n${question}`, APPLIED_LIMITS.question), answer: cut(answer, APPLIED_LIMITS.answer) }
    : { session, question: cut(question, APPLIED_LIMITS.question), answer: cut(answer, APPLIED_LIMITS.answer) };
  return [...list.filter((r) => r.session !== session), entry].slice(-APPLIED_LIMITS.requests);
}

/** The list without the request of a change that was undone. */
export function withoutApplied(list: AppliedRequest[], session: string): AppliedRequest[] {
  return list.filter((r) => r.session !== session);
}
