// What a diagram shows (Capella's Insert/Remove, SysON's Manage visibility):
// hide elements from it, add what an element is connected to, a new
// diagram around an element. A viewpoint view is drawn from the whole
// model, so hiding there makes a named diagram without the element.
import { EditRequest, ElementRecord, namedDiagramOf, neighbours, showsLiteral, Snapshot, topLevelIds, ViewerGraph } from '../src/pure/workbench';

export interface ContentsHost {
  graph: () => ViewerGraph | null;
  snapshot: () => Snapshot | null;
  /** The model id of a box drawn at the top of the diagram. */
  idOf: (drawingId: string) => string | null;
  submit: (requests: EditRequest[], label: string) => void;
  toast: (text: string, kind: 'info' | 'error') => void;
  /** Show this diagram once the snapshot that creates it arrives. */
  showLater: (view: string) => void;
}

export class DiagramContents {
  constructor(private readonly host: ContentsHost) {}

  private drawnAtTop(graph: ViewerGraph): string[] {
    return topLevelIds(graph).map((id) => this.host.idOf(id)).filter((id): id is string => !!id);
  }

  /** A diagram that shows `rest`: the named one on screen changes, a viewpoint view gets a named copy. */
  private showOnly(graph: ViewerGraph, rest: string[], what: string): void {
    const named = namedDiagramOf(graph.id, graph.arc.view);
    if (named) return this.host.submit([{ op: 'set', element: named, key: 'shows', value: showsLiteral(rest) }], `hide ${what} from ${named}`);
    const name = this.uniqueName(`${graph.arc.title} (custom)`);
    this.host.showLater(`${graph.arc.view}:${name}`);
    this.host.submit([{ op: 'add_raw', declaration: `diagram ${JSON.stringify(name)} {\n  view: ${graph.arc.view}\n  shows: ${showsLiteral(rest)}\n}`, top_level: true }], `new diagram ${name} without ${what}`);
  }

  /** Hide one element: out of a named diagram's `shows:`, or a new named diagram without it. */
  hide(element: ElementRecord): void {
    const graph = this.host.graph();
    if (!graph) return;
    const top = this.drawnAtTop(graph);
    if (!top.includes(element.id)) return this.host.toast(`${element.name || element.id} is drawn inside another box: hide that box, or open it as a diagram.`, 'info');
    this.showOnly(graph, top.filter((id) => id !== element.id), element.id);
  }

  hideMany(elements: ElementRecord[]): void {
    const graph = this.host.graph();
    if (!graph) return;
    const ids = new Set(elements.map((e) => e.id));
    this.showOnly(graph, this.drawnAtTop(graph).filter((id) => !ids.has(id)), `${ids.size} elements`);
  }

  private relatedOf(element: ElementRecord, snapshot: Snapshot): string[] {
    const around = neighbours(element, snapshot);
    return [...around.incoming, ...around.outgoing].map((n) => n.other?.id).filter((id): id is string => !!id);
  }

  /** Add what an element is connected to into the named diagram on screen (or a new one). */
  related(element: ElementRecord): void {
    const graph = this.host.graph();
    const snapshot = this.host.snapshot();
    if (!graph || !snapshot) return;
    const named = namedDiagramOf(graph.id, graph.arc.view);
    if (!named) return this.around(element);
    const current = this.drawnAtTop(graph);
    const shows = [...new Set([...current, element.id, ...this.relatedOf(element, snapshot)])];
    if (shows.length === current.length) return this.host.toast('Everything connected to it is already on this diagram.', 'info');
    this.host.submit([{ op: 'set', element: named, key: 'shows', value: showsLiteral(shows) }], `show what ${element.id} is connected to`);
  }

  /** A new named diagram with the element and what it is connected to (Capella's contextual diagram). */
  around(element: ElementRecord): void {
    const graph = this.host.graph();
    const snapshot = this.host.snapshot();
    if (!graph || !snapshot) return;
    const view = ['oab', 'sab', 'lab', 'pab', 'cap', 'cdb'].includes(graph.arc.view) ? graph.arc.view : 'lab';
    const name = this.uniqueName(`${element.name || element.id} context`);
    this.host.showLater(`${view}:${name}`);
    const shows = showsLiteral([...new Set([element.id, ...this.relatedOf(element, snapshot)])]);
    this.host.submit([{ op: 'add_raw', declaration: `diagram ${JSON.stringify(name)} {\n  view: ${view}\n  shows: ${shows}\n  context: true\n}`, top_level: true }], `new diagram ${name}`);
  }

  /** A new named diagram that shows the selected elements (and what they hold), on the view on screen. */
  fromSelection(elements: ElementRecord[]): void {
    const graph = this.host.graph();
    if (!graph || elements.length === 0) return;
    const view = ['oab', 'sab', 'lab', 'pab', 'cap', 'cdb'].includes(graph.arc.view) ? graph.arc.view : 'lab';
    const name = this.uniqueName(`${elements.length === 1 ? elements[0].name || elements[0].id : `${elements.length} elements`} diagram`);
    this.host.showLater(`${view}:${name}`);
    const shows = showsLiteral([...new Set(elements.map((e) => e.id))]);
    this.host.submit([{ op: 'add_raw', declaration: `diagram ${JSON.stringify(name)} {\n  view: ${view}\n  shows: ${shows}\n}`, top_level: true }], `new diagram ${name}`);
  }

  private uniqueName(base: string): string {
    const taken = new Set((this.host.snapshot()?.diagrams?.graphs ?? []).map((g) => g.arc.title));
    let name = base;
    for (let n = 2; taken.has(name); n++) name = `${base} ${n}`;
    return name;
  }
}
