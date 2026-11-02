// draw.io's Arrange, without VS Code: the tags an element carries (its
// first tag is its layer), the edit that moves it to another layer, the
// edits of "Group into a new component", the layers of a model with their
// counts, and the keys of the Arrange commands. Pure functions, tested with
// node:test; the webview imports from here.
import { EditRequest, ElementRecord, LAYER_OF_VIEW, stringLiteral, toolsFor } from './workbench';
import { freshIdLike } from './editing';

/** The layer of the elements without a tag. */
export const DEFAULT_LAYER = '';
export const layerLabel = (name: string): string => (name === DEFAULT_LAYER ? 'Default' : name);

/** The tags of an element, as `arclang workbench` lists its attributes (a list, one text, or the source of a list). */
export function tagsOf(attributes: Record<string, unknown> | undefined): string[] {
  const value = attributes?.tags;
  if (Array.isArray(value)) return value.filter((t): t is string => typeof t === 'string');
  if (typeof value !== 'string' || !value.trim()) return [];
  const text = value.trim();
  if (!text.startsWith('[')) return [text.replace(/^"(.*)"$/, '$1')];
  return text.replace(/^\[/, '').replace(/\]$/, '').split(',').map((t) => t.trim().replace(/^"(.*)"$/, '$1')).filter(Boolean);
}

/** The layer an element is on: its first tag. */
export const layerOfTags = (tags: string[]): string => tags[0] ?? DEFAULT_LAYER;

/**
 * The edit that puts an element on `layer`: its first tag (its layer) is
 * replaced, its other tags kept; the Default layer drops the first tag.
 * Null when it is on that layer already.
 */
export function moveToLayerEdit(element: ElementRecord, layer: string): EditRequest | null {
  const tags = tagsOf(element.attributes);
  const current = layerOfTags(tags);
  if (current === layer) return null;
  const rest = (current === DEFAULT_LAYER ? tags : tags.slice(1)).filter((t) => t !== layer);
  const next = layer === DEFAULT_LAYER ? rest : [layer, ...rest];
  if (next.length === 0) return { op: 'unset', element: element.id, key: 'tags' };
  return { op: 'set', element: element.id, key: 'tags', value: `[${next.map(stringLiteral).join(', ')}]` };
}

