// The diff of an AI proposal, as the page shows it.
import { el } from './dom';

/** A unified diff, colored by line, its first lines shown. */
export function renderDiff(lines: string[], added: number, removed: number, max = 40): HTMLElement {
  const pre = el('pre', { class: 'wb-ai-diff', 'aria-label': `Source diff: ${added} lines added, ${removed} removed` });
  for (const line of lines.slice(0, max)) {
    const kind = line.startsWith('@@') ? 'hunk' : line[0] === '+' ? 'add' : line[0] === '-' ? 'del' : 'ctx';
    pre.appendChild(el('span', { class: `wb-ai-line is-${kind}`, text: line || ' ' }));
  }
  if (lines.length > max) pre.appendChild(el('span', { class: 'wb-ai-line is-more', text: `… ${lines.length - max} more lines: Open diff` }));
  return el('div', { class: 'wb-ai-diff-wrap' }, [el('div', { class: 'wb-ai-diff-head' }, [el('span', { text: 'Source' }), el('span', { class: 'wb-ai-plus', text: `+${added}` }), el('span', { class: 'wb-ai-minus', text: `−${removed}` })]), pre]);
}
