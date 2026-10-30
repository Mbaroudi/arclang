/*
 * ArcViz scope — which part of each view is on screen and how the reader
 * arranged it: the containers folded, the container opened as a diagram of
 * its own, and the boxes moved by hand.
 *
 * No DOM. The arrangement starts from the layout file of the model, when it
 * has one, and is written back in the same form. Meanwhile it is remembered
 * in the storage it is given, so a reload shows the reader what they left.
 * Whatever is read, from the file or from storage, is checked against the
 * model: an id it does not draw is ignored.
 */
(function (root, factory) {
  const api = factory();
  if (typeof module === 'object' && module.exports) module.exports = api;
  else root.ArcVizScope = api;
})(typeof self !== 'undefined' ? self : this, function () {
  'use strict';

  const LAYOUT_VERSION = '1';

  /**
   * @param {object} deps
   * @param {Array<object>} deps.graphs      layout graphs of every view
   * @param {object} deps.collapse           ArcVizCollapse
   * @param {?Storage} deps.storage          where to remember, or null
   * @param {string} deps.storageKey         one key per model
   * @param {?object} deps.initial           the layout file of the model, if any
   * @param {?object} deps.sections          more of the file kept elsewhere: { toFile() -> object, load(file) } (layers, groups, locks)
   */
  function create(deps) {
    const { graphs, collapse, storage, storageKey, initial } = deps;
    const sections = deps.sections || null; // zorder hook
    const whole = () => ({ folded: new Set(), focus: null });
    const WHOLE = Object.freeze(whole());
    const byId = new Map(graphs.map((graph) => [graph.id, graph]));
    const containers = new Map(graphs.map((graph) => [graph.id, new Set(collapse.containersOf(graph))]));
    const scopes = new Map(); // graph id -> { folded: Set, focus: ?string }
    // Layout key -> { node id: { dx, dy } }: a placement belongs to one
    // layout, so each fold or open container has its own.
    const places = new Map();
    // Layout key -> { edge id: [{ x, y }] }: the exchanges bent by hand.
    const routes = new Map();
    // Layout key -> { edge id: 'straight' | 'curved' }: lines drawn otherwise than at right angles.
    const styles = new Map();
    // Layout key -> { edge id: { dx, dy } }: names dragged off their line.
    const labels = new Map();
    // Layout key -> { port id: { dx, dy } }: ports dragged along the border of their box.
    const ports = new Map();
    // Layout key -> { node id: z }: boxes brought to the front (z > 0) or sent to the back (z < 0).
    const order = new Map();
    const STYLES = new Set(['straight', 'curved']);
    const MAX_Z = 100000;
    const isShift = (value) => Boolean(value) && Number.isFinite(value.dx) && Number.isFinite(value.dy) && Math.abs(value.dx) <= 100000 && Math.abs(value.dy) <= 100000;
    const edges = new Map(graphs.map((graph) => {
      const ids = new Set();
      const walk = (holder) => {
        for (const edge of holder.edges || []) ids.add(edge.id);
        (holder.children || []).forEach(walk);
      };
      walk(graph);
      return [graph.id, ids];
    }));
    const isPoint = (point) => Boolean(point) && Number.isFinite(point.x) && Number.isFinite(point.y) && Math.abs(point.x) <= 100000 && Math.abs(point.y) <= 100000;
    const MAX_BENDS = 64; // points a bent exchange may pass through
    const isRoute = (points) => Array.isArray(points) && points.length > 0 && points.length <= MAX_BENDS && points.every(isPoint);
    const nodes = new Map(graphs.map((graph) => {
      const ids = new Set();
      const walk = (node) => {
        ids.add(node.id);
        (node.children || []).forEach(walk);
      };
      (graph.children || []).forEach(walk);
      return [graph.id, ids];
    }));
    const portIds = new Map(graphs.map((graph) => {
      const ids = new Set();
      const walk = (node) => {
        for (const port of node.ports || []) ids.add(port.id);
        (node.children || []).forEach(walk);
      };
      (graph.children || []).forEach(walk);
      return [graph.id, ids];
    }));
    /** Change one entry of a per-layout map of the view on screen; `value` undefined removes it. */
    const setIn = (map, id, item, value) => {
      const key = keyOf(id, of(id));
      const { [item]: was, ...others } = map.get(key) || {};
      const next = value === undefined ? others : { ...others, [item]: value };
      if (Object.keys(next).length > 0) map.set(key, next);
      else map.delete(key);
      remember();
    };
    const sorted = (object) => Object.fromEntries(Object.keys(object).sort().map((k) => [k, object[k]]));
    // The open container is not folded in its own diagram: it is not part
    // of what tells one layout from another.
    const keyOf = (id, scope) => JSON.stringify([id, scope.focus, ...[...scope.folded].filter((node) => node !== scope.focus).sort()]);
    const FARTHEST = 100000; // sheet units: beyond this an offset is not a placement
    const sound = (value) => value === undefined || (Number.isFinite(value) && Math.abs(value) <= FARTHEST);
    const isOffset = (value) => Boolean(value) && Number.isFinite(value.dx) && Number.isFinite(value.dy) && Math.abs(value.dx) <= FARTHEST && Math.abs(value.dy) <= FARTHEST && sound(value.dw) && sound(value.dh);
    /** A placement as kept: the move, and the resize only when there is one. */
    const asPlacement = ({ dx, dy, dw, dh }) => ({ dx, dy, ...(dw ? { dw } : {}), ...(dh ? { dh } : {}) });
    const nothing = (value) => !value.dx && !value.dy && !value.dw && !value.dh;

    const of = (id) => scopes.get(id) || whole();
    const isContainer = (id, node) => Boolean(node) && containers.has(id) && containers.get(id).has(node);

    // What is remembered is a working copy of the layout file this page
    // was built with: it belongs to this model (two may share a title) and
    // to that file. A new file starts a new working copy.
    let print = 5381;
    const stamp = (text) => {
      for (const char of text) print = ((print * 33) ^ char.charCodeAt(0)) >>> 0;
    };
    for (const [id, held] of containers) stamp([id, ...[...held].sort()].join('\u0000'));
    stamp(JSON.stringify(initial === undefined ? null : initial));
    const slot = `${storageKey}:${print.toString(36)}`;

    /** The arrangement in the form of the layout file. */
    function toFile() {
      const views = {};
      for (const id of [...scopes.keys()].sort()) {
        const scope = scopes.get(id);
        if (scope.folded.size > 0 || scope.focus) views[id] = { folded: [...scope.folded].sort(), open: scope.focus };
      }
      const keys = [...new Set([...places.keys(), ...routes.keys(), ...styles.keys(), ...labels.keys(), ...ports.keys(), ...order.keys()])].sort();
      const arrangements = keys.map((key) => {
        const [view, open, ...folded] = JSON.parse(key);
        const placed = places.get(key) || {};
        const bent = routes.get(key);
        // Only what the view draws: an element removed or renamed since leaves nothing behind.
        const drawsNode = (node) => nodes.has(view) && nodes.get(view).has(node);
        const drawsPort = (port) => portIds.has(view) && portIds.get(view).has(port);
        const only = (object, keep) => Object.fromEntries(Object.keys(object).filter(keep).sort().map((k) => [k, object[k]]));
        const entry = { view, open, folded, places: only(placed, drawsNode) };
        // Written only when an exchange was bent: files without bends stay as they were.
        // Only exchanges the view draws: a renamed or removed one leaves nothing behind.
        // A line's style and the offset of its name go with its points, each written only when set.
        const drawsEdge = (edge) => edges.has(view) && edges.get(view).has(edge);
        const styled = styles.get(key) || {};
        const named = labels.get(key) || {};
        const drawn = [...new Set([...Object.keys(bent || {}), ...Object.keys(styled), ...Object.keys(named)])].filter(drawsEdge).sort();
        if (drawn.length) {
          entry.edges = Object.fromEntries(drawn.map((edge) => [edge, {
            ...(bent && bent[edge] ? { points: bent[edge] } : {}),
            ...(styled[edge] ? { style: styled[edge] } : {}),
            ...(named[edge] ? { label: named[edge] } : {}),
          }]));
        }
        const moved = ports.has(key) ? only(ports.get(key), drawsPort) : {};
        const stacked = order.has(key) ? only(order.get(key), drawsNode) : {};
        if (Object.keys(moved).length) entry.ports = moved;
        if (Object.keys(stacked).length) entry.order = stacked;
        return entry;
      }).filter((entry) => Object.keys(entry.places).length || entry.edges || entry.ports || entry.order);
      return { arclang_layout: LAYOUT_VERSION, views, arrangements, ...(sections ? sections.toFile() : {}) }; // zorder hook
    }

    /** Replace the arrangement by what a layout says, piece by valid piece. */
    function load(file) {
      scopes.clear();
      places.clear();
      routes.clear();
      styles.clear();
      labels.clear();
      ports.clear();
      order.clear();
      if (sections) sections.load(file); // zorder hook
      const own = (object, key) => (object && typeof object === 'object' && Object.prototype.hasOwnProperty.call(object, key) ? object[key] : undefined);
      const containersIn = (id, list) => (Array.isArray(list) ? list.filter((node) => isContainer(id, node)) : []);
      const views = own(file, 'views');
      for (const id of byId.keys()) {
        const entry = own(views, id);
        if (!entry || typeof entry !== 'object') continue;
        const folded = containersIn(id, entry.folded);
        const focus = isContainer(id, entry.open) ? entry.open : null;
        if (folded.length > 0 || focus) scopes.set(id, { folded: new Set(folded), focus });
      }
      const arrangements = own(file, 'arrangements');
      for (const entry of Array.isArray(arrangements) ? arrangements : []) {
        const view = own(entry, 'view');
        const placed = own(entry, 'places') || {};
        if (!nodes.has(view) || typeof placed !== 'object') continue;
        const open = own(entry, 'open');
        if (open && !isContainer(view, open)) continue; // made in a container that is gone
        const kept = {};
        for (const node of Object.keys(placed)) {
          if (node !== '__proto__' && nodes.get(view).has(node) && isOffset(placed[node])) kept[node] = asPlacement(placed[node]);
        }
        const key = keyOf(view, { focus: open || null, folded: new Set(containersIn(view, entry.folded)) });
        if (Object.keys(kept).length > 0) places.set(key, kept);
        const bent = own(entry, 'edges');
        const keptRoutes = {};
        const keptStyles = {};
        const keptLabels = {};
        for (const edge of bent && typeof bent === 'object' ? Object.keys(bent) : []) {
          if (edge === '__proto__' || !edges.get(view).has(edge)) continue;
          const points = own(bent[edge], 'points');
          if (isRoute(points)) keptRoutes[edge] = points.map(({ x, y }) => ({ x, y }));
          const style = own(bent[edge], 'style');
          if (STYLES.has(style)) keptStyles[edge] = style;
          const label = own(bent[edge], 'label');
          if (isShift(label) && (label.dx || label.dy)) keptLabels[edge] = { dx: label.dx, dy: label.dy };
        }
        if (Object.keys(keptRoutes).length > 0) routes.set(key, keptRoutes);
        if (Object.keys(keptStyles).length > 0) styles.set(key, keptStyles);
        if (Object.keys(keptLabels).length > 0) labels.set(key, keptLabels);
        const moved = own(entry, 'ports');
        const keptPorts = {};
        for (const port of moved && typeof moved === 'object' ? Object.keys(moved) : []) {
          const shift = moved[port];
          if (port !== '__proto__' && portIds.get(view).has(port) && isShift(shift) && (shift.dx || shift.dy)) keptPorts[port] = { dx: shift.dx, dy: shift.dy };
        }
        if (Object.keys(keptPorts).length > 0) ports.set(key, keptPorts);
        const stacked = own(entry, 'order');
        const keptOrder = {};
        for (const node of stacked && typeof stacked === 'object' ? Object.keys(stacked) : []) {
          const z = stacked[node];
          if (node !== '__proto__' && nodes.get(view).has(node) && Number.isInteger(z) && z !== 0 && Math.abs(z) <= MAX_Z) keptOrder[node] = z;
        }
        if (Object.keys(keptOrder).length > 0) order.set(key, keptOrder);
      }
    }

    function remember() {
      if (!storage) return;
      try {
        storage.setItem(slot, JSON.stringify(toFile()));
      } catch (error) {
        // Private browsing or a full store: the scope still holds for this visit.
      }
    }

    load(initial);
    const fromFile = JSON.stringify(toFile());
    if (storage) {
      try {
        const saved = JSON.parse(storage.getItem(slot));
        if (saved && typeof saved === 'object' && !Array.isArray(saved)) load(saved);
      } catch (error) {
        // Nothing usable was remembered: the file stands.
      }
    }

    return {
      WHOLE,
      toFile,
      /** Whether the arrangement differs from the layout file of the model. */
      dirty: () => JSON.stringify(toFile()) !== fromFile,
      /** Drop the working copy: back to the layout file. */
      revert() {
        load(initial);
        remember();
      },
      foldedOf: (id) => of(id).folded,
      focusOf: (id) => of(id).focus,
      /** Identifies a layout: a view, what is folded in it and what is open. */
      key: (id, scope = of(id)) => keyOf(id, scope),
      /** The graph to lay out for a scope: the focus first, then the folds. */
      shape: (graph, scope = of(graph.id)) => collapse.fold(collapse.focus(graph, scope.focus), new Set([...scope.folded].filter((node) => node !== scope.focus))),
      /** Containers that can fold in what the view shows now. */
      foldable: (id) => collapse.containersOf(collapse.focus(byId.get(id), of(id).focus)).filter((node) => node !== of(id).focus),
      canFocus: (id, node) => isContainer(id, node) && of(id).focus !== node,
      /** The containers around the open one, then the open one. */
      path: (id) => (of(id).focus ? [...collapse.ancestorsOf(byId.get(id), of(id).focus), of(id).focus] : []),
      /** Whether the element is part of what the view shows, folds aside. */
      shows: (id, node) => {
        const focus = of(id).focus;
        return !focus || node === focus || collapse.ancestorsOf(byId.get(id), node).includes(focus);
      },
      /** Where the reader moved boxes in a layout: { node id: { dx, dy } }. */
      placedOf: (id, scope = of(id)) => ({ ...(places.get(keyOf(id, scope)) || {}) }),
      /** Move a box of the layout on screen, on top of earlier moves. */
      moveBy(id, node, dx, dy) {
        if (!nodes.has(id) || !nodes.get(id).has(node) || !isOffset({ dx, dy })) return;
        const key = keyOf(id, of(id));
        const { [node]: was = { dx: 0, dy: 0 }, ...others } = places.get(key) || {};
        const now = asPlacement({ ...was, dx: was.dx + dx, dy: was.dy + dy });
        if (!isOffset(now)) return;
        const next = nothing(now) ? others : { ...others, [node]: now };
        if (Object.keys(next).length > 0) places.set(key, next);
        else places.delete(key);
        remember();
      },
      /** Make a box of the layout on screen wider and taller (negative: smaller). */
      resizeBy(id, node, dw, dh) {
        if (!nodes.has(id) || !nodes.get(id).has(node) || !Number.isFinite(dw) || !Number.isFinite(dh)) return;
        const key = keyOf(id, of(id));
        const { [node]: was = { dx: 0, dy: 0 }, ...others } = places.get(key) || {};
        const now = asPlacement({ ...was, dw: (was.dw || 0) + dw, dh: (was.dh || 0) + dh });
        if (!isOffset(now)) return;
        const next = nothing(now) ? others : { ...others, [node]: now };
        if (Object.keys(next).length > 0) places.set(key, next);
        else places.delete(key);
        remember();
      },
      /**
       * Replace the placement of the layout on screen by where the boxes
       * really went (a container holds back what is dragged out of it), so
       * the next move starts from what the reader sees.
       */
      settle(id, actual) {
        const key = keyOf(id, of(id));
        const kept = {};
        for (const node of Object.keys(actual || {})) {
          if (nodes.get(id).has(node) && isOffset(actual[node]) && !nothing(actual[node])) kept[node] = asPlacement(actual[node]);
        }
        if (Object.keys(kept).length > 0) places.set(key, kept);
        else places.delete(key);
        remember();
      },
      /**
       * A tidy layout as the placement: the boxes where it puts them; bends,
       * moved names and ports dropped (the lines are redrawn); line styles
       * and stacking kept.
       */
      tidy(id, placed) {
        const key = keyOf(id, of(id));
        const kept = {};
        for (const node of Object.keys(placed || {})) {
          if (nodes.has(id) && nodes.get(id).has(node) && isOffset(placed[node]) && !nothing(placed[node])) kept[node] = asPlacement(placed[node]);
        }
        if (Object.keys(kept).length > 0) places.set(key, kept);
        else places.delete(key);
        for (const map of [routes, labels, ports]) map.delete(key);
        remember();
      },
      /** Back to the automatic layout: boxes, bends, styles, names, ports and stacking. */
      resetPlaces(id) {
        const key = keyOf(id, of(id));
        for (const map of [places, routes, styles, labels, ports, order]) map.delete(key);
        remember();
      },
      /** Whether anything of the layout on screen was arranged by hand. */
      arranged: (id, scope = of(id)) => [places, routes, styles, labels, ports, order].some((map) => map.has(keyOf(id, scope))),
      /** What else the reader arranged in a layout, as ArcVizPlace takes it: { styles, labels, ports, order }. */
      extrasOf: (id, scope = of(id)) => {
        const key = keyOf(id, scope);
        return { styles: { ...(styles.get(key) || {}) }, labels: { ...(labels.get(key) || {}) }, ports: { ...(ports.get(key) || {}) }, order: { ...(order.get(key) || {}) } };
      },
      /** How an exchange of the layout on screen is drawn: 'orthogonal' (the default), 'straight' or 'curved'. */
      styleOf: (id, edge) => (styles.get(keyOf(id, of(id))) || {})[edge] || 'orthogonal',
      setStyle(id, edge, style) {
        if (!edges.has(id) || !edges.get(id).has(edge)) return;
        setIn(styles, id, edge, STYLES.has(style) ? style : undefined);
      },
      /** Move the name of an exchange off its line, on top of earlier moves. */
      moveLabelBy(id, edge, dx, dy) {
        if (!edges.has(id) || !edges.get(id).has(edge) || !isShift({ dx, dy })) return;
        const was = (labels.get(keyOf(id, of(id))) || {})[edge] || { dx: 0, dy: 0 };
        const now = { dx: was.dx + dx, dy: was.dy + dy };
        setIn(labels, id, edge, isShift(now) && (now.dx || now.dy) ? now : undefined);
      },
      /** Move a port along the border of its box (the placement puts it back on the border). */
      movePortBy(id, port, dx, dy) {
        if (!portIds.has(id) || !portIds.get(id).has(port) || !isShift({ dx, dy })) return;
        const was = (ports.get(keyOf(id, of(id))) || {})[port] || { dx: 0, dy: 0 };
        const now = { dx: was.dx + dx, dy: was.dy + dy };
        setIn(ports, id, port, isShift(now) && (now.dx || now.dy) ? now : undefined);
      },
      /** Bring a box in front of the others (`front`), or behind them. */
      restack(id, node, front) {
        if (!nodes.has(id) || !nodes.get(id).has(node)) return;
        const all = order.get(keyOf(id, of(id))) || {};
        const values = Object.values(all);
        const z = front ? Math.max(0, ...values) + 1 : Math.min(0, ...values) - 1;
        setIn(order, id, node, Math.abs(z) <= MAX_Z ? z : undefined);
      },
      // zorder hook: the stacking of several boxes at once ({ node: z }, 0 or none: where the layout put it).
      setStacking(id, zs) {
        if (!nodes.has(id)) return;
        const key = keyOf(id, of(id));
        const next = { ...(order.get(key) || {}) };
        for (const node of Object.keys(zs || {})) {
          if (node === '__proto__' || !nodes.get(id).has(node)) continue;
          const z = zs[node];
          if (Number.isInteger(z) && z !== 0 && Math.abs(z) <= MAX_Z) next[node] = z;
          else delete next[node];
        }
        if (Object.keys(next).length > 0) order.set(key, next);
        else order.delete(key);
        remember();
      },
      /** The points an exchange of the layout on screen was bent through, [] when none. */
      routeOf: (id, edge, scope = of(id)) => ((routes.get(keyOf(id, scope)) || {})[edge] || []).map(({ x, y }) => ({ x, y })),
      /** Where the reader bent exchanges in a layout: { edge id: [{ x, y }] }. */
      routesOf: (id, scope = of(id)) => ({ ...(routes.get(keyOf(id, scope)) || {}) }),
      /** Bend an exchange of the layout on screen through `points`; none (or []) straightens it. */
      setRoute(id, edge, points) {
        if (!edges.has(id) || !edges.get(id).has(edge)) return;
        const key = keyOf(id, of(id));
        const { [edge]: was, ...others } = routes.get(key) || {};
        const next = isRoute(points) ? { ...others, [edge]: points.map(({ x, y }) => ({ x, y })) } : others;
        if (Object.keys(next).length > 0) routes.set(key, next);
        else routes.delete(key);
        remember();
      },
      setFolded(id, folded) {
        scopes.set(id, { folded: new Set([...folded].filter((node) => isContainer(id, node))), focus: of(id).focus });
        remember();
      },
      setFocus(id, node) {
        scopes.set(id, { folded: new Set(of(id).folded), focus: isContainer(id, node) ? node : null });
        remember();
      },
    };
  }

  return { create };
});
