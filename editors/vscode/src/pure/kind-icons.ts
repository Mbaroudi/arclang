// One icon per element kind, drawn as the diagram draws the kind: Capella's
// colour code (blue structure, green functions, yellow hardware, orange
// activities) and the viewer's glyphs (the stick figure of an actor, the
// circled f of a function, the 3D box of a node). Pure data: the side bar
// tree writes them as SVG files, the properties pane builds them as nodes.

export interface IconShape {
  tag: 'circle' | 'rect' | 'path' | 'ellipse' | 'line';
  attrs: Record<string, string | number>;
}

export interface KindIcon {
  /** The family of kinds the icon stands for (`function`, `component`...): the file name of the tree's icon. */
  family: string;
  fill: string;
  stroke: string;
  shapes: IconShape[];
}

const box = (x: number, y: number, w: number, h: number, rx = 1.5): IconShape => ({ tag: 'rect', attrs: { x, y, width: w, height: h, rx } });
const path = (d: string, extra: Record<string, string | number> = {}): IconShape => ({ tag: 'path', attrs: { d, ...extra } });
const circle = (cx: number, cy: number, r: number, extra: Record<string, string | number> = {}): IconShape => ({ tag: 'circle', attrs: { cx, cy, r, ...extra } });
const NONE = { fill: 'none' };

