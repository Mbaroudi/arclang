/*
 * ArcViz tidy — how orderly a drawing is, and tidying it up without losing
 * how the reader arranged it.
 *
 * Pure functions, no DOM. `measure` counts what makes a drawing hard to
 * read: boxes that overlap, lines that run across a box, lines that cross
 * each other. `worse` tells whether a change made it harder. `seeded` turns
 * the drawing on screen into a graph ELK lays out *interactively*: the
 * layers, the order within them and the direction of cycles follow where
 * the boxes are now, so a box dragged to the left stays on the left; ELK
 * then aligns the boxes, spaces them and removes the overlaps. `placements`
 * gives the result as the moves the layout file keeps (offsets from the
 * automatic layout).
 */
(function (root, factory) {
  const api = factory();
  if (typeof module === 'object' && module.exports) module.exports = api;
  else root.ArcVizTidy = api;
})(typeof self !== 'undefined' ? self : this, function () {
  'use strict';

  const INSET = 2; // touching is not overlapping
  const MAX_EDGES = 400; // past this, crossings are not counted (the page never freezes)

  /** ELK's layered options that follow the positions given. */
  const INTERACTIVE = {
    'elk.layered.layering.strategy': 'INTERACTIVE',
    'elk.layered.crossingMinimization.strategy': 'INTERACTIVE',
    'elk.layered.cycleBreaking.strategy': 'INTERACTIVE',
    'elk.interactiveLayout': 'true',
  };

  /** Every box with its absolute rectangle, its parent and whether it holds others. */
  function boxes(graph) {
    const out = new Map();
    const walk = (holder, bx, by, parent) => {
      for (const node of holder.children || []) {
        const x = bx + (node.x || 0);
        const y = by + (node.y || 0);
        out.set(node.id, { id: node.id, x, y, width: node.width || 0, height: node.height || 0, parent, container: Boolean(node.children && node.children.length) });
        for (const port of node.ports || []) out.set(port.id, { id: port.id, port: true, owner: node.id, x: x + (port.x || 0), y: y + (port.y || 0), width: port.width || 0, height: port.height || 0 });
        walk(node, x, y, node.id);
      }
    };
    walk(graph, 0, 0, null);
    return out;
  }

  /** The box an end is drawn on: the box itself, or the box a port sits on. */
  function ownerOf(end, map) {
    const at = map.get(end);
    return at && at.port ? at.owner : end;
  }

  function ancestors(id, map) {
    const out = new Set();
    for (let at = map.get(id); at && at.parent; at = map.get(at.parent)) out.add(at.parent);
    return out;
  }

  /** The absolute points of every line, with its two end boxes. */
  function lines(graph, map) {
    const out = [];
    const walk = (holder) => {
      for (const edge of holder.edges || []) {
        const base = edge.container && edge.container !== graph.id && map.has(edge.container) ? map.get(edge.container) : { x: 0, y: 0 };
        const points = (edge.sections || []).flatMap((s) => [s.startPoint, ...(s.bendPoints || []), s.endPoint]).filter(Boolean).map((p) => ({ x: p.x + base.x, y: p.y + base.y }));
        if (points.length >= 2) out.push({ id: edge.id, points, ends: [ownerOf((edge.sources || [])[0], map), ownerOf((edge.targets || [])[0], map)] });
      }
      for (const child of holder.children || []) walk(child);
    };
    walk(graph);
    return out;
  }

  function overlap(a, b) {
    return a.x < b.x + b.width - INSET && b.x < a.x + a.width - INSET && a.y < b.y + b.height - INSET && b.y < a.y + a.height - INSET;
  }

  /** Whether the segment p-q runs through the inside of r. */
  function segmentCrosses(p, q, r) {
    const x0 = r.x + INSET, x1 = r.x + r.width - INSET, y0 = r.y + INSET, y1 = r.y + r.height - INSET;
    if (x1 <= x0 || y1 <= y0) return false;
    // Liang-Barsky clipping of the segment against the inner rectangle.
    let t0 = 0, t1 = 1;
    const dx = q.x - p.x, dy = q.y - p.y;
    for (const [pp, qq] of [[-dx, p.x - x0], [dx, x1 - p.x], [-dy, p.y - y0], [dy, y1 - p.y]]) {
      if (pp === 0) {
        if (qq < 0) return false;
      } else {
        const t = qq / pp;
        if (pp < 0) t0 = Math.max(t0, t);
        else t1 = Math.min(t1, t);
        if (t0 > t1) return false;
      }
    }
    return t1 - t0 > 1e-9;
  }

  /** Whether two segments cross at a point inside both (a shared end or a touch is not a crossing). */
  function segmentsCross(a, b, c, d) {
    const cross = (o, p, q) => (p.x - o.x) * (q.y - o.y) - (p.y - o.y) * (q.x - o.x);
    const d1 = cross(c, d, a), d2 = cross(c, d, b), d3 = cross(a, b, c), d4 = cross(a, b, d);
    return ((d1 > 0 && d2 < 0) || (d1 < 0 && d2 > 0)) && ((d3 > 0 && d4 < 0) || (d3 < 0 && d4 > 0));
  }

  /**
   * How orderly a drawing is: { overlaps, through, crossings } — boxes
   * overlapping a sibling, lines running across a box that is neither one
   * of their ends nor around them, and crossings between two lines.
   */
  function measure(graph) {
    const map = boxes(graph);
    const nodes = [...map.values()].filter((b) => !b.port);
    let overlaps = 0;
    for (let i = 0; i < nodes.length; i += 1) {
      for (let j = i + 1; j < nodes.length; j += 1) {
        if (nodes[i].parent === nodes[j].parent && overlap(nodes[i], nodes[j])) overlaps += 1;
      }
    }
    const drawn = lines(graph, map);
    let through = 0;
    for (const line of drawn) {
      const spared = new Set(line.ends);
      for (const end of line.ends) for (const up of ancestors(end, map)) spared.add(up);
      const crossed = nodes.some((box) => !spared.has(box.id) && line.points.some((p, k) => k > 0 && segmentCrosses(line.points[k - 1], p, box)));
      if (crossed) through += 1;
    }
    let crossings = 0;
    if (drawn.length <= MAX_EDGES) {
      for (let i = 0; i < drawn.length; i += 1) {
        for (let j = i + 1; j < drawn.length; j += 1) {
          const a = drawn[i].points, b = drawn[j].points;
          for (let k = 1; k < a.length; k += 1) for (let m = 1; m < b.length; m += 1) if (segmentsCross(a[k - 1], a[k], b[m - 1], b[m])) crossings += 1;
        }
      }
    }
    return { overlaps, through, crossings };
  }

  /** Whether a change left the drawing harder to read than it was. */
  function worse(before, after) {
    if (!before || !after) return false;
    return after.overlaps > before.overlaps || after.through > before.through || after.crossings > before.crossings;
  }

  /** `2 boxes overlap, 3 lines cross` — what a reader is told. */
  function describe(m) {
    const parts = [];
    if (m.overlaps) parts.push(`${m.overlaps} ${m.overlaps > 1 ? 'boxes overlap' : 'box overlaps another'}`);
    if (m.through) parts.push(`${m.through} ${m.through > 1 ? 'lines run' : 'line runs'} across a box`);
    if (m.crossings) parts.push(`${m.crossings} ${m.crossings > 1 ? 'crossings' : 'crossing'} between lines`);
    return parts.join(', ');
  }

  /**
   * The graph to lay out again: the shape of the view, every box where it is
   * drawn now (and as large), ELK told to follow those positions.
   */
  function seeded(shape, arranged) {
    const at = new Map();
    const index = (holder) => {
      for (const node of holder.children || []) {
        at.set(node.id, node);
        index(node);
      }
    };
    index(arranged);
    const graph = JSON.parse(JSON.stringify(shape));
    graph.layoutOptions = { ...(graph.layoutOptions || {}), ...INTERACTIVE };
    const walk = (holder) => {
      for (const node of holder.children || []) {
        const now = at.get(node.id);
        if (now) {
          node.x = now.x;
          node.y = now.y;
          if (!(node.children && node.children.length)) {
            node.width = now.width;
            node.height = now.height;
          }
        }
        if (node.children && node.children.length) {
          node.layoutOptions = { ...(node.layoutOptions || {}), ...INTERACTIVE };
          walk(node);
        }
      }
    };
    walk(graph);
    return graph;
  }

  /**
   * The tidy layout as the moves the layout file keeps: { box: { dx, dy[, dw, dh] } }
   * from the automatic layout, each relative to the box that holds it.
   */
  function placements(auto, tidy) {
    const was = new Map();
    const index = (holder) => {
      for (const node of holder.children || []) {
        was.set(node.id, node);
        index(node);
      }
    };
    index(auto);
    const out = {};
    const round = (v) => Math.round(v * 10) / 10;
    const walk = (holder) => {
      for (const node of holder.children || []) {
        const before = was.get(node.id);
        if (before) {
          const move = { dx: round((node.x || 0) - (before.x || 0)), dy: round((node.y || 0) - (before.y || 0)) };
          if (!(node.children && node.children.length)) {
            const dw = round((node.width || 0) - (before.width || 0));
            const dh = round((node.height || 0) - (before.height || 0));
            if (dw) move.dw = dw;
            if (dh) move.dh = dh;
          }
          if (move.dx || move.dy || move.dw || move.dh) out[node.id] = move;
        }
        walk(node);
      }
    };
    walk(tidy);
    return out;
  }

  return { measure, worse, describe, seeded, placements, INTERACTIVE };
});
