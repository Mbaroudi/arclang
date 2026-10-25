// The text of what the "New ..." commands write: a model header, the Arcadia
// skeleton, a layer block, a diagram declaration. No VS Code here: the
// compiler is the judge of all of it (`arclang check`, `arclang edit`).
import { stringLiteral } from './workbench';

/** A model name taken from a file name: `lane_keeping.arc` is `lane_keeping`. */
export function modelNameOf(fileName: string): string {
  const base = fileName.replace(/^.*[\\/]/, '').replace(/\.(arc|sysml)$/i, '');
  const name = base.replace(/[^A-Za-z0-9_]+/g, '_').replace(/^_+|_+$/g, '');
  return /^[A-Za-z_]/.test(name) ? name : name ? `M_${name}` : 'Model';
}

/** The header of a model: nothing else. */
export function blankModel(name: string, title = name): string {
  return `model ${modelNameOf(name)} {\n  name: ${stringLiteral(title)}\n  version: "0.1"\n}\n`;
}

/** What a level of the method declares, in the order of the method. */
export const SKELETON_LAYERS: { keyword: string; title: string }[] = [
  { keyword: 'operational_analysis', title: 'Operational Analysis' },
  { keyword: 'system_analysis', title: 'System Analysis' },
  { keyword: 'logical_architecture', title: 'Logical Architecture' },
  { keyword: 'physical_architecture', title: 'Physical Architecture' },
];

/** The four Arcadia levels, empty: it compiles, each level waits for its first element. */
export function arcadiaSkeleton(name: string, title = name): string {
  const layers = SKELETON_LAYERS.map((l) => `${l.keyword} ${stringLiteral(l.title)} {\n}\n`).join('\n');
  return `${blankModel(name, title)}\n${layers}`;
}

/** A level or a section a model may hold once. */
export interface LayerChoice {
  /** The keyword that opens the block. */
  keyword: string;
  /** What the quick pick says. */
  label: string;
  detail: string;
  /** The block's name, when it has one. */
  title?: string;
}

export const LAYER_CHOICES: LayerChoice[] = [
  { keyword: 'operational_analysis', label: 'Operational', detail: 'actors, entities, activities, operational capabilities', title: 'Operational Analysis' },
  { keyword: 'system_analysis', label: 'System', detail: 'the system, its actors, functions and capabilities', title: 'System Analysis' },
  { keyword: 'logical_architecture', label: 'Logical', detail: 'logical components and functions', title: 'Logical Architecture' },
  { keyword: 'physical_architecture', label: 'Physical', detail: 'nodes, behaviour components, physical links', title: 'Physical Architecture' },
  { keyword: 'epbs', label: 'EPBS', detail: 'the end product breakdown: systems, subsystems, configuration items', title: 'EPBS' },
  { keyword: 'requirements', label: 'Requirements', detail: 'the requirements the architecture answers' },
  { keyword: 'safety_analysis', label: 'Safety', detail: 'hazards, failure modes, safety goals' },
];

/** The top-level blocks a source already holds (a keyword at the start of a line). */
export function heldBlocks(source: string): Set<string> {
  const held = new Set<string>();
  for (const match of source.matchAll(/^([a-z_]+)\b/gm)) held.add(match[1]);
  return held;
}

/** The levels and sections the model does not hold yet. */
export function missingLayers(source: string): LayerChoice[] {
  const held = heldBlocks(source);
  return LAYER_CHOICES.filter((choice) => !held.has(choice.keyword));
}

/** The block that opens a layer, empty. */
export function layerBlock(choice: LayerChoice): string {
  return choice.title ? `${choice.keyword} ${stringLiteral(choice.title)} {\n}` : `${choice.keyword} {\n}`;
}

/** A diagram declaration: a view and the elements it starts from. */
export function diagramDeclaration(name: string, view: string, shows: string[]): string {
  const list = shows.length > 0 ? `\n  shows: [${shows.map((id) => stringLiteral(id)).join(', ')}]` : '';
  return `diagram ${stringLiteral(name)} {\n  view: ${view}${list}\n}`;
}

/** `base`, else `base 2`, `base 3`... the first name not in `taken`. */
export function uniqueName(base: string, taken: Iterable<string>): string {
  const names = new Set(taken);
  let name = base;
  for (let n = 2; names.has(name); n++) name = `${base} ${n}`;
  return name;
}

/** A file name for a model: the name given, with its extension. */
export function arcFileName(name: string): string {
  const trimmed = name.trim();
  return /\.arc$/i.test(trimmed) ? trimmed : `${trimmed}.arc`;
}
