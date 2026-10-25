// The what-if panel: what the proposed change does to the model, computed
// by compiling a copy. The gate before and after, the blockers it adds or
// clears, the constraints that flip, the latency budgets (bars against the
// bound), the requirement coverage, the verdicts left stale, the elements
// added, removed or changed. Then: apply (one undoable edit) or discard.
import { WhatIfResult } from '../../src/pure/impact';
import { button, el } from '../dom';

export interface WhatIfHost {
  apply: () => void;
  discard: () => void;
  reveal: (id: string) => void;
}

const verdict = (passed: boolean) => (passed ? 'PASS' : 'FAIL');
const msText = (v: number | null) => (v === null ? '—' : `${Number.isInteger(v) ? v : v.toFixed(1)} ms`);

function card(title: string, tone: 'ok' | 'bad' | 'warn' | 'plain', children: (Node | string | null)[]): HTMLElement {
  return el('section', { class: `ip-card tone-${tone}` }, [el('h3', { class: 'ip-card-title', text: title }), ...children]);
}

function gateCard(w: WhatIfResult): HTMLElement {
  const worse = w.gate_before.passed && !w.gate_after.passed;
  const better = !w.gate_before.passed && w.gate_after.passed;
  const arrow = el('div', { class: 'ip-gate', role: 'status' }, [
    el('span', { class: `ip-gate-state ${w.gate_before.passed ? 'is-pass' : 'is-fail'}`, text: verdict(w.gate_before.passed) }),
    el('span', { class: 'ip-gate-arrow', 'aria-hidden': 'true', text: '→' }),
    el('span', { class: `ip-gate-state ${w.gate_after.passed ? 'is-pass' : 'is-fail'}`, text: verdict(w.gate_after.passed) }),
    el('span', { class: 'ip-gate-std', text: w.gate_after.standard }),
  ]);
  const list = el('ul', { class: 'ip-blockers' });
  for (const b of w.new_blockers) list.appendChild(el('li', { class: 'is-new' }, [el('b', { text: 'new ' }), b]));
  for (const b of w.removed_blockers) list.appendChild(el('li', { class: 'is-cleared' }, [el('b', { text: 'cleared ' }), b]));
  const nothing = !w.new_blockers.length && !w.removed_blockers.length;
  return card('Production gate', worse ? 'bad' : better ? 'ok' : 'plain', [arrow, nothing ? el('p', { class: 'ip-muted', text: 'No blocker added or cleared.' }) : list]);
}

function coverageCard(w: WhatIfResult, host: WhatIfHost): HTMLElement {
  const b = w.coverage_before;
  const a = w.coverage_after;
  const stat = (label: string, from: number, to: number, total: number) =>
    el('div', { class: 'ip-stat' }, [el('div', { class: 'ip-stat-label', text: label }), el('div', { class: `ip-stat-value ${to < from ? 'is-worse' : to > from ? 'is-better' : ''}` }, [`${from}`, el('span', { class: 'ip-gate-arrow', text: ' → ' }), `${to}`, el('span', { class: 'ip-muted', text: ` / ${total}` })])]);
  const rows = el('ul', { class: 'ip-deltas' });
  for (const c of w.coverage) {
    const link = el('button', { type: 'button', class: 'ip-seg', text: c.requirement, title: `Show ${c.name || c.requirement}` });
    link.addEventListener('click', () => host.reveal(c.requirement));
    rows.appendChild(el('li', {}, [link, ` ${c.before.status} → `, el('b', { class: c.after.status === 'covered' ? 'is-better' : 'is-worse', text: c.after.status })]));
  }
  const worse = a.satisfied < b.satisfied || w.coverage.some((c) => c.after.status === 'failing');
  return card('Requirement coverage', worse ? 'bad' : 'plain', [
    el('div', { class: 'ip-stats' }, [stat('Satisfied', b.satisfied, a.satisfied, a.requirements), stat('Verified', b.verified, a.verified, a.requirements)]),
    w.coverage.length ? rows : el('p', { class: 'ip-muted', text: 'No requirement changes status.' }),
  ]);
}

