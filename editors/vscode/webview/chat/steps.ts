// Numbered steps the AI wrote in one line, found as a list (no DOM: the unit tests run it).

/** "1. a 2. b 3. c" written in one line: the items, or null when it is not a list. */
export function inlineSteps(paragraph: string): { lead: string; items: string[] } | null {
  const marks = [...paragraph.matchAll(/(?:^|\s)(\d{1,2})\.\s+(?=\S)/g)];
  if (marks.length < 2) return null;
  // The numbers must count up from 1 (or from the first one) without gaps.
  const numbers = marks.map((m) => Number(m[1]));
  if (numbers.some((n, i) => n !== numbers[0] + i) || numbers[0] > 1) return null;
  const starts = marks.map((m) => m.index! + m[0].indexOf(m[1]));
  const lead = paragraph.slice(0, starts[0]).trim();
  const items = starts.map((s, i) => paragraph.slice(s, starts[i + 1] ?? paragraph.length).replace(/^\d{1,2}\.\s+/, '').trim());
  return { lead, items };
}
