/*
 * ArcViz collapse — folding a container into a single box.
 *
 * Pure functions over a layout graph, no DOM. A folded container keeps its
 * name, its own ports and a count of what it hides; exchanges of the hidden
 * elements end on the container; exchanges wholly inside it disappear with
 * their ends. The input graph is never modified.
 */
(function (root, factory) {
  const api = factory();
  if (typeof module === 'object' && module.exports) module.exports = api;
  else root.ArcVizCollapse = api;
})(typeof self !== 'undefined' ? self : this, function () {
  'use strict';

  const CHAR_WIDTH = 7.2;
  const MIN_WIDTH = 112;
  const MIN_HEIGHT = 42;
  const HORIZONTAL_PADDING = 44;
  const PORT_PITCH = 20;

  // A folded box has no inside to pad, but its ports keep their sides.
  const withoutPadding = (options) => Object.fromEntries(Object.entries(options || {}).filter(([key]) => key !== 'elk.padding'));

  const isContainer = (node) => Array.isArray(node.children) && node.children.length > 0;

  /** Ids of the nodes that hold others, outermost first. */
  function containersOf(graph) {
    const found = [];
    const walk = (node) => {
      if (!isContainer(node)) return;
      found.push(node.id);
      node.children.forEach(walk);
    };
    (graph.children || []).forEach(walk);
    return found;
  }

  /** Ids of the containers around `id`, outermost first. */
  function ancestorsOf(graph, id) {
    const search = (nodes, path) => {
      for (const node of nodes || []) {
        if (node.id === id) return path;
        const below = search(node.children, [...path, node.id]);
        if (below) return below;
      }
      return null;
    };
    return search(graph.children, []) || [];
  }

  /**
   * The graph with the containers in `folded` (a Set of ids) shown as
   * single boxes.
   */
  function fold(graph, folded) {
    if (!folded || folded.size === 0) return graph;
    // Node id or port id -> the folded container that now stands for it.
    const standIn = new Map();
    const hide = (node, container) => {
      standIn.set(node.id, container);
      for (const port of node.ports || []) standIn.set(port.id, container);
      (node.children || []).forEach((child) => hide(child, container));
    };
    const count = (node) => (node.children || []).reduce((sum, child) => sum + 1 + count(child), 0);

    const copy = (node) => {
      if (!isContainer(node)) return node;
      if (!folded.has(node.id)) return { ...node, children: node.children.map(copy) };
      node.children.forEach((child) => hide(child, node.id));
      const name = (node.labels && node.labels[0] && node.labels[0].text) || '';
      const { children, ...rest } = node;
      return {
        ...rest,
        width: Math.max(MIN_WIDTH, name.length * CHAR_WIDTH + HORIZONTAL_PADDING),
        height: Math.max(MIN_HEIGHT, (node.ports || []).length * PORT_PITCH + 16),
        layoutOptions: withoutPadding(node.layoutOptions),
        arc: { ...node.arc, folded: count(node) },
      };
    };
    const children = (graph.children || []).map(copy);

    // The folded containers and their own ports: ends that stay where they
    // are, but belong to the box.
    const ownPorts = new Map();
    const notePorts = (node) => {
      if (folded.has(node.id) && node.arc && node.arc.folded !== undefined) {
        ownPorts.set(node.id, node.id);
        for (const port of node.ports || []) ownPorts.set(port.id, node.id);
      }
      (node.children || []).forEach(notePorts);
    };
    children.forEach(notePorts);

    const moved = (ends) => [...new Set(ends.map((end) => standIn.get(end) || end))];
    const edges = [];
    for (const edge of graph.edges || []) {
      const touched = [...edge.sources, ...edge.targets].some((end) => standIn.has(end));
      if (!touched) {
        edges.push(edge);
        continue;
      }
      const sources = moved(edge.sources);
      const targets = moved(edge.targets);
      // An exchange that now starts and ends on one folded box — hidden
      // element to hidden element, or to the box's own port — was inside it.
      const boxes = new Set([...edge.sources, ...edge.targets].map((end) => standIn.get(end) || ownPorts.get(end)));
      if (boxes.size === 1 && !boxes.has(undefined)) continue;
      edges.push({
        ...edge,
        sources,
        targets,
        arc: {
          ...edge.arc,
          source_node: standIn.get(edge.sources[0]) || edge.arc.source_node,
          target_node: standIn.get(edge.targets[0]) || edge.arc.target_node,
        },
      });
    }
    return { ...graph, children, edges };
  }

  /**
   * The container `id` as a diagram of its own: its whole content, and
   * around it, folded, the elements it exchanges with. Exchanges between
   * those other elements are left out: the diagram is about the container.
   */
  function focus(graph, id) {
    const find = (nodes) => {
      for (const node of nodes || []) {
        if (node.id === id) return node;
        const below = find(node.children);
        if (below) return below;
      }
      return null;
    };
    const target = id ? find(graph.children) : null;
    if (!target || !isContainer(target)) return graph;

    const around = new Set(ancestorsOf(graph, id));
    const inside = new Set();
    const nodeOf = new Map(); // node id or port id -> node id
    // For an element outside the target: the largest box that holds it
    // without holding the target.
    const contextOf = new Map();
    const nodes = new Map();
    const walk = (node, within, context) => {
      nodes.set(node.id, node);
      const here = within || node.id === id;
      const box = here || around.has(node.id) ? null : context || node.id;
      for (const key of [node.id, ...(node.ports || []).map((port) => port.id)]) {
        nodeOf.set(key, node.id);
        if (here) inside.add(key);
        else if (box) contextOf.set(key, box);
      }
      (node.children || []).forEach((child) => walk(child, here, box));
    };
    (graph.children || []).forEach((node) => walk(node, false, null));

    const context = []; // in the order the exchanges name them
    const edges = [];
    const leftOut = [];
    for (const edge of graph.edges || []) {
      const ends = [...edge.sources, ...edge.targets];
      if (!ends.some((end) => inside.has(end))) continue;
      // An end on a box around the target has no place here: the exchange
      // is named so the reader knows it exists.
      if (ends.some((end) => !inside.has(end) && !contextOf.has(end))) {
        leftOut.push({ id: edge.id, label: (edge.labels && edge.labels[0] && edge.labels[0].text) || edge.id });
        continue;
      }
      const placed = (end) => {
        if (inside.has(end)) return end;
        const box = contextOf.get(end);
        if (!context.includes(box)) context.push(box);
        return box;
      };
      const sources = [...new Set(edge.sources.map(placed))];
      const targets = [...new Set(edge.targets.map(placed))];
      edges.push({
        ...edge,
        sources,
        targets,
        arc: {
          ...edge.arc,
          source_node: inside.has(edge.sources[0]) ? edge.arc.source_node : sources[0],
          target_node: inside.has(edge.targets[0]) ? edge.arc.target_node : targets[0],
        },
      });
    }

    const order = [...nodes.keys()];
    context.sort((a, b) => order.indexOf(a) - order.indexOf(b));
    const boxes = context.map((box) => {
      const node = nodes.get(box);
      const shown = isContainer(node) ? fold({ children: [node], edges: [] }, new Set([box])).children[0] : node;
      // Ports of a context box are not ends of anything drawn here.
      return { ...shown, ports: [], arc: { ...shown.arc, context: true } };
    });
    return { ...graph, children: [target, ...boxes], edges, arc: { ...graph.arc, left_out: leftOut } };
  }

  return { fold, focus, containersOf, ancestorsOf };
});