/** A name a new layer may take: a tag is a text, short, not the Default layer's. */
export function layerNameProblem(name: string, taken: string[]): string | null {
  const text = name.trim();
  if (!text) return 'A layer needs a name.';
  if (text.length > 60) return 'A layer name is at most 60 characters.';
  if (/["\\\n]/.test(text)) return 'A layer name has no quote, backslash or line break.';
  if (text.toLowerCase() === 'default') return '"Default" is the layer of the elements without a tag.';
  if (taken.includes(text)) return `The layer "${text}" exists already.`;
  return null;
}

export interface LayerRow {
  name: string;
  label: string;
  /** Elements of the model on it. */
  count: number;
}

/**
 * The layers of a model, top first: those the reader ordered, then the
 * others (a new one on top, as in draw.io), Default last unless ordered.
 */
export function layerRows(elements: { attributes?: Record<string, unknown> }[], order: string[], extra: string[] = []): LayerRow[] {
  const counts = new Map<string, number>([[DEFAULT_LAYER, 0]]);
  for (const name of extra) if (!counts.has(name)) counts.set(name, 0);
  for (const element of elements) {
    const tags = tagsOf(element.attributes);
    const layer = layerOfTags(tags);
    counts.set(layer, (counts.get(layer) ?? 0) + 1);
    for (const tag of tags.slice(1)) if (!counts.has(tag)) counts.set(tag, 0);
  }
  const names = [...counts.keys()];
  const ordered = order.filter((n) => counts.has(n));
  const others = names.filter((n) => !ordered.includes(n) && n !== DEFAULT_LAYER).sort((a, b) => a.localeCompare(b));
  const all = ordered.length ? [...others, ...ordered, ...(ordered.includes(DEFAULT_LAYER) ? [] : [DEFAULT_LAYER])] : [...others, DEFAULT_LAYER];
  return all.map((name) => ({ name, label: layerLabel(name), count: counts.get(name) ?? 0 }));
}

/** The layers one step up or down: `name` swapped with its neighbor (top first). */
export function stepLayer(names: string[], name: string, step: -1 | 1): string[] {
  const at = names.indexOf(name);
  const to = at + step;
  if (at < 0 || to < 0 || to >= names.length) return names;
  const next = names.slice();
  [next[at], next[to]] = [next[to], next[at]];
  return next;
}

/** `name` dragged before `before` (null: to the bottom). */
export function dropLayer(names: string[], name: string, before: string | null): string[] {
  const rest = names.filter((n) => n !== name);
  const at = before === null ? rest.length : rest.indexOf(before);
  return [...rest.slice(0, at < 0 ? rest.length : at), name, ...rest.slice(at < 0 ? rest.length : at)];
}

// --- Group into a new component --------------------------------------------------

/** The create tool that makes a container for elements of a kind, per view (a component holds components, a function functions). */
const MAKES: Record<string, Record<string, string>> = {
  lab: { LogicalComponent: 'component', LogicalFunction: 'function' },
  sab: { SystemFunction: 'function' },
  pab: { PhysicalNode: 'node', PhysicalFunction: 'function' },
  msm: { State: 'state' },
};

export interface GroupIntoPlan {
  requests: EditRequest[];
  /** The id the new container is given. */
  id: string;
  keyword: string;
}

/** What "Group into a new …" makes for this selection on this view ('component', 'function'…), or why not. */
export function groupIntoKind(view: string, elements: ElementRecord[]): { keyword: string; label: string } | { error: string } {
  if (elements.length < 2) return { error: 'Select two or more elements to group them into a new one.' };
  const kinds = [...new Set(elements.map((e) => e.kind))];
  if (kinds.length > 1) return { error: `The selection mixes ${kinds.join(' and ')}: a new container holds one kind.` };
  const toolId = MAKES[view]?.[kinds[0]];
  const tool = toolId ? toolsFor(view).find((t) => t.id === toolId && t.kind === 'create') : undefined;
  if (!tool?.keyword || !tool.containerKinds?.includes(kinds[0])) return { error: `On this view, the method gives no ${kinds[0]} a container of its own kind.` };
  const owners = new Set(elements.map((e) => e.owner ?? ''));
  if (owners.size > 1) return { error: 'The elements are in different containers: a new one goes where they all are.' };
  return { keyword: tool.keyword, label: tool.label.toLowerCase() };
}

/**
 * The edits of "Group into a new component": the container is added where
 * the elements are (in their owner, or in the layer of the view), then each
 * element is moved into it. One request list: one compiler check, one undo.
 */
export function groupIntoPlan(view: string, elements: ElementRecord[], all: ElementRecord[], name: string): GroupIntoPlan | { error: string } {
  const kind = groupIntoKind(view, elements);
  if ('error' in kind) return kind;
  const label = name.trim();
  if (!label) return { error: 'The new container needs a name.' };
  const byUuid = new Map(all.map((e) => [e.uuid, e]));
  const owner = elements[0].owner ? byUuid.get(elements[0].owner) : undefined;
  const tool = toolsFor(view).find((t) => t.keyword === kind.keyword && t.kind === 'create');
  const inOwner = owner && tool?.containerKinds?.includes(owner.kind);
  const layer = LAYER_OF_VIEW[view];
  if (!inOwner && !layer) return { error: `No layer for the ${view} view.` };
  const taken = new Set(all.map((e) => e.id));
  const id = freshIdLike(elements[0].id, taken) ?? freshName(label, taken);
  const add: EditRequest = { op: 'add', keyword: kind.keyword, name: label, attributes: [['id', stringLiteral(id)]], ...(inOwner ? { container: owner!.id } : { layer }) };
  return { requests: [add, ...elements.map((e): EditRequest => ({ op: 'move', element: e.id, container: id }))], id, keyword: kind.keyword };
}

/** An id from a name when the elements' ids have no prefix to number: `Sensing`, `Sensing-2`… */
function freshName(name: string, taken: Set<string>): string {
  const base = name.replace(/[^A-Za-z0-9_-]+/g, '-').replace(/^-+|-+$/g, '') || 'Group';
  if (!taken.has(base)) return base;
  for (let n = 2; ; n++) if (!taken.has(`${base}-${n}`)) return `${base}-${n}`;
}

// --- the keys -----------------------------------------------------------------------

/** The Arrange commands VS Code's keybindings send to the canvas. */
export const ARRANGE_COMMANDS = ['bring-front', 'send-back', 'bring-forward', 'send-backward', 'group', 'ungroup', 'lock', 'layers', 'select-vertices', 'select-edges', 'select-none'] as const;
export type ArrangeCommand = (typeof ARRANGE_COMMANDS)[number];

/**
 * Their keys, draw.io's where it has them: Mod is ⌘ on a Mac, Ctrl
 * elsewhere. VS Code binds them under the canvas's when-clause (so ⌘⇧F
 * there is not Find in Files) and resolves them on the keyboard's layout.
 * ⌘] and ⌘[ are VS Code's indent: forward and backward take ⌥⇧⌘F and ⌥⇧⌘B.
 */
export const ARRANGE_KEYS: Record<ArrangeCommand, { keys: string; name: string; label: string }> = {
  'bring-front': { keys: 'Mod+Shift+F', name: 'bringToFront', label: 'Bring to front' },
  'send-back': { keys: 'Mod+Shift+B', name: 'sendToBack', label: 'Send to back' },
  'bring-forward': { keys: 'Mod+Alt+Shift+F', name: 'bringForward', label: 'Bring forward' },
  'send-backward': { keys: 'Mod+Alt+Shift+B', name: 'sendBackward', label: 'Send backward' },
  group: { keys: 'Mod+G', name: 'group', label: 'Group' },
  ungroup: { keys: 'Mod+Shift+G', name: 'ungroup', label: 'Ungroup' },
  lock: { keys: 'Mod+L', name: 'lock', label: 'Lock / unlock' },
  layers: { keys: 'Mod+Shift+L', name: 'layers', label: 'Layers' },
  'select-vertices': { keys: 'Mod+Shift+I', name: 'selectVertices', label: 'Select boxes' },
  'select-edges': { keys: 'Mod+Shift+E', name: 'selectEdges', label: 'Select lines' },
  'select-none': { keys: 'Mod+Shift+A', name: 'selectNone', label: 'Select none' },
};

/** The command a key press is, as the page sees it outside VS Code (and to let VS Code's own through). */
export function arrangeCommandOf(event: { key: string; code?: string; metaKey: boolean; ctrlKey: boolean; shiftKey: boolean; altKey: boolean }): ArrangeCommand | null {
  if (!(event.metaKey || event.ctrlKey)) return null;
  // With Alt, a Mac types another letter (⌥F is ƒ): the key's code says which one it is.
  const letter = (event.code && /^Key[A-Z]$/.test(event.code) && event.altKey ? event.code.slice(3) : event.key).toLowerCase();
  for (const command of ARRANGE_COMMANDS) {
    const parts = ARRANGE_KEYS[command].keys.split('+');
    const key = parts[parts.length - 1].toLowerCase();
    if (key !== letter) continue;
    if (parts.includes('Shift') !== event.shiftKey || parts.includes('Alt') !== event.altKey) continue;
    return command;
  }
  return null;
}

/** The keybindings of package.json for the Arrange commands (what the test of package.json checks). */
export function arrangeKeybindings(when: string): { command: string; key: string; mac: string; when: string }[] {
  return ARRANGE_COMMANDS.map((command) => {
    const keys = ARRANGE_KEYS[command].keys.toLowerCase();
    return { command: `arclang.workbench.${ARRANGE_KEYS[command].name}`, key: keys.replace('mod', 'ctrl'), mac: keys.replace('mod', 'cmd'), when };
  });
}

/** The shortcuts list (?): what Arrange adds, written for a Mac as the rest of it is. */
export const ARRANGE_HELP: [string, string][] = [
  ['⇧⌘F  ⇧⌘B', 'Bring to front, send to back'],
  ['⌥⇧⌘F  ⌥⇧⌘B', 'Bring forward, send backward (past the next box it overlaps)'],
  ['⌘G  ⇧⌘G', 'Group the selected boxes, ungroup (double-click a group to edit inside, Esc to leave)'],
  ['⌘L', 'Lock or unlock the selection on the canvas'],
  ['⇧⌘L', 'Layers: show, hide, lock and order the tags'],
  ['⇧⌘I  ⇧⌘E  ⇧⌘A', 'Select every box, every line, nothing'],
];
