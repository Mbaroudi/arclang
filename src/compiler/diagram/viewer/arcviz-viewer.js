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
    req: 'Requirements',
    par: 'Parametric',
    gv: 'General View',
    iv: 'Interconnection',
    afv: 'Action Flow',
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
   *          stage, place, theme } — the layout engine constructor and the
   *          other ArcViz modules, each one named after its file (`theme`,
   *          ArcVizTheme, is optional: without it the built-in colours).
   *          payload.theme, a theme (arclang-theme.json) or a preset id,
   *          colours every view; one the validator refuses is not applied
   *          and the notes say why.
   * options: optional. A host that is a workbench of its own, with a
   *          project explorer and a properties view, places the chrome
   *          where it wants it:
   *   slots:    { tabs, tools, head, tree, details, related, foot } —
   *             elements that receive, in order, the view tabs, the
   *             toolbar, the title and crumbs and chains, the model tree,
   *             the details of the selection, the "across layers" section,
   *             and the hint, picker, legend and notes. A slot left out
   *             keeps that chrome in the host, as the standalone page has it.
   *   onSelect: (selection | null) => void, called whenever the selection
   *             changes, with { view, id, name, kind, isEdge }.
   *   onDrop:   ({ view, from, to }) => void, when a box is dragged and let
   *             go on another box (`to`) or on the background (`to` null),
   *             outside arrange mode. Given, dragging a box is that gesture.
   *   onOpen:   ({ view, id, name, kind }) => boolean, on a double-click on
   *             a box; true means the host took it (an editor opened),
   *             false lets the viewer fold or unfold the container.
   *   onConnect: (fromId, toId, { fromKind, toKind, view }) => void, when
   *             a line is drawn from the handle of a box or port to another
   *             box or port (the kinds are those of the boxes; a port's,
   *             of the box it sits on). Given, every box and port shows a
   *             round handle under the pointer, and `c` with a box
   *             selected starts the line from the keyboard.
   *   connectTargets: (fromId) => ids | null, asked once as a line starts:
   *             the boxes and ports it may end on, or null for any of
   *             them. Left out, any.
   *   arrange:  true to start with boxes placed by dragging (an editor).
   *             With onDrop, a box let go on another box that is not its
   *             container goes into it (onDrop); anywhere else, it stays
   *             where it was let go.
   *   onLayout: (layout file) => void, whenever the arrangement (places,
   *             folds, open containers) changes: the host keeps it.
   *   toolbar:  { history, align, arrangeToggle } — undo and redo from the
   *             host's history ({ undo(), redo(), labels() }), Align on its
   *             selection ({ count(), align(how), distribute(axis) }), and
   *             false to leave out the Arrange switch (always arranging).
   *   exportItems, themeItems: [{ id, label, title, shortcut?, checked?, run }]
   *             — more entries in Export (above SVG and PNG) and a Theme
   *             menu (shown only when given); see "extension hooks".
   *   theme:    { theme, dark } — the colour theme to draw with (deps.theme
   *             checks it), its dark variant on screen when `dark`; it wins
   *             over payload.theme. See setTheme for later changes.
   *   edit:     a host that edits the model from the drawing, with
   *             deps.edit (ArcVizEdit): see `attach` in arcviz-edit.js.
   *   reorganize: 'ask' (the default), 'always' or 'never': what follows a
   *             box moved or resized by hand that left the drawing harder
   *             to read (boxes overlapping, lines across a box or crossing):
   *             an offer to tidy it up, a tidy-up, or nothing. Needs
   *             deps.tidy (ArcVizTidy).
   *   onReorganize: (mode) => void, when the reader changes that choice
   *             (the host keeps it); without it, the page remembers it.
   */
  function mount(host, payload, deps, options) {
    const graphs = payload.graphs || [];
    const diagnostics = payload.diagnostics || [];
    const render = deps.render;
    // --- theme ----------------------------------------------------------
    // The colours of the screen and those of the exports: the same theme,
    // the screen in its dark variant when the host asks for one.
    const themes = deps.theme || null;
    const themeState = { screen: undefined, exports: undefined, theme: null, refused: null };
    const themeOptions = (purpose) => (themeState[purpose] === undefined ? {} : { theme: themeState[purpose] });
    /** Check a theme and make it the one drawn; { ok, errors, warnings }. */
    function adoptTheme(theme, settings) {
      if (theme === null) {
        Object.assign(themeState, { screen: undefined, exports: undefined, theme: null, refused: null });
        return { ok: true, errors: [], warnings: [] };
      }
      if (!themes) return { ok: false, errors: [{ rule: 'structure', message: 'this page has no theme support (ArcVizTheme)' }], warnings: [] };
      const given = typeof theme === 'string' ? themes.preset(theme) : theme;
      if (!given) return { ok: false, errors: [{ rule: 'structure', message: `no preset "${theme}"` }], warnings: [] };
      const checked = themes.validate(given);
      // A draft (a theme being edited) is previewed whatever it breaks; it is never the theme kept.
      const draft = Boolean(settings && settings.draft && checked.theme);
      if (!checked.ok && !draft) {
        themeState.refused = checked.errors;
        return checked;
      }
      Object.assign(themeState, {
        screen: themes.palette(checked.theme, { dark: Boolean(settings && settings.dark), draft }),
        exports: themes.palette(checked.theme, { draft }),
        theme: draft ? themeState.theme : checked.theme,
        refused: null,
      });
      return checked;
    }
    // options.theme: { theme, dark } — the host's theme, in its dark variant on a dark editor.
    if (options && options.theme && options.theme.theme) adoptTheme(options.theme.theme, options.theme);
    else if (payload.theme) adoptTheme(payload.theme);
    const slots = (options && options.slots) || {};
    const onSelect = (options && options.onSelect) || (() => {});
    const onDrop = options && options.onDrop;
    const onOpen = options && options.onOpen;
    const onConnect = options && options.onConnect;
    const connectTargets = options && options.connectTargets;
    // The host may take the whole window itself (with its own tools around the sheet).
    const onFullscreen = options && options.onFullscreen;
    // The host may save files itself (an editor's webview cannot download):
    // (blob, filename) -> void. Without it, the browser downloads them.
    const save = (options && options.onDownload) || download;
    const onLayout = options && options.onLayout;
    // Tidying up after a change by hand: ArcVizTidy, and what the reader chose to do.
    const tidyKit = deps.tidy || root.ArcVizTidy || null;
    const REORGANIZE = ['ask', 'always', 'never'];
    const onReorganize = options && options.onReorganize;
    let reorganize = (() => {
      if (options && REORGANIZE.includes(options.reorganize)) return options.reorganize;
      try {
        const kept = root.localStorage && root.localStorage.getItem('arcviz.reorganize');
        if (REORGANIZE.includes(kept)) return kept;
      } catch (error) {
        // Storage refused (a private window): ask.
      }
      return 'ask';
    })();
    function setReorganize(mode) {
      if (!REORGANIZE.includes(mode)) return;
      reorganize = mode;
      if (onReorganize) onReorganize(mode);
      else {
        try {
          if (root.localStorage) root.localStorage.setItem('arcviz.reorganize', mode);
        } catch (error) {
          // Not remembered: kept for this page only.
        }
      }
    }
    // zorder hook: layers (tags), groups and locks (arcviz-layers.js); options.layers = { tagsOf(view, id, isEdge) -> [tag] }.
    const layersKit = deps.layers || root.ArcVizLayers || null;
    const layerStore = layersKit ? layersKit.create({ tagsOf: (options && options.layers && options.layers.tagsOf) || null }) : null;
    const viewSigs = new Map();
    const state = { active: null, views: new Map(), layouts: new Map(), arranging: Boolean(options && options.arrange), chain: null, selected: null, history: [], settled: new Map(), filters: new Map() };

    host.textContent = '';
    host.classList.add('arcviz');
    for (const slot of Object.values(slots)) {
      slot.textContent = '';
      slot.classList.add('arcviz');
    }
    // Chrome goes to its slot when the host names one, else where the
    // standalone page keeps it.
    const into = (name, fallback, element) => (slots[name] || fallback).appendChild(element);

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

    // --- the tool row (arcviz-toolbar.js) ---------------------------------
    // Moving about | undo, redo | zoom | layout, align, filters | export,
    // full screen. A narrow frame drops the words of the menus first, then
    // moves whole tools into "⋯", the least used first.
    const toolkit = deps.toolbar || root.ArcVizToolbar;
    const kit = toolkit.create({ el });
    const extra = (options && options.toolbar) || {};
    // A host with a history of its own: { undo(), redo(), labels() -> { undo, redo } }.
    const history = extra.history || null;
    // A host that aligns its selection: { count(), align(how), distribute(axis) }.
    const aligner = extra.align || null;
    // What the Layout menu may do in the view on screen (set as it is drawn).
    const can = { fold: false, unfold: false, reset: false, arrange: true };

    const back = kit.iconButton({ id: 'arcviz-back', icon: 'back', label: 'Back', hint: 'Back to the element you came from', run: () => goBack() });
    back.disabled = true;
    const treeToggle = kit.iconButton({ id: 'arcviz-tree-toggle', icon: 'tree', label: 'Model tree', hint: 'Show or hide the model tree', pressed: true, run: () => toggleTree() });
    // The tree lives in the project explorer of a workbench host: the
    // button that hides it would hide a panel that is not the viewer's.
    if (slots.tree) treeToggle.hidden = true;
    const undo = history ? kit.iconButton({ id: 'arcviz-undo', icon: 'undo', label: 'Undo', keys: 'Mod+Z', run: () => history.undo() }) : null;
    const redo = history ? kit.iconButton({ id: 'arcviz-redo', icon: 'redo', label: 'Redo', keys: kit.mac ? 'Mod+Shift+Z' : 'Ctrl+Y', run: () => history.redo() }) : null;

    const zoomOut = kit.iconButton({ id: 'arcviz-zoom-out', icon: 'zoom-out', label: 'Zoom out', keys: '-', run: () => sheet.zoomOut() });
    const zoomIn = kit.iconButton({ id: 'arcviz-zoom-in', icon: 'zoom-in', label: 'Zoom in', keys: '+', run: () => sheet.zoomIn() });
    const fitButton = kit.iconButton({ id: 'arcviz-fit', icon: 'fit', label: 'Fit to window', keys: '0', run: () => fit() });
    const ZOOM_PRESETS = [0.5, 0.75, 1, 1.5, 2];
    const zoomLevel = kit.menuButton({
      id: 'arcviz-zoom-level',
      label: toolkit.formatZoom(1),
      heading: (label) => `Zoom · ${label}`,
      title: 'Zoom level: choose one',
      describe: (label) => `Zoom ${label}`,
      className: 'arcviz-zoom',
      keepLabel: true,
      items: () => [
        { label: 'Fit to window', keys: '0', action: 'zoom-fit', run: () => fit() },
        'separator',
        ...ZOOM_PRESETS.map((scale) => ({
          label: toolkit.formatZoom(scale),
          action: `zoom-${Math.round(scale * 100)}`,
          radio: true,
          checked: Math.abs(sheet.scale() - scale) < 0.005,
          run: () => sheet.zoomTo(scale),
        })),
      ],
    });
    const showZoom = (scale) => {
      const text = toolkit.formatZoom(scale);
      if (zoomLevel.button.textContent !== text) zoomLevel.setLabel(text);
    };

    const layoutName = payload.layout_name || `${slug(payload.title)}.layout.json`;
    const arrangeToggle = extra.arrangeToggle !== false;
    const layoutMenu = kit.menuButton({
      id: 'arcviz-layout-menu',
      icon: 'layout',
      label: 'Layout',
      className: 'arcviz-layout',
      items: () => [
        { label: 'Re-arrange automatically', action: 'reset-layout', title: 'Put every box of this view back where the automatic layout placed it', disabled: !can.reset, reason: 'No box of this view was placed by hand', run: () => resetLayout() },
        ...(tidyKit && options && (options.edit || options.arrange) ? [
          { label: 'Tidy up, keeping my arrangement', action: 'tidy-up', title: 'Align and space the boxes, remove overlaps and redraw every line, keeping the order in which the boxes are placed now', disabled: !can.arrange, reason: 'A scenario is drawn as declared', run: () => tidyUp() },
          { label: 'After a move by hand: ask to tidy up', action: 'reorganize-ask', checked: reorganize === 'ask', title: 'When a box moved or resized by hand leaves the diagram harder to read, offer to tidy it up', run: () => setReorganize('ask') },
          { label: 'After a move by hand: always tidy up', action: 'reorganize-always', checked: reorganize === 'always', title: 'Tidy the diagram up by itself whenever a move by hand leaves it harder to read', run: () => setReorganize('always') },
          { label: 'After a move by hand: never ask', action: 'reorganize-never', checked: reorganize === 'never', title: 'Leave the diagram as it is placed by hand', run: () => setReorganize('never') },
        ] : []),
        'separator',
        { label: 'Fold all', action: 'fold-all', title: 'Show every container as a single box', disabled: !can.fold, reason: 'This view has no container to fold', run: () => refold(new Set(scope.foldable(state.active))) },
        { label: 'Unfold all', action: 'unfold-all', title: 'Show what every container holds', disabled: !can.unfold, reason: 'No container is folded', run: () => refold(new Set()) },
        // An editor places boxes by dragging all the time: no mode to switch.
        ...(extra.layoutItems ? ['separator', ...extra.layoutItems()] : []), // zorder hook: Order, Group, Size, Lock, Waypoints, Select
        ...(arrangeToggle ? ['separator', { label: 'Arrange by hand', action: 'arrange', checked: state.arranging, title: 'Move boxes by hand: drag one, or select it and press Shift with an arrow key', disabled: !can.arrange, reason: 'A scenario is drawn as declared', run: () => toggleArrange() }] : []),
      ],
    });
    const SELECT_MORE = 'select two or more boxes (Shift+click) to align them';
    const alignMenu = aligner ? kit.menuButton({
      id: 'arcviz-align-menu',
      icon: 'align',
      label: 'Align',
      className: 'arcviz-align',
      items: () => {
        const few = aligner.count() < 3;
        const align = (label, how) => ({ label, action: `align-${how}`, run: () => aligner.align(how) });
        const spread = (label, axis) => ({ label, action: `distribute-${axis}`, disabled: few, reason: 'Select three or more boxes to distribute them', run: () => aligner.distribute(axis) });
        return [
          align('Align left', 'left'), align('Align centre', 'center'), align('Align right', 'right'),
          'separator',
          align('Align top', 'top'), align('Align middle', 'middle'), align('Align bottom', 'bottom'),
          'separator',
          spread('Distribute horizontally', 'horizontal'), spread('Distribute vertically', 'vertical'),
        ];
      },
    }) : null;
    if (alignMenu) alignMenu.setDisabled(true, SELECT_MORE);
    // Filters, as a diagram has them in Capella: kinds of boxes and lines
    // left out of the drawing, ports and labels hidden.
    const filterList = el('div', { class: 'arcviz-filter-list' });
    const filterMenu = kit.menuButton({ id: 'arcviz-filters-menu', icon: 'filter', label: 'Filters', title: 'Filters: what this view leaves out', className: 'arcviz-filters', panel: () => filterList });
    // --- extension hooks: more exports, a theme chooser ------------------
    // options.exportItems and options.themeItems: [{ id, label, title,
    // shortcut?, checked?, run }] (or a function returning that list, read
    // each time the menu opens). The export items come first in Export,
    // above a rule; Theme ▾ is shown only when its list is not empty. The
    // toolbar draws them; what they do is the host's.
    const listOf = (items) => (typeof items === 'function' ? items() : items) || [];
    const hookItems = (items) => listOf(items).map((item) => ({
      label: item.label,
      title: item.title,
      keys: item.shortcut,
      action: item.id,
      checked: item.checked,
      radio: item.checked !== undefined,
      run: () => item.run(),
    }));
    const exportHooks = (options && options.exportItems) || [];
    const themeHooks = (options && options.themeItems) || [];
    const themeMenu = listOf(themeHooks).length > 0 ? kit.menuButton({
      id: 'arcviz-theme-menu',
      icon: 'theme',
      label: 'Theme',
      className: 'arcviz-theme',
      items: () => hookItems(themeHooks),
    }) : null;
    // --- end of the extension hooks ----------------------------------------

    // zorder hook: exports follow the layers shown.
    const hiddenNote = () => (layerStore && layerStore.state().hidden.length ? ` (hidden layers are left out: ${layerStore.state().hidden.map((n) => n || 'Default').join(', ')})` : '');
    const exportMenu = kit.menuButton({
      id: 'arcviz-export-menu',
      icon: 'export',
      label: 'Export',
      className: 'arcviz-export',
      items: () => [
        ...(() => { const extra = hookItems(exportHooks); return extra.length ? [...extra, 'separator'] : []; })(),
        { label: 'Save as SVG', action: 'save-svg', title: `This view as drawn, folded containers included, as an SVG file${hiddenNote()}`, run: () => saveSvg() },
        { label: 'Save as PNG', action: 'save-png', title: `This view as drawn, folded containers included, as a PNG image${hiddenNote()}`, run: () => savePng() },
        // A host that keeps the arrangement itself (onLayout) needs no download of it.
        ...(onLayout ? [] : ['separator', { label: scope.dirty() ? 'Save layout •' : 'Save layout', action: 'save-layout', title: `Download the folds, the open containers and the placement as ${layoutName}. Kept next to the model, it is what everyone sees.`, run: () => exporter.saveLayout(scope.toFile(), layoutName) }]),
      ],
    });
    const fullscreen = onFullscreen
      ? kit.iconButton({ id: 'arcviz-fullscreen', icon: 'fullscreen', label: 'Full screen', hint: 'Full screen, with the palette and every tool (Esc to leave)', run: () => onFullscreen() })
      : kit.iconButton({ id: 'arcviz-fullscreen', icon: 'fullscreen', label: 'Full screen', hint: 'Full screen (Esc to leave)', pressed: false, run: () => sheet.toggleFullscreen() });
    if (!onFullscreen) document.addEventListener('fullscreenchange', () => fullscreen.setAttribute('aria-pressed', String(document.fullscreenElement === stage)));

    const drop = (node, rank) => { node.dataset.drop = String(rank); return node; };
    const menus = [zoomLevel, layoutMenu, alignMenu, filterMenu, exportMenu, themeMenu].filter(Boolean);
    const menuOf = new Map(menus.map((menu) => [menu.root, menu]));
    const tools = kit.row([
      kit.group('Navigation', [treeToggle, back]),
      history ? kit.group('History', [undo, redo]) : null,
      kit.group('Zoom', [zoomOut, drop(zoomLevel.root, 1), zoomIn, fitButton]),
      kit.group('Arrangement', [drop(layoutMenu.root, 4), alignMenu ? drop(alignMenu.root, 6) : null, drop(filterMenu.root, 5), ...(extra.tools || []).map((spec) => drop(kit.iconButton(spec), spec.rank || 6))]), // zorder hook: host tools (Layers)
      kit.group('Output', [drop(exportMenu.root, 7), themeMenu ? drop(themeMenu.root, 8) : null, drop(fullscreen, 3)]),
    ].filter(Boolean), menus, (tool) => (menuOf.has(tool)
      ? menuOf.get(tool).entries()
      : [{ label: tool.getAttribute('aria-label'), keys: tool.dataset.keys, disabled: kit.isOff(tool), run: () => tool.click() }]));
    const toolbar = tools.toolbar;

    /** Put the history and the selection the host has into the buttons. */
    function refreshTools() {
      if (history) {
        const labels = history.labels() || {};
        undo.disabled = !labels.undo;
        kit.setLabel(undo, labels.undo ? `Undo ${labels.undo}` : 'Undo');
        redo.disabled = !labels.redo;
        kit.setLabel(redo, labels.redo ? `Redo ${labels.redo}` : 'Redo');
      }
      if (alignMenu) alignMenu.setDisabled(aligner.count() < 2, SELECT_MORE);
    }
    refreshTools();

    function resetLayout() {
      return replace(() => scope.resetPlaces(state.active)).then(() => announce('Layout reset.'));
    }

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
      text: 'Drag or scroll to pan, or use the arrow keys. Zoom with + and -, with Ctrl or ⌘ and the wheel, by pinching, or by double-clicking the desk (with Shift to zoom out). Press 0 to fit. Double-click a container, or select it and press Enter, to fold or unfold it. With Arrange on, drag a box to place it.'
        + (onConnect ? ' To connect two elements, drag from the round handle at the edge of a box, or on a port, to the other one; or select a box and press c, then choose the target with the arrow keys and Enter.' : ''),
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

    if (slots.tabs || slots.tools) {
      into('tabs', host, tablist);
      into('tools', host, toolbar);
    } else {
      host.appendChild(el('div', { class: 'arcviz-bar' }, [tablist, toolbar]));
    }
    // Where the reader is inside a view once a container is open.
    const crumbs = el('nav', { class: 'arcviz-crumbs', 'aria-label': 'Location in the model', hidden: true });
    // What changed on screen, for readers who do not see it change.
    const status = el('p', { class: 'arcviz-status', role: 'status' });
    // Cleared first: the same words twice in a row would not be read again.
    const announce = (text) => {
      status.textContent = '';
      setTimeout(() => { status.textContent = text; }, 30);
    };
    into('head', host, title);
    into('head', host, whichRow);
    into('head', host, crumbs);
    host.appendChild(status);
    into('head', host, chains);
    // Tree and details share the column beside the sheet, like a project
    // explorer over a properties view: what is selected, and where it
    // leads, stays in sight while the drawing is read. A workbench host
    // has those panels already and takes the pieces into them.
    const beside = [];
    if (slots.tree) slots.tree.appendChild(nav.tree); else beside.push(nav.tree);
    if (slots.details) slots.details.appendChild(details); else beside.push(details);
    const side = el('div', { class: 'arcviz-side' }, beside);
    const work = el('div', { class: `arcviz-work${beside.length === 0 ? ' is-sideless' : ''}` }, [side, stage]);
    host.appendChild(work);
    // The "across layers" section follows the details unless it has a
    // panel of its own, like a semantic browser beside the properties.
    // An element of the viewer's own, so a host that moves the chrome
    // about keeps receiving the selection.
    const related = slots.related ? el('div', { class: 'arcviz-related-host' }) : null;
    if (related) slots.related.appendChild(related);
    into('foot', host, hint);
    into('foot', host, pick);
    into('foot', host, el('div', { class: 'arcviz-below' }, [legend]));
    into('foot', host, notes);
    host.appendChild(printHost);

    // --- layout ---------------------------------------------------------
    const activeGraph = () => graphs.find((graph) => graph.id === state.active);
    let storage = null;
    try {
      storage = root.localStorage || null;
    } catch (error) {
      // Storage is refused: folds hold for this visit only.
    }
    const scope = deps.scope.create({ graphs, collapse: deps.collapse, storage, storageKey: `arcviz:${payload.title}`, initial: payload.layout || null, sections: layerStore }); // zorder hook: sections
    /** Tell the host the arrangement when it changed. */
    const reportLayout = deps.place.layoutReporter(() => scope.toFile(), onLayout);
    const foldedOf = scope.foldedOf;

    // A view is laid out once per scope: what is folded, what is open.
    // The automatic layout is computed once; the reader's placement is
    // applied on top of it each time a box is moved.
    // --- filters --------------------------------------------------------
    const filtersOf = (view) => {
      if (!state.filters.has(view)) state.filters.set(view, { nodes: new Set(), edges: new Set(), ports: false, labels: false });
      return state.filters.get(view);
    };
    const filterSignature = (view) => {
      const filters = filtersOf(view);
      return `|f:${[...filters.nodes].sort().join(',')}|e:${[...filters.edges].sort().join(',')}`;
    };

    /** The graph without the kinds filtered out, and the edges that touched them. */
    function applyFilters(graph, filters) {
      if (filters.nodes.size === 0 && filters.edges.size === 0) return graph;
      const gone = new Set();
      const prune = (node) => {
        node.children = (node.children || []).filter((child) => {
          const out = filters.nodes.has((child.arc || {}).kind);
          if (out) collect(child);
          return !out;
        });
        node.children.forEach(prune);
      };
      const collect = (node) => {
        gone.add(node.id);
        for (const port of node.ports || []) gone.add(port.id);
        (node.children || []).forEach(collect);
      };
      prune(graph);
      const touches = (edge) => [...(edge.sources || []), ...(edge.targets || [])].some((end) => gone.has(end))
        || gone.has((edge.arc || {}).source_node) || gone.has((edge.arc || {}).target_node);
      const sweep = (holder) => {
        if (holder.edges) holder.edges = holder.edges.filter((edge) => !filters.edges.has((edge.arc || {}).kind) && !touches(edge));
        (holder.children || []).forEach(sweep);
      };
      sweep(graph);
      return graph;
    }

    /** Every kind of box and of line a view may draw, for the filter menu. */
    function kindsIn(graph) {
      const nodes = new Set();
      const edges = new Set();
      const walk = (holder) => {
        for (const child of holder.children || []) { nodes.add((child.arc || {}).kind); walk(child); }
        for (const edge of holder.edges || []) edges.add((edge.arc || {}).kind);
      };
      walk(scope.shape(graph));
      nodes.delete(undefined);
      edges.delete(undefined);
      return { nodes: [...nodes], edges: [...edges] };
    }

    function drawFilterMenu(graph) {
      filterList.textContent = '';
      const filters = filtersOf(graph.id);
      const sequence = render.isSequence(graph);
      const check = (label, on, change) => el('label', { class: 'arcviz-filter' }, [
        el('input', { type: 'checkbox', checked: on, onchange: (event) => change(event.currentTarget.checked) }),
        el('span', { text: label }),
      ]);
      const apply = () => { state.views.delete(scope.key(graph.id) + filterSignature(graph.id)); redraw(); };
      const kinds = kindsIn(graph);
      if (!sequence) {
        filterList.appendChild(el('p', { class: 'arcviz-filter-title', text: 'Show' }));
        filterList.appendChild(check('Ports', !filters.ports, (on) => { filters.ports = !on; canvas.classList.toggle('hide-ports', filters.ports); }));
        filterList.appendChild(check('Labels of lines', !filters.labels, (on) => { filters.labels = !on; canvas.classList.toggle('hide-labels', filters.labels); }));
      }
      if (kinds.nodes.length > 0) filterList.appendChild(el('p', { class: 'arcviz-filter-title', text: 'Boxes' }));
      for (const kind of kinds.nodes) {
        if (kind === 'lifeline' || kind === 'initial_state' || kind === 'junction') continue;
        filterList.appendChild(check(render.nodeStyle(kind).label, !filters.nodes.has(kind), (on) => { if (on) filters.nodes.delete(kind); else filters.nodes.add(kind); apply(); }));
      }
      if (kinds.edges.length > 0) filterList.appendChild(el('p', { class: 'arcviz-filter-title', text: 'Lines' }));
      for (const kind of kinds.edges) {
        filterList.appendChild(check(render.edgeStyle(kind).label, !filters.edges.has(kind), (on) => { if (on) filters.edges.delete(kind); else filters.edges.add(kind); apply(); }));
      }
      const active = filters.nodes.size + filters.edges.size + (filters.ports ? 1 : 0) + (filters.labels ? 1 : 0);
      filterMenu.setBadge(active > 0 ? active : '');
      canvas.classList.toggle('hide-ports', filters.ports);
      canvas.classList.toggle('hide-labels', filters.labels);
    }

    function layout(graph, shown) {
      const key = scope.key(graph.id, shown) + filterSignature(graph.id);
      const sig = layerStore ? layerStore.signature(graph.id) : ''; // zorder hook: a drawing per layer and group state
      if (state.views.has(key) && viewSigs.get(key) === sig) return state.views.get(key);
      if (!state.layouts.has(key)) {
        // Scenarios are drawn as declared; everything else goes through ELK.
        const copy = applyFilters(JSON.parse(JSON.stringify(scope.shape(graph, shown))), filtersOf(graph.id));
        state.layouts.set(key, render.isSequence(graph) ? Promise.resolve(copy) : render.layOut(new deps.ELK(), copy));
      }
      const pending = state.layouts.get(key).then((laidOut) => {
        // An editor (options.edit) also routes redrawn exchanges around the boxes.
        const extras = { ...scope.extrasOf(graph.id, shown), avoid: Boolean(options && options.edit && deps.edit) };
        const arranged = render.isSequence(graph) ? laidOut : deps.place.place(laidOut, scope.placedOf(graph.id, shown), scope.routesOf(graph.id, shown), extras);
        const placed = layerStore && !render.isSequence(graph) ? layerStore.apply(arranged, graph.id) : arranged; // zorder hook
        return { graph: placed, index: indexGraph(placed), ...render.renderSvg(placed, themeOptions('screen')) };
      });
      state.views.set(key, pending);
      viewSigs.set(key, sig); // zorder hook
      return pending;
    }

    /** The active view once laid out, or null if it failed or is absent. */
    async function activeView() {
      const pending = state.views.get(scope.key(state.active) + filterSignature(state.active));
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
      const before = scope.key(id) + filterSignature(id);
      if (scope.key(id) !== scope.key(id, scope.WHOLE)) {
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
    function carried(id, raw) {
      const graph = activeGraph();
      if (!graph || render.isSequence(graph)) return [];
      // zorder hook: a locked box stays; a member of a group moves with it; a group carries its members and what they hold.
      if (layerStore && layerStore.isLocked(graph.id, id, false)) return [];
      const standsFor = layerStore && !raw ? layersKit.redirect(layerStore, graph.id, id) : id;
      if (standsFor !== id) return carried(standsFor);
      if (layersKit && layersKit.isGroupId(id)) {
        const members = layersApi ? layersApi.membersDeep(id) : [];
        if (members.some((member) => layerStore.isLocked(graph.id, member, false))) return [];
        return [...new Set([id, ...layersApi.framesDeep(id), ...members.flatMap((member) => carried(member, true))])];
      }
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
    /** How orderly the drawing on screen is, when the page can tell. */
    async function orderOfActive() {
      if (!tidyKit) return null;
      const shown = await activeView();
      return shown ? tidyKit.measure(shown.graph) : null;
    }

    /**
     * Change the placement and draw it again. A change by hand (`manual`)
     * that leaves the drawing harder to read is followed by what the reader
     * chose: an offer to tidy it up, a tidy-up, or nothing.
     */
    async function replace(change, how) {
      const manual = Boolean(how && how.manual) && tidyKit && reorganize !== 'never';
      const before = manual ? await orderOfActive() : null;
      closeTidyOffer();
      const view = state.active;
      const where = sheet.position();
      state.views.delete(scope.key(view) + filterSignature(view));
      change();
      await redraw();
      if (state.active !== view) return;
      sheet.restore(where);
      const shown = await activeView();
      if (shown && shown.graph.arc.placed) scope.settle(view, shown.graph.arc.placed);
      reportLayout();
      if (manual && shown) {
        const after = tidyKit.measure(shown.graph);
        if (tidyKit.worse(before, after)) {
          if (reorganize === 'always') setTimeout(() => tidyUp(), 0);
          else offerTidy(after);
        }
      }
      // Print pages show each view whole, as the reader arranged it.
      try {
        state.settled.set(view, await layout(graphs.find((graph) => graph.id === view), scope.WHOLE));
      } catch (error) {
        // The view reports its own layout failure when opened.
      }
    }

    /**
     * Tidy the view up: ELK lays it out again from where the boxes are now
     * (ArcVizTidy.seeded), so their order stays, and the result is kept as
     * the placement. Bends, moved names and ports go: the lines are redrawn.
     */
    async function tidyUp() {
      closeTidyOffer();
      const view = state.active;
      const graph = activeGraph();
      if (!tidyKit || !graph || render.isSequence(graph)) return;
      const shown = await activeView();
      const key = scope.key(view) + filterSignature(view);
      const automatic = state.layouts.get(key);
      if (!shown || !automatic) return;
      try {
        const auto = await automatic;
        const copy = applyFilters(JSON.parse(JSON.stringify(scope.shape(graph))), filtersOf(view));
        const tidy = await render.layOut(new deps.ELK(), tidyKit.seeded(copy, shown.graph));
        if (state.active !== view) return;
        const before = tidyKit.measure(shown.graph);
        await replace(() => scope.tidy(view, tidyKit.placements(auto, tidy)));
        const after = await orderOfActive();
        const left = after ? tidyKit.describe(after) : '';
        announce(`Diagram tidied up${before && after ? ` (${tidyKit.describe(before) || 'nothing to fix'} before${left ? `, ${left} now` : ', nothing left'})` : ''}.`);
      } catch (error) {
        announce(`The diagram could not be tidied up: ${error && error.message ? error.message : error}`);
      }
    }

    // The offer to tidy up, over the sheet: never modal, gone at the next change.
    let tidyOffer = null;
    function closeTidyOffer() {
      if (tidyOffer) tidyOffer.remove();
      tidyOffer = null;
    }
    function offerTidy(order) {
      closeTidyOffer();
      const choose = (mode, tidy) => () => {
        if (mode) setReorganize(mode);
        closeTidyOffer();
        if (tidy) tidyUp();
      };
      const text = `This change left the diagram harder to read: ${tidyKit.describe(order)}. Tidy it up, keeping your arrangement?`;
      tidyOffer = el('div', { class: 'arcviz-tidy-offer', role: 'group', 'aria-label': 'Tidy up the diagram' }, [
        el('span', { class: 'arcviz-tidy-text', text }),
        el('button', { type: 'button', class: 'arcviz-tidy-yes', text: 'Tidy up', onclick: choose(null, true) }),
        el('button', { type: 'button', class: 'arcviz-tidy-keep', text: 'Keep my layout', onclick: choose(null, false) }),
        el('button', { type: 'button', class: 'arcviz-tidy-always', text: 'Always tidy up', title: 'Tidy up by itself after every move by hand that leaves the diagram harder to read (Layout menu to change it)', onclick: choose('always', true) }),
      ]);
      // The sheet pans and selects on a press: a press on the offer is the offer's.
      for (const type of ['pointerdown', 'mousedown', 'touchstart', 'dblclick', 'wheel', 'contextmenu']) tidyOffer.addEventListener(type, (event) => event.stopPropagation());
      tidyOffer.addEventListener('keydown', (event) => {
        if (event.key === 'Escape') {
          event.stopPropagation();
          closeTidyOffer();
          stage.focus();
        }
      });
      stage.appendChild(tidyOffer);
      announce(text);
    }

    async function moveBy(id, dx, dy) {
      await replace(() => scope.moveBy(state.active, id, dx, dy), { manual: true });
      announce(`${nav.nameOf(id)} moved.`);
    }

    /** Resize a box; by a side handle, what it holds stays where it is (arcviz-place.js). */
    async function resizeBy(id, dw0, dh0, dx0 = 0, dy0 = 0) {
      const shown = (dx0 || dy0) && (await activeView());
      const entry = shown && shown.index.nodes.get(id);
      const { dw, dh, dx, dy, children } = deps.place.sideResize ? deps.place.sideResize(entry && entry.node, dw0, dh0, dx0, dy0) : { dw: dw0, dh: dh0, dx: 0, dy: 0, children: [] };
      await replace(() => {
        if (dx || dy) {
          scope.moveBy(state.active, id, dx, dy);
          for (const child of children) scope.moveBy(state.active, child.id, -dx, -dy);
        }
        scope.resizeBy(state.active, id, dw, dh);
      }, { manual: true });
      announce(`${nav.nameOf(id)} resized.`);
    }

    /** A box let go: into the box under it when that is a new container, else placed there. */
    async function placeOrDrop(id, dx, dy, into) {
      const view = state.active;
      // zorder hook: a group moves its members (a member dragged drags its group); a box let go on a group frame stays where it is let go.
      if (layerStore) id = layersKit.redirect(layerStore, view, id);
      if (layersKit && layersKit.isGroupId(id)) {
        await replace(() => layersApi.membersDeep(id).forEach((member) => scope.moveBy(view, member, dx, dy)), { manual: true });
        announce('Group moved.');
        return;
      }
      if (layersKit && into && layersKit.isGroupId(into)) into = null;
      const graph = activeGraph();
      const parent = graph ? deps.place.parentOf(scope.shape(graph), id) : null;
      if (onDrop && into && into !== parent && !carried(id).includes(into)) {
        await replace(() => {}); // back where it was until the model says otherwise
        onDrop({ view, from: id, to: into });
        return;
      }
      await moveBy(id, dx, dy);
    }

    function toggleArrange() {
      state.arranging = !state.arranging;
      sheet.setArranging(state.arranging);
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
      reportLayout();
      // A mark on Export while the arrangement differs from the file it saves.
      exportMenu.setBadge(!onLayout && scope.dirty() ? '•' : '');
      const foldable = scope.foldable(graph.id).length > 0;
      can.fold = foldable;
      can.unfold = foldable && foldedOf(graph.id).size > 0;
      can.arrange = !render.isSequence(graph);
      can.reset = Boolean(scope.arranged(graph.id));
      sheet.setArranging(state.arranging && !render.isSequence(graph));
      sheet.cancelConnect();
      try {
        const view = await layout(graph);
        if (state.turn !== turn) return;
        canvas.innerHTML = view.svg; // escaped by ArcVizRender
        if (layerStore) layersKit.decorate(canvas, layerStore, graph.id); // zorder hook
        fit();
        if (editor) editor.refresh();
        drawChains(graph);
        drawCrumbs(graph);
        drawFilterMenu(graph);
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
    const edit = options && options.edit && deps.edit ? options.edit : null;
    const editingHooks = {};
    const kindOf = (end) => deps.place.endKind(end);
    const endName = (end) => deps.place.endName(end, nav.nameOf);
    const sheet = deps.stage.create({
      editing: edit ? editingHooks : null,
      stage,
      canvas,
      onZoom: showZoom,
      viewSize: activeView,
      onPick: (element) => select(element),
      onOpen: (element) => {
        const node = nodeUnder(element);
        if (!node) return;
        if (layersApi && layersApi.open(node.dataset.id)) return; // zorder hook: enter a group
        if (onOpen && onOpen({ view: state.active, id: node.dataset.id, name: nav.nameOf(node.dataset.id), kind: node.dataset.kind })) return;
        toggleFold(node.dataset.id);
      },
      onKey: (event) => {
        const step = { ArrowLeft: [-NUDGE, 0], ArrowRight: [NUDGE, 0], ArrowUp: [0, -NUDGE], ArrowDown: [0, NUDGE] }[event.key];
        if (event.key === 'Escape' && layersApi && layersApi.enteredGroup()) layersApi.exit(); // zorder hook
        else if (event.key === 'Escape') select(null);
        else if (event.key === 'Enter' && state.selected && canFold(state.selected)) toggleFold(state.selected);
        else if (state.arranging && event.shiftKey && step && state.selected && carried(state.selected).length > 0) moveBy(state.selected, step[0], step[1]);
        else return false;
        return true;
      },
      carried: (id) => carried(id),
      onMove: (id, dx, dy, into) => placeOrDrop(id, dx, dy, into),
      onResize: (id, dw, dh, dx, dy) => resizeBy(id, dw, dh, dx, dy),
      onDrop: onDrop ? (from, to) => onDrop({ view: state.active, from, to }) : null,
      onConnect: onConnect
        ? (from, to) => onConnect(from.dataset.id, to.dataset.id, { fromKind: kindOf(from), toKind: kindOf(to), view: state.active })
        : null,
      connectTargets: connectTargets || null,
      nameOf: endName,
    });
    const fit = sheet.fit;
    const editor = edit ? deps.edit.attach({ stage, canvas, sheet, hooks: editingHooks, options: edit, connects: Boolean(onConnect), view: () => state.active, select, carried, scope, replace, announce, drop: onDrop ? (from, to) => onDrop({ view: state.active, from, to }) : null }) : null;
    // zorder hook: the Arrange commands (arcviz-layers.js) on this viewer, and what the host's API gains.
    const layersApi = layerStore ? (() => {
      const commands = layersKit.attach({ store: layerStore, scope, replace, view: () => state.active, shown: activeView, announce });
      const isGroup = layersKit.isGroupId;
      const framesDeep = (id) => {
        const group = layerStore.groupById(state.active, id);
        return group ? group.members.filter(isGroup).flatMap((inner) => [inner, ...framesDeep(inner)]) : [];
      };
      const redecorate = () => layersKit.decorate(canvas, layerStore, state.active);
      return {
        ...commands,
        framesDeep,
        /** A double-click: into the group under it (a collapsed one opens); false when it is not in one. */
        open(id) {
          const to = layersKit.redirect(layerStore, state.active, id);
          if (!isGroup(to)) return false;
          const group = layerStore.groupById(state.active, to);
          if (group && group.collapsed) {
            commands.collapseGroup(to, false);
            return true;
          }
          layerStore.enter(state.active, to);
          redecorate();
          announce('Editing inside the group: Esc leaves it.');
          return true;
        },
        exit() {
          layerStore.enter(state.active, null);
          redecorate();
          select(null);
          announce('Left the group.');
        },
        api: {
          arrange: { ...commands, enterGroup: (id) => { const done = commands.enterGroup(id); redecorate(); return done; }, exitGroup: () => { commands.exitGroup(); redecorate(); } },
          bringToFront: commands.bringToFront,
          sendToBack: commands.sendToBack,
          bringForward: commands.bringForward,
          sendBackward: commands.sendBackward,
          /** Several boxes moved at once: a group moves its members, a locked box stays. */
          moveMany: (moves) => replace(() => {
            for (const { id, dx, dy } of moves || []) {
              for (const member of isGroup(id) ? commands.membersDeep(id) : [id]) if (carried(member, true).length > 0) scope.moveBy(state.active, member, dx, dy);
            }
          }, { manual: true }),
        },
      };
    })() : null;

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
      // An editor selects ports too (a port is dragged, and has properties of its own).
      const match = [...canvas.querySelectorAll(editor ? '.av-node, .av-edge, .av-port' : '.av-node, .av-edge')].find((item) => item.dataset.id === id);
      return select(match || null);
    }

    async function select(target) {
      const asked = state.active;
      const view = await activeView();
      if (!view || state.active !== asked) return; // the view changed meanwhile
      const port = editor && target && target.closest ? target.closest('.av-port') : null;
      let group = !port && target && target.closest ? target.closest('.av-edge, .av-node') : null;
      // zorder hook: a member of a group stands for its group, unless the group is entered.
      if (group && layerStore && group.classList.contains('av-node')) {
        const to = layersKit.redirect(layerStore, state.active, group.dataset.id);
        if (to !== group.dataset.id) group = [...canvas.querySelectorAll('.av-node')].find((item) => item.dataset.id === to) || group;
      }
      picker.value = group ? group.dataset.id : '';
      for (const marked of canvas.querySelectorAll('.is-selected, .is-linked')) marked.classList.remove('is-selected', 'is-linked');
      if (port) {
        // A port: the host shows its properties; the drawing marks it and the lines through it.
        port.classList.add('is-selected');
        deps.place.markPortLines(canvas, view.index.edges, port.dataset.id);
        state.selected = null;
        editor.refresh();
        sheet.refreshHandle();
        showDetails(null);
        onSelect({ view: state.active, id: port.dataset.id, name: endName(port), kind: 'port', isEdge: false });
        return;
      }
      if (group) group.classList.add('is-selected');
      if (editor) editor.refresh();
      if (!group) {
        state.selected = null;
        showDetails(null);
        nav.mark(state.active, null);
        sheet.refreshHandle();
        onSelect(null);
        return;
      }
      const id = group.dataset.id;
      state.selected = id;
      group.classList.add('is-selected');
      sheet.refreshHandle();
      nav.mark(state.active, group.classList.contains('av-node') ? id : null);
      if (group.classList.contains('av-node')) {
        for (const edge of canvas.querySelectorAll('.av-edge')) {
          if (edge.dataset.source === id || edge.dataset.target === id) edge.classList.add('is-linked');
        }
        const entry = view.index.nodes.get(id);
        showDetails(entry, view);
        onSelect({ view: state.active, id, name: displayName(entry.node, ''), kind: entry.node.arc.kind, isEdge: false });
      } else {
        const edge = view.index.edges.get(id);
        showDetails(edge, view, true);
        onSelect({ view: state.active, id, name: displayName(edge, ''), kind: edge.arc.kind, isEdge: true });
      }
    }

    /** A node's ports as `in Name (Interface)`. */
    function portLabels(node) {
      return (node.ports || []).map((port) => {
        const arc = port.arc;
        const carried = arc.interface ? ` (${arc.interface})` : '';
        return `${arc.direction === 'undirected' ? '' : arc.direction + ' '}${arc.name}${carried}`;
      });
    }

    function row(list, term, value) {
      if (!value) return;
      list.appendChild(el('dt', { text: term }));
      list.appendChild(el('dd', { text: value }));
    }

    function showDetails(entry, view, isEdge) {
      details.textContent = '';
      if (related) related.textContent = '';
      if (!entry) {
        details.appendChild(el('p', { class: 'arcviz-quiet', text: 'Nothing selected.' }));
        if (related) related.appendChild(el('p', { class: 'arcviz-quiet', text: 'Select an element to see where it leads.' }));
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
        row(list, 'Ports', portLabels(node).join(', '));
      }
      if (isEdge) {
        details.appendChild(list);
        if (related) related.appendChild(el('p', { class: 'arcviz-quiet', text: 'An exchange leads nowhere else: select one of its ends.' }));
        return;
      }
      // Where the element leads comes first: it is what the reader acts on.
      const id = entry.node.id;
      if (related) {
        const node = entry.node;
        const parent = entry.parent ? view.index.nodes.get(entry.parent).node : null;
        related.appendChild(nav.browser(state.active, id, {
          name: displayName(node, render.nodeStyle(node.arc.kind).label),
          kind: render.nodeStyle(node.arc.kind).label,
          inside: parent ? parent.labels[0].text : null,
          insideId: parent ? parent.id : null,
          ports: portLabels(node),
        }));
      } else {
        details.appendChild(nav.related(state.active, id));
      }
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
      for (const old of canvas.querySelectorAll('.av-chain-graph')) old.remove();
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
      drawChainGraph(chain);
    }

    /**
     * The control nodes (AND, OR, ITERATE) and sequence links of the lit
     * chain, over the sheet: a control node sits between the steps it
     * joins, a sequence link is a dashed arrow with its condition.
     */
    function drawChainGraph(chain) {
      for (const old of canvas.querySelectorAll('.av-chain-graph')) old.remove();
      const controls = chain.controls || [];
      const sequences = chain.sequences || [];
      if (controls.length === 0 && sequences.length === 0) return;
      const svg = canvas.querySelector('svg.av-svg');
      if (!svg) return;
      const NS = 'http://www.w3.org/2000/svg';
      const centreOf = (id) => {
        const item = canvas.querySelector(`.av-node[data-id="${CSS.escape(id)}"], .av-edge[data-id="${CSS.escape(id)}"]`);
        if (!item) return null;
        const box = item.getBBox();
        const matrix = item.getCTM();
        const root = svg.getCTM();
        const point = svg.createSVGPoint();
        point.x = box.x + box.width / 2;
        point.y = box.y + box.height / 2;
        const moved = root ? point.matrixTransform(matrix).matrixTransform(root.inverse()) : point.matrixTransform(matrix);
        return { x: moved.x, y: moved.y };
      };
      // A referenced chain stands at the mean of its own nodes.
      const chainAt = (id) => {
        const other = (activeGraph().arc.chains || []).find((c) => c.id === id);
        const points = other ? other.nodes.map(centreOf).filter(Boolean) : [];
        return points.length ? { x: points.reduce((s, p) => s + p.x, 0) / points.length, y: points.reduce((s, p) => s + p.y, 0) / points.length } : null;
      };
      // A control node is placed at the mean of the steps linked to it.
      const places = new Map();
      for (const control of controls) {
        const around = sequences
          .filter((link) => link.source === control.id || link.target === control.id)
          .map((link) => (link.source === control.id ? link.target : link.source))
          .filter((id) => !controls.some((other) => other.id === id))
          .map((id) => centreOf(id) || chainAt(id))
          .filter(Boolean);
        if (around.length === 0) continue;
        places.set(control.id, {
          x: around.reduce((sum, p) => sum + p.x, 0) / around.length,
          y: around.reduce((sum, p) => sum + p.y, 0) / around.length,
        });
      }
      const where = (id) => places.get(id) || centreOf(id) || chainAt(id);
      const layer = document.createElementNS(NS, 'g');
      layer.setAttribute('class', 'av-chain-graph');
      for (const link of sequences) {
        const from = where(link.source);
        const to = where(link.target);
        if (!from || !to) continue;
        const line = document.createElementNS(NS, 'line');
        line.setAttribute('x1', from.x); line.setAttribute('y1', from.y);
        line.setAttribute('x2', to.x); line.setAttribute('y2', to.y);
        line.setAttribute('class', 'av-sequence');
        const title = document.createElementNS(NS, 'title');
        title.textContent = link.condition ? `Sequence [${link.condition}]` : 'Sequence';
        line.appendChild(title);
        layer.appendChild(line);
        if (link.condition) {
          const text = document.createElementNS(NS, 'text');
          text.setAttribute('x', (from.x + to.x) / 2);
          text.setAttribute('y', (from.y + to.y) / 2 - 6);
          text.setAttribute('class', 'av-sequence-label');
          text.setAttribute('text-anchor', 'middle');
          text.textContent = `[${link.condition}]`;
          layer.appendChild(text);
        }
      }
      for (const control of controls) {
        const at = places.get(control.id);
        if (!at) continue;
        const group = document.createElementNS(NS, 'g');
        group.setAttribute('class', 'av-control');
        group.setAttribute('data-kind', control.kind);
        const shape = document.createElementNS(NS, control.kind === 'and' ? 'rect' : 'polygon');
        if (control.kind === 'and') {
          shape.setAttribute('x', at.x - 22); shape.setAttribute('y', at.y - 4);
          shape.setAttribute('width', 44); shape.setAttribute('height', 8);
        } else {
          shape.setAttribute('points', `${at.x},${at.y - 14} ${at.x + 14},${at.y} ${at.x},${at.y + 14} ${at.x - 14},${at.y}`);
        }
        const title = document.createElementNS(NS, 'title');
        title.textContent = `${control.kind.toUpperCase()}: ${control.name}`;
        group.appendChild(title);
        group.appendChild(shape);
        const text = document.createElementNS(NS, 'text');
        text.setAttribute('x', at.x); text.setAttribute('y', at.y + 26);
        text.setAttribute('text-anchor', 'middle');
        text.setAttribute('class', 'av-control-label');
        text.textContent = control.kind.toUpperCase();
        group.appendChild(text);
        layer.appendChild(group);
      }
      svg.appendChild(layer);
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
      download: save,
      filename,
      activeView,
      svgFor: (view) => render.renderSvg(view.graph, themeOptions('exports')),
      onFailure: (message) => notes.appendChild(el('p', { class: 'arcviz-note', role: 'status', text: message })),
    });
    const saveSvg = exporter.saveSvg;
    const savePng = exporter.savePng;

    // Print pages cannot wait for a layout, so every view is laid out in
    // the background once the first one is on screen.
    function buildPrintPages() {
      const pages = graphs
        .filter((graph) => state.settled.has(graph.id))
        .map((graph) => ({ title: graph.arc.title, svg: render.renderSvg(state.settled.get(graph.id).graph, themeOptions('exports')).svg }));
      exporter.printPages(printHost, pages);
    }
    window.addEventListener('beforeprint', buildPrintPages);
    window.addEventListener('afterprint', () => { printHost.textContent = ''; });

    if (themeState.refused) {
      notes.appendChild(el('p', { class: 'arcviz-note', role: 'status', text: `The theme was refused, the built-in colours are shown: ${themeState.refused.map((e) => e.message).join('; ')}` }));
    }

    /**
     * Colour every view with `theme` (a theme object, a preset id, or null
     * for the built-in colours); `settings.dark` draws its dark variant on
     * screen, `settings.draft` previews a theme being edited even when the
     * validator refuses it. A refused theme changes nothing otherwise.
     * Returns the validator's verdict.
     */
    async function setTheme(theme, settings) {
      const verdict = adoptTheme(theme, settings);
      if (!verdict.ok && !(settings && settings.draft && verdict.theme)) return verdict;
      drawLegendNow();
      state.views.clear(); // the layouts stay: only the colours change
      for (const [id, entry] of state.settled) state.settled.set(id, { ...entry, ...render.renderSvg(entry.graph, themeOptions('screen')) });
      await redraw();
      return verdict;
    }
    function drawLegendNow() {
      // The legend's swatches come from the renderer's own theme.
      if (render.setTheme) render.setTheme(themeState.screen || null);
    }
    drawLegendNow();

    /** The laid-out view `id` (the one on screen when left out), drawn for export. */
    async function exportDrawing(id) {
      const graph = id ? graphs.find((candidate) => candidate.id === id) : activeGraph();
      if (!graph) return null;
      const view = id && id !== state.active ? await layout(graph) : await activeView();
      if (!view) return null;
      return { view, graph, drawn: render.renderSvg(view.graph, themeOptions('exports')) };
    }

    /** What a view draws, for the notes of a slide: [{ id, name, kind }], boxes then lines. */
    function contentsOf(view) {
      const items = [];
      for (const { node } of view.index.nodes.values()) {
        const kind = (node.arc || {}).kind;
        if (kind === 'initial_state' || kind === 'junction') continue;
        items.push({ id: node.id, name: displayName(node, render.nodeStyle(kind).label), kind: render.nodeStyle(kind).label });
      }
      for (const edge of view.index.edges.values()) {
        const kind = (edge.arc || {}).kind;
        items.push({ id: edge.id, name: displayName(edge, render.edgeStyle(kind).label), kind: render.edgeStyle(kind).label, line: true });
      }
      return items;
    }

    /**
     * The view on screen (or `options.view`) as an SVG for the clipboard
     * or a file: only the box around `options.ids` (the selection when
     * true) when given and drawn. { svg, width, height, cropped }.
     */
    async function exportSvg(options) {
      const found = await exportDrawing(options && options.view);
      if (!found) return null;
      const ids = options && options.ids === true ? (state.selected ? [state.selected] : []) : (options && options.ids) || [];
      const box = ids.length ? deps.exporter.boundsOf(found.view.graph, ids) : null;
      const drawn = box ? deps.exporter.crop(found.drawn, box) : found.drawn;
      const svg = options && options.transparent ? deps.exporter.withoutPaper(drawn.svg) : drawn.svg;
      return { svg, width: drawn.width, height: drawn.height, cropped: Boolean(box), title: found.graph.arc.title, view: found.graph.id, elements: contentsOf(found.view) };
    }

    /** The same as a PNG blob, `options.scale` pixels per unit (2 by default). */
    async function exportImage(options) {
      const drawing = await exportSvg(options);
      if (!drawing) return null;
      const blob = await exporter.rasterize(drawing, { scale: (options && options.scale) || 2, transparent: Boolean(options && options.transparent) });
      return { ...drawing, blob };
    }

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
      select: selectById,
      fit,
      /** The view on screen, what is selected in it and where the sheet is. */
      current: () => ({ view: state.active, selected: state.selected, position: sheet.position() }),
      /** Put the sheet back where `current()` saw it. */
      restore: (position) => sheet.restore(position),
      /** The zoom of the sheet: screen pixels per sheet unit. */
      scale: () => sheet.scale(),
      /** Read the host's history and selection into the toolbar again (after either changed). */
      refreshTools,
      // moveMany, routeOf, setRoute: an editing host's (arcviz-edit.js).
      ...(editor ? editor.api : {}),
      // zorder hook: front, back, forward, backward, groups, locks, layers (arcviz-layers.js).
      ...(layersApi ? layersApi.api : {}),
      views: () => graphs.map((graph) => ({ id: graph.id, view: graph.arc.view, title: graph.arc.title })),
      svgOf: (id) => (state.settled.get(id) || {}).svg || null,
      buildPrintPages,
      setTheme,
      /** Tidy the view up, keeping its arrangement (see tidyUp); and what follows a move by hand, from the host's setting. */
      tidyUp,
      setReorganize: (mode) => {
        if (REORGANIZE.includes(mode)) reorganize = mode;
      },
      /** The theme drawn (complete), null for the built-in colours; and the reasons the last one was refused. */
      theme: () => ({ theme: themeState.theme, refused: themeState.refused }),
      exportSvg,
      exportImage,
    };
  }

  root.ArcViz = { mount };
})(typeof globalThis !== 'undefined' ? globalThis : this);
