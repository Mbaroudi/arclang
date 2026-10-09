/*
 * ArcViz place — manual placement on top of the automatic layout.
 *
 * Pure functions over a laid-out graph, no DOM. A box the reader moved goes
 * exactly where it was dropped; what it holds moves with it; only the
 * exchanges that reach it are redrawn, at right angles, straight from one
 * end to the other. They do not steer around other boxes: that is the
 * reader's arrangement to make. The input is never modified.
 */
(function (root, factory) {
  const api = factory();
  if (typeof module === 'object' && module.exports) module.exports = api;
  else root.ArcVizPlace = api;
})(typeof self !== 'undefined' ? self : this, function () {
  'use strict';

  const MARGIN = 12; // free sheet around the drawing
  const STUB = 14; // how far an exchange leaves a box before turning
  const INSET = { top: 34, side: 18, bottom: 20 }; // room a container keeps around its content
  const FARTHEST = 100000; // sheet units: beyond this an offset is not a placement

  const isOffset = (value) => Boolean(value) && Number.isFinite(value.dx) && Number.isFinite(value.dy) && Math.abs(value.dx) <= FARTHEST && Math.abs(value.dy) <= FARTHEST;

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

  /**
   * The layout with the reader's placement applied.
   * @param {object} layout  a graph laid out by ELK
   * @param {Object<string, {dx: number, dy: number}>} offsets  per node id
   */
  function place(layout, offsets) {
    const before = survey(layout);
    const wanted = Object.keys(offsets || {}).filter((id) => before.nodes.has(id) && isOffset(offsets[id]) && (offsets[id].dx || offsets[id].dy));
    if (wanted.length === 0) return layout;

    const graph = JSON.parse(JSON.stringify(layout));
    const start = survey(graph);
    const moved = new Set();
    const placed = {}; // where each box really went, once held inside its container
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
        placed[id] = { dx: x - node.x, dy: y - node.y };
      }
      node.x = x;
      node.y = y;
    }

    const map = survey(graph);
    // An end has moved, as its container sees it, when a moved box is the
    // end itself or holds it, below that container.
    const displaced = (end, container) => {
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
    const redrawn = [];
    const redraw = (holder) => {
      for (const edge of holder.edges || []) {
        const ends = [edge.sources[0], edge.targets[0]];
        if (!ends.some((end) => displaced(end, edge.container))) continue;
        const owner = (end) => (map.ports.has(end) ? map.ports.get(end).owner : end);
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
        const centre = (end) => {
          const port = map.ports.get(end);
          const box = map.nodes.get(port ? port.owner : end);
          return box ? { x: box.x + box.node.width / 2, y: box.y + box.node.height / 2 } : null;
        };
        const [sourceCentre, targetCentre] = ends.map(centre);
        if (!sourceCentre || !targetCentre) continue;
        const from = anchor(ends[0], targetCentre, map);
        const to = anchor(ends[1], sourceCentre, map);
        const points = route(from, to);
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
        const length = Math.abs(local[i].x - local[i - 1].x) + Math.abs(local[i].y - local[i - 1].y);
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
    }
    graph.width = Math.max(layout.width || 0, width + MARGIN);
    graph.height = Math.max(layout.height || 0, height + MARGIN);
    graph.arc = { ...graph.arc, placed };
    return graph;
  }

  return { place };
});
