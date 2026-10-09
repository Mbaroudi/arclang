/*
 * ArcViz navigation — moving through the model rather than through tabs.
 *
 * Two pieces of chrome, both built from ArcVizLinks:
 *  - the model tree: every view with the nesting of what it draws;
 *  - "across layers": for the selected element, the other views that draw
 *    it and the elements it is traced to, each one a way to go there.
 *
 * Built with DOM APIs and textContent only: model text is never markup.
 */
(function (root) {
  'use strict';

  /**
   * @param {object} deps
   * @param {Function} deps.el            element builder of the viewer
   * @param {object}   deps.model         ArcVizLinks.index(...)
   * @param {Function} deps.placeLabel    ({graph, view, title}) -> tab name
   * @param {Function} deps.nodeStyle     (kind) -> { fill, stroke, label }
   * @param {Function} deps.goTo          (graphId, elementId) -> Promise
   */
  function create(deps) {
    const { el, model, placeLabel, nodeStyle, goTo } = deps;

    // --- model tree -----------------------------------------------------
    const groups = new Map(); // graph id -> <details>
    const entries = new Map(); // `${graph}\n${element}` -> <button>
    const key = (graph, id) => `${graph}\n${id}`;

    const item = (graph, node) => {
      const style = nodeStyle(node.kind);
      const swatch = el('span', { class: 'arcviz-swatch', 'aria-hidden': 'true' });
      swatch.style.background = style.fill;
      swatch.style.borderColor = style.stroke;
      const button = el('button', {
        type: 'button',
        class: 'arcviz-tree-item',
        title: `${style.label}: ${node.name}`,
        onclick: () => goTo(graph, node.id),
      }, [swatch, el('span', { text: node.name || style.label })]);
      entries.set(key(graph, node.id), button);
      const children = node.children.length > 0 ? [el('ul', {}, node.children.map((child) => item(graph, child)))] : [];
      return el('li', {}, [button, ...children]);
    };

    const tree = el('nav', { class: 'arcviz-tree', 'aria-label': 'Model tree' }, [
      el('h3', { class: 'arcviz-tree-title', text: 'Model' }),
      ...model.outline().map((entry) => {
        const count = entries.size;
        const list = el('ul', {}, entry.nodes.map((node) => item(entry.graph, node)));
        const summary = el('summary', {}, [
          el('span', { text: placeLabel(entry) }),
          el('span', { class: 'arcviz-count', text: String(entries.size - count) }),
        ]);
        const group = el('details', { class: 'arcviz-tree-view' }, [summary, list]);
        groups.set(entry.graph, group);
        return group;
      }),
    ]);

    /** Open the active view's branch and mark the selected element. */
    let shown = null;
    function mark(graph, id) {
      // The tree follows the view on screen: its branch opens and the others
      // close when the view changes, not on every selection, so a branch
      // the reader opened by hand stays open.
      if (shown !== graph) {
        for (const [other, group] of groups) group.open = other === graph;
        shown = graph;
      }
      for (const button of entries.values()) button.removeAttribute('aria-current');
      const current = id && entries.get(key(graph, id));
      if (!current) return;
      current.setAttribute('aria-current', 'true');
      for (let up = current.parentElement; up && up !== tree; up = up.parentElement) {
        if (up.tagName === 'DETAILS') up.open = true;
      }
      if (current.scrollIntoView) current.scrollIntoView({ block: 'nearest' });
    }

    // --- across layers --------------------------------------------------
    const jump = (place, id, text) => el('button', {
      type: 'button',
      class: 'arcviz-jump',
      'data-graph': place.graph,
      'data-element': id,
      text,
      onclick: () => goTo(place.graph, id),
    });

    /**
     * What leads out of the element `id`, shown in view `graph`. The
     * section is always there, so its absence never has to be interpreted.
     */
    function related(graph, id) {
      const elsewhere = model.placesOf(id).filter((place) => place.graph !== graph);
      const relations = model.relationsOf(id);
      const heading = el('h4', { text: 'Across layers' });
      if (elsewhere.length === 0 && relations.length === 0) {
        return el('section', { class: 'arcviz-related', 'aria-label': 'Across layers' }, [
          heading,
          el('p', { class: 'arcviz-quiet', text: 'No other view draws this element and no trace names it.' }),
        ]);
      }

      const list = el('dl', {});
      if (elsewhere.length > 0) {
        list.appendChild(el('dt', { text: 'Also shown in' }));
        list.appendChild(el('dd', {}, elsewhere.map((place) => jump(place, id, placeLabel(place)))));
      }
      // One heading per kind of relation, however many elements it reaches.
      const byLabel = new Map();
      for (const relation of relations) byLabel.set(relation.label, [...(byLabel.get(relation.label) || []), relation]);
      for (const [label, group] of byLabel) {
        list.appendChild(el('dt', { text: label }));
        const ways = group.flatMap((relation) => (relation.places.length > 0
          ? relation.places.map((place) => jump(place, relation.id, `${relation.name} — ${placeLabel(place)}`))
          : [el('span', { class: 'arcviz-quiet', text: `${relation.name} (not on a diagram)` })]));
        list.appendChild(el('dd', {}, ways));
      }
      return el('section', { class: 'arcviz-related', 'aria-label': 'Across layers' }, [
        heading,
        list,
      ]);
    }

    /**
     * The way into an open container: the whole view, each container around
     * the open one, then the open one. `open(id)` goes to a step; `null` is
     * the whole view. An empty path hides the host.
     */
    function crumbs(host, whole, path, open) {
      host.textContent = '';
      host.hidden = path.length === 0;
      if (path.length === 0) return;
      const step = (text, target) => el('button', { type: 'button', class: 'arcviz-crumb', text, onclick: () => open(target) });
      host.appendChild(step(whole, null));
      path.forEach((id, index) => {
        host.appendChild(el('span', { class: 'arcviz-crumb-rule', 'aria-hidden': 'true', text: '/' }));
        host.appendChild(index === path.length - 1
          ? el('span', { class: 'arcviz-crumb-here', 'aria-current': 'location', text: model.nameOf(id) })
          : step(model.nameOf(id), id));
      });
    }

    /** The key to what a view draws: one entry per kind of box and of line. */
    function legend(host, nodeKinds, edgeKinds, edgeStyle) {
      host.textContent = '';
      for (const kind of nodeKinds) {
        const style = nodeStyle(kind);
        const swatch = el('span', { class: 'arcviz-swatch' });
        swatch.style.background = style.fill;
        swatch.style.borderColor = style.stroke;
        host.appendChild(el('span', { class: 'arcviz-key' }, [swatch, el('span', { text: style.label })]));
      }
      for (const kind of edgeKinds) {
        const style = edgeStyle(kind);
        const line = el('span', { class: 'arcviz-line' });
        line.style.borderTopColor = style.stroke;
        line.style.borderTopStyle = style.dash ? 'dashed' : 'solid';
        line.style.borderTopWidth = `${Math.max(2, Math.round(style.width))}px`;
        host.appendChild(el('span', { class: 'arcviz-key' }, [line, el('span', { text: style.label })]));
      }
    }

    return { tree, mark, related, crumbs, legend, nameOf: model.nameOf };
  }

  root.ArcVizNavigation = { create };
})(typeof globalThis !== 'undefined' ? globalThis : this);
