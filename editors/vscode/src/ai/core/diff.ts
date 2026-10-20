// A line diff of two texts, as a unified diff with a few lines of context:
// the preview of what an AI proposal changes in the source.

export interface LineDiff {
  lines: string[];
  added: number;
  removed: number;
}

/** The longest common subsequence of the middle of two line lists (trimmed of their common ends). */
function operations(a: string[], b: string[]): ('=' | '-' | '+')[] {
  const n = a.length;
  const m = b.length;
  // Too big for a table: the whole middle is replaced (still a correct diff).
  if (n * m > 4_000_000) return [...a.map(() => '-' as const), ...b.map(() => '+' as const)];
  const table = new Uint32Array((n + 1) * (m + 1));
  for (let i = n - 1; i >= 0; i--) {
    for (let j = m - 1; j >= 0; j--) {
      table[i * (m + 1) + j] = a[i] === b[j] ? table[(i + 1) * (m + 1) + j + 1] + 1 : Math.max(table[(i + 1) * (m + 1) + j], table[i * (m + 1) + j + 1]);
    }
  }
  const ops: ('=' | '-' | '+')[] = [];
  let i = 0;
  let j = 0;
  while (i < n && j < m) {
    if (a[i] === b[j]) {
      ops.push('=');
      i++;
      j++;
    } else if (table[(i + 1) * (m + 1) + j] >= table[i * (m + 1) + j + 1]) {
      ops.push('-');
      i++;
    } else {
      ops.push('+');
      j++;
    }
  }
  while (i++ < n) ops.push('-');
  while (j++ < m) ops.push('+');
  return ops;
}

export function lineDiff(before: string, after: string, context = 2): LineDiff {
  const a = before.split('\n');
  const b = after.split('\n');
  let head = 0;
  while (head < a.length && head < b.length && a[head] === b[head]) head++;
  let tail = 0;
  while (tail < a.length - head && tail < b.length - head && a[a.length - 1 - tail] === b[b.length - 1 - tail]) tail++;
  const middle = operations(a.slice(head, a.length - tail), b.slice(head, b.length - tail));
  const ops: ('=' | '-' | '+')[] = [...Array(head).fill('='), ...middle, ...Array(tail).fill('=')];
  // Walk the operations, keeping changed lines and `context` lines around them.
  const rows: { op: '=' | '-' | '+'; text: string; ai: number; bi: number }[] = [];
  let ai = 0;
  let bi = 0;
  for (const op of ops) {
    if (op === '=') rows.push({ op, text: a[ai], ai: ai++, bi: bi++ });
    else if (op === '-') rows.push({ op, text: a[ai], ai: ai++, bi });
    else rows.push({ op, text: b[bi], ai, bi: bi++ });
  }
  const keep = rows.map(() => false);
  rows.forEach((row, index) => {
    if (row.op === '=') return;
    for (let k = Math.max(0, index - context); k <= Math.min(rows.length - 1, index + context); k++) keep[k] = true;
  });
  const lines: string[] = [];
  let added = 0;
  let removed = 0;
  for (let index = 0; index < rows.length; index++) {
    if (!keep[index]) continue;
    if (index === 0 || !keep[index - 1]) {
      let end = index;
      while (end + 1 < rows.length && keep[end + 1]) end++;
      const hunk = rows.slice(index, end + 1);
      const aCount = hunk.filter((r) => r.op !== '+').length;
      const bCount = hunk.filter((r) => r.op !== '-').length;
      lines.push(`@@ -${rows[index].ai + 1},${aCount} +${rows[index].bi + 1},${bCount} @@`);
    }
    const row = rows[index];
    if (row.op === '+') added++;
    if (row.op === '-') removed++;
    lines.push(`${row.op === '=' ? ' ' : row.op}${row.text}`);
  }
  return { lines, added, removed };
}