/** The glyphs, on a 16 x 16 grid; a shape is filled with the family's fill unless it says otherwise. */
const FAMILIES: Record<string, Omit<KindIcon, 'family'>> = {
  actor: { fill: '#DCEAF8', stroke: '#3F6FA6', shapes: [circle(8, 3.6, 2.4), path('M8 6v4.6M4.2 7.8h7.6M8 10.6l-3.2 4M8 10.6l3.2 4', NONE)] },
  entity: { fill: '#EEF0F2', stroke: '#66717D', shapes: [box(1.5, 3, 13, 10), path('M1.5 6h13', NONE)] },
  activity: { fill: '#FFE3AE', stroke: '#B0730C', shapes: [box(1.5, 3, 13, 10, 4.5), path('M9.6 5.6c-1.8-.5-2.5.6-2.5 1.8v4.4M5.6 8.6h3.4', NONE)] },
  function: { fill: '#CDEDBE', stroke: '#3C8A3F', shapes: [box(1.5, 3, 13, 10, 4.5), path('M9.6 5.6c-1.8-.5-2.5.6-2.5 1.8v4.4M5.6 8.6h3.4', NONE)] },
  component: { fill: '#D3E4F8', stroke: '#2A62A8', shapes: [box(4, 2, 10.5, 12, 1.2), box(1.5, 4.5, 5, 2.4, 0.5), box(1.5, 9.1, 5, 2.4, 0.5)] },
  system: { fill: '#E3EEFB', stroke: '#2A62A8', shapes: [box(1.5, 2.5, 13, 11, 1.5), path('M4 6h8M4 9h5', NONE)] },
  node: { fill: '#FFF4AE', stroke: '#9C8500', shapes: [path('M1.5 5.5h9.5v9H1.5z'), path('M1.5 5.5l3-3h9.5l-3 3M14 2.5v9l-3 3')] },
  hardware: { fill: '#FFE98A', stroke: '#9C8500', shapes: [box(2, 3.5, 12, 9, 1), path('M4.5 1.5v2M8 1.5v2M11.5 1.5v2M4.5 12.5v2M8 12.5v2M11.5 12.5v2', NONE)] },
  function_port: { fill: '#3C8A3F', stroke: '#2E6B30', shapes: [box(4, 4, 8, 8, 0.8)] },
  component_port: { fill: '#FFFFFF', stroke: '#1D4E96', shapes: [box(3.5, 3.5, 9, 9, 0.8), path('M6.4 5.6L10 8l-3.6 2.4z', { fill: '#1D4E96', stroke: 'none' })] },
  physical_port: { fill: '#FFF4AE', stroke: '#9C8500', shapes: [box(3.5, 3.5, 9, 9, 0.8), circle(8, 8, 1.6, { fill: '#9C8500', stroke: 'none' })] },
  functional_exchange: { fill: 'none', stroke: '#1F7A33', shapes: [path('M1.5 8h11'), path('M10 5l3.5 3-3.5 3', NONE)] },
  component_exchange: { fill: 'none', stroke: '#1D4E96', shapes: [path('M1.5 8h11'), path('M10 5l3.5 3-3.5 3', NONE), box(0.5, 6, 3, 4, 0.4)] },
  operational_exchange: { fill: 'none', stroke: '#7A5200', shapes: [path('M1.5 8h11'), path('M10 5l3.5 3-3.5 3', NONE)] },
  physical_link: { fill: 'none', stroke: '#C92A2A', shapes: [path('M1.5 8h13', { 'stroke-width': 2.4 }), circle(2, 8, 1.4, { fill: '#C92A2A', stroke: 'none' }), circle(14, 8, 1.4, { fill: '#C92A2A', stroke: 'none' })] },
  physical_exchange: { fill: 'none', stroke: '#1D4E96', shapes: [path('M1.5 8h11', { 'stroke-dasharray': '2.4 1.4' }), path('M10 5l3.5 3-3.5 3', NONE)] },
  capability: { fill: '#E3EEFB', stroke: '#2A62A8', shapes: [{ tag: 'ellipse', attrs: { cx: 8, cy: 8, rx: 6.5, ry: 4.4 } }] },
  operational_capability: { fill: '#F3E8D3', stroke: '#8A6A2E', shapes: [{ tag: 'ellipse', attrs: { cx: 8, cy: 8, rx: 6.5, ry: 4.4 } }] },
  mission: { fill: '#FFE9B8', stroke: '#9A6400', shapes: [path('M3.5 14.5v-13', NONE), path('M3.5 2.5h9l-2.4 3 2.4 3h-9z')] },
  chain: { fill: '#FCE3D4', stroke: '#B4500F', shapes: [circle(3.5, 8, 2.2), circle(12.5, 8, 2.2), path('M5.7 8h4.6', NONE)] },
  requirement: { fill: '#EDE4F7', stroke: '#6A3FA6', shapes: [path('M3 1.5h7l3 3v10H3z'), path('M10 1.5v3h3M5.5 7.5h5M5.5 10h5M5.5 12.5h3', NONE)] },
  test_case: { fill: '#E1F2EC', stroke: '#2E7D5B', shapes: [box(2, 2, 12, 12, 2), path('M4.8 8.2l2.2 2.3 4.4-4.8', NONE)] },
  constraint: { fill: 'none', stroke: '#4A545E', shapes: [path('M6 2c-2 0-2 1-2 2.6S3 7.4 2 8c1 .6 2 1 2 3.4S4 14 6 14M10 2c2 0 2 1 2 2.6s1 2.8 2 3.4c-1 .6-2 1-2 3.4S12 14 10 14')] },
  hazard: { fill: '#FDE2E1', stroke: '#B42318', shapes: [path('M8 1.8l6.6 12H1.4z'), path('M8 6v3.8M8 11.6v.4', NONE)] },
  threat: { fill: '#FCE8D8', stroke: '#B4500F', shapes: [path('M8 1.5l5.5 2v4.2c0 3.2-2.4 5.6-5.5 6.8-3.1-1.2-5.5-3.6-5.5-6.8V3.5z')] },
  asset: { fill: '#FFF1CC', stroke: '#9A6400', shapes: [path('M8 1.5l6 6.5-6 6.5-6-6.5z')] },
  state: { fill: '#EEF0F2', stroke: '#4A545E', shapes: [box(1.5, 3.5, 13, 9, 4.5)] },
  mode: { fill: '#E3EEFB', stroke: '#2A62A8', shapes: [box(1.5, 3.5, 13, 9, 4.5)] },
  state_machine: { fill: '#EEF0F2', stroke: '#4A545E', shapes: [box(0.8, 1.5, 7, 5, 2.4), box(8.2, 9.5, 7, 5, 2.4), path('M4.3 6.5v5h3.4', NONE), path('M6.4 10.2l1.4 1.3-1.4 1.3', NONE)] },
  transition: { fill: 'none', stroke: '#343A40', shapes: [path('M2 12C4 4 10 4 13 9'), path('M10.6 9.4l2.6-.4.2-2.6', NONE)] },
  scenario: { fill: 'none', stroke: '#18222D', shapes: [box(1, 1.5, 5, 3, 0.6), box(10, 1.5, 5, 3, 0.6), path('M3.5 4.5v10M12.5 4.5v10', { 'stroke-dasharray': '1.6 1.2' }), path('M3.5 9h8M10 7.5l1.6 1.5L10 10.5', NONE)] },
  message: { fill: 'none', stroke: '#18222D', shapes: [path('M1.5 8h11'), path('M10 5.5l3 2.5-3 2.5', NONE)] },
  fragment: { fill: 'none', stroke: '#55616D', shapes: [box(1.5, 2.5, 13, 11, 0.6), path('M1.5 6h4.5l1.2-1.4V2.5', NONE)] },
  class: { fill: '#F7F1DC', stroke: '#7A6A2A', shapes: [box(1.5, 1.5, 13, 13, 0.8), path('M1.5 5.5h13M1.5 10h13', NONE)] },
  enumeration: { fill: '#EEF0F2', stroke: '#4A545E', shapes: [box(1.5, 1.5, 13, 13, 0.8), path('M1.5 5.5h13M4 8h8M4 11h6', NONE)] },
  data_type: { fill: '#EEF0F2', stroke: '#4A545E', shapes: [box(1.5, 3, 13, 10, 0.8), path('M5 6l-2 2 2 2M11 6l2 2-2 2', NONE)] },
  exchange_item: { fill: '#E4F3E0', stroke: '#3C8A3F', shapes: [box(1.5, 3, 13, 10, 0.8), path('M1.5 3l6.5 5.4L14.5 3', NONE)] },
  interface: { fill: '#FFFFFF', stroke: '#2A62A8', shapes: [circle(10.5, 8, 3.5), path('M1.5 8h5.5', NONE)] },
  association: { fill: 'none', stroke: '#7A6A2A', shapes: [path('M1.5 12.5l13-9'), path('M1.5 12.5l2.2-3.6 1.4 2 2.2-.2z', { fill: '#7A6A2A' })] },
  trace: { fill: 'none', stroke: '#55616D', shapes: [path('M1.5 8h10', { 'stroke-dasharray': '2.2 1.6' }), path('M10 5l3.5 3-3.5 3', NONE)] },
  deployment: { fill: 'none', stroke: '#9C8500', shapes: [path('M8 1.5v9'), path('M5 7.5l3 3 3-3', NONE), path('M2 13.5h12', { 'stroke-width': 1.8 })] },
  type: { fill: '#F1EAFB', stroke: '#5B3E96', shapes: [box(1.5, 1.5, 13, 13, 2), path('M4.8 5h6.4M8 5v7', NONE)] },
  configuration: { fill: '#E9ECEF', stroke: '#3D4852', shapes: [circle(8, 8, 3), path('M8 1.5v2.5M8 12v2.5M1.5 8H4M12 8h2.5M3.4 3.4l1.8 1.8M10.8 10.8l1.8 1.8M3.4 12.6l1.8-1.8M10.8 5.2l1.8-1.8', NONE)] },
  epbs: { fill: '#E9ECEF', stroke: '#3D4852', shapes: [path('M1.5 5.5h9.5v9H1.5z'), path('M1.5 5.5l3-3h9.5l-3 3M14 2.5v9l-3 3')] },
  model: { fill: '#E3EEFB', stroke: '#2A62A8', shapes: [path('M1.5 4.5h5l1.5-2h6.5v11h-13z')] },
  package: { fill: '#F0F1F3', stroke: '#55616D', shapes: [path('M1.5 4.5h5l1.5-2h6.5v11h-13z')] },
  diagram: { fill: '#FFFFFF', stroke: '#2A62A8', shapes: [box(1, 2, 5, 4, 0.8), box(10, 2, 5, 4, 0.8), box(5.5, 10, 5, 4, 0.8), path('M6 4h4M3.5 6l3 4M12.5 6l-3 4', NONE)] },
  table: { fill: '#FFFFFF', stroke: '#3D4852', shapes: [box(1.5, 2.5, 13, 11, 0.8), path('M1.5 6h13M1.5 9.5h13M6 2.5v11', NONE)] },
  element: { fill: '#FFFFFF', stroke: '#444B52', shapes: [box(2, 3, 12, 10, 1.2)] },
};

