// One line of status under the canvas: errors, findings and the gate; the
// list of findings unfolds on demand (errors unfold by themselves).
import type { Finding, FromWebview, Snapshot } from '../src/pure/workbench';
import { button, clear, el } from './dom';

export interface StatusHost {
  post: (message: FromWebview) => void;
  /** Select the element a finding is about. */
  select: (id: string) => void;
  /** The line changed height: the canvas takes what is left. */
  resized: () => void;
}

export function createStatusLine(bottom: HTMLElement, owner: StatusHost): (snapshot: Snapshot) => void {
  let open = false;
  const command = (name: string) => () => owner.post({ type: 'command', command: name });
  const item = (finding: Finding): HTMLElement => {
    const line = el('li', { class: `is-${finding.severity}` }, [el('code', { text: finding.rule }), ' ', finding.message]);
    if (finding.element) {
      const id = finding.element;
      line.appendChild(button(id, () => {
        owner.select(id);
        owner.post({ type: 'reveal', id });
      }, { class: 'wb-link' }));
    }
    return line;
  };
  const show = (snapshot: Snapshot): void => {
    clear(bottom);
    const errors = snapshot.errors.length;
    const { findings, gate } = snapshot;
    const bySeverity = new Map<string, number>();
    for (const finding of findings) bySeverity.set(finding.severity, (bySeverity.get(finding.severity) ?? 0) + 1);
    const counts = [...bySeverity.entries()].map(([severity, n]) => `${n} ${severity}${n > 1 ? 's' : ''}`).join(', ');
    const unfolded = open || errors > 0;
    const toggle = button(unfolded ? '▾' : '▸', () => {
      open = !open;
      show(snapshot);
    }, { class: 'wb-toggle', title: unfolded ? 'Hide the findings' : 'Show the findings' });
    bottom.appendChild(el('div', { class: 'wb-problems-head' }, [
      toggle,
      el('strong', { class: errors ? 'is-error' : '', text: errors ? `${errors} error${errors > 1 ? 's' : ''}: the model does not compile` : findings.length ? counts : 'no findings' }),
      gate ? el('span', { class: `wb-gate ${gate.passed ? 'is-pass' : 'is-fail'}`, text: gate.passed ? 'gate PASS' : `gate FAIL (${gate.requirements_verified}/${gate.requirements_total} requirements verified)` }) : null,
      el('span', { class: 'wb-spacer' }),
      button('Check', command('arclang.check')),
      button('Gate', command('arclang.runGate')),
      button('Transition plan', command('arclang.transitionPlan')),
      button('Source', () => owner.post({ type: 'open-text' })),
    ]));
    bottom.classList.toggle('is-open', unfolded);
    owner.resized();
    if (!unfolded) return;
    const list = el('ul', { class: 'wb-problems' });
    for (const error of snapshot.errors) list.appendChild(el('li', { class: 'is-error', text: error }));
    for (const finding of findings.slice(0, 200)) list.appendChild(item(finding));
    if (findings.length > 200) list.appendChild(el('li', { class: 'wb-muted', text: `… ${findings.length - 200} more in the Problems view` }));
    bottom.appendChild(list);
  };
  return show;
}
