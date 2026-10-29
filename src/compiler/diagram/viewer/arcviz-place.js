/*
 * ArcViz place — manual placement on top of the automatic layout.
 *
 * Pure functions over a laid-out graph, no DOM. A box the reader moved goes
 * exactly where it was dropped; what it holds moves with it; only the
 * exchanges that reach it are redrawn, at right angles, straight from one
 * end to the other. They do not steer around other boxes: that is the
 * reader's arrangement to make. An exchange the reader bent passes through
 * the points they gave it, in sheet units, from one end to the other. The
 * input is never modified.
 */
(function (root, factory) {
  // The router is optional: without it, redrawn exchanges keep the plain right-angled route.
  const router = typeof module === 'object' && module.exports ? require('./arcviz-route.js') : () => root.ArcVizRoute || null;
  const api = factory(typeof router === 'function' ? router : () => router);
  if (typeof module === 'object' && module.exports) module.exports = api;
  else root.ArcVizPlace = api;
})(typeof self !== 'undefined' ? self : this, function (router) {
  'use strict';

  const MARGIN = 12; // free sheet around the drawing
  const STUB = 14; // how far an exchange leaves a box before turning
  const INSET = { top: 34, side: 18, bottom: 20 }; // room a container keeps around its content
  const FARTHEST = 100000; // sheet units: beyond this an offset is not a placement

  const sound = (value) => value === undefined || (Number.isFinite(value) && Math.abs(value) <= FARTHEST);
  const isOffset = (value) => Boolean(value) && Number.isFinite(value.dx) && Number.isFinite(value.dy) && Math.abs(value.dx) <= FARTHEST && Math.abs(value.dy) <= FARTHEST && sound(value.dw) && sound(value.dh);
  const MIN = { width: 48, height: 28 }; // the smallest a box may be made

  const isPoint = (point) => Boolean(point) && Number.isFinite(point.x) && Number.isFinite(point.y) && Math.abs(point.x) <= FARTHEST && Math.abs(point.y) <= FARTHEST;
  const isRoute = (points) => Array.isArray(points) && points.length > 0 && points.length <= 64 && points.every(isPoint);

  const NORMAL = { east: { x: 1, y: 0 }, west: { x: -1, y: 0 }, north: { x: 0, y: -1 }, south: { x: 0, y: 1 } };

  /** Index a graph: every node and port with its parent and absolute origin. */
  function survey(graph) {
    const nodes = new Map(); // id -> { node, parent, x, y }
    const ports = new Map(); // port id -> { port, owner }
    const walk = (node, parent, ox, oy) => {
      const entry = { node, parent, x: ox + node.x, y: oy + node.y };
      nodes.set(node.id, entry);
      for (const port of node.ports || []) ports.set(port.id, { port, owner: node.id });
      for (const child of node.children || []) walk(child, node.id, entry.x, entry.y);
    };
    for (const child of graph.children || []) walk(child, null, 0, 0);
    return { nodes, ports };
  }

  /** Where an exchange meets one of its ends, and which way it leaves. */
  function anchor(end, towards, map) {
    const port = map.ports.get(end);
    const box = map.nodes.get(port ? port.owner : end);
    if (!box) return null;
    const { x, y } = box;
    const { width, height } = box.node;
    if (port) {
      const px = port.port.x + port.port.width / 2;
      const py = port.port.y + port.port.height / 2;
      // The side a port sits on is read as the renderer reads it.
      const side = port.port.x <= 0 ? 'west' : port.port.x + port.port.width >= width ? 'east' : port.port.y <= 0 ? 'north' : 'south';
      return { side, x: side === 'west' ? x : side === 'east' ? x + width : x + px, y: side === 'north' ? y : side === 'south' ? y + height : y + py };
    }
    const cx = x + width / 2;
    const cy = y + height / 2;
    const side = towards.x >= x + width ? 'east' : towards.x <= x ? 'west' : towards.y < cy ? 'north' : 'south';
    return { side, x: side === 'west' ? x : side === 'east' ? x + width : cx, y: side === 'north' ? y : side === 'south' ? y + height : cy };
  }

  const STYLES = new Set(['straight', 'curved']); // orthogonal is the default, never kept
  const isShift = (value) => Boolean(value) && Number.isFinite(value.dx) && Number.isFinite(value.dy) && Math.abs(value.dx) <= FARTHEST && Math.abs(value.dy) <= FARTHEST;
  /** The entries of a map given by id that pass `ok`, as a Map. */
  const entries = (object, ok) => new Map(Object.keys(object && typeof object === 'object' ? object : {}).filter((id) => id !== '__proto__' && ok(object[id])).map((id) => [id, object[id]]));

  /** The side of its box a port sits on, as the renderer reads it. */
  const sideOf = (port, width, height) => (port.x <= 0 ? 'west' : port.x + port.width >= width ? 'east' : port.y <= 0 ? 'north' : 'south');
  /** How far a port sticks out of its box. */
  function protrusion(port, width, height) {
    const side = sideOf(port, width, height);
    const out = side === 'west' ? -port.x : side === 'east' ? port.x + port.width - width : side === 'north' ? -port.y : port.y + port.height - height;
    return Math.max(0, out);
  }
  /**
   * The nearest place on the border of a box (width × height) to a port
   * centred at (cx, cy): its side and how far along it from the top or the
   * left, in sheet units (so a resize from the bottom or the right leaves
   * it where it is, as it leaves the other ports).
   */
  function onBorder(cx, cy, width, height) {
    const x = Math.min(Math.max(cx, 0), width);
    const y = Math.min(Math.max(cy, 0), height);
    const distances = [['west', x], ['east', width - x], ['north', y], ['south', height - y]];
    const [side] = distances.reduce((a, b) => (b[1] < a[1] ? b : a));
    return { side, along: side === 'west' || side === 'east' ? y : x };
  }
  /** Put a port on a side of a box, `along` from its top or left, sticking out by `out`. */
  function seat(port, width, height, { side, along, out }) {
    const keep = 2; // away from the corners, so the side reads unambiguously
    if (side === 'west' || side === 'east') {
      const cy = Math.min(Math.max(along, port.height / 2 + keep), Math.max(port.height / 2 + keep, height - port.height / 2 - keep));
      port.y = cy - port.height / 2;
      port.x = side === 'west' ? -out : width - port.width + out;
    } else {
      const cx = Math.min(Math.max(along, port.width / 2 + keep), Math.max(port.width / 2 + keep, width - port.width / 2 - keep));
      port.x = cx - port.width / 2;
      port.y = side === 'north' ? -out : height - port.height + out;
    }
  }

  /** Where a straight line from the centre of a box towards a point leaves the box. */
  function floating(box, towards) {
    const cx = box.x + box.width / 2;
    const cy = box.y + box.height / 2;
    const dx = towards.x - cx;
    const dy = towards.y - cy;
    if (!dx && !dy) return { x: cx, y: box.y, side: 'north' };
    const sx = dx ? box.width / 2 / Math.abs(dx) : Infinity;
    const sy = dy ? box.height / 2 / Math.abs(dy) : Infinity;
    const s = Math.min(sx, sy);
    const side = sx < sy ? (dx > 0 ? 'east' : 'west') : dy > 0 ? 'south' : 'north';
    return { x: cx + dx * s, y: cy + dy * s, side };
  }

  /** Smooth a corner of a straight end: the middle of the overlap of two ranges, else null. */
  const overlapMiddle = (lo1, hi1, lo2, hi2, inset) => {
    const lo = Math.max(lo1, lo2) + inset;
    const hi = Math.min(hi1, hi2) - inset;
    return hi >= lo ? (lo + hi) / 2 : null;
  };

  /** Right-angled path from one anchor to another. */
  function route(from, to) {
    const a = { x: from.x + NORMAL[from.side].x * STUB, y: from.y + NORMAL[from.side].y * STUB };
    const b = { x: to.x + NORMAL[to.side].x * STUB, y: to.y + NORMAL[to.side].y * STUB };
    const horizontal = (side) => side === 'east' || side === 'west';
    let middle;
    if (horizontal(from.side) && horizontal(to.side)) {
      // Facing each other: one jog half way. Otherwise go around by the middle row.
      const facing = from.side !== to.side && (from.side === 'east' ? a.x <= b.x : a.x >= b.x);
      const mx = (a.x + b.x) / 2;
      const my = (a.y + b.y) / 2;
      middle = facing ? [{ x: mx, y: a.y }, { x: mx, y: b.y }] : [{ x: a.x, y: my }, { x: b.x, y: my }];
    } else if (!horizontal(from.side) && !horizontal(to.side)) {
      const my = (a.y + b.y) / 2;
      middle = [{ x: a.x, y: my }, { x: b.x, y: my }];
    } else {
      middle = horizontal(from.side) ? [{ x: b.x, y: a.y }] : [{ x: a.x, y: b.y }];
    }
    const points = [{ x: from.x, y: from.y }, a, ...middle, b, { x: to.x, y: to.y }];
    // Drop repeats and points in the middle of a straight run.
    return points.filter((point, index) => {
      const before = points[index - 1];
      const after = points[index + 1];
      if (!before || !after) return true;
      if (point.x === before.x && point.y === before.y) return false;
      return !((before.x === point.x && point.x === after.x) || (before.y === point.y && point.y === after.y));
    });
  }

  /** Drop repeats and points in the middle of a straight run. */
  function simplify(points) {
    return points.filter((point, index) => {
      const before = points[index - 1];
      const after = points[index + 1];
      if (!before || !after) return true;
      if (point.x === before.x && point.y === before.y) return false;
      return !((before.x === point.x && point.x === after.x) || (before.y === point.y && point.y === after.y));
    });
  }

  /**
   * Right-angled path from one anchor to another through the reader's
   * points: it leaves square to its side, turns at each point (each run
   * crosses the previous one), and comes in square to the other side.
   */
  function routeThrough(from, points, to) {
    const a = { x: from.x + NORMAL[from.side].x * STUB, y: from.y + NORMAL[from.side].y * STUB };
    const b = { x: to.x + NORMAL[to.side].x * STUB, y: to.y + NORMAL[to.side].y * STUB };
    const horizontal = (side) => side === 'east' || side === 'west';
    const path = [{ x: from.x, y: from.y }, a];
    let along = horizontal(from.side); // the way the last run went
    const reach = (point, last) => {
      const at = path[path.length - 1];
      if (at.x === point.x || at.y === point.y) {
        along = at.y === point.y;
      } else {
        // Cross the last run: after a horizontal run go vertical first, and the other way.
        // The final point is reached square to the side it comes in by.
        const verticalFirst = last ? horizontal(to.side) : along;
        path.push(verticalFirst ? { x: at.x, y: point.y } : { x: point.x, y: at.y });
        along = verticalFirst;
      }
      path.push({ x: point.x, y: point.y });
    };
    points.forEach((point) => reach(point, false));
    reach(b, true);
    path.push({ x: to.x, y: to.y });
    return simplify(path);
  }

  /**
   * The layout with the reader's placement applied.
   * @param {object} layout  a graph laid out by ELK
   * @param {Object<string, {dx: number, dy: number, dw?: number, dh?: number}>} offsets  per node id:
   *        the move, and how much wider and taller the box was made
   * @param {Object<string, Array<{x: number, y: number}>>} [routes]  per edge id: the points
   *        the reader bent it through, in sheet units
   */
  function place(layout, offsets, routes, extras) {
    const more = extras || {};
    const before = survey(layout);
    const wanted = Object.keys(offsets || {}).filter((id) => before.nodes.has(id) && isOffset(offsets[id]) && (offsets[id].dx || offsets[id].dy || offsets[id].dw || offsets[id].dh));
    const bent = new Map(Object.keys(routes || {}).filter((id) => id !== '__proto__' && isRoute(routes[id])).map((id) => [id, routes[id].map(({ x, y }) => ({ x, y }))]));
    const styles = entries(more.styles, (style) => STYLES.has(style));
    // A note is tied to what it is about by a straight line, unless the reader styled it.
    const tieNotes = (holder) => {
      for (const edge of holder.edges || []) if (edge.arc && edge.arc.kind === 'annotation' && !styles.has(edge.id)) styles.set(edge.id, 'straight');
      (holder.children || []).forEach(tieNotes);
    };
    tieNotes(layout);
    const labelShifts = entries(more.labels, isShift);
    const portShifts = entries(more.ports, isShift);
    const order = entries(more.order, (z) => Number.isFinite(z) && Math.abs(z) <= FARTHEST);
    for (const id of [...portShifts.keys()]) if (!before.ports.has(id)) portShifts.delete(id);
    const avoid = Boolean(more.avoid) && Boolean(router());
    if (wanted.length === 0 && bent.size === 0 && styles.size === 0 && labelShifts.size === 0 && portShifts.size === 0 && order.size === 0) return layout;

    const graph = JSON.parse(JSON.stringify(layout));
    // Brought to the front or sent to the back: among its siblings, drawn later or earlier.
    if (order.size > 0) {
      const restack = (holder) => {
        if (!holder.children) return;
        const rank = (node) => (order.has(node.id) ? order.get(node.id) : 0);
        holder.children = holder.children.map((node, index) => ({ node, index })).sort((a, b) => rank(a.node) - rank(b.node) || a.index - b.index).map(({ node }) => node);
        holder.children.forEach(restack);
      };
      restack(graph);
    }
    const start = survey(graph);
    const moved = new Set();
    const placed = {}; // where each box really went, once held inside its container
    // Ports dragged along the border: their side and how far along, kept through a resize.
    const seated = new Map();
    for (const [id, shift] of portShifts) {
      const { port, owner } = start.ports.get(id);
      const { node } = start.nodes.get(owner);
      const out = protrusion(port, node.width, node.height);
      const spot = onBorder(port.x + port.width / 2 + shift.dx, port.y + port.height / 2 + shift.dy, node.width, node.height);
      seated.set(id, { ...spot, out });
      seat(port, node.width, node.height, seated.get(id));
    }
    // Sizes first, inner boxes before the boxes that hold them, so a
    // container never shrinks over what it holds.
    const depth = (id) => {
      let d = 0;
      for (let at = start.nodes.get(id).parent; at; at = start.nodes.get(at).parent) d += 1;
      return d;
    };
    for (const id of [...wanted].sort((a, b) => depth(b) - depth(a))) {
      const { dw = 0, dh = 0 } = offsets[id];
      if (!dw && !dh) continue;
      const { node } = start.nodes.get(id);
      const content = (node.children || []).reduce((room, child) => ({ width: Math.max(room.width, child.x + child.width + INSET.side), height: Math.max(room.height, child.y + child.height + INSET.bottom) }), { width: 0, height: 0 });
      const width = Math.max(MIN.width, content.width, node.width + dw);
      const height = Math.max(MIN.height, content.height, node.height + dh);
      // Ports on the right and bottom sides stay on their side.
      for (const port of node.ports || []) {
        if (seated.has(port.id)) {
          seat(port, width, height, seated.get(port.id));
          continue;
        }
        if (port.x + port.width >= node.width - 0.5) port.x += width - node.width;
        if (port.y + port.height >= node.height - 0.5) port.y += height - node.height;
      }
      if (width !== node.width || height !== node.height) {
        moved.add(id);
        placed[id] = { dx: 0, dy: 0, dw: width - before.nodes.get(id).node.width, dh: height - before.nodes.get(id).node.height };
      }
      node.width = width;
      node.height = height;
    }
    for (const id of wanted) {
      const { node, parent } = start.nodes.get(id);
      const holder = parent ? start.nodes.get(parent).node : null;
      let x = node.x + offsets[id].dx;
      let y = node.y + offsets[id].dy;
      if (holder) {
        // A box stays inside the container that holds it.
        x = Math.min(Math.max(x, INSET.side), Math.max(INSET.side, holder.width - INSET.side - node.width));
        y = Math.min(Math.max(y, INSET.top), Math.max(INSET.top, holder.height - INSET.bottom - node.height));
      }
      if (x !== node.x || y !== node.y) {
        moved.add(id);
        placed[id] = { ...(placed[id] || {}), dx: x - node.x, dy: y - node.y };
      }
      node.x = x;
      node.y = y;
    }

    const map = survey(graph);
    // An end has moved, as its container sees it, when a moved box is the
    // end itself or holds it, below that container, or when it is a port
    // dragged along its border.
    const displaced = (end, container) => {
      if (seated.has(end)) return true;
      const port = map.ports.get(end);
      for (let id = port ? port.owner : end; id && id !== container; id = map.nodes.get(id) && map.nodes.get(id).parent) {
        if (moved.has(id)) return true;
      }
      return false;
    };
    const reach = { x: 0, y: 0 }; // how far redrawn exchanges extend
    const low = { x: Infinity, y: Infinity };
    const stretch = (point) => {
      reach.x = Math.max(reach.x, point.x);
      reach.y = Math.max(reach.y, point.y);
      low.x = Math.min(low.x, point.x);
      low.y = Math.min(low.y, point.y);
    };
    const owner = (end) => (map.ports.has(end) ? map.ports.get(end).owner : end);
    const rectOf = (id) => {
      const box = map.nodes.get(id);
      return box ? { x: box.x, y: box.y, width: box.node.width, height: box.node.height } : null;
    };
    /** The boxes an exchange must go around: all but those that hold one of its ends. */
    const obstaclesOf = (ends) => {
      const holding = new Set();
      for (const end of ends) {
        for (let id = map.nodes.get(owner(end)) && map.nodes.get(owner(end)).parent; id; id = map.nodes.get(id).parent) holding.add(id);
      }
      const ends2 = ends.map(owner);
      if (holding.has(ends2[0]) || holding.has(ends2[1])) for (const id of ends2) holding.add(id);
      const rects = [];
      const walk = (node) => {
        if (holding.has(node.id)) (node.children || []).forEach(walk);
        else rects.push({ id: node.id, ...rectOf(node.id) });
      };
      (graph.children || []).forEach(walk);
      return rects;
    };
    /** Absolute points of an exchange as it is drawn now. */
    const drawnPoints = (edge) => {
      const base = edge.container && edge.container !== graph.id && map.nodes.has(edge.container) ? map.nodes.get(edge.container) : { x: 0, y: 0 };
      return (edge.sections || []).flatMap((section) => [section.startPoint, ...(section.bendPoints || []), section.endPoint]).map((p) => ({ x: p.x + base.x, y: p.y + base.y }));
    };
    /**
     * Where the exchange meets its two ends: a port where it has one; on a
     * box, the side facing the other end, level with it when the two face
     * each other (draw.io's floating end), recomputed whenever a box moves.
     */
    const anchors = (ends, firstPoint, lastPoint) => {
      const centre = (end) => {
        const r = rectOf(owner(end));
        return r ? { x: r.x + r.width / 2, y: r.y + r.height / 2 } : null;
      };
      const [sourceCentre, targetCentre] = ends.map(centre);
      if (!sourceCentre || !targetCentre) return null;
      const from = anchor(ends[0], firstPoint || targetCentre, map);
      const to = anchor(ends[1], lastPoint || sourceCentre, map);
      if (!from || !to || !avoid) return from && to ? [from, to] : null;
      // Two plain boxes facing each other: both ends level, so the line runs straight.
      const [a, b] = ends.map((end) => (map.ports.has(end) ? null : rectOf(end)));
      const level = (end, other, here, there) => {
        if (!here) return end;
        const horizontal = end.side === 'east' || end.side === 'west';
        const towards = there || { x: other.x, y: other.y, width: 0, height: 0 };
        const middle = horizontal ? overlapMiddle(here.y, here.y + here.height, towards.y, towards.y + towards.height, 6) : overlapMiddle(here.x, here.x + here.width, towards.x, towards.x + towards.width, 6);
        if (middle === null) return end;
        return horizontal ? { ...end, y: middle } : { ...end, x: middle };
      };
      if (!firstPoint) Object.assign(from, level(from, to, a, b));
      if (!lastPoint) Object.assign(to, level(to, from, b, a));
      return [from, to];
    };
    /** A route around the other boxes, or null when the router finds none. */
    const around = (ends, from, to, via) => {
      if (!avoid) return null;
      return router().route({ from, to, via, obstacles: obstaclesOf(ends), stub: STUB });
    };
    /** A straight line from end to end, through the reader's points. */
    const straight = (ends, via) => {
      const at = (end, towards) => {
        if (map.ports.has(end)) return anchor(end, towards, map);
        const r = rectOf(end);
        return r ? floating(r, towards) : null;
      };
      const centre = (end) => {
        const r = rectOf(owner(end));
        return r ? { x: r.x + r.width / 2, y: r.y + r.height / 2 } : null;
      };
      const from = at(ends[0], via[0] || centre(ends[1]));
      const to = at(ends[1], via[via.length - 1] || centre(ends[0]));
      return from && to ? [{ x: from.x, y: from.y }, ...via, { x: to.x, y: to.y }] : null;
    };
    // Boxes that moved, for the exchanges they now lie across.
    const movedRects = [...moved].map((id) => ({ id, ...rectOf(id) }));
    const redrawn = [];
    const redraw = (holder) => {
      for (const edge of holder.edges || []) {
        const ends = [edge.sources[0], edge.targets[0]];
        const style = styles.get(edge.id);
        if (style) edge.lineStyle = style;
        if (style === 'straight' && owner(ends[0]) !== owner(ends[1])) {
          const points = straight(ends, bent.get(edge.id) || []);
          if (points) {
            points.forEach(stretch);
            redrawn.push({ edge, points });
            continue;
          }
        }
        if (bent.has(edge.id)) {
          // Bent by the reader: from its source, through the points, to its target.
          const points = bent.get(edge.id);
          const ends2 = anchors(ends, points[0], points[points.length - 1]);
          if (ends2) {
            const [from, to] = ends2;
            const all = around(ends, from, to, points) || routeThrough(from, points, to);
            all.forEach(stretch);
            redrawn.push({ edge, points: all });
            continue;
          }
        }
        const moving = ends.some((end) => displaced(end, edge.container));
        // With routing around boxes, an exchange a moved box now lies across is redrawn too.
        const crossed = !moving && avoid && movedRects.length > 0 && owner(ends[0]) !== owner(ends[1]) && (() => {
          const points = drawnPoints(edge);
          const others = new Set(obstaclesOf(ends).map((r) => r.id));
          return movedRects.some((r) => others.has(r.id) && router().crosses(points, r));
        })();
        if (!moving && !crossed) continue;
        if (owner(ends[0]) === owner(ends[1])) {
          // A loop keeps its shape: it follows the box it loops on.
          const now = map.nodes.get(owner(ends[0]));
          const was = before.nodes.get(owner(ends[0]));
          const base = edge.container && edge.container !== graph.id && map.nodes.has(edge.container) ? [map.nodes.get(edge.container), before.nodes.get(edge.container)] : [{ x: 0, y: 0 }, { x: 0, y: 0 }];
          const by = { x: now.x - base[0].x - (was.x - base[1].x), y: now.y - base[0].y - (was.y - base[1].y) };
          for (const section of edge.sections || []) {
            for (const point of [section.startPoint, ...(section.bendPoints || []), section.endPoint]) {
              point.x += by.x;
              point.y += by.y;
              stretch({ x: point.x + base[0].x, y: point.y + base[0].y });
            }
          }
          for (const label of edge.labels || []) {
            if (label.x === undefined) continue;
            label.x += by.x;
            label.y += by.y;
          }
          continue;
        }
        const ends2 = anchors(ends);
        if (!ends2) continue;
        const [from, to] = ends2;
        const points = around(ends, from, to, []) || route(from, to);
        points.forEach(stretch);
        redrawn.push({ edge, points });
      }
      for (const child of holder.children || []) redraw(child);
    };
    redraw(graph);

    // Keep everything on the sheet: shift the whole drawing when a box or a
    // redrawn exchange went past the top or the left.
    for (const child of graph.children || []) stretch({ x: child.x, y: child.y });
    const shift = { x: Math.max(0, MARGIN - low.x), y: Math.max(0, MARGIN - low.y) };
    if (shift.x || shift.y) {
      for (const child of graph.children || []) {
        child.x += shift.x;
        child.y += shift.y;
      }
      // Exchanges held by the root are in sheet coordinates: they follow.
      for (const edge of graph.edges || []) {
        if (redrawn.some((entry) => entry.edge === edge)) continue;
        for (const section of edge.sections || []) {
          for (const point of [section.startPoint, ...(section.bendPoints || []), section.endPoint]) {
            point.x += shift.x;
            point.y += shift.y;
          }
        }
        for (const label of edge.labels || []) {
          if (label.x === undefined) continue;
          label.x += shift.x;
          label.y += shift.y;
        }
      }
    }
    const shifted = survey(graph);
    const origin = (id) => (id && id !== graph.id && shifted.nodes.has(id) ? shifted.nodes.get(id) : { x: 0, y: 0 });
    for (const { edge, points } of redrawn) {
      const base = origin(edge.container);
      const local = points.map((point) => ({ x: point.x + shift.x - base.x, y: point.y + shift.y - base.y }));
      edge.sections = [{ id: `${edge.id}_placed`, startPoint: local[0], endPoint: local[local.length - 1], bendPoints: local.slice(1, -1) }];
      // The name sits on the longest run of the new route.
      let longest = 0;
      let at = local[0];
      for (let i = 1; i < local.length; i += 1) {
        const length = Math.hypot(local[i].x - local[i - 1].x, local[i].y - local[i - 1].y);
        if (length > longest) {
          longest = length;
          at = { x: (local[i].x + local[i - 1].x) / 2, y: (local[i].y + local[i - 1].y) / 2 };
        }
      }
      for (const label of edge.labels || []) {
        label.x = at.x - (label.width || 0) / 2;
        label.y = at.y + 3;
      }
    }
    // A name dragged off its line keeps that offset from where it would sit.
    if (labelShifts.size > 0) {
      const walk = (holder) => {
        for (const edge of holder.edges || []) {
          const by = labelShifts.get(edge.id);
          if (!by) continue;
          for (const label of edge.labels || []) {
            if (label.x === undefined) continue;
            label.x += by.dx;
            label.y += by.dy;
          }
        }
        (holder.children || []).forEach(walk);
      };
      walk(graph);
    }

    let width = reach.x + shift.x;
    let height = reach.y + shift.y;
    for (const child of graph.children || []) {
      width = Math.max(width, child.x + child.width);
      height = Math.max(height, child.y + child.height);
    }
    // Routes that were not redrawn still need their room.
    for (const edge of graph.edges || []) {
      for (const section of edge.sections || []) {
        for (const point of [section.startPoint, ...(section.bendPoints || []), section.endPoint]) {
          width = Math.max(width, point.x);
          height = Math.max(height, point.y);
        }
      }
      for (const label of labelShifts.has(edge.id) ? edge.labels || [] : []) {
        if (label.x === undefined) continue;
        width = Math.max(width, label.x + (label.width || 0));
        height = Math.max(height, label.y + (label.height || 0));
      }
    }
    graph.width = Math.max(layout.width || 0, width + MARGIN);
    graph.height = Math.max(layout.height || 0, height + MARGIN);
    graph.arc = { ...graph.arc, placed };
    return graph;
  }

  // --- what an editing viewer asks of the placement (arcviz-viewer.js) -------

  /**
   * A resize by a side handle: `dx`/`dy` move the left or top side, the
   * width or height changing the other way, never past what the box holds
   * (its children keep their place on the sheet).
   */
  function sideResize(node, dw, dh, dx = 0, dy = 0) {
    const children = node ? node.children || [] : [];
    if (!node || (!dx && !dy)) return { dw, dh, dx, dy, children };
    const roomX = Math.min(node.width - MIN.width, ...children.map((child) => child.x - INSET.side));
    const roomY = Math.min(node.height - MIN.height, ...children.map((child) => child.y - INSET.top));
    const x = dx ? Math.min(dx, roomX) : 0;
    const y = dy ? Math.min(dy, roomY) : 0;
    return { dx: x, dy: y, dw: dx ? -x : dw, dh: dy ? -y : dh, children };
  }

  /** The id of the box holding `id` in a laid-out shape, or null at the top. */
  function parentOf(shape, id) {
    let found = null;
    const walk = (node, parent) => {
      if (node.id === id) found = parent;
      (node.children || []).forEach((child) => walk(child, node.id));
    };
    (shape && shape.children ? shape.children : []).forEach((node) => walk(node, null));
    return found;
  }

  /** The kind of a drawn box, or of the box a drawn port sits on. */
  function endKind(end) {
    const node = end.closest('.av-node');
    return node ? node.dataset.kind : null;
  }

  /** What to call a drawn box or port (`Port of Box` for a port), boxes named by `nameOf`. */
  function endName(end, nameOf) {
    const owner = end.closest('.av-node');
    const name = owner ? nameOf(owner.dataset.id) : end.dataset.id;
    if (!end.classList.contains('av-port')) return name;
    const tip = end.querySelector('title');
    const port = tip ? tip.textContent.split(' — ')[0] : end.dataset.id;
    return `${port} of ${name}`;
  }

  /** Marks the lines of `edges` (an index) that pass through the port `id`. */
  function markPortLines(canvas, edges, id) {
    for (const edge of canvas.querySelectorAll('.av-edge')) {
      const ends = edges.get(edge.dataset.id);
      if (ends && [...(ends.sources || []), ...(ends.targets || [])].includes(id)) edge.classList.add('is-linked');
    }
  }

  /** A function telling `onLayout` the layout file whenever it changed (not the one the page was built with). */
  function layoutReporter(toFile, onLayout) {
    let last = null;
    return () => {
      if (!onLayout) return;
      const now = JSON.stringify(toFile());
      if (last === null || now === last) {
        last = now;
        return;
      }
      last = now;
      onLayout(JSON.parse(now));
    };
  }

  return { place, INSET, MIN, sideResize, parentOf, endKind, endName, markPortLines, layoutReporter };
});
