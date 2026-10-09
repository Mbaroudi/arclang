/*
 * ArcViz stage — the frame the sheet sits in: pan, zoom, fit, full screen,
 * and the gestures on it. A click picks what is under the pointer, a
 * double-click opens it, and in arrange mode a drag moves a box instead of
 * the sheet. What those gestures mean is the viewer's business.
 */
(function (root) {
  'use strict';

  const MIN_SCALE = 0.1;
  const MAX_SCALE = 4;
  const ZOOM_STEP = 1.25;
  const PAN_STEP = 60;
  const FIT_PADDING = 24;
  const CLICK_SLOP = 4; // pixels a pointer may wander and still be a click

  /**
   * @param {object} deps
   * @param {Element}  deps.stage      the frame
   * @param {Element}  deps.canvas     what is panned and zoomed inside it
   * @param {Function} deps.viewSize   () -> Promise<{width, height}|null> of the sheet
   * @param {Function} deps.onPick     (element under the pointer) on a click
   * @param {Function} deps.onOpen     (element under the pointer) on a double-click
   * @param {Function} deps.onKey      (event) -> true when the viewer handled the key
   * @param {Function} deps.carried    (node id) -> ids that move with it, [] if it cannot move
   * @param {Function} deps.onMove     (node id, dx, dy) in sheet units, when a box is dropped
   */
  function create(deps) {
    const { stage, canvas, viewSize, onPick, onOpen, onKey, carried, onMove } = deps;
    const at = { scale: 1, x: 0, y: 0 };
    let arranging = false;

    const apply = () => {
      canvas.style.transform = `translate(${at.x}px, ${at.y}px) scale(${at.scale})`;
    };

    async function fit() {
      const size = await viewSize();
      if (!size) return;
      const frame = stage.getBoundingClientRect();
      at.scale = Math.max(MIN_SCALE, Math.min((frame.width - FIT_PADDING) / size.width, (frame.height - FIT_PADDING) / size.height, 1.5));
      at.x = (frame.width - size.width * at.scale) / 2;
      at.y = Math.max(FIT_PADDING / 2, (frame.height - size.height * at.scale) / 2);
      apply();
    }

    function zoomBy(factor, originX, originY) {
      const frame = stage.getBoundingClientRect();
      const ox = originX === undefined ? frame.width / 2 : originX;
      const oy = originY === undefined ? frame.height / 2 : originY;
      const next = Math.min(MAX_SCALE, Math.max(MIN_SCALE, at.scale * factor));
      const ratio = next / at.scale;
      at.x = ox - (ox - at.x) * ratio;
      at.y = oy - (oy - at.y) * ratio;
      at.scale = next;
      apply();
    }

    function panBy(dx, dy) {
      at.x += dx;
      at.y += dy;
      apply();
    }

    /** Bring a drawn element to the middle of the frame. */
    function centerOn(element) {
      if (!element) return;
      const box = element.getBoundingClientRect();
      const frame = stage.getBoundingClientRect();
      panBy(frame.left + frame.width / 2 - (box.left + box.width / 2), frame.top + frame.height / 2 - (box.top + box.height / 2));
    }

    // The stage captures the pointer, so events name the stage: the element
    // is the one under the pointer.
    const under = (event) => document.elementFromPoint(event.clientX, event.clientY);
    const groupsOf = (ids) => [...canvas.querySelectorAll('.av-node')].filter((group) => ids.includes(group.dataset.id));

    let drag = null;
    stage.addEventListener('pointerdown', (event) => {
      if (event.button !== 0) return;
      drag = { x: event.clientX, y: event.clientY, startX: event.clientX, startY: event.clientY, moved: false, box: null };
      if (arranging) {
        const hit = under(event);
        const node = hit && hit.closest ? hit.closest('.av-node') : null;
        const ids = node ? carried(node.dataset.id) : [];
        if (ids.length > 0) drag.box = { id: node.dataset.id, groups: groupsOf(ids) };
      }
      stage.setPointerCapture(event.pointerId);
    });
    stage.addEventListener('pointermove', (event) => {
      if (!drag) return;
      const dx = event.clientX - drag.x;
      const dy = event.clientY - drag.y;
      if (!drag.moved && Math.hypot(event.clientX - drag.startX, event.clientY - drag.startY) < CLICK_SLOP) return;
      drag.moved = true;
      if (drag.box) {
        // The box follows the pointer; its exchanges are redrawn on drop.
        const shift = `translate(${(event.clientX - drag.startX) / at.scale} ${(event.clientY - drag.startY) / at.scale})`;
        for (const group of drag.box.groups) group.setAttribute('transform', shift);
        stage.classList.add('is-moving');
        return;
      }
      stage.classList.add('is-panning');
      drag.x = event.clientX;
      drag.y = event.clientY;
      panBy(dx, dy);
    });
    const release = () => {
      drag = null;
      stage.classList.remove('is-panning', 'is-moving');
    };
    stage.addEventListener('pointerup', (event) => {
      const done = drag;
      release();
      if (!done) return;
      if (!done.moved) onPick(under(event));
      else if (done.box) onMove(done.box.id, (event.clientX - done.startX) / at.scale, (event.clientY - done.startY) / at.scale);
    });
    stage.addEventListener('pointercancel', () => {
      if (drag && drag.box) for (const group of drag.box.groups) group.removeAttribute('transform');
      release();
    });
    stage.addEventListener('dblclick', (event) => onOpen(under(event)));
    stage.addEventListener('wheel', (event) => {
      if (!event.ctrlKey && !event.metaKey) return; // leave page scrolling alone
      event.preventDefault();
      const frame = stage.getBoundingClientRect();
      zoomBy(event.deltaY < 0 ? 1.1 : 1 / 1.1, event.clientX - frame.left, event.clientY - frame.top);
    }, { passive: false });
    stage.addEventListener('keydown', (event) => {
      if (event.ctrlKey || event.metaKey || event.altKey) return; // browser shortcuts
      if (onKey(event)) {
        event.preventDefault();
        return;
      }
      const pan = { ArrowLeft: [PAN_STEP, 0], ArrowRight: [-PAN_STEP, 0], ArrowUp: [0, PAN_STEP], ArrowDown: [0, -PAN_STEP] }[event.key];
      if (pan) panBy(pan[0], pan[1]);
      else if (event.key === '+' || event.key === '=') zoomBy(ZOOM_STEP);
      else if (event.key === '-') zoomBy(1 / ZOOM_STEP);
      else if (event.key === '0') fit();
      else return;
      event.preventDefault();
    });

    function toggleFullscreen() {
      if (document.fullscreenElement) document.exitFullscreen();
      else if (stage.requestFullscreen) stage.requestFullscreen().catch(() => {});
    }
    document.addEventListener('fullscreenchange', () => setTimeout(fit, 120));
    if (typeof ResizeObserver !== 'undefined') {
      let width = 0;
      new ResizeObserver(() => {
        const now = stage.getBoundingClientRect().width;
        if (Math.abs(now - width) > 2) {
          width = now;
          fit();
        }
      }).observe(stage);
    }

    return {
      fit,
      zoomIn: () => zoomBy(ZOOM_STEP),
      zoomOut: () => zoomBy(1 / ZOOM_STEP),
      centerOn,
      toggleFullscreen,
      /** Where the sheet is: kept across a redraw that does not change its size much. */
      position: () => ({ ...at }),
      restore: (position) => {
        Object.assign(at, position);
        apply();
      },
      setArranging: (on) => {
        arranging = on;
        stage.classList.toggle('is-arranging', on);
      },
    };
  }

  root.ArcVizStage = { create };
})(typeof globalThis !== 'undefined' ? globalThis : this);
