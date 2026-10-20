/*
 * ArcViz export — a view as an SVG file, as a PNG image, and the print
 * pages; a view or the part of it around a selection as an image for the
 * clipboard (a slide, a document). The SVG it writes or inserts comes from
 * ArcVizRender, which escapes every model value. Exports are drawn with the
 * export theme (`deps.svgFor`), not with what a dark editor shows.
 */
(function (root) {
  'use strict';

  const PNG_LONG_SIDE = 3200;
  const SVG_TYPE = 'image/svg+xml;charset=utf-8';

  /**
   * @param {object} deps
   * @param {Function} deps.el          element builder of the viewer
   * @param {Function} deps.download    (blob, filename) -> void
   * @param {Function} deps.filename    (extension) -> name for the view on screen
   * @param {Function} deps.activeView  () -> Promise of the laid-out view, or null
   * @param {Function} deps.onFailure   (message) -> void, shown to the reader
   * @param {Function} [deps.svgFor]    (view) -> { svg, width, height }: the
   *                                    view drawn for export; the drawing on
   *                                    screen when left out
   */
  function create(deps) {
    const { el, download, filename, activeView, onFailure } = deps;
    const svgFor = deps.svgFor || ((view) => view);
    const failed = () => onFailure('The PNG could not be produced by this browser. Save the SVG instead: it has no size limit.');

    async function saveSvg() {
      const view = await activeView();
      if (view) download(new Blob([svgFor(view).svg], { type: SVG_TYPE }), filename('svg'));
    }

    async function savePng() {
      const view = await activeView();
      if (!view) return;
      const drawn = svgFor(view);
      const scale = Math.min(4, PNG_LONG_SIDE / Math.max(drawn.width, drawn.height));
      try {
        download(await rasterize(drawn, { scale }), filename('png'));
      } catch (error) {
        failed();
      }
    }

    /**
     * A drawing as a PNG blob: `scale` pixels per sheet unit, on its own
     * paper (`transparent` drops it).
     */
    function rasterize(drawn, options) {
      const scale = (options && options.scale) || 2;
      const markup = options && options.transparent ? withoutPaper(drawn.svg) : drawn.svg;
      return new Promise((resolve, reject) => {
        const url = URL.createObjectURL(new Blob([markup], { type: SVG_TYPE }));
        const image = new Image();
        image.onload = () => {
          const surface = el('canvas', { width: Math.max(1, Math.round(drawn.width * scale)), height: Math.max(1, Math.round(drawn.height * scale)) });
          const context = surface.getContext('2d');
          context.drawImage(image, 0, 0, surface.width, surface.height);
          URL.revokeObjectURL(url);
          surface.toBlob((blob) => (blob ? resolve(blob) : reject(new Error('the canvas gave no PNG'))), 'image/png');
        };
        image.onerror = () => {
          URL.revokeObjectURL(url);
          reject(new Error('the drawing could not be loaded as an image'));
        };
        image.src = url;
      });
    }

    /**
     * Fill `host` with one page per view. `pages` is [{ title, svg }], laid
     * out beforehand: printing cannot wait for a layout.
     */
    function printPages(host, pages) {
      host.textContent = '';
      for (const { title, svg } of pages) {
        const frame = el('div', { class: 'arcviz-print-frame' });
        frame.innerHTML = svg; // escaped by ArcVizRender
        const drawing = frame.querySelector('svg');
        drawing.removeAttribute('width');
        drawing.removeAttribute('height');
        host.appendChild(el('div', { class: 'arcviz-print-page' }, [el('h3', { text: title }), frame]));
      }
    }

    /** The arrangement as a layout file, to keep next to the model. */
    function saveLayout(layout, name) {
      download(new Blob([`${JSON.stringify(layout, null, 2)}\n`], { type: 'application/json' }), name);
    }

    return { saveSvg, savePng, saveLayout, printPages, rasterize };
  }

  /** The SVG without its sheet of paper: what a transparent image shows. */
  function withoutPaper(svg) {
    return svg.replace(/<rect class="av-paper"[^>]*\/>/, '');
  }

  /**
   * The part of a drawing inside `box` ({ x, y, width, height } in sheet
   * units) with a margin around it: the same markup, its view box moved.
   */
  function crop(drawn, box, margin) {
    const pad = margin === undefined ? 16 : margin;
    const x = box.x - pad;
    const y = box.y - pad;
    const width = Math.ceil(box.width + pad * 2);
    const height = Math.ceil(box.height + pad * 2);
    const svg = drawn.svg
      .replace(/^<svg([^>]*?) width="[^"]*" height="[^"]*" viewBox="[^"]*"/, `<svg$1 width="${width}" height="${height}" viewBox="${x} ${y} ${width} ${height}"`)
      .replace(/<rect class="av-paper"[^>]*\/>/, (paper) => paper.replace(/x="[^"]*" y="[^"]*" width="[^"]*" height="[^"]*"/, `x="${x}" y="${y}" width="${width}" height="${height}"`));
    return { svg, width, height };
  }

  /**
   * The box around the drawn elements `ids` of a laid-out graph (sheet
   * units: the drawing's own coordinates), or null when none is drawn.
   */
  function boundsOf(graph, ids) {
    const wanted = new Set(ids);
    let box = null;
    const add = (x, y, width, height) => {
      if (!box) box = { x1: x, y1: y, x2: x + width, y2: y + height };
      else box = { x1: Math.min(box.x1, x), y1: Math.min(box.y1, y), x2: Math.max(box.x2, x + width), y2: Math.max(box.y2, y + height) };
    };
    const origins = new Map();
    const walk = (node, ox, oy) => {
      const x = ox + (node.x || 0);
      const y = oy + (node.y || 0);
      origins.set(node.id, { x, y });
      if (wanted.has(node.id)) add(x, y, node.width || 0, node.height || 0);
      for (const child of node.children || []) walk(child, x, y);
    };
    for (const child of graph.children || []) walk(child, 0, 0);
    for (const edge of graph.edges || []) {
      if (!wanted.has(edge.id)) continue;
      const origin = (edge.container !== graph.id && origins.get(edge.container)) || { x: 0, y: 0 };
      for (const section of edge.sections || []) {
        for (const point of [section.startPoint, ...(section.bendPoints || []), section.endPoint]) add(origin.x + point.x, origin.y + point.y, 0, 0);
      }
    }
    return box && { x: box.x1, y: box.y1, width: box.x2 - box.x1, height: box.y2 - box.y1 };
  }

  root.ArcVizExport = { create, crop, boundsOf, withoutPaper };
})(typeof globalThis !== 'undefined' ? globalThis : this);
