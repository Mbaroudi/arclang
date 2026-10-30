// The run on the workbench page: the active states, the message just sent
// or the chain steps holding a token are outlined, the transition fired or
// the message sent drawn bold, on the diagram of what runs, as the debugger
// (`arclang debug --dap`) reports them.
import { debugView, DebugStateMessage } from '../src/pure/debug';

let current: DebugStateMessage | null = null;

function find(host: HTMLElement, id: string): Element | null {
  const stage = host.querySelector('.arcviz-stage');
  if (!stage) return null;
  return stage.querySelector(`[data-id="${CSS.escape(id)}"]`) ?? stage.querySelector(`.av-edge[data-id$=":${CSS.escape(id)}"]`);
}

function paint(host: HTMLElement): void {
  for (const node of host.querySelectorAll('.wb-debug-active, .wb-debug-fired')) node.classList.remove('wb-debug-active', 'wb-debug-fired');
  if (!current) return;
  for (const id of current.nodes) find(host, id)?.classList.add('wb-debug-active');
  for (const id of current.edges) find(host, id)?.classList.add('wb-debug-fired');
}

/** `show` brings the diagram of what runs (a machine, a scenario) on screen. */
export function installDebugMarks(host: HTMLElement, show: (view: string) => void, shown: () => string | null): void {
  window.addEventListener('message', (event: MessageEvent) => {
    const message = event.data as DebugStateMessage | undefined;
    if (message?.type !== 'debug-state') return;
    const starting = !current && !!message.kind;
    current = message.kind ? message : null;
    const view = current ? debugView(current) : null;
    // At the start only: the reader may look at another view while it runs.
    if (view && starting && shown() !== view) show(view);
    paint(host);
  });
  // The drawing is built again (another view, a new snapshot): paint again.
  let timer: ReturnType<typeof setTimeout> | undefined;
  new MutationObserver(() => {
    if (!current) return;
    clearTimeout(timer);
    timer = setTimeout(() => paint(host), 120);
  }).observe(host, { childList: true, subtree: true });
}
