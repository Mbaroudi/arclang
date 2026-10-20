/*
 * ArcViz edit — direct manipulation for a host that edits the model from
 * the drawing (an editor, not the site): the ends of the selected exchange
 * dragged to another box or port (reconnect), its middle dragged to bend
 * it, boxes moved or resized against the edges and centres of the others
 * (smart guides, Alt to move freely), a drag on the empty sheet to select
 * the boxes it encloses, and an arrow on each side of the box under the
 * pointer: dragged, it connects; tapped, it creates and connects.
 *
 * Everything lives in the frame, over the sheet, so it keeps its size at
 * any zoom. What a gesture means for the model is the host's business: it
 * hears of each one through a callback.
 */
(function (root) {
  'use strict';

  const SNAP = 6; // frame pixels within which an edge or a centre snaps
  const SLOP = 4; // pixels a pointer may wander and still be a click
  const GRACE = 22; // pixels around a box within which its arrows stay
  const DOUBLE_TAP_MS = 450; // two taps on a bend within this remove it
  const SIDES = ['north', 'east', 'south', 'west'];
  const ARROWS = { north: '↑', east: '→', south: '↓', west: '←' };

  /**
   * @param {object} deps
   * @param {Element}  deps.stage, deps.canvas   the frame and the sheet in it
   * @param {object}   deps.sheet               the stage (ArcVizStage): scale(), connectFrom()
   * @param {object}   deps.hooks               filled here, read by the stage (its `editing`)
   * @param {Function} deps.pick                (element|null): a click on the empty sheet
   * @param {Function} deps.carried             (node id) -> ids moving with it
   * @param {Function} deps.routeOf             (edge id) -> points it is bent through, sheet units
   * @param {Function} deps.onBend              optional (edge id, points): an exchange bent or straightened
   * @param {Function} deps.onReconnect         optional (edge id, 'source'|'target', node or port id)
   * @param {Function} deps.reconnectTargets    optional (edge id, end) -> ids it may move to, null for any
   * @param {Function} deps.onMarquee           optional (node ids, additive) after a drag on the empty sheet
   * @param {Function} deps.onQuick             optional (node id, side): an arrow of a box tapped
   * @param {Function} deps.onEdit              optional (element) -> true: a double-click edits it
   * @param {boolean}  deps.snap                smart guides while moving and resizing
   */
  function create(deps) {
    const { stage, canvas, sheet, hooks } = deps;
    const div = (className, attributes = {}) => {
      const node = document.createElement('div');
      node.className = className;
      for (const [key, value] of Object.entries(attributes)) node.setAttribute(key, value);
      return node;
    };
    const frame = () => stage.getBoundingClientRect();
    const toFrame = (x, y) => {
      const f = frame();
      return { x: x - f.left, y: y - f.top };
    };
    const shapeOf = (node) => node.querySelector(':scope > .av-shape') || node;
    const svg = () => canvas.querySelector('svg');
    /** A point of the sheet (sheet units) at a point of the screen. */
    const toSheet = (clientX, clientY) => {
      const root = svg();
      const matrix = root && root.getScreenCTM();
      if (!matrix) return null;
      const point = new DOMPoint(clientX, clientY).matrixTransform(matrix.inverse());
      return { x: Math.round(point.x), y: Math.round(point.y) };
    };
    const toScreen = (point) => {
      const root = svg();
      const matrix = root && root.getScreenCTM();
      if (!matrix) return null;
      const at = new DOMPoint(point.x, point.y).matrixTransform(matrix);
      return { x: at.x, y: at.y };
    };
    const place = (node, at) => {
      node.style.translate = `${at.x}px ${at.y}px`;
      node.hidden = false;
    };
    const under = (event, through = []) => {
      for (const node of through) node.style.pointerEvents = 'none';
      const hit = document.elementFromPoint(event.clientX, event.clientY);
      for (const node of through) node.style.pointerEvents = '';
      return hit;
    };
    /** Follow a pointer from a press on `node` until it is let go. */
    const follow = (node, down, onMove, onUp) => {
      down.preventDefault();
      down.stopPropagation();
      node.setPointerCapture(down.pointerId);
      let moved = false;
      const move = (event) => {
        if (!moved && Math.hypot(event.clientX - down.clientX, event.clientY - down.clientY) < SLOP) return;
        moved = true;
        onMove(event);
      };
      const up = (event) => {
        node.removeEventListener('pointermove', move);
        node.removeEventListener('pointerup', up);
        node.removeEventListener('pointercancel', up);
        onUp(event, moved && event.type === 'pointerup');
      };
      node.addEventListener('pointermove', move);
      node.addEventListener('pointerup', up);
      node.addEventListener('pointercancel', up);
    };
    const ghost = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
    ghost.setAttribute('class', 'arcviz-ghost');
    ghost.setAttribute('aria-hidden', 'true');
    const ghostLine = document.createElementNS('http://www.w3.org/2000/svg', 'line');
    ghost.appendChild(ghostLine);
    const band = (from, to) => {
      if (!ghost.isConnected) stage.appendChild(ghost);
      ghostLine.setAttribute('x1', String(from.x));
      ghostLine.setAttribute('y1', String(from.y));
      ghostLine.setAttribute('x2', String(to.x));
      ghostLine.setAttribute('y2', String(to.y));
    };

    // --- the selected exchange: its two ends and its bends ---------------
    const ends = { source: div('arcviz-end', { role: 'button', 'data-end': 'source', title: 'Drag to another box or port to reconnect the source' }), target: div('arcviz-end', { role: 'button', 'data-end': 'target', title: 'Drag to another box or port to reconnect the target' }) };
    for (const node of Object.values(ends)) {
      node.hidden = true;
      if (deps.onReconnect) stage.appendChild(node);
    }
    let bends = []; // handles of the selected exchange: waypoints and the middles that add one
    const selectedEdge = () => canvas.querySelector('.av-edge.is-selected');
    const linesOf = (edge) => [...edge.querySelectorAll('.av-line')];
    /** Where the exchange starts and ends, in screen pixels. */
    function endPoints(edge) {
      const lines = linesOf(edge);
      if (lines.length === 0) return null;
      const at = (line, length) => {
        const point = line.getPointAtLength(length);
        const matrix = line.getScreenCTM();
        return matrix ? new DOMPoint(point.x, point.y).matrixTransform(matrix) : null;
      };
      const first = lines[0];
      const last = lines[lines.length - 1];
      const start = at(first, 0);
      const end = at(last, last.getTotalLength());
      // A third of the way: the name of the exchange usually sits in its middle.
      const middle = at(first, first.getTotalLength() / 3);
      return start && end && middle ? { source: { x: start.x, y: start.y }, target: { x: end.x, y: end.y }, middle: { x: middle.x, y: middle.y } } : null;
    }
    function clearBends() {
      for (const node of bends) node.remove();
      bends = [];
    }
    function placeEdge() {
      const edge = selectedEdge();
      const points = edge ? endPoints(edge) : null;
      for (const [end, node] of Object.entries(ends)) {
        if (points && deps.onReconnect) place(node, toFrame(points[end].x, points[end].y));
        else node.hidden = true;
      }
      clearBends();
      if (!points || !deps.onBend) return;
      const id = edge.dataset.id;
      const route = deps.routeOf(id);
      const screenRoute = route.map(toScreen);
      if (screenRoute.some((p) => !p)) return;
      const chain = [points.source, ...screenRoute, points.target];
      // A square on each waypoint (drag to move it, double-click to remove it),
      // a diamond in the middle of each run between them (drag to add one).
      screenRoute.forEach((point, index) => {
        const handle = div('arcviz-bend is-waypoint', { role: 'button', title: 'Drag to move this bend; double-click to remove it', 'data-index': String(index) });
        place(handle, toFrame(point.x, point.y));
        handle.addEventListener('pointerdown', (down) => dragBend(handle, down, id, route, index, false));
        stage.appendChild(handle);
        bends.push(handle);
      });
      for (let index = 0; index + 1 < chain.length; index += 1) {
        const a = chain[index];
        const b = chain[index + 1];
        const middle = route.length === 0 ? points.middle : { x: (a.x + b.x) / 2, y: (a.y + b.y) / 2 };
        const handle = div('arcviz-bend is-new', { role: 'button', title: 'Drag to bend the exchange here', 'data-index': String(index) });
        place(handle, toFrame(middle.x, middle.y));
        handle.addEventListener('pointerdown', (down) => dragBend(handle, down, id, route, index, true));
        stage.appendChild(handle);
        bends.push(handle);
      }
    }
    let lastTap = { at: 0, index: -1 };
    function dragBend(handle, down, id, route, index, adding) {
      follow(handle, down, (event) => place(handle, toFrame(event.clientX, event.clientY)), (event, moved) => {
        if (!moved) {
          // Two taps on one bend remove it (the handles are redrawn between clicks: no dblclick).
          const now = performance.now();
          const twice = !adding && lastTap.index === index && now - lastTap.at < DOUBLE_TAP_MS;
          lastTap = { at: now, index: adding ? -1 : index };
          if (twice) deps.onBend(id, deps.routeOf(id).filter((_, at) => at !== index));
          return;
        }
        const point = toSheet(event.clientX, event.clientY);
        if (!point) return placeEdge();
        // The route as it is now: another bend may have landed since these handles were drawn.
        const now = deps.routeOf(id);
        const next = adding ? [...now.slice(0, index), point, ...now.slice(index)] : now.map((p, at) => (at === index ? point : p));
        deps.onBend(id, next);
      });
    }
    for (const [end, node] of Object.entries(ends)) {
      node.addEventListener('pointerdown', (down) => reconnect(node, end, down));
    }
    function reconnect(node, end, down) {
      const edge = selectedEdge();
      const points = edge && endPoints(edge);
      if (!edge || !points) return;
      const allowed = deps.reconnectTargets ? deps.reconnectTargets(edge.dataset.id, end) : null;
      const valid = allowed ? new Set(allowed) : null;
      const candidates = new Set([...canvas.querySelectorAll('.av-node, .av-port')].filter((item) => !valid || valid.has(item.dataset.id)));
      for (const item of candidates) item.classList.add('is-connect-target');
      stage.classList.add('is-reconnecting');
      const fixed = toFrame(points[end === 'source' ? 'target' : 'source'].x, points[end === 'source' ? 'target' : 'source'].y);
      let aimed = null;
      const aim = (target) => {
        if (aimed === target) return;
        if (aimed) aimed.classList.remove('is-drop-target');
        aimed = target;
        if (aimed) aimed.classList.add('is-drop-target');
      };
      const targetUnder = (event) => {
        const hit = under(event, [node]);
        const item = hit && hit.closest ? hit.closest('.av-port, .av-node') : null;
        return item && candidates.has(item) ? item : null;
      };
      follow(node, down, (event) => {
        const at = toFrame(event.clientX, event.clientY);
        place(node, at);
        band(fixed, at);
        aim(targetUnder(event));
      }, (event, moved) => {
        const target = moved ? targetUnder(event) : null;
        aim(null);
        for (const item of candidates) item.classList.remove('is-connect-target');
        stage.classList.remove('is-reconnecting');
        if (ghost.isConnected) ghost.remove();
        placeEdge();
        if (target) deps.onReconnect(edge.dataset.id, end, target.dataset.id);
      });
    }

    // --- smart guides: edges and centres of the other boxes --------------
    const guides = { x: div('arcviz-guide is-vertical'), y: div('arcviz-guide is-horizontal') };
    for (const guide of Object.values(guides)) guide.hidden = true;
    if (deps.snap) stage.append(guides.x, guides.y);
    let snapping = null; // { mode, box, others }
    hooks.snapStart = (id, mode) => {
      if (!deps.snap) return;
      const node = [...canvas.querySelectorAll('.av-node')].find((item) => item.dataset.id === id);
      if (!node) return;
      const moving = new Set(mode === 'move' ? deps.carried(id) : [id]);
      const rect = (item) => {
        const r = shapeOf(item).getBoundingClientRect();
        return { left: r.left, right: r.right, top: r.top, bottom: r.bottom, cx: (r.left + r.right) / 2, cy: (r.top + r.bottom) / 2 };
      };
      const others = [...canvas.querySelectorAll('.av-node')].filter((item) => !moving.has(item.dataset.id) && !item.contains(node)).map(rect);
      snapping = { mode, box: rect(node), others };
    };
    /** The nearest line of another box within reach of one of ours, as the shift that lands on it. */
    const nearest = (ours, theirs) => {
      let best = null;
      for (const mine of ours) {
        for (const line of theirs) {
          const shift = line - mine;
          if (Math.abs(shift) <= SNAP && (!best || Math.abs(shift) < Math.abs(best.shift))) best = { shift, line };
        }
      }
      return best;
    };
    hooks.snap = (dx, dy, event) => {
      if (!snapping || (event && event.altKey)) {
        hooks.snapEnd(true);
        return { dx, dy };
      }
      const { box, others, mode } = snapping;
      const xs = others.flatMap((o) => [o.left, o.right, o.cx]);
      const ys = others.flatMap((o) => [o.top, o.bottom, o.cy]);
      const ours = mode === 'move'
        ? { x: [box.left + dx, box.right + dx, box.cx + dx], y: [box.top + dy, box.bottom + dy, box.cy + dy] }
        : { x: [box.right + dx], y: [box.bottom + dy] };
      const sx = nearest(ours.x, xs);
      const sy = nearest(ours.y, ys);
      const f = frame();
      if (sx) Object.assign(guides.x.style, { translate: `${sx.line - f.left}px 0px` });
      if (sy) Object.assign(guides.y.style, { translate: `0px ${sy.line - f.top}px` });
      guides.x.hidden = !sx;
      guides.y.hidden = !sy;
      return { dx: dx + (sx ? sx.shift : 0), dy: dy + (sy ? sy.shift : 0) };
    };
    hooks.snapEnd = (keep) => {
      guides.x.hidden = true;
      guides.y.hidden = true;
      if (!keep) snapping = null;
    };

    // --- a drag on the empty sheet selects what it encloses -------------
    const marquee = div('arcviz-marquee');
    let spaceDown = false;
    stage.addEventListener('keydown', (event) => {
      if (event.key === ' ') spaceDown = true;
    });
    stage.addEventListener('keyup', (event) => {
      if (event.key === ' ') spaceDown = false;
    });
    const isChrome = (target) => target.closest && target.closest('.arcviz-handle, .arcviz-grip, .arcviz-end, .arcviz-bend, .arcviz-quick');
    if (deps.onMarquee) {
      stage.addEventListener('pointerdown', (down) => {
        if (down.button !== 0 || spaceDown || !down.isPrimary || stage.classList.contains('is-connecting')) return;
        const hit = down.target;
        if (isChrome(hit) || (hit.closest && hit.closest('.av-node, .av-edge'))) return;
        const start = toFrame(down.clientX, down.clientY);
        follow(stage, down, (event) => {
          const at = toFrame(event.clientX, event.clientY);
          Object.assign(marquee.style, { left: `${Math.min(start.x, at.x)}px`, top: `${Math.min(start.y, at.y)}px`, width: `${Math.abs(at.x - start.x)}px`, height: `${Math.abs(at.y - start.y)}px` });
          if (!marquee.isConnected) stage.appendChild(marquee);
        }, (event, moved) => {
          if (marquee.isConnected) marquee.remove();
          if (!moved) {
            deps.pick(null);
            stage.focus({ preventScroll: true });
            return;
          }
          const box = { left: Math.min(down.clientX, event.clientX), right: Math.max(down.clientX, event.clientX), top: Math.min(down.clientY, event.clientY), bottom: Math.max(down.clientY, event.clientY) };
          const inside = [...canvas.querySelectorAll('.av-node')].filter((node) => {
            const r = shapeOf(node).getBoundingClientRect();
            return r.width > 0 && r.left >= box.left && r.right <= box.right && r.top >= box.top && r.bottom <= box.bottom;
          });
          // A box and what it holds: the box stands for them.
          const outer = inside.filter((node) => !inside.some((other) => other !== node && other.contains(node)));
          stage.focus({ preventScroll: true });
          deps.onMarquee(outer.map((node) => node.dataset.id), event.shiftKey);
        });
      }, true);
    }

    // --- quick connect: an arrow on each side of the box under the pointer
    const arrows = Object.fromEntries(SIDES.map((side) => [side, div(`arcviz-quick is-${side}`, { role: 'button', 'aria-label': `Connect ${side}`, title: 'Drag to another box to connect; click to create a connected element' })]));
    let quickOn = null; // the box the arrows sit on
    if (deps.onQuick) {
      hooks.portsOnly = true;
      for (const [side, arrow] of Object.entries(arrows)) {
        arrow.textContent = ARROWS[side];
        arrow.hidden = true;
        stage.appendChild(arrow);
        arrow.addEventListener('pointerdown', (down) => {
          if (down.button !== 0 || !quickOn) return;
          down.preventDefault();
          down.stopPropagation();
          const box = quickOn;
          hideArrows();
          sheet.connectFrom(box, down, (from) => deps.onQuick(from.dataset.id, side));
        });
      }
      stage.addEventListener('pointermove', (event) => {
        if (event.buttons || stage.matches('.is-moving, .is-connecting, .is-reconnecting, .is-resizing, .is-panning')) return hideArrows();
        const hit = under(event);
        if (hit && Object.values(arrows).includes(hit)) return;
        const node = hit && hit.closest ? hit.closest('.av-node') : null;
        if (node && !hit.closest('.av-port')) return showArrows(node);
        if (quickOn && quickOn.isConnected) {
          const r = shapeOf(quickOn).getBoundingClientRect();
          const near = event.clientX > r.left - GRACE && event.clientX < r.right + GRACE && event.clientY > r.top - GRACE && event.clientY < r.bottom + GRACE;
          if (near) return;
        }
        hideArrows();
      });
      stage.addEventListener('pointerleave', () => hideArrows());
    }
    function showArrows(node) {
      quickOn = node;
      const r = shapeOf(node).getBoundingClientRect();
      const spots = { north: [(r.left + r.right) / 2, r.top], east: [r.right, (r.top + r.bottom) / 2], south: [(r.left + r.right) / 2, r.bottom], west: [r.left, (r.top + r.bottom) / 2] };
      for (const side of SIDES) place(arrows[side], toFrame(spots[side][0], spots[side][1]));
    }
    function hideArrows() {
      quickOn = null;
      for (const arrow of Object.values(arrows)) arrow.hidden = true;
    }

    // --- the stage's hooks -----------------------------------------------
    if (deps.onEdit) hooks.onEdit = (hit) => deps.onEdit(hit);
    hooks.onFrame = () => {
      placeEdge();
      if (quickOn) {
        if (quickOn.isConnected) showArrows(quickOn);
        else hideArrows();
      }
    };

    return {
      /** Put the handles where they belong after the selection or the drawing changed. */
      refresh: () => {
        hideArrows();
        placeEdge();
      },
    };
  }

  /**
   * Mount the editing gestures on a viewer (ArcViz.mount with options.edit):
   *   options.edit = { onEdit({ view, id, isEdge }) -> true when a double-click
   *   edits the box or exchange; onReconnect({ view, edge, end, to }) when an
   *   end of the selected exchange is dragged to another box or port, and
   *   reconnectTargets({ view, edge, end }) -> ids | null, the ends it may move
   *   to; onMarquee({ view, ids, additive }) after a drag on the empty sheet;
   *   onQuick({ view, id, side }) when an arrow on the side of a box is tapped
   *   (dragged, it connects through onConnect); bends: true to bend exchanges
   *   by their middle, kept in the layout; snap: true for smart guides }.
   * `viewer` lends what the gestures act through: the stage, the sheet, the
   * scope of the views, the active view, select, carried, replace, announce.
   * Returns the editor, with `api`: what the viewer's own API gains
   * (moveMany, routeOf, setRoute).
   */
  function attach(viewer) {
    const { options: edit, scope, replace, carried, view } = viewer;
    const editor = create({
      stage: viewer.stage,
      canvas: viewer.canvas,
      sheet: viewer.sheet,
      hooks: viewer.hooks,
      pick: (element) => viewer.select(element),
      carried,
      routeOf: (edge) => scope.routeOf(view(), edge),
      onBend: edit.bends ? (edge, points) => replace(() => scope.setRoute(view(), edge, points)).then(() => viewer.announce('Exchange bent.')) : null,
      onReconnect: edit.onReconnect ? (edge, end, to) => edit.onReconnect({ view: view(), edge, end, to }) : null,
      reconnectTargets: edit.reconnectTargets ? (edge, end) => edit.reconnectTargets({ view: view(), edge, end }) : null,
      onMarquee: edit.onMarquee ? (ids, additive) => edit.onMarquee({ view: view(), ids, additive }) : null,
      onQuick: edit.onQuick && viewer.connects ? (id, side) => edit.onQuick({ view: view(), id, side }) : null,
      onEdit: edit.onEdit
        ? (hit) => {
            const group = hit.closest ? hit.closest('.av-edge, .av-node') : null;
            return Boolean(group) && Boolean(edit.onEdit({ view: view(), id: group.dataset.id, isEdge: group.classList.contains('av-edge') }));
          }
        : null,
      snap: Boolean(edit.snap),
    });
    editor.api = {
      /** Move several boxes of the view on screen at once, in sheet units: one redraw, one layout change. */
      moveMany: (moves) =>
        replace(() => {
          for (const { id, dx, dy } of moves || []) if (carried(id).length > 0) scope.moveBy(view(), id, dx, dy);
        }),
      /** The points an exchange of the view on screen is bent through ([] when straight). */
      routeOf: (edge) => scope.routeOf(view(), edge),
      /** Bend an exchange of the view on screen through `points`; [] straightens it. */
      setRoute: (edge, points) => replace(() => scope.setRoute(view(), edge, points)),
    };
    return editor;
  }

  root.ArcVizEdit = { create, attach };
})(typeof globalThis !== 'undefined' ? globalThis : this);
