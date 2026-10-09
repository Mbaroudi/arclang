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
   */
  function create(deps) {
    const { graphs, collapse, storage, storageKey, initial } = deps;
    const whole = () => ({ folded: new Set(), focus: null });
    const WHOLE = Object.freeze(whole());
    const byId = new Map(graphs.map((graph) => [graph.id, graph]));
    const containers = new Map(graphs.map((graph) => [graph.id, new Set(collapse.containersOf(graph))]));
    const scopes = new Map(); // graph id -> { folded: Set, focus: ?string }
    // Layout key -> { node id: { dx, dy } }: a placement belongs to one
    // layout, so each fold or open container has its own.
    const places = new Map();
    const nodes = new Map(graphs.map((graph) => {
      const ids = new Set();
      const walk = (node) => {
        ids.add(node.id);
        (node.children || []).forEach(walk);
      };
      (graph.children || []).forEach(walk);
      return [graph.id, ids];
    }));
    // The open container is not folded in its own diagram: it is not part
    // of what tells one layout from another.
    const keyOf = (id, scope) => JSON.stringify([id, scope.focus, ...[...scope.folded].filter((node) => node !== scope.focus).sort()]);
    const FARTHEST = 100000; // sheet units: beyond this an offset is not a placement
    const isOffset = (value) => Boolean(value) && Number.isFinite(value.dx) && Number.isFinite(value.dy) && Math.abs(value.dx) <= FARTHEST && Math.abs(value.dy) <= FARTHEST;

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
      const arrangements = [...places.keys()].sort().map((key) => {
        const [view, open, ...folded] = JSON.parse(key);
        const placed = places.get(key);
        return { view, open, folded, places: Object.fromEntries(Object.keys(placed).sort().map((node) => [node, placed[node]])) };
      });
      return { arclang_layout: LAYOUT_VERSION, views, arrangements };
    }

    /** Replace the arrangement by what a layout says, piece by valid piece. */
    function load(file) {
      scopes.clear();
      places.clear();
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
        const placed = own(entry, 'places');
        if (!nodes.has(view) || !placed || typeof placed !== 'object') continue;
        const open = own(entry, 'open');
        if (open && !isContainer(view, open)) continue; // made in a container that is gone
        const kept = {};
        for (const node of Object.keys(placed)) {
          if (node !== '__proto__' && nodes.get(view).has(node) && isOffset(placed[node])) kept[node] = { dx: placed[node].dx, dy: placed[node].dy };
        }
        const key = keyOf(view, { focus: open || null, folded: new Set(containersIn(view, entry.folded)) });
        if (Object.keys(kept).length > 0) places.set(key, kept);
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
        const now = { dx: was.dx + dx, dy: was.dy + dy };
        if (!isOffset(now)) return;
        const next = now.dx === 0 && now.dy === 0 ? others : { ...others, [node]: now };
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
          if (nodes.get(id).has(node) && isOffset(actual[node]) && (actual[node].dx || actual[node].dy)) kept[node] = { dx: actual[node].dx, dy: actual[node].dy };
        }
        if (Object.keys(kept).length > 0) places.set(key, kept);
        else places.delete(key);
        remember();
      },
      resetPlaces(id) {
        places.delete(keyOf(id, of(id)));
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
