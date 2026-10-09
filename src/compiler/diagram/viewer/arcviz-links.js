/*
 * ArcViz links — what ties the views together.
 *
 * Pure functions over the diagram payload, no DOM: which views draw an
 * element, which elements it is traced to, and the nesting of each view.
 * Shared by the browser viewer and the Node tests.
 */
(function (root, factory) {
  const api = factory();
  if (typeof module === 'object' && module.exports) module.exports = api;
  else root.ArcVizLinks = api;
})(typeof self !== 'undefined' ? self : this, function () {
  'use strict';

  // How a trace reads from its target back to its source.
  const INCOMING = {
    realizes: 'Realized by',
    satisfies: 'Satisfied by',
    implements: 'Implemented by',
    refines: 'Refined by',
    verifies: 'Verified by',
    validates: 'Validated by',
    allocates: 'Allocated by',
    derives: 'Derived by',
    traces: 'Traced from',
  };

  const capitalized = (text) => text.charAt(0).toUpperCase() + text.slice(1).replace(/_/g, ' ');
  const outgoingLabel = (kind) => capitalized(kind);
  // A trace type is the author's text: `constructor` must not find a built-in.
  const incomingLabel = (kind) => (Object.prototype.hasOwnProperty.call(INCOMING, kind) ? INCOMING[kind] : `${capitalized(kind)} (from)`);

  /**
   * Index the payload.
   * @param {Array<object>} graphs ELK graphs of every view, in tab order.
   * @param {Array<{kind: string, source: string, target: string}>} links traces of the model.
   */
  function index(graphs, links) {
    const places = new Map(); // element id -> [{ graph, view, title }]
    const names = new Map(); // element id -> display name
    const relations = new Map(); // element id -> [{ label, id }]
    const relate = (id, label, other) => {
      const list = relations.get(id) || [];
      if (!list.some((known) => known.label === label && known.id === other)) list.push({ label, id: other });
      relations.set(id, list);
    };

    const outlineOf = (node) => ({
      id: node.id,
      name: (node.labels && node.labels[0] && node.labels[0].text) || node.id,
      kind: (node.arc || {}).kind,
      children: (node.children || []).map(outlineOf),
    });
    const outline = graphs.map((graph) => ({
      graph: graph.id,
      view: graph.arc.view,
      title: graph.arc.title,
      nodes: (graph.children || []).map(outlineOf),
    }));

    for (const entry of outline) {
      const place = { graph: entry.graph, view: entry.view, title: entry.title };
      const walk = (node) => {
        const known = places.get(node.id) || [];
        if (!known.some((other) => other.graph === place.graph)) known.push(place);
        places.set(node.id, known);
        if (!names.has(node.id)) names.set(node.id, node.name);
        node.children.forEach(walk);
      };
      entry.nodes.forEach(walk);
    }
    // A component shown on the node hosting it realizes the logical one.
    const realizing = (node) => {
      for (const realized of (node.arc && node.arc.realizes) || []) {
        relate(node.id, outgoingLabel('realizes'), realized);
        relate(realized, incomingLabel('realizes'), node.id);
      }
      (node.children || []).forEach(realizing);
    };
    for (const link of links || []) {
      relate(link.source, outgoingLabel(link.kind), link.target);
      relate(link.target, incomingLabel(link.kind), link.source);
    }
    for (const graph of graphs) (graph.children || []).forEach(realizing);

    const placesOf = (id) => places.get(id) || [];
    const nameOf = (id) => names.get(id) || id;
    return {
      placesOf,
      nameOf,
      /** Elements `id` is traced to or from, each with the views drawing it. */
      relationsOf: (id) => (relations.get(id) || []).map((relation) => ({
        label: relation.label,
        id: relation.id,
        name: nameOf(relation.id),
        places: placesOf(relation.id),
      })),
      outline: () => outline,
    };
  }

  return { index };
});
