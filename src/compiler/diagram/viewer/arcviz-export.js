/*
 * ArcViz export — a view as an SVG file, as a PNG image, and the print
 * pages. The SVG it writes or inserts comes from ArcVizRender, which
 * escapes every model value.
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
   */
  function create(deps) {
    const { el, download, filename, activeView, onFailure } = deps;
    const failed = () => onFailure('The PNG could not be produced by this browser. Save the SVG instead: it has no size limit.');

    async function saveSvg() {
      const view = await activeView();
      if (view) download(new Blob([view.svg], { type: SVG_TYPE }), filename('svg'));
    }

    async function savePng() {
      const view = await activeView();
      if (!view) return;
      const url = URL.createObjectURL(new Blob([view.svg], { type: SVG_TYPE }));
      const image = new Image();
      image.onload = () => {
        const scale = Math.min(4, PNG_LONG_SIDE / Math.max(view.width, view.height));
        const surface = el('canvas', { width: Math.round(view.width * scale), height: Math.round(view.height * scale) });
        const context = surface.getContext('2d');
        context.fillStyle = '#FFFFFF';
        context.fillRect(0, 0, surface.width, surface.height);
        context.drawImage(image, 0, 0, surface.width, surface.height);
        URL.revokeObjectURL(url);
        surface.toBlob((blob) => {
          if (blob) download(blob, filename('png'));
          else failed();
        }, 'image/png');
      };
      image.onerror = () => {
        URL.revokeObjectURL(url);
        failed();
      };
      image.src = url;
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

    return { saveSvg, savePng, saveLayout, printPages };
  }

  root.ArcVizExport = { create };
})(typeof globalThis !== 'undefined' ? globalThis : this);
