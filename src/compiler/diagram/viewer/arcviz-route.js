/*
 * ArcViz route — right-angled routes that go around boxes.
 *
 * Pure functions, no DOM. A route leaves its start square to the side it
 * leaves by, comes into its end square to the side it enters by, passes
 * through the points the reader gave it, and keeps a margin from every
 * obstacle. The search runs on the grid of lines drawn along the sides of
 * the obstacles (an orthogonal visibility graph) with A*, counting each
 * turn as a length: the result is short, with few turns, and the same for
 * the same input. When no route exists (an end inside another box), null.
 */
(function (root, factory) {
  const api = factory();
  if (typeof module === 'object' && module.exports) module.exports = api;
  else root.ArcVizRoute = api;
})(typeof self !== 'undefined' ? self : this, function () {
  'use strict';

  const MARGIN = 10; // sheet units kept between a route and a box
  const BEND = 24; // a turn costs as much as this length
  const REACH = 160;
  const OUTSIDE = 30; // the outermost lines lie this far beyond everything // first search: obstacles this far around the ends only
  const DIRS = [{ x: 1, y: 0 }, { x: 0, y: 1 }, { x: -1, y: 0 }, { x: 0, y: -1 }]; // east, south, west, north
  const SIDE_DIR = { east: 0, south: 1, west: 2, north: 3 };

  /** A small binary heap of [cost, value] pairs. */
  function heap() {
    const items = [];
    return {
      get size() {
        return items.length;
      },
      push(cost, order, value) {
        items.push([cost, order, value]);
        let i = items.length - 1;
        while (i > 0) {
          const parent = (i - 1) >> 1;
          if (!less(items[i], items[parent])) break;
          [items[i], items[parent]] = [items[parent], items[i]];
          i = parent;
        }
      },
      pop() {
        const top = items[0];
        const last = items.pop();
        if (items.length > 0) {
          items[0] = last;
          let i = 0;
          for (;;) {
            const l = 2 * i + 1;
            const r = l + 1;
            let best = i;
            if (l < items.length && less(items[l], items[best])) best = l;
            if (r < items.length && less(items[r], items[best])) best = r;
            if (best === i) break;
            [items[i], items[best]] = [items[best], items[i]];
            i = best;
          }
        }
        return top[2];
      },
    };
    // Ties go to the state pushed first: the search is deterministic.
    function less(a, b) {
      return a[0] < b[0] || (a[0] === b[0] && a[1] < b[1]);
    }
  }

  const sortedUnique = (values) => [...new Set(values.map((v) => Math.round(v * 100) / 100))].sort((a, b) => a - b);

  /** Every coordinate of a grid, with the middle of each gap so that a gap is a channel. */
  function axis(values) {
    const base = sortedUnique(values);
    const all = [...base];
    for (let i = 0; i + 1 < base.length; i += 1) all.push((base[i] + base[i + 1]) / 2);
    return sortedUnique(all);
  }

  /**
   * The cheapest right-angled path between two points around rectangles.
   * @param {{x, y}} a, b             the points (outside every obstacle)
   * @param {Array<number>} startDirs the ways the path may leave a (0-3)
   * @param {?number} endDir          the way the path must arrive at b, or null
   * @param {Array<{x, y, width, height}>} rects  obstacles, already grown by the margin
   * @returns {?{points: Array<{x, y}>, dir: number}}  corners from a to b and the way it arrived, or null
   */
  function search(a, b, startDirs, endDir, rects) {
    // Lines along every obstacle, and one beyond them all on each side to go around the lot.
    const around = (values) => [Math.min(...values) - OUTSIDE, Math.max(...values) + OUTSIDE, ...values];
    const xs = axis(around([a.x, b.x, ...rects.flatMap((r) => [r.x, r.x + r.width])]));
    const ys = axis(around([a.y, b.y, ...rects.flatMap((r) => [r.y, r.y + r.height])]));
    const ix = new Map(xs.map((v, i) => [v, i]));
    const iy = new Map(ys.map((v, i) => [v, i]));
    const key = (v) => Math.round(v * 100) / 100;
    const W = xs.length;
    const inside = (x, y) => rects.some((r) => x > r.x && x < r.x + r.width && y > r.y && y < r.y + r.height);
    const blocked = new Uint8Array(W * ys.length);
    for (let j = 0; j < ys.length; j += 1) for (let i = 0; i < W; i += 1) blocked[j * W + i] = inside(xs[i], ys[j]) ? 1 : 0;
    const start = iy.get(key(a.y)) * W + ix.get(key(a.x));
    const goal = iy.get(key(b.y)) * W + ix.get(key(b.x));
    if (blocked[start] || blocked[goal]) return null;
    const h = (cell) => Math.abs(xs[cell % W] - b.x) + Math.abs(ys[Math.floor(cell / W)] - b.y);
    // A state is a cell and the way the path entered it.
    const best = new Float64Array(W * ys.length * 4).fill(Infinity);
    const from = new Int32Array(W * ys.length * 4).fill(-1);
    const open = heap();
    let order = 0;
    for (const d of startDirs) {
      best[start * 4 + d] = 0;
      open.push(h(start), order++, start * 4 + d);
    }
    let reached = -1;
    while (open.size > 0) {
      const state = open.pop();
      const cell = state >> 2;
      const dir = state & 3;
      const cost = best[state];
      if (cell === goal) {
        // Arriving the wrong way costs a turn more; the right way, nothing.
        if (endDir === null || endDir === undefined || dir === endDir) {
          reached = state;
          break;
        }
      }
      const i = cell % W;
      const j = Math.floor(cell / W);
      for (let nd = 0; nd < 4; nd += 1) {
        if (nd === ((dir + 2) & 3)) continue; // no turning back
        const ni = i + DIRS[nd].x;
        const nj = j + DIRS[nd].y;
        if (ni < 0 || nj < 0 || ni >= W || nj >= ys.length) continue;
        const next = nj * W + ni;
        if (blocked[next]) continue;
        const step = Math.abs(xs[ni] - xs[i]) + Math.abs(ys[nj] - ys[j]) + (nd === dir ? 0 : BEND);
        const ns = next * 4 + nd;
        if (cost + step < best[ns]) {
          best[ns] = cost + step;
          from[ns] = state;
          open.push(cost + step + h(next), order++, ns);
        }
      }
    }
    if (reached < 0) return null;
    const cells = [];
    for (let s = reached; s >= 0; s = from[s]) cells.push(s >> 2);
    cells.reverse();
    const points = cells.map((cell) => ({ x: xs[cell % W], y: ys[Math.floor(cell / W)] }));
    return { points: simplify(points), dir: reached & 3 };
  }

  /** Drop repeats and points in the middle of a straight run. */
  function simplify(points) {
    const out = [];
    for (const point of points) {
      const last = out[out.length - 1];
      if (last && last.x === point.x && last.y === point.y) continue;
      const before = out[out.length - 2];
      if (before && last && ((before.x === last.x && last.x === point.x) || (before.y === last.y && last.y === point.y))) out.pop();
      out.push({ x: point.x, y: point.y });
    }
    return out;
  }

  const grow = (r, by) => ({ x: r.x - by, y: r.y - by, width: r.width + 2 * by, height: r.height + 2 * by });
  const near = (r, points, by) => points.some((p) => p.x > r.x - by && p.x < r.x + r.width + by && p.y > r.y - by && p.y < r.y + r.height + by)
    || (() => {
      // Between the ends: what lies in the box they span, widened.
      const xs = points.map((p) => p.x);
      const ys = points.map((p) => p.y);
      const lo = { x: Math.min(...xs) - by, y: Math.min(...ys) - by };
      const hi = { x: Math.max(...xs) + by, y: Math.max(...ys) + by };
      return r.x < hi.x && r.x + r.width > lo.x && r.y < hi.y && r.y + r.height > lo.y;
    })();

  /**
   * A right-angled route between two anchors, around the obstacles.
   * @param {object} spec
   * @param {{x, y, side}} spec.from, spec.to   where the route meets its ends, and the side of the box
   * @param {Array<{x, y}>} [spec.via]          points it must pass through, in order
   * @param {Array<{x, y, width, height}>} spec.obstacles  boxes to go around (the ends' own boxes included)
   * @param {number} [spec.stub]                how far it leaves a box before turning
   * @returns {?Array<{x, y}>}                  the route, from the first anchor to the last, or null
   */
  function route(spec) {
    const { from, to } = spec;
    const via = (spec.via || []).map(({ x, y }) => ({ x, y }));
    const stub = spec.stub || 14;
    const margin = Math.min(MARGIN, stub - 2);
    const out = (anchor) => {
      const d = DIRS[SIDE_DIR[anchor.side]];
      return { x: anchor.x + d.x * stub, y: anchor.y + d.y * stub };
    };
    const a = out(from);
    const b = out(to);
    const legs = [a, ...via, b];
    const all = (spec.obstacles || []).map((r) => grow(r, margin));
    // A point inside a box (two boxes overlap, a bend dropped on one) frees that box.
    const free = all.filter((r) => !legs.some((p) => p.x > r.x && p.x < r.x + r.width && p.y > r.y && p.y < r.y + r.height));
    const solve = (rects) => {
      const path = [{ x: from.x, y: from.y }];
      let arrived = null;
      for (let i = 0; i + 1 < legs.length; i += 1) {
        // Out of a box square to its side; on through a bend without turning back.
        const startDirs = i === 0 ? [SIDE_DIR[from.side]] : [0, 1, 2, 3].filter((d) => arrived === null || d !== ((arrived + 2) & 3));
        const endDir = i + 2 === legs.length ? (SIDE_DIR[to.side] + 2) & 3 : null;
        const leg = search(legs[i], legs[i + 1], startDirs, endDir, rects);
        if (!leg) return null;
        path.push(...leg.points);
        arrived = leg.dir;
      }
      path.push({ x: to.x, y: to.y });
      return simplify(path);
    };
    // Near the ends first (fast), then the whole sheet when that finds nothing.
    const local = free.filter((r) => near(r, legs, REACH));
    if (local.length < free.length) {
      const quick = solve(local);
      // The stubs start on the ends' own boxes: only what lies between is checked.
      if (quick && routeClear(quick.slice(1, -1), free)) return quick;
    }
    return solve(free);
  }

  /** Whether a route keeps out of every box. */
  function routeClear(points, rects) {
    if (!points) return false;
    for (let i = 0; i + 1 < points.length; i += 1) {
      const p = points[i];
      const q = points[i + 1];
      for (const r of rects) {
        const lx = Math.min(p.x, q.x);
        const hx = Math.max(p.x, q.x);
        const ly = Math.min(p.y, q.y);
        const hy = Math.max(p.y, q.y);
        if (hx > r.x && lx < r.x + r.width && hy > r.y && ly < r.y + r.height) return false;
      }
    }
    return true;
  }

  /** Whether a polyline crosses the inside of a rectangle (its border shrunk by `inset`). */
  function crosses(points, rect, inset = 2) {
    const r = { x: rect.x + inset, y: rect.y + inset, width: rect.width - 2 * inset, height: rect.height - 2 * inset };
    if (r.width <= 0 || r.height <= 0) return false;
    for (let i = 0; i + 1 < points.length; i += 1) {
      const p = points[i];
      const q = points[i + 1];
      // Liang–Barsky clipping of the segment against the rectangle.
      let t0 = 0;
      let t1 = 1;
      const dx = q.x - p.x;
      const dy = q.y - p.y;
      const edges = [[-dx, p.x - r.x], [dx, r.x + r.width - p.x], [-dy, p.y - r.y], [dy, r.y + r.height - p.y]];
      let hit = true;
      for (const [pp, qq] of edges) {
        if (pp === 0) {
          if (qq < 0) {
            hit = false;
            break;
          }
          continue;
        }
        const t = qq / pp;
        if (pp < 0) t0 = Math.max(t0, t);
        else t1 = Math.min(t1, t);
        if (t0 > t1) {
          hit = false;
          break;
        }
      }
      if (hit && t1 - t0 > 1e-9) return true;
    }
    return false;
  }

  return { route, crosses, MARGIN };
});
