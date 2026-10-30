// The running state machine on the workbench page: the active states are
// outlined and the transition just fired is drawn bold, on the diagram of
// the machine, as the debugger (`arclang debug --dap`) reports them.
import { debugTargets, DebugStateMessage } from '../src/pure/debug';

let current: DebugStateMessage | null = null;

function paint(host: HTMLElement): void {
  for (const node of host.querySelectorAll('.wb-debug-active, .wb-debug-fired')) node.classList.remove('wb-debug-active', 'wb-debug-fired');
  if (!current) return;
  const { states, transitions } = debugTargets(current);
  const mark = (ids: string[], cls: string) => {
    for (const id of ids) host.querySelector(`.arcviz-stage [data-id="${CSS.escape(id)}"]`)?.classList.add(cls);
  };
  mark(states, 'wb-debug-active');
  mark(transitions, 'wb-debug-fired');
}

export function installDebugMarks(host: HTMLElement): void {
  window.addEventListener('message', (event: MessageEvent) => {
    const message = event.data as DebugStateMessage | undefined;
    if (message?.type !== 'debug-state') return;
    current = message.machine ? message : null;
    paint(host);
  });
  // The drawing is built again (another view, a new snapshot): paint again.
  let timer: ReturnType<typeof setTimeout> | undefined;
  new MutationObserver((records) => {
    if (!current || records.every((r) => [...r.addedNodes].every((n) => !(n instanceof Element) || !n.closest?.('.arcviz-stage')))) return;
    clearTimeout(timer);
    timer = setTimeout(() => paint(host), 120);
  }).observe(host, { childList: true, subtree: true });
}