function budgetBar(b: WhatIfResult['budgets'][number]): HTMLElement {
  const limit = b.budget_ms ?? Math.max(b.before_ms ?? 0, b.after_ms ?? 0, 1);
  const scale = Math.max(limit, b.before_ms ?? 0, b.after_ms ?? 0) * 1.08;
  const bar = (label: string, value: number | null, ok: boolean) => {
    // CSSOM, not a style attribute: the page's CSP allows no inline style.
    const fill = value === null ? null : el('span', { class: `ip-bar ${ok ? 'is-ok' : 'is-over'}` });
    if (fill) fill.style.width = `${Math.min(100, (value! / scale) * 100)}%`;
    const mark = el('span', { class: 'ip-bar-limit', title: `budget ${msText(b.budget_ms)}` });
    mark.style.left = `${(limit / scale) * 100}%`;
    return el('div', { class: 'ip-bar-row' }, [el('span', { class: 'ip-bar-label', text: label }), el('span', { class: 'ip-bar-track' }, [fill, mark]), el('span', { class: `ip-bar-value ${ok ? '' : 'is-worse'}`, text: msText(value) })]);
  };
  return el('div', { class: 'ip-budget' }, [
    el('div', { class: 'ip-budget-name' }, [b.name || b.chain, el('span', { class: 'ip-muted', text: ` · budget ${msText(b.budget_ms)}` })]),
    bar('before', b.before_ms, b.ok_before),
    bar('after', b.after_ms, b.ok_after),
    b.missing.length ? el('div', { class: 'ip-muted', text: `no latency: ${b.missing.join(', ')}` }) : null,
  ]);
}

function timingCard(w: WhatIfResult): HTMLElement | null {
  const budgets = w.budgets.filter((b) => b.before_ms !== b.after_ms || !b.ok_after);
  const flips = w.constraints.filter((c) => c.flipped);
  if (!budgets.length && !flips.length) return null;
  const tone = budgets.some((b) => b.ok_before && !b.ok_after) || flips.some((c) => c.after === false) ? 'bad' : 'plain';
  const list = el('ul', { class: 'ip-deltas' });
  for (const c of flips) {
    list.appendChild(el('li', {}, [el('b', { text: c.name || c.id }), ` ${c.before === false ? 'violated' : 'holds'} → `, el('b', { class: c.after ? 'is-better' : 'is-worse', text: c.after === false ? 'violated' : 'holds' }), el('span', { class: 'ip-muted', text: `  ${c.left_before} → ${c.left_after} against ${c.right}` })]));
  }
  return card('Budgets and constraints', tone, [...budgets.map(budgetBar), flips.length ? list : null]);
}

function verdictCard(w: WhatIfResult, host: WhatIfHost): HTMLElement | null {
  if (!w.verdicts.length) return null;
  const list = el('ul', { class: 'ip-deltas' });
  for (const v of w.verdicts) {
    const link = el('button', { type: 'button', class: 'ip-seg', text: v.test, title: v.why });
    link.addEventListener('click', () => host.reveal(v.test));
    list.appendChild(el('li', {}, [link, ` ${v.before} → ${v.after} `, v.stale ? el('b', { class: 'is-worse', text: 'stale: run again' }) : null]));
  }
  return card('Verdicts', w.verdicts.some((v) => v.stale || v.after === 'fail') ? 'warn' : 'plain', [list]);
}

function changesCard(w: WhatIfResult): HTMLElement {
  const d = w.diff;
  const list = el('ul', { class: 'ip-deltas' });
  for (const e of d.added) list.appendChild(el('li', {}, [el('b', { class: 'is-better', text: 'added ' }), `${e.id} (${e.element_type})`]));
  for (const e of d.removed) list.appendChild(el('li', {}, [el('b', { class: 'is-worse', text: 'removed ' }), `${e.id} (${e.element_type})`]));
  for (const e of d.modified) list.appendChild(el('li', {}, [el('b', { text: 'changed ' }), `${e.id}: `, el('span', { class: 'ip-muted', text: e.changes.map((c) => `${c.field} ${c.old} → ${c.new}`).join('; ') })]));
  const none = !d.added.length && !d.removed.length && !d.modified.length;
  return card('Elements', 'plain', [none ? el('p', { class: 'ip-muted', text: 'No element added, removed or changed (only links).' }) : list]);
}

/** The panel of a what-if; `applied` once the change was written. */
export function renderWhatIf(w: WhatIfResult, applied: boolean, host: WhatIfHost): HTMLElement {
  const actions = el('div', { class: 'ip-whatif-actions' });
  if (applied) {
    actions.appendChild(el('span', { class: 'ip-applied', text: 'Applied to the model (⌘Z in the text undoes it)' }));
  } else {
    const apply = button('Apply the change', host.apply, { class: 'ip-primary', title: 'Write it to the model: one undo step' });
    actions.append(apply, button('Discard', host.discard, { class: 'ip-secondary', title: 'Leave the model as it is' }));
  }
  const cards = [gateCard(w), coverageCard(w, host), timingCard(w), verdictCard(w, host), changesCard(w)].filter((c): c is HTMLElement => !!c);
  return el('section', { class: 'ip-whatif', 'aria-label': 'What-if' }, [
    el('div', { class: 'ip-whatif-head' }, [el('h2', { class: 'ip-whatif-title', text: 'If this change were made' }), el('span', { class: 'ip-muted', text: 'computed on a copy: nothing is written yet' }), actions]),
    el('div', { class: 'ip-cards' }, cards),
  ]);
}