/** The family of each kind of the metamodel; kinds not listed draw as a plain element. */
const KIND_FAMILY: Record<string, string> = {
  Actor: 'actor',
  SystemActor: 'actor',
  OperationalEntity: 'entity',
  OperationalActivity: 'activity',
  OperationalExchange: 'operational_exchange',
  OperationalProcess: 'chain',
  OperationalCapability: 'operational_capability',
  Requirement: 'requirement',
  Mission: 'mission',
  Capability: 'capability',
  CapabilityRealization: 'capability',
  SystemFunction: 'function',
  LogicalFunction: 'function',
  PhysicalFunction: 'function',
  FunctionPort: 'function_port',
  FunctionalExchange: 'functional_exchange',
  SystemComponent: 'component',
  LogicalComponent: 'component',
  BehaviorComponent: 'component',
  FunctionalChain: 'chain',
  PhysicalPath: 'chain',
  ControlNode: 'chain',
  SequenceLink: 'trace',
  ComponentPort: 'component_port',
  LogicalInterface: 'interface',
  ComponentExchange: 'component_exchange',
  PhysicalNode: 'node',
  PhysicalPort: 'physical_port',
  HardwareComponent: 'hardware',
  PhysicalLink: 'physical_link',
  PhysicalExchange: 'physical_exchange',
  Deployment: 'deployment',
  EpbsSystem: 'epbs',
  EpbsSubsystem: 'epbs',
  EpbsItem: 'epbs',
  Class: 'class',
  Association: 'association',
  Enumeration: 'enumeration',
  DataType: 'data_type',
  ExchangeItem: 'exchange_item',
  Hazard: 'hazard',
  FmeaEntry: 'hazard',
  Threat: 'threat',
  PrimaryAsset: 'asset',
  Trace: 'trace',
  Realization: 'trace',
  Verification: 'trace',
  Configuration: 'configuration',
  TestCase: 'test_case',
  Model: 'model',
  Type: 'type',
  Constraint: 'constraint',
  StateMachine: 'state_machine',
  State: 'state',
  Mode: 'mode',
  Transition: 'transition',
  Diagram: 'diagram',
  Table: 'table',
  Scenario: 'scenario',
  Fragment: 'fragment',
  StateFragment: 'fragment',
  Message: 'message',
  Package: 'package',
  System: 'system',
};

/** Every family, for the files the tree writes once. */
export function iconFamilies(): string[] {
  return Object.keys(FAMILIES);
}

export function familyOf(kind: string): string {
  return KIND_FAMILY[kind] ?? (FAMILIES[kind] ? kind : 'element');
}

export function kindIcon(kind: string): KindIcon {
  const family = familyOf(kind);
  return { family, ...FAMILIES[family] };
}

/** The icon of a family as an SVG document (the tree's icon files). */
export function iconSvg(family: string, size = 16): string {
  const icon = FAMILIES[family] ?? FAMILIES.element;
  const shapes = icon.shapes
    .map((shape) => {
      const attrs: Record<string, string | number> = { fill: icon.fill, stroke: icon.stroke, ...shape.attrs };
      const written = Object.entries(attrs)
        .map(([key, value]) => `${key}="${String(value).replace(/[&<>"]/g, '')}"`)
        .join(' ');
      return `<${shape.tag} ${written}/>`;
    })
    .join('');
  return `<svg xmlns="http://www.w3.org/2000/svg" width="${size}" height="${size}" viewBox="0 0 16 16" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round">${shapes}</svg>`;
}
