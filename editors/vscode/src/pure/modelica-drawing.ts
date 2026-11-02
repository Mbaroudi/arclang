// The diagram of a Modelica class, drawn from what OpenModelica knows of it
// (the JSON of getModelInstance, `arclang verify --drawing CLASS`): its
// components with the icons of their classes, where they are placed, the
// connectors on them, the lines of the connections, the class's own
// graphics. Every coordinate is computed here (placement, rotation, flip,
// the icon's coordinate system), so the text stays upright and a rotated
// rectangle or an arc comes out right; the result is SVG text, its strings
// escaped. Free of the DOM and of VS Code.

type Json = unknown;
type Obj = Record<string, Json>;
type Point = [number, number];
/** An affine map of the plane: x' = a x + c y + e, y' = b x + d y + f. */
type Matrix = [number, number, number, number, number, number];

const IDENTITY: Matrix = [1, 0, 0, 1, 0, 0];
const DEFAULT_EXTENT: [Point, Point] = [[-100, -100], [100, 100]];
const MAX_DEPTH = 4;

const isObj = (v: Json): v is Obj => typeof v === 'object' && v !== null && !Array.isArray(v);
const num = (v: Json, fallback: number): number => (typeof v === 'number' && Number.isFinite(v) ? v : fallback);
const point = (v: Json, fallback: Point = [0, 0]): Point => (Array.isArray(v) && v.length === 2 ? [num(v[0], fallback[0]), num(v[1], fallback[1])] : fallback);
const points = (v: Json): Point[] => (Array.isArray(v) ? v.filter((p) => Array.isArray(p) && p.length === 2).map((p) => point(p)) : []);
const extentOf = (v: Json, fallback: [Point, Point] = DEFAULT_EXTENT): [Point, Point] => (Array.isArray(v) && v.length === 2 ? [point(v[0], fallback[0]), point(v[1], fallback[1])] : fallback);
const color = (v: Json, fallback: [number, number, number]): [number, number, number] =>
  Array.isArray(v) && v.length === 3 && v.every((c) => typeof c === 'number') ? [v[0] as number, v[1] as number, v[2] as number] : fallback;
/** `LinePattern.Dash` → `Dash`; anything else (an expression) → undefined. */
const enumName = (v: Json): string | undefined => (isObj(v) && v.$kind === 'enum' && typeof v.name === 'string' ? v.name.split('.').pop() : typeof v === 'string' ? v.split('.').pop() : undefined);

