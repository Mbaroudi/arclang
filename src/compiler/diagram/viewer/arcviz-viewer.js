/*
 * ArcViz viewer — the interactive shell around the renderer: one tab per
 * Arcadia viewpoint, a model tree, links from an element to the other
 * layers, folding of containers, a container opened as a diagram of its
 * own, functional-chain highlighting, pan and
 * zoom, element details,
 * SVG/PNG export and print pages.
 *
 * Chrome is built with DOM APIs and textContent. The only markup injected
 * as a string is the SVG from ArcVizRender, which escapes every value.
 */
(function (root) {
  'use strict';

  const VIEW_NAMES = {
    oab: 'Operational',
    sab: 'System',
    lab: 'Logical',
    pab: 'Physical',
    msm: 'Modes and states',
    es: 'Scenarios',
    cap: 'Capabilities',
    cdb: 'Data',
    pbs: 'Product breakdown',
  };
  const NUDGE = 10; // sheet units a box moves per key press
  const HISTORY_LIMIT = 50;

  function el(tag, attributes, children) {
    const node = document.createElement(tag);
    for (const [key, value] of Object.entries(attributes || {})) {
      if (value === null || value === undefined || value === false) continue;
      if (key === 'text') node.textContent = value;
      else if (key === 'class') node.className = value;
      else if (key.startsWith('on')) node.addEventListener(key.slice(2), value);
      else node.setAttribute(key, value === true ? '' : value);
    }
    for (const child of children || []) if (child) node.appendChild(child);
    return node;
  }

  function download(blob, filename) {
    const url = URL.createObjectURL(blob);
    const link = el('a', { href: url, download: filename });
    document.body.appendChild(link);
    link.click();
    link.remove();
    setTimeout(() => URL.revokeObjectURL(url), 1000);
  }

  function slug(text) {
    return String(text).toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '') || 'diagram';
  }

  /** What to call an element in lists and headings: its label, else its kind. */
  function displayName(item, kindLabel) {
    const text = item.labels && item.labels[0] ? item.labels[0].text : '';
    return text || kindLabel;
  }

  function indexGraph(graph) {
    const nodes = new Map();
    const walk = (node, parent) => {
      nodes.set(node.id, { node, parent });
      for (const child of node.children || []) walk(child, node.id);
    };
    for (const child of graph.children || []) walk(child, null);
    const edges = new Map((graph.edges || []).map((edge) => [edge.id, edge]));
    return { nodes, edges };
  }

  /**
   * Mount the viewer in `host`.
   * payload: { title, graphs: [ELK graph], diagnostics: [string] }
   * deps:    { ELK, render, links, navigation, collapse, scope, exporter,
   *          stage, place } — the layout engine constructor and the other
   *          ArcViz modules, each one named after its file.
   */
  function mount(host, payload, deps) {
    const graphs = payload.graphs || [];
    const diagnostics = payload.diagnostics || [];
    const render = deps.render;
    const state = { active: null, views: new Map(), layouts: new Map(), arranging: false, chain: null, selected: null, history: [], settled: new Map() };

    host.textContent = '';
    host.classList.add('arcviz');

    if (graphs.length === 0) {
      host.appendChild(el('p', {
        class: 'arcviz-empty',
        text: 'This model has nothing to draw yet. Declare actors, functions, components or nodes to get a diagram.',
      }));
      return { ready: Promise.resolve(), svgOf: () => null };
    }

    // --- chrome ---------------------------------------------------------
    // One tab per kind of view. A kind drawn once per machine or scenario
    // holds several diagrams, chosen in a list under the tabs.
    const kinds = [...new Set(graphs.map((graph) => graph.arc.view))];
    const ofKind = (kind) => graphs.filter((graph) => graph.arc.view === kind);
    const lastShown = new Map();
    const showKind = (kind) => show(lastShown.get(kind) || ofKind(kind)[0].id);
    const tabs = kinds.map((kind) => el('button', {
      type: 'button',
      role: 'tab',
      class: 'arcviz-tab',
      id: `arcviz-tab-${kind}`,
      'aria-controls': 'arcviz-stage',
      'aria-selected': 'false',
      tabindex: '-1',
      'data-view': kind,
      text: VIEW_NAMES[kind] || kind,
      onclick: () => showKind(kind),
    }));
    const tablist = el('div', { role: 'tablist', class: 'arcviz-tabs', 'aria-label': 'Architecture viewpoints' }, tabs);
    tablist.addEventListener('keydown', (event) => {
      const index = tabs.indexOf(document.activeElement);
      const target = {
        ArrowRight: (index + 1) % tabs.length,
        ArrowLeft: (index - 1 + tabs.length) % tabs.length,
        Home: 0,
        End: tabs.length - 1,
      }[event.key];
      if (index < 0 || target === undefined) return;
      const next = tabs[target];
      next.focus();
      showKind(next.dataset.view);
      event.preventDefault();
    });

    // A tab's name, or `Tab: Diagram` when its kind holds several diagrams.
    const placeLabel = (place) => {
      const name = VIEW_NAMES[place.view] || place.view;
      return ofKind(place.view).length > 1 ? `${name}: ${place.title}` : name;
    };
    const nav = deps.navigation.create({
      el,
      model: deps.links.index(graphs, payload.links || []),
      placeLabel,
      nodeStyle: render.nodeStyle,
      goTo: (graph, id) => goTo(graph, id),
    });

    const tool = (label, title, action) => el('button', { type: 'button', class: 'arcviz-tool', title, text: label, onclick: action });
    const back = tool('Back', 'Return to the element you came from', () => goBack());
    back.disabled = true;
    const treeToggle = tool('Model tree', 'Show or hide the model tree', () => toggleTree());
    treeToggle.setAttribute('aria-pressed', 'true');
    const arrange = tool('Arrange', 'Move boxes by hand: drag one, or select it and press Shift with an arrow key', () => toggleArrange());
    arrange.setAttribute('aria-pressed', 'false');
    const resetPlaces = tool('Reset layout', 'Put every box of this view back where the automatic layout placed it', () => replace(() => scope.resetPlaces(state.active)).then(() => announce('Layout reset.')));
    const layoutName = payload.layout_name || `${slug(payload.title)}.layout.json`;
    const saveLayout = tool('Save layout', `Download the folds, the open containers and the placement as ${layoutName}. Kept next to the model, it is what everyone sees.`, () => exporter.saveLayout(scope.toFile(), layoutName));
    const foldAll = tool('Fold all', 'Show every container as a single box', () => refold(new Set(scope.foldable(state.active))));
    const unfoldAll = tool('Unfold all', 'Show what every container holds', () => refold(new Set()));
    const toolbar = el('div', { class: 'arcviz-tools', role: 'toolbar', 'aria-label': 'Diagram tools' }, [
      treeToggle,
      back,
      foldAll,
      unfoldAll,
      arrange,
      resetPlaces,
      tool('Fit', 'Fit the diagram to the frame (0)', () => fit()),
      tool('−', 'Zoom out (-)', () => sheet.zoomOut()),
      tool('+', 'Zoom in (+)', () => sheet.zoomIn()),
      tool('Full screen', 'Show the diagram full screen', () => sheet.toggleFullscreen()),
      tool('Save SVG', 'Download this view as drawn, folded containers included, as an SVG file', () => saveSvg()),
      saveLayout,
      tool('Save PNG', 'Download this view as drawn, folded containers included, as a PNG image', () => savePng()),
    ]);

    const title = el('p', { class: 'arcviz-title' });
    const which = el('select', { id: 'arcviz-which', onchange: (event) => show(event.currentTarget.value) });
    const whichRow = el('div', { class: 'arcviz-pick arcviz-which' }, [
      el('label', { for: 'arcviz-which', class: 'arcviz-label', text: 'Diagram' }),
      which,
    ]);
    const canvas = el('div', { class: 'arcviz-canvas' });
    const stage = el('div', {
      class: 'arcviz-stage',
      id: 'arcviz-stage',
      role: 'tabpanel',
      tabindex: '0',
      'aria-describedby': 'arcviz-hint',
    }, [canvas]);
    const hint = el('p', {
      class: 'arcviz-hint',
      id: 'arcviz-hint',
      text: 'Drag to pan, or use the arrow keys. Zoom with + and -, or hold Ctrl or ⌘ and scroll. Press 0 to fit. Double-click a container, or select it and press Enter, to fold or unfold it. With Arrange on, drag a box to place it.',
    });
    // Everything drawn is also listed here, so an element can be chosen
    // without a pointer.
    const picker = el('select', { id: 'arcviz-pick', onchange: (event) => selectById(event.currentTarget.value) });
    const pick = el('div', { class: 'arcviz-pick' }, [
      el('label', { for: 'arcviz-pick', class: 'arcviz-label', text: 'Show details of' }),
      picker,
    ]);
    const chains = el('div', { class: 'arcviz-chains' });
    const details = el('div', { class: 'arcviz-details', 'aria-live': 'polite' });
    const legend = el('div', { class: 'arcviz-legend' });
    const notes = el('div', { class: 'arcviz-notes' });
    const printHost = el('div', { class: 'arcviz-print', 'aria-hidden': 'true' });

    host.appendChild(el('div', { class: 'arcviz-bar' }, [tablist, toolbar]));
    // Where the reader is inside a view once a container is open.
    const crumbs = el('nav', { class: 'arcviz-crumbs', 'aria-label': 'Location in the model', hidden: true });
    // What changed on screen, for readers who do not see it change.
    const status = el('p', { class: 'arcviz-status', role: 'status' });
    // Cleared first: the same words twice in a row would not be read again.
    const announce = (text) => {
      status.textContent = '';
      setTimeout(() => { status.textContent = text; }, 30);
    };
    host.appendChild(title);
    host.appendChild(whichRow);
    host.appendChild(crumbs);
    host.appendChild(status);
    host.appendChild(chains);
    // Tree and details share the column beside the sheet, like a project
    // explorer over a properties view: what is selected, and where it
    // leads, stays in sight while the drawing is read.
    const side = el('div', { class: 'arcviz-side' }, [nav.tree, details]);
    const work = el('div', { class: 'arcviz-work' }, [side, stage]);
    host.appendChild(work);
    host.appendChild(hint);
    host.appendChild(pick);
    host.appendChild(el('div', { class: 'arcviz-below' }, [legend]));
    host.appendChild(notes);
    host.appendChild(printHost);

    // --- layout ---------------------------------------------------------
    const activeGraph = () => graphs.find((graph) => graph.id === state.active);
    let storage = null;
    try {
      storage = root.localStorage || null;
    } catch (error) {
      // Storage is refused: folds hold for this visit only.
    }
    const scope = deps.scope.create({ graphs, collapse: deps.collapse, storage, storageKey: `arcviz:${payload.title}`, initial: payload.layout || null });
    const foldedOf = scope.foldedOf;

    // A view is laid out once per scope: what is folded, what is open.
    // The automatic layout is computed once; the reader's placement is
    // applied on top of it each time a box is moved.
    function layout(graph, shown) {
      const key = scope.key(graph.id, shown);
      if (state.views.has(key)) return state.views.get(key);
      if (!state.layouts.has(key)) {
        // Scenarios are drawn as declared; everything else goes through ELK.
        const copy = JSON.parse(JSON.stringify(scope.shape(graph, shown)));
        state.layouts.set(key, render.isSequence(graph) ? Promise.resolve(copy) : new deps.ELK().layout(copy));
      }
      const pending = state.layouts.get(key).then((laidOut) => {
        const placed = render.isSequence(graph) ? laidOut : deps.place.place(laidOut, scope.placedOf(graph.id, shown));
        return { graph: placed, index: indexGraph(placed), ...render.renderSvg(placed) };
      });
      state.views.set(key, pending);
      return pending;
    }

    /** The active view once laid out, or null if it failed or is absent. */
    async function activeView() {
      const pending = state.views.get(scope.key(state.active));
      if (!pending) return null;
      try {
        return await pending;
      } catch (error) {
        return null;
      }
    }

    // --- navigation -----------------------------------------------------
    function toggleTree() {
      const hidden = work.classList.toggle('is-treeless');
      treeToggle.setAttribute('aria-pressed', String(!hidden));
      setTimeout(fit, 0);
    }

    /** Bring a drawn element to the middle of the frame. */
    function center(id) {
      sheet.centerOn([...canvas.querySelectorAll('.av-node')].find((item) => item.dataset.id === id));
    }

    // --- folding --------------------------------------------------------
    const canFold = (id) => scope.foldable(state.active).includes(id);

    /** Change the scope of a view, dropping the layout it replaces. */
    function rescope(id, change) {
      // The whole view is kept for printing; any other is recomputed.
      const before = scope.key(id);
      if (before !== scope.key(id, scope.WHOLE)) {
        state.views.delete(before);
        state.layouts.delete(before);
      }
      change();
    }

    /** Redraw the active view, keeping the selection and the chain. */
    async function redraw() {
      const id = state.active;
      const keep = state.selected;
      const chain = state.chain;
      await show(id);
      if (state.active !== id) return;
      const button = chain && [...chains.querySelectorAll('.arcviz-chain')].find((item) => item.dataset.chain === chain);
      if (button) button.click();
      if (keep) await selectById(keep);
    }

    function refold(next) {
      rescope(state.active, () => scope.setFolded(state.active, next));
      return redraw();
    }

    async function toggleFold(id) {
      if (!canFold(id)) return;
      const view = state.active;
      const next = new Set(foldedOf(view));
      const unfolding = next.delete(id);
      if (!unfolding) next.add(id);
      await refold(next);
      if (state.active !== view) return;
      await selectById(id);
      center(id);
      announce(`${nav.nameOf(id)} ${unfolding ? 'unfolded' : 'folded'}.`);
      // The button that was pressed is redrawn with the details.
      const again = details.querySelector('.arcviz-fold');
      if (again) again.focus({ preventScroll: true });
    }

    // --- manual placement ----------------------------------------------
    /** What moves with a box: itself and what it holds. Lifelines stay put. */
    function carried(id) {
      const graph = activeGraph();
      if (!graph || render.isSequence(graph)) return [];
      const held = [];
      const collect = (node, inside) => {
        const here = inside || node.id === id;
        if (here) held.push(node.id);
        (node.children || []).forEach((child) => collect(child, here));
      };
      (scope.shape(graph).children || []).forEach((node) => collect(node, false));
      return held;
    }

    /** Redraw the active view after its placement changed, where it is. */
    async function replace(change) {
      const view = state.active;
      const where = sheet.position();
      state.views.delete(scope.key(view));
      change();
      await redraw();
      if (state.active !== view) return;
      sheet.restore(where);
      const shown = await activeView();
      if (shown && shown.graph.arc.placed) scope.settle(view, shown.graph.arc.placed);
      // Print pages show each view whole, as the reader arranged it.
      try {
        state.settled.set(view, await layout(graphs.find((graph) => graph.id === view), scope.WHOLE));
      } catch (error) {
        // The view reports its own layout failure when opened.
      }
    }

    async function moveBy(id, dx, dy) {
      await replace(() => scope.moveBy(state.active, id, dx, dy));
      announce(`${nav.nameOf(id)} moved.`);
    }

    function toggleArrange() {
      state.arranging = !state.arranging;
      sheet.setArranging(state.arranging);
      arrange.setAttribute('aria-pressed', String(state.arranging));
      announce(state.arranging ? 'Arrange mode: drag a box, or select it and press Shift with an arrow key.' : 'Arrange mode off.');
    }

    /** Open a container as a diagram of its own; `null` shows the whole view. */
    async function focusOn(id) {
      const view = state.active;
      rescope(view, () => scope.setFocus(view, id));
      await redraw();
      if (state.active !== view) return;
      if (id) await selectById(id);
      stage.focus({ preventScroll: true });
      announce(id ? `${nav.nameOf(id)} opened as a diagram.` : 'Whole view.');
    }

    function drawCrumbs(graph) {
      const whole = placeLabel({ view: graph.arc.view, title: graph.arc.title });
      nav.crumbs(crumbs, whole, scope.path(graph.id), focusOn);
    }

    /** Unfold whatever hides `id` in view `graph`. */
    async function reveal(graph, id) {
      const source = graphs.find((candidate) => candidate.id === graph);
      const hiding = deps.collapse.ancestorsOf(source, id).filter((ancestor) => foldedOf(graph).has(ancestor));
      const outside = !scope.shows(graph, id);
      if (hiding.length === 0 && !outside) return;
      rescope(graph, () => {
        // An element outside the open container is reached in the whole view.
        if (outside) scope.setFocus(graph, null);
        scope.setFolded(graph, new Set([...foldedOf(graph)].filter((ancestor) => !hiding.includes(ancestor))));
      });
      if (state.active === graph) await show(graph);
    }

    async function open(graph, id) {
      if (id) await reveal(graph, id);
      // Always wait for the view to be drawn: it may still be laying out
      // from an earlier request for the same view.
      if (state.active !== graph) await show(graph);
      else await state.drawn;
      if (state.active !== graph) return false; // another view was asked for meanwhile
      await selectById(id);
      if (state.active !== graph) return false;
      center(id);
      // The control that was activated is gone with the old details: the
      // reader continues from the element they arrived at.
      const heading = details.querySelector('h3');
      if (heading) {
        heading.tabIndex = -1;
        heading.focus({ preventScroll: true });
      } else {
        stage.focus({ preventScroll: true });
      }
      return true;
    }

    /** Go to an element in a view, remembering where the reader was. */
    async function goTo(graph, id) {
      if (state.active === graph && state.selected === id) return;
      const from = { graph: state.active, id: state.selected };
      if (!(await open(graph, id))) return;
      const top = state.history[state.history.length - 1];
      if (!top || top.graph !== from.graph || top.id !== from.id) state.history.push(from);
      if (state.history.length > HISTORY_LIMIT) state.history.shift();
      back.disabled = false;
    }

    async function goBack() {
      const previous = state.history.pop();
      if (state.history.length === 0) {
        stage.focus({ preventScroll: true }); // before the button under focus is disabled
        back.disabled = true;
      }
      if (previous) await open(previous.graph, previous.id);
    }

    function clearViewChrome() {
      canvas.textContent = '';
      canvas.classList.remove('has-chain');
      chains.textContent = '';
      legend.textContent = '';
      notes.textContent = '';
      details.textContent = '';
      picker.textContent = '';
    }

    /** Show a view. `state.drawn` settles once it is on screen. */
    function show(id) {
      state.drawn = draw(id);
      return state.drawn;
    }

    async function draw(id) {
      const graph = graphs.find((candidate) => candidate.id === id) || graphs[0];
      // Only the latest request may paint: an earlier one still laying out
      // would otherwise overwrite it with another view or fold state.
      const turn = (state.turn = (state.turn || 0) + 1);
      state.active = graph.id;
      state.chain = null;
      state.selected = null;
      const kind = graph.arc.view;
      lastShown.set(kind, graph.id);
      for (const tab of tabs) {
        const on = tab.dataset.view === kind;
        tab.setAttribute('aria-selected', String(on));
        tab.tabIndex = on ? 0 : -1;
      }
      stage.setAttribute('aria-labelledby', `arcviz-tab-${kind}`);
      const siblings = ofKind(kind);
      title.textContent = graph.arc.title;
      title.hidden = siblings.length > 1;
      whichRow.hidden = siblings.length < 2;
      which.textContent = '';
      for (const sibling of siblings) {
        which.appendChild(el('option', { value: sibling.id, text: sibling.arc.title, selected: sibling.id === graph.id }));
      }
      clearViewChrome();
      // A mark on the button while the arrangement differs from the file.
      saveLayout.textContent = scope.dirty() ? 'Save layout •' : 'Save layout';
      saveLayout.setAttribute('aria-label', scope.dirty() ? 'Save layout, changed since the layout file' : 'Save layout');
      const foldable = scope.foldable(graph.id).length > 0;
      const nothingFolded = !foldable || foldedOf(graph.id).size === 0;
      // A button about to be disabled must not keep the focus.
      if (nothingFolded && document.activeElement === unfoldAll) stage.focus({ preventScroll: true });
      foldAll.disabled = !foldable;
      arrange.disabled = render.isSequence(graph);
      sheet.setArranging(state.arranging && !render.isSequence(graph));
      const nothingPlaced = Object.keys(scope.placedOf(graph.id)).length === 0;
      if (nothingPlaced && document.activeElement === resetPlaces) stage.focus({ preventScroll: true });
      resetPlaces.disabled = nothingPlaced;
      unfoldAll.disabled = nothingFolded;
      try {
        const view = await layout(graph);
        if (state.turn !== turn) return;
        canvas.innerHTML = view.svg; // escaped by ArcVizRender
        fit();
        drawChains(graph);
        drawCrumbs(graph);
        drawLegend(view);
        drawNotes(graph);
        drawPicker(view);
        showDetails(null);
        nav.mark(graph.id, null);
      } catch (error) {
        if (state.turn !== turn) return;
        clearViewChrome();
        drawNotes(graph);
        canvas.appendChild(el('p', {
          class: 'arcviz-empty',
          text: `This view could not be laid out: ${error && error.message ? error.message : error}`,
        }));
      }
    }

    // --- the stage: pan, zoom and gestures ------------------------------
    const nodeUnder = (element) => (element && element.closest ? element.closest('.av-node') : null);
    const sheet = deps.stage.create({
      stage,
      canvas,
      viewSize: activeView,
      onPick: (element) => select(element),
      onOpen: (element) => {
        const node = nodeUnder(element);
        if (node) toggleFold(node.dataset.id);
      },
      onKey: (event) => {
        const step = { ArrowLeft: [-NUDGE, 0], ArrowRight: [NUDGE, 0], ArrowUp: [0, -NUDGE], ArrowDown: [0, NUDGE] }[event.key];
        if (event.key === 'Escape') select(null);
        else if (event.key === 'Enter' && state.selected && canFold(state.selected)) toggleFold(state.selected);
        else if (state.arranging && event.shiftKey && step && state.selected && carried(state.selected).length > 0) moveBy(state.selected, step[0], step[1]);
        else return false;
        return true;
      },
      carried: (id) => carried(id),
      onMove: (id, dx, dy) => moveBy(id, dx, dy),
    });
    const fit = sheet.fit;

    // --- selection and details -----------------------------------------
    function drawPicker(view) {
      picker.textContent = '';
      picker.appendChild(el('option', { value: '', text: 'Nothing selected' }));
      const nodes = el('optgroup', { label: 'Elements' });
      for (const [id, entry] of view.index.nodes) {
        const kind = render.nodeStyle(entry.node.arc.kind).label;
        nodes.appendChild(el('option', { value: id, text: entry.node.labels[0].text ? `${entry.node.labels[0].text} (${kind})` : kind }));
      }
      const edges = el('optgroup', { label: 'Exchanges and links' });
      for (const [id, edge] of view.index.edges) {
        const kind = render.edgeStyle(edge.arc.kind).label;
        edges.appendChild(el('option', { value: id, text: edge.labels[0].text ? `${edge.labels[0].text} (${kind})` : `${kind}, unnamed` }));
      }
      picker.appendChild(nodes);
      if (view.index.edges.size > 0) picker.appendChild(edges);
    }

    function selectById(id) {
      const match = [...canvas.querySelectorAll('.av-node, .av-edge')].find((item) => item.dataset.id === id);
      return select(match || null);
    }

    async function select(target) {
      const asked = state.active;
      const view = await activeView();
      if (!view || state.active !== asked) return; // the view changed meanwhile
      const group = target && target.closest ? target.closest('.av-edge, .av-node') : null;
      picker.value = group ? group.dataset.id : '';
      for (const marked of canvas.querySelectorAll('.is-selected, .is-linked')) marked.classList.remove('is-selected', 'is-linked');
      if (!group) {
        state.selected = null;
        showDetails(null);
        nav.mark(state.active, null);
        return;
      }
      const id = group.dataset.id;
      state.selected = id;
      group.classList.add('is-selected');
      nav.mark(state.active, group.classList.contains('av-node') ? id : null);
      if (group.classList.contains('av-node')) {
        for (const edge of canvas.querySelectorAll('.av-edge')) {
          if (edge.dataset.source === id || edge.dataset.target === id) edge.classList.add('is-linked');
        }
        showDetails(view.index.nodes.get(id), view);
      } else {
        showDetails(view.index.edges.get(id), view, true);
      }
    }

    function row(list, term, value) {
      if (!value) return;
      list.appendChild(el('dt', { text: term }));
      list.appendChild(el('dd', { text: value }));
    }

    function showDetails(entry, view, isEdge) {
      details.textContent = '';
      if (!entry) {
        details.appendChild(el('p', { class: 'arcviz-quiet', text: 'Nothing selected.' }));
        return;
      }
      const list = el('dl', {});
      if (isEdge) {
        const style = render.edgeStyle(entry.arc.kind);
        const nameOf = (id) => {
          const hit = view.index.nodes.get(id);
          return hit ? displayName(hit.node, render.nodeStyle(hit.node.arc.kind).label) : id;
        };
        details.appendChild(el('h3', { text: displayName(entry, style.label) }));
        row(list, 'Kind', style.label);
        row(list, 'From', nameOf(entry.arc.source_node));
        row(list, 'To', nameOf(entry.arc.target_node));
        row(list, 'Carries', entry.arc.exchange_item);
        for (const [key, value] of Object.entries(entry.arc.properties || {})) row(list, key.replace(/_/g, ' '), value);
      } else {
        const node = entry.node;
        details.appendChild(el('h3', { text: displayName(node, render.nodeStyle(node.arc.kind).label) }));
        row(list, 'Kind', render.nodeStyle(node.arc.kind).label);
        if (entry.parent) row(list, 'Inside', view.index.nodes.get(entry.parent).node.labels[0].text);
        row(list, 'Realizes', (node.arc.realizes || []).join(', '));
        for (const [key, value] of Object.entries(node.arc.properties || {})) row(list, key.replace(/_/g, ' '), value);
        const ports = (node.ports || []).map((port) => {
          const arc = port.arc;
          const carried = arc.interface ? ` (${arc.interface})` : '';
          return `${arc.direction === 'undirected' ? '' : arc.direction + ' '}${arc.name}${carried}`;
        });
        row(list, 'Ports', ports.join(', '));
      }
      if (isEdge) {
        details.appendChild(list);
        return;
      }
      // Where the element leads comes first: it is what the reader acts on.
      const id = entry.node.id;
      details.appendChild(nav.related(state.active, id));
      details.appendChild(list);
      if (canFold(id)) {
        const folded = foldedOf(state.active).has(id);
        details.appendChild(el('button', {
          type: 'button',
          class: 'arcviz-jump arcviz-fold',
          'aria-expanded': String(!folded),
          text: folded ? `Unfold ${entry.node.arc.folded} inside` : 'Fold into one box',
          onclick: () => toggleFold(id),
        }));
      }
      if (scope.canFocus(state.active, id)) {
        details.appendChild(el('button', {
          type: 'button',
          class: 'arcviz-jump arcviz-open',
          text: 'Open as a diagram',
          onclick: () => focusOn(id),
        }));
      }
    }

    // --- functional chains ----------------------------------------------
    function drawChains(graph) {
      chains.textContent = '';
      const all = graph.arc.chains || [];
      if (all.length === 0) return;
      chains.appendChild(el('span', { class: 'arcviz-label', text: 'Functional chains' }));
      for (const chain of all) {
        chains.appendChild(el('button', {
          type: 'button',
          class: 'arcviz-chain',
          'data-chain': chain.id,
          'aria-pressed': 'false',
          text: chain.name,
          onclick: (event) => toggleChain(chain, event.currentTarget),
        }));
      }
    }

    function toggleChain(chain, button) {
      const turnOn = state.chain !== chain.id;
      state.chain = turnOn ? chain.id : null;
      for (const other of chains.querySelectorAll('.arcviz-chain')) other.setAttribute('aria-pressed', 'false');
      for (const marked of canvas.querySelectorAll('.in-chain')) marked.classList.remove('in-chain');
      canvas.classList.toggle('has-chain', turnOn);
      if (!turnOn) return;
      button.setAttribute('aria-pressed', 'true');
      // A chain passing through a folded container lights the container.
      const drawn = new Set([...canvas.querySelectorAll('.av-node')].map((item) => item.dataset.id));
      const standIns = chain.nodes
        .filter((id) => !drawn.has(id))
        .flatMap((id) => deps.collapse.ancestorsOf(activeGraph(), id).filter((ancestor) => drawn.has(ancestor)).slice(-1));
      const members = new Set([...chain.nodes, ...chain.edges, ...standIns]);
      for (const item of canvas.querySelectorAll('.av-node, .av-edge')) {
        if (members.has(item.dataset.id)) item.classList.add('in-chain');
      }
    }

    // --- legend and notes -----------------------------------------------
    function drawLegend(view) {
      const kinds = (items, pick) => [...new Set([...items].map(pick))];
      nav.legend(legend, kinds(view.index.nodes.values(), (entry) => entry.node.arc.kind), kinds(view.index.edges.values(), (edge) => edge.arc.kind), render.edgeStyle);
    }

    function drawNotes(graph) {
      notes.textContent = '';
      // What the open container exchanges with the containers around it.
      const leftOut = (scope.shape(graph).arc.left_out || []).map((edge) => edge.label);
      if (leftOut.length > 0) {
        notes.appendChild(el('p', {
          class: 'arcviz-note',
          text: `Not drawn here, because they reach a container around this one: ${leftOut.join(', ')}. The whole view shows them.`,
        }));
      }
      const prefix = `[${graph.id}] `;
      // What the layout file names and the model does not draw concerns
      // every view: saving the layout will write it without those entries.
      const general = '[layout] ';
      const issues = diagnostics
        .filter((line) => line.startsWith(prefix) || line.startsWith(general))
        .map((line) => (line.startsWith(prefix) ? line.slice(prefix.length) : `layout file: ${line.slice(general.length)} — saving the layout will drop it`));
      if (issues.length > 0) {
        const summary = el('summary', { text: `${issues.length} model ${issues.length === 1 ? 'issue' : 'issues'} found while drawing this view` });
        const list = el('ul', {}, issues.map((issue) => el('li', { text: issue })));
        notes.appendChild(el('details', { class: 'arcviz-issues', open: true }, [summary, list]));
      }
    }

    // --- export and print -----------------------------------------------
    function filename(extension) {
      return `${slug(payload.title)}-${slug(state.active)}.${extension}`;
    }

    const exporter = deps.exporter.create({
      el,
      download,
      filename,
      activeView,
      onFailure: (message) => notes.appendChild(el('p', { class: 'arcviz-note', role: 'status', text: message })),
    });
    const saveSvg = exporter.saveSvg;
    const savePng = exporter.savePng;

    // Print pages cannot wait for a layout, so every view is laid out in
    // the background once the first one is on screen.
    function buildPrintPages() {
      const pages = graphs
        .filter((graph) => state.settled.has(graph.id))
        .map((graph) => ({ title: graph.arc.title, svg: state.settled.get(graph.id).svg }));
      exporter.printPages(printHost, pages);
    }
    window.addEventListener('beforeprint', buildPrintPages);
    window.addEventListener('afterprint', () => { printHost.textContent = ''; });

    const ready = show(graphs[0].id).then(async () => {
      for (const graph of graphs) {
        try {
          state.settled.set(graph.id, await layout(graph, scope.WHOLE)); // print pages show every view whole
        } catch (error) {
          // The view reports its own layout failure when opened.
        }
      }
    });

    return {
      ready,
      /** Settles once the view last asked for is on screen. */
      settled: () => state.drawn,
      show,
      goTo,
      goBack,
      focusOn,
      svgOf: (id) => (state.settled.get(id) || {}).svg || null,
      buildPrintPages,
    };
  }

  root.ArcViz = { mount };
})(typeof globalThis !== 'undefined' ? globalThis : this);