export function escapeXml(text: string): string {
  return text.replace(/[&<>"']/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' })[c] as string);
}

const multiply = (m: Matrix, n: Matrix): Matrix => [
  m[0] * n[0] + m[2] * n[1],
  m[1] * n[0] + m[3] * n[1],
  m[0] * n[2] + m[2] * n[3],
  m[1] * n[2] + m[3] * n[3],
  m[0] * n[4] + m[2] * n[5] + m[4],
  m[1] * n[4] + m[3] * n[5] + m[5],
];
const apply = (m: Matrix, [x, y]: Point): Point => [m[0] * x + m[2] * y + m[4], m[1] * x + m[3] * y + m[5]];
const translate = (x: number, y: number): Matrix => [1, 0, 0, 1, x, y];
const rotate = (degrees: number): Matrix => {
  const r = (degrees * Math.PI) / 180;
  return [Math.cos(r), Math.sin(r), -Math.sin(r), Math.cos(r), 0, 0];
};

/**
 * The map from a class's icon coordinates to its place in the parent: the
 * icon's extent onto the placement's (a reversed extent flips), turned by
 * the rotation, moved to the origin (Modelica 3.6, 18.6.5).
 */
export function placementMatrix(icon: [Point, Point], placed: { origin: Point; extent: [Point, Point]; rotation: number }): Matrix {
  const [[ix1, iy1], [ix2, iy2]] = icon;
  const [[ex1, ey1], [ex2, ey2]] = placed.extent;
  const sx = ix2 === ix1 ? 1 : (ex2 - ex1) / (ix2 - ix1);
  const sy = iy2 === iy1 ? 1 : (ey2 - ey1) / (iy2 - iy1);
  const fit: Matrix = [sx, 0, 0, sy, ex1 - ix1 * sx, ey1 - iy1 * sy];
  return multiply(translate(placed.origin[0], placed.origin[1]), multiply(rotate(placed.rotation), fit));
}

/** A graphic's own origin and rotation (GraphicItem), on top of the map it is drawn with. */
const itemMatrix = (m: Matrix, origin: Point, rotation: number): Matrix => multiply(m, multiply(translate(origin[0], origin[1]), rotate(rotation)));

// ---- The graphics records ---------------------------------------------------

const FIELDS: Record<string, string[]> = {
  Line: ['visible', 'origin', 'rotation', 'points', 'color', 'pattern', 'thickness', 'arrow', 'arrowSize', 'smooth'],
  Polygon: ['visible', 'origin', 'rotation', 'lineColor', 'fillColor', 'pattern', 'fillPattern', 'lineThickness', 'points', 'smooth'],
  Rectangle: ['visible', 'origin', 'rotation', 'lineColor', 'fillColor', 'pattern', 'fillPattern', 'lineThickness', 'borderPattern', 'extent', 'radius'],
  Ellipse: ['visible', 'origin', 'rotation', 'lineColor', 'fillColor', 'pattern', 'fillPattern', 'lineThickness', 'extent', 'startAngle', 'endAngle', 'closure'],
  Text: ['visible', 'origin', 'rotation', 'lineColor', 'fillColor', 'pattern', 'fillPattern', 'lineThickness', 'extent', 'textString', 'fontSize', 'textColor', 'fontName', 'textStyle', 'horizontalAlignment'],
  Bitmap: ['visible', 'origin', 'rotation', 'extent', 'fileName', 'imageSource'],
};

/** A graphic as named fields: OpenModelica gives a record's fields in order (positional), a connection's Line by name. */
export function recordFields(kind: string, value: Json): Obj {
  if (isObj(value) && Array.isArray(value.elements)) {
    const names = FIELDS[kind] ?? [];
    const fields: Obj = {};
    names.forEach((name, i) => (fields[name] = (value.elements as Json[])[i]));
    return fields;
  }
  return isObj(value) ? value : {};
}

interface Style {
  stroke: [number, number, number] | null;
  fill: [number, number, number] | null;
  dash: string;
  width: number;
}

const DASHES: Record<string, string> = { Dash: '6 3', Dot: '1.5 3', DashDot: '6 3 1.5 3', DashDotDot: '6 3 1.5 3 1.5 3' };
const rgb = (c: [number, number, number]) => `rgb(${c[0]},${c[1]},${c[2]})`;

function styleOf(f: Obj, line: boolean): Style {
  const pattern = enumName(f.pattern) ?? 'Solid';
  const strokeColor = color(line ? f.color : f.lineColor, [0, 0, 0]);
  const fillPattern = enumName(f.fillPattern) ?? 'None';
  const thickness = num(line ? f.thickness : f.lineThickness, 0.25);
  return {
    stroke: pattern === 'None' ? null : strokeColor,
    fill: line || fillPattern === 'None' ? null : color(f.fillColor, [255, 255, 255]),
    dash: DASHES[pattern] ?? '',
    width: Math.max(1, thickness * 4),
  };
}

function styleAttributes(s: Style): string {
  const parts = [`fill="${s.fill ? rgb(s.fill) : 'none'}"`, `stroke="${s.stroke ? rgb(s.stroke) : 'none'}"`];
  if (s.stroke) parts.push(`stroke-width="${s.width}"`, 'vector-effect="non-scaling-stroke"', 'stroke-linejoin="round"', 'stroke-linecap="round"');
  if (s.dash) parts.push(`stroke-dasharray="${s.dash}"`);
  return parts.join(' ');
}

/** The output: SVG elements in the diagram's coordinates (y up), and the box they cover. */
class Canvas {
  readonly parts: string[] = [];
  minX = Infinity;
  minY = Infinity;
  maxX = -Infinity;
  maxY = -Infinity;

  cover(p: Point): void {
    this.minX = Math.min(this.minX, p[0]);
    this.maxX = Math.max(this.maxX, p[0]);
    this.minY = Math.min(this.minY, p[1]);
    this.maxY = Math.max(this.maxY, p[1]);
  }

  /** SVG's y goes down: the diagram's is flipped once, here. */
  path(ps: Point[], closed: boolean, style: Style): void {
    if (ps.length < 2) return;
    ps.forEach((p) => this.cover(p));
    const d = ps.map((p, i) => `${i ? 'L' : 'M'}${round(p[0])} ${round(-p[1])}`).join('') + (closed ? 'Z' : '');
    this.parts.push(`<path d="${d}" ${styleAttributes(style)}/>`);
  }
}

const round = (v: number) => Math.round(v * 100) / 100;

/** The points of a smooth (Bézier) line through the midpoints, as Modelica tools draw it. */
function smoothed(ps: Point[]): Point[] {
  if (ps.length < 3) return ps;
  const out: Point[] = [ps[0]];
  const mid = (a: Point, b: Point): Point => [(a[0] + b[0]) / 2, (a[1] + b[1]) / 2];
  for (let i = 1; i < ps.length - 1; i++) {
    const start = i === 1 ? ps[0] : mid(ps[i - 1], ps[i]);
    const end = i === ps.length - 2 ? ps[ps.length - 1] : mid(ps[i], ps[i + 1]);
    for (let t = 0.1; t <= 1.0001; t += 0.1) {
      const u = 1 - t;
      out.push([u * u * start[0] + 2 * u * t * ps[i][0] + t * t * end[0], u * u * start[1] + 2 * u * t * ps[i][1] + t * t * end[1]]);
    }
  }
  return out;
}

/** An ellipse (or its arc) in its extent, as points: drawn exactly whatever the map does to it. */
function ellipsePoints(extent: [Point, Point], start: number, end: number): Point[] {
  const [[x1, y1], [x2, y2]] = extent;
  const cx = (x1 + x2) / 2;
  const cy = (y1 + y2) / 2;
  const rx = Math.abs(x2 - x1) / 2;
  const ry = Math.abs(y2 - y1) / 2;
  const span = end - start;
  const steps = Math.max(8, Math.ceil(Math.abs(span) / 6));
  const out: Point[] = [];
  for (let i = 0; i <= steps; i++) {
    const a = ((start + (span * i) / steps) * Math.PI) / 180;
    out.push([cx + rx * Math.cos(a), cy + ry * Math.sin(a)]);
  }
  return out;
}

/** A rectangle with rounded corners, as points. */
function rectanglePoints(extent: [Point, Point], radius: number): Point[] {
  const [[ax, ay], [bx, by]] = extent;
  const [x1, x2] = [Math.min(ax, bx), Math.max(ax, bx)];
  const [y1, y2] = [Math.min(ay, by), Math.max(ay, by)];
  const r = Math.min(radius, (x2 - x1) / 2, (y2 - y1) / 2);
  if (r <= 0) return [[x1, y1], [x2, y1], [x2, y2], [x1, y2]];
  const corner = (cx: number, cy: number, from: number): Point[] =>
    Array.from({ length: 7 }, (_, i) => {
      const a = ((from + i * 15) * Math.PI) / 180;
      return [cx + r * Math.cos(a), cy + r * Math.sin(a)] as Point;
    });
  return [...corner(x2 - r, y1 + r, 270), ...corner(x2 - r, y2 - r, 0), ...corner(x1 + r, y2 - r, 90), ...corner(x1 + r, y1 + r, 180)];
}

function arrowHead(canvas: Canvas, tip: Point, from: Point, size: number, filled: boolean, style: Style): void {
  const dx = tip[0] - from[0];
  const dy = tip[1] - from[1];
  const length = Math.hypot(dx, dy) || 1;
  const [ux, uy] = [dx / length, dy / length];
  const back: Point = [tip[0] - ux * size, tip[1] - uy * size];
  const left: Point = [back[0] - uy * size * 0.4, back[1] + ux * size * 0.4];
  const right: Point = [back[0] + uy * size * 0.4, back[1] - ux * size * 0.4];
  const stroke = style.stroke ?? [0, 0, 0];
  canvas.path(filled ? [left, tip, right] : [left, tip, right], filled, { ...style, dash: '', fill: filled ? stroke : null });
}

/** What a text of an icon says: %name and %class filled, a parameter's value when it is known. */
export function fillText(text: string, context: TextContext): string {
  return text.replace(/%(\{[A-Za-z_][\w.]*\}|[A-Za-z_][\w.]*|%)/g, (whole, name: string) => {
    const key = name.replace(/^\{|\}$/g, '');
    if (key === '%') return '%';
    if (key === 'name') return context.name;
    if (key === 'class') return context.className;
    return context.parameters[key] ?? whole;
  });
}

export interface TextContext {
  name: string;
  className: string;
  parameters: Record<string, string>;
}

function drawText(canvas: Canvas, m: Matrix, f: Obj, context: TextContext): void {
  const raw = typeof f.textString === 'string' ? f.textString : '';
  const text = fillText(raw, context).trim();
  if (!text) return;
  // The box where the text goes, wherever the map put it; the text itself stays upright.
  const [[x1, y1], [x2, y2]] = extentOf(f.extent, [[-10, -10], [10, 10]]);
  const corners = [apply(m, [x1, y1]), apply(m, [x2, y1]), apply(m, [x2, y2]), apply(m, [x1, y2])];
  const xs = corners.map((p) => p[0]);
  const ys = corners.map((p) => p[1]);
  const [left, right, bottom, top] = [Math.min(...xs), Math.max(...xs), Math.min(...ys), Math.max(...ys)];
  const width = right - left;
  const height = top - bottom;
  if (width <= 0 || height <= 0) return;
  const lines = text.split('\n');
  const longest = Math.max(...lines.map((l) => l.length), 1);
  const scale = Math.hypot(m[0], m[1]);
  const asked = num(f.fontSize, 0) * scale * 0.35;
  const fitted = Math.min(height / lines.length / 1.15, width / (longest * 0.58));
  const size = asked > 0 ? Math.min(asked, fitted * 1.4) : fitted;
  if (size < 0.5) return;
  const alignment = enumName(f.horizontalAlignment) ?? 'Center';
  const [anchor, x] = alignment === 'Left' ? ['start', left] : alignment === 'Right' ? ['end', right] : ['middle', (left + right) / 2];
  const styles = Array.isArray(f.textStyle) ? f.textStyle.map(enumName) : [];
  const fill = color(f.textColor, color(f.lineColor, [0, 0, 0]));
  [[left, bottom], [right, top]].forEach((p) => canvas.cover(p as Point));
  const firstBaseline = (top + bottom) / 2 + ((lines.length - 1) * size * 1.15) / 2 - size * 0.35;
  lines.forEach((line, i) => {
    const y = firstBaseline - i * size * 1.15;
    canvas.parts.push(
      `<text x="${round(x)}" y="${round(-y)}" font-size="${round(size)}" text-anchor="${anchor}" fill="${rgb(fill)}"${styles.includes('Bold') ? ' font-weight="bold"' : ''}${styles.includes('Italic') ? ' font-style="italic"' : ''}>${escapeXml(line)}</text>`,
    );
  });
}

function drawBitmap(canvas: Canvas, m: Matrix, f: Obj): void {
  if (typeof f.imageSource !== 'string' || !f.imageSource) return;
  const [[x1, y1], [x2, y2]] = extentOf(f.extent, [[-10, -10], [10, 10]]);
  const a = apply(m, [Math.min(x1, x2), Math.max(y1, y2)]);
  const b = apply(m, [Math.max(x1, x2), Math.min(y1, y2)]);
  canvas.cover(a);
  canvas.cover(b);
  const source = f.imageSource.replace(/[^A-Za-z0-9+/=]/g, '');
  canvas.parts.push(`<image x="${round(Math.min(a[0], b[0]))}" y="${round(-Math.max(a[1], b[1]))}" width="${round(Math.abs(b[0] - a[0]))}" height="${round(Math.abs(b[1] - a[1]))}" preserveAspectRatio="none" href="data:image/png;base64,${source}"/>`);
}

/** One graphic of a layer (Icon or Diagram), drawn through the map. */
export function drawGraphic(canvas: Canvas, m: Matrix, graphic: Json, context: TextContext): void {
  if (!isObj(graphic)) return;
  const kind = typeof graphic.name === 'string' ? graphic.name : '';
  const f = recordFields(kind, graphic);
  if (f.visible === false) return;
  const local = itemMatrix(m, point(f.origin), num(f.rotation, 0));
  const map = (ps: Point[]) => ps.map((p) => apply(local, p));
  switch (kind) {
    case 'Line': {
      const style = styleOf(f, true);
      const raw = points(f.points);
      const ps = map(enumName(f.smooth) === 'Bezier' ? smoothed(raw) : raw);
      canvas.path(ps, false, style);
      const arrows = Array.isArray(f.arrow) ? f.arrow.map(enumName) : [];
      const size = num(f.arrowSize, 3) * Math.hypot(local[0], local[1]);
      if (ps.length >= 2) {
        if (arrows[0] && arrows[0] !== 'None') arrowHead(canvas, ps[0], ps[1], size, arrows[0] === 'Filled', style);
        if (arrows[1] && arrows[1] !== 'None') arrowHead(canvas, ps[ps.length - 1], ps[ps.length - 2], size, arrows[1] === 'Filled', style);
      }
      return;
    }
    case 'Polygon': {
      const raw = points(f.points);
      canvas.path(map(enumName(f.smooth) === 'Bezier' ? smoothed([...raw, raw[0]]) : raw), true, styleOf(f, false));
      return;
    }
    case 'Rectangle':
      canvas.path(map(rectanglePoints(extentOf(f.extent), num(f.radius, 0))), true, styleOf(f, false));
      return;
    case 'Ellipse': {
      const start = num(f.startAngle, 0);
      const end = num(f.endAngle, 360);
      const full = Math.abs(end - start) >= 360;
      const closure = enumName(f.closure) ?? (full ? 'Chord' : 'Radial');
      const arc = ellipsePoints(extentOf(f.extent), start, end);
      const [[x1, y1], [x2, y2]] = extentOf(f.extent);
      const ps = !full && closure === 'Radial' ? [[(x1 + x2) / 2, (y1 + y2) / 2] as Point, ...arc] : arc;
      canvas.path(map(ps), full || closure !== 'None', styleOf(f, false));
      return;
    }
    case 'Text':
      drawText(canvas, local, f, context);
      return;
    case 'Bitmap':
      drawBitmap(canvas, local, f);
      return;
  }
}

// ---- Classes, their layers, their components --------------------------------

/** The classes a class extends, nearest first, as their instances. */
function bases(cls: Obj): Obj[] {
  const out: Obj[] = [];
  for (const element of Array.isArray(cls.elements) ? cls.elements : []) {
    if (isObj(element) && element.$kind === 'extends' && isObj(element.baseClass)) out.push(element.baseClass);
  }
  return out;
}

/** A layer (Icon or Diagram) of a class: its coordinate system and its graphics, the bases' first. */
export function layerOf(cls: Obj, layer: 'Icon' | 'Diagram', depth = 0): { extent: [Point, Point] | null; graphics: Json[] } {
  const own = isObj(cls.annotation) && isObj(cls.annotation[layer]) ? (cls.annotation[layer] as Obj) : null;
  let extent: [Point, Point] | null = own && isObj(own.coordinateSystem) && Array.isArray(own.coordinateSystem.extent) ? extentOf(own.coordinateSystem.extent) : null;
  const graphics: Json[] = [];
  if (depth < 8) {
    for (const base of bases(cls)) {
      const inherited = layerOf(base, layer, depth + 1);
      graphics.push(...inherited.graphics);
      extent ??= inherited.extent;
    }
  }
  if (own && Array.isArray(own.graphics)) graphics.push(...own.graphics);
  return { extent, graphics };
}

export interface Component {
  name: string;
  className: string;
  comment: string;
  type: Obj | null;
  placement: { origin: Point; extent: [Point, Point]; rotation: number; visible: boolean } | null;
  iconPlacement: { origin: Point; extent: [Point, Point]; rotation: number; visible: boolean } | null;
  connector: boolean;
  parameters: Record<string, string>;
}

function transformationOf(v: Json): Component['placement'] {
  if (!isObj(v)) return null;
  return { origin: point(v.origin), extent: extentOf(v.extent, [[-10, -10], [10, 10]]), rotation: num(v.rotation, 0), visible: true };
}

/** A component's annotation: its own, or the one written after `constrainedby` (where a replaceable part's placement goes). */
function annotationOf(element: Obj): Obj {
  if (isObj(element.annotation)) return element.annotation;
  const replaceable = isObj(element.prefixes) && isObj(element.prefixes.replaceable) ? element.prefixes.replaceable : null;
  return replaceable && isObj(replaceable.annotation) ? replaceable.annotation : {};
}

/** The literal values of a component's modifiers (for %parameter in its icon's texts). */
function parametersOf(element: Obj): Record<string, string> {
  const out: Record<string, string> = {};
  if (isObj(element.modifiers)) {
    for (const [key, value] of Object.entries(element.modifiers)) {
      if (typeof value === 'string' || typeof value === 'number') out[key] = String(value);
      else if (isObj(value) && (typeof value.$value === 'string' || typeof value.$value === 'number')) out[key] = String(value.$value);
    }
  }
  return out;
}

/** The components of a class, the inherited ones first. */
export function componentsOf(cls: Obj, depth = 0): Component[] {
  const out: Component[] = [];
  if (depth < 8) for (const base of bases(cls)) out.push(...componentsOf(base, depth + 1));
  for (const element of Array.isArray(cls.elements) ? cls.elements : []) {
    if (!isObj(element) || element.$kind !== 'component' || typeof element.name !== 'string') continue;
    const type = isObj(element.type) ? element.type : null;
    const className = type && typeof type.name === 'string' ? type.name : typeof element.type === 'string' ? element.type : '';
    const annotation = annotationOf(element);
    const placementAnnotation = isObj(annotation.Placement) ? annotation.Placement : null;
    const visible = placementAnnotation?.visible !== false;
    const placement = placementAnnotation ? transformationOf(placementAnnotation.transformation) : null;
    const iconPlacement = placementAnnotation ? transformationOf(placementAnnotation.iconTransformation) ?? placement : null;
    const restriction = type && typeof type.restriction === 'string' ? type.restriction : '';
    const component: Component = {
      name: element.name,
      className,
      comment: typeof element.comment === 'string' ? element.comment : '',
      type,
      placement: placement && { ...placement, visible },
      iconPlacement: iconPlacement && { ...iconPlacement, visible },
      connector: restriction.includes('connector'),
      parameters: parametersOf(element),
    };
    // A redeclared part replaces the inherited one of the same name, keeping its place when it has none.
    const inherited = out.findIndex((c) => c.name === component.name);
    if (inherited >= 0) {
      component.placement ??= out[inherited].placement;
      component.iconPlacement ??= out[inherited].iconPlacement;
      out.splice(inherited, 1);
    }
    out.push(component);
  }
  return out;
}

/** Draws a class's icon through `m`: its graphics, then its connectors with their own icons. */
function drawIcon(canvas: Canvas, cls: Obj, m: Matrix, context: TextContext, depth: number): void {
  const icon = layerOf(cls, 'Icon');
  for (const graphic of icon.graphics) drawGraphic(canvas, m, graphic, context);
  if (depth >= MAX_DEPTH) return;
  for (const connector of componentsOf(cls).filter((c) => c.connector && c.iconPlacement?.visible && c.type)) {
    const place = connector.iconPlacement;
    if (!place || !connector.type) continue;
    const inner = layerOf(connector.type, 'Icon');
    const map = multiply(m, placementMatrix(inner.extent ?? DEFAULT_EXTENT, place));
    drawIcon(canvas, connector.type, map, { name: connector.name, className: connector.className, parameters: connector.parameters }, depth + 1);
  }
}

interface Connection {
  from: string;
  to: string;
  line: Obj | null;
}

const crefText = (v: Json): string => (isObj(v) && Array.isArray(v.parts) ? v.parts.map((p) => (isObj(p) && typeof p.name === 'string' ? p.name : '?')).join('.') : '');

function connectionsOf(cls: Obj, depth = 0): Connection[] {
  const out: Connection[] = [];
  if (depth < 8) for (const base of bases(cls)) out.push(...connectionsOf(base, depth + 1));
  for (const c of Array.isArray(cls.connections) ? cls.connections : []) {
    if (!isObj(c)) continue;
    const annotation = isObj(c.annotation) ? c.annotation : {};
    out.push({ from: crefText(c.lhs), to: crefText(c.rhs), line: isObj(annotation.Line) ? annotation.Line : null });
  }
  return out;
}

export interface DrawnComponent {
  name: string;
  className: string;
  comment: string;
  /** Its box on the drawing, in SVG coordinates (y down). */
  box: { x: number; y: number; width: number; height: number };
  /** A class of its own worth opening (not a connector, not a basic type). */
  opens: boolean;
}

export interface Drawing {
  svg: string;
  viewBox: { x: number; y: number; width: number; height: number };
  components: DrawnComponent[];
  connections: number;
  /** Components without a placement: not drawn, listed. */
  unplaced: string[];
}

/** The diagram of the class an instance describes: what the view draws. */
export function drawDiagram(instance: Json): Drawing {
  const cls = isObj(instance) ? instance : {};
  const canvas = new Canvas();
  const diagram = layerOf(cls, 'Diagram');
  const own: TextContext = { name: '', className: typeof cls.name === 'string' ? cls.name : '', parameters: {} };
  for (const graphic of diagram.graphics) drawGraphic(canvas, IDENTITY, graphic, own);

  const components: DrawnComponent[] = [];
  const unplaced: string[] = [];
  for (const component of componentsOf(cls)) {
    const place = component.placement;
    if (!component.type) continue;
    if (!place) {
      // Parameters and variables have no place: only the parts that should have one are told.
      if (component.type.restriction !== 'type' && !/^(Real|Integer|Boolean|String)$/.test(component.className)) unplaced.push(component.name);
      continue;
    }
    if (!place.visible) continue;
    const iconExtent = layerOf(component.type, 'Icon').extent ?? DEFAULT_EXTENT;
    const m = placementMatrix(iconExtent, place);
    const own = new Canvas();
    drawIcon(own, component.type, m, { name: component.name, className: component.className.split('.').pop() ?? '', parameters: component.parameters }, 0);
    if (own.parts.length === 0) {
      // An icon with no graphics: a plain box with the part's name, so the part is still there.
      own.path(rectanglePoints(iconExtent, 0).map((p) => apply(m, p)), true, { stroke: [90, 90, 90], fill: [245, 245, 245], dash: '', width: 1 });
      drawText(own, m, { extent: iconExtent, textString: component.name }, { name: component.name, className: '', parameters: {} });
    }
    canvas.parts.push(`<g class="component" data-name="${escapeXml(component.name)}">`, ...own.parts, '</g>');
    const corners = [apply(m, iconExtent[0]), apply(m, iconExtent[1])];
    [...corners, [own.minX, own.minY] as Point, [own.maxX, own.maxY] as Point].filter((p) => Number.isFinite(p[0])).forEach((p) => canvas.cover(p));
    const xs = corners.map((p) => p[0]);
    const ys = corners.map((p) => -p[1]);
    components.push({
      name: component.name,
      className: component.className,
      comment: component.comment,
      box: { x: Math.min(...xs), y: Math.min(...ys), width: Math.max(...xs) - Math.min(...xs), height: Math.max(...ys) - Math.min(...ys) },
      opens: !component.connector && component.type.restriction !== 'type' && component.className.includes('.'),
    });
  }

  const connections = connectionsOf(cls);
  for (const connection of connections) {
    if (!connection.line) continue;
    const f = recordFields('Line', connection.line);
    if (f.visible === false) continue;
    drawGraphic(canvas, IDENTITY, { name: 'Line', ...f }, own);
  }

  const empty = !Number.isFinite(canvas.minX);
  const extent = diagram.extent ?? DEFAULT_EXTENT;
  const [minX, maxX, minY, maxY] = empty ? [extent[0][0], extent[1][0], extent[0][1], extent[1][1]] : [canvas.minX, canvas.maxX, canvas.minY, canvas.maxY];
  const margin = Math.max(4, (maxX - minX) * 0.03, (maxY - minY) * 0.03);
  const viewBox = { x: round(minX - margin), y: round(-maxY - margin), width: round(maxX - minX + 2 * margin), height: round(maxY - minY + 2 * margin) };
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="${viewBox.x} ${viewBox.y} ${viewBox.width} ${viewBox.height}" font-family="Arial, Helvetica, sans-serif">${canvas.parts.join('')}</svg>`;
  return { svg, viewBox, components, connections: connections.length, unplaced };
}
