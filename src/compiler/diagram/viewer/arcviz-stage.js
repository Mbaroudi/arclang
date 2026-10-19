/*
 * ArcViz stage — the frame the sheet sits in: pan, zoom, fit, full screen,
 * and the gestures on it. A click picks what is under the pointer, a
 * double-click on a box opens it and on the desk zooms in (with Shift,
 * out), the wheel pans and with Ctrl or ⌘ (a trackpad pinch) zooms about
 * the pointer, two fingers pinch, a flung sheet glides to a stop, and in
 * arrange mode a drag moves a box instead of the sheet. A host that
 * connects elements gets a round handle on the box or port under the
 * pointer: dragged to another box or port, it draws the line (Capella's
 * gesture); tapped, or started with the keyboard, it waits for the target
 * to be chosen. What the picking, opening and connecting gestures mean is
 * the viewer's business.
 */
(function (root) {
  'use strict';

  const MIN_SCALE = 0.1;
  const MAX_SCALE = 4;
  const ZOOM_STEP = 1.25;
  const DOUBLE_CLICK_ZOOM = 2;
  const WHEEL_ZOOM_RATE = 0.0022; // scale = exp(-deltaY * rate): smooth on a trackpad, brisk on a wheel
  const WHEEL_LINE = 16; // pixels per line when a wheel reports lines
  const ZOOM_ANIMATION_MS = 180;
  const PAN_STEP = 60;
  const FIT_PADDING = 24;
  const CLICK_SLOP = 4; // pixels a pointer may wander and still be a click
  const GLIDE_FRICTION = 0.85; // of the velocity kept each frame after a fling: a flick glides about as far as it was dragged
  const GLIDE_STOP = 0.4; // pixels per frame under which the glide ends
  const FLING_WINDOW_MS = 90; // how recent the last movement must be to fling

  const easeOut = (t) => 1 - Math.pow(1 - t, 3);

  /**
   * @param {object} deps
   * @param {Element}  deps.stage      the frame
   * @param {Element}  deps.canvas     what is panned and zoomed inside it
   * @param {Function} deps.viewSize   () -> Promise<{width, height}|null> of the sheet
   * @param {Function} deps.onPick     (element under the pointer) on a click
   * @param {Function} deps.onOpen     (element under the pointer) on a double-click on a box
   * @param {Function} deps.onKey      (event) -> true when the viewer handled the key
   * @param {Function} deps.carried    (node id) -> ids that move with it, [] if it cannot move
   * @param {Function} deps.onMove     (node id, dx, dy) in sheet units, when a box is dropped
   * @param {Function} deps.onDrop     optional: (node id, node id under the pointer or null)
   *                                   when a box is dragged and let go outside arrange mode;
   *                                   given, a drag on a box is that gesture, not a pan
   * @param {Function} deps.onConnect  optional: (from element, to element), the `.av-node`
   *                                   or `.av-port` groups, when a line drawn from the
   *                                   handle of the first ends on the second; given, the
   *                                   box or port under the pointer (else the selected box)
   *                                   shows the handle
   * @param {Function} deps.connectTargets optional: (node or port id) -> ids a line from
   *                                   it may end on, or null for any; absent, any
   * @param {Function} deps.nameOf     optional: (element) -> what to call a box or port,
   *                                   for the handle's accessible name
   */
  function create(deps) {
    const { stage, canvas, viewSize, onPick, onOpen, onKey, carried, onMove, onDrop, onConnect, connectTargets } = deps;
    const nameOf = deps.nameOf || ((element) => element.dataset.id);
    // The line drawn from a dragged box to the pointer.
    const ghost = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
    ghost.setAttribute('class', 'arcviz-ghost');
    ghost.setAttribute('aria-hidden', 'true');
    const ghostLine = document.createElementNS('http://www.w3.org/2000/svg', 'line');
    ghost.appendChild(ghostLine);
    let dropTarget = null;
    const markTarget = (node) => {
      if (dropTarget === node) return;
      if (dropTarget) dropTarget.classList.remove('is-drop-target');
      dropTarget = node;
      if (dropTarget) dropTarget.classList.add('is-drop-target');
    };
    const at = { scale: 1, x: 0, y: 0 };
    let arranging = false;

    const apply = () => {
      canvas.style.transform = `translate(${at.x}px, ${at.y}px) scale(${at.scale})`;
      placeHandle();
    };

    // --- motion: one animation at a time, cancelled by any new gesture ---
    let motion = null;
    const stopMotion = () => {
      if (motion) cancelAnimationFrame(motion);
      motion = null;
    };
    /** Glide from where the sheet is to `target` over a short moment. */
    function animateTo(target) {
      stopMotion();
      const from = { ...at };
      const started = performance.now();
      const step = (now) => {
        const t = Math.min(1, (now - started) / ZOOM_ANIMATION_MS);
        const k = easeOut(t);
        at.scale = from.scale + (target.scale - from.scale) * k;
        at.x = from.x + (target.x - from.x) * k;
        at.y = from.y + (target.y - from.y) * k;
        apply();
        motion = t < 1 ? requestAnimationFrame(step) : null;
      };
      motion = requestAnimationFrame(step);
    }

    async function fit() {
      const size = await viewSize();
      if (!size) return;
      stopMotion();
      const frame = stage.getBoundingClientRect();
      at.scale = Math.max(MIN_SCALE, Math.min((frame.width - FIT_PADDING) / size.width, (frame.height - FIT_PADDING) / size.height, 1.5));
      at.x = (frame.width - size.width * at.scale) / 2;
      at.y = Math.max(FIT_PADDING / 2, (frame.height - size.height * at.scale) / 2);
      apply();
    }

    /** Where the sheet lands when scaled by `factor` about a point of the frame. */
    function scaledBy(factor, originX, originY) {
      const frame = stage.getBoundingClientRect();
      const ox = originX === undefined ? frame.width / 2 : originX;
      const oy = originY === undefined ? frame.height / 2 : originY;
      const scale = Math.min(MAX_SCALE, Math.max(MIN_SCALE, at.scale * factor));
      const ratio = scale / at.scale;
      return { scale, x: ox - (ox - at.x) * ratio, y: oy - (oy - at.y) * ratio };
    }

    /** Zoom about a point of the frame (its middle by default), at once or animated. */
    function zoomBy(factor, originX, originY, animated) {
      const target = scaledBy(factor, originX, originY);
      if (animated) {
        animateTo(target);
      } else {
        stopMotion();
        Object.assign(at, target);
        apply();
      }
    }

    function panBy(dx, dy) {
      at.x += dx;
      at.y += dy;
      apply();
    }

    /** Let the sheet glide on after a fling, slowing each frame. */
    function glide(vx, vy) {
      stopMotion();
      let [dx, dy] = [vx, vy];
      const step = () => {
        dx *= GLIDE_FRICTION;
        dy *= GLIDE_FRICTION;
        if (Math.hypot(dx, dy) < GLIDE_STOP) {
          motion = null;
          return;
        }
        panBy(dx, dy);
        motion = requestAnimationFrame(step);
      };
      motion = requestAnimationFrame(step);
    }

    /** Bring a drawn element to the middle of the frame. */
    function centerOn(element) {
      if (!element) return;
      stopMotion();
      const box = element.getBoundingClientRect();
      const frame = stage.getBoundingClientRect();
      panBy(frame.left + frame.width / 2 - (box.left + box.width / 2), frame.top + frame.height / 2 - (box.top + box.height / 2));
    }

    // The stage captures the pointer, so events name the stage: the element
    // is the one under the pointer.
    const under = (event) => document.elementFromPoint(event.clientX, event.clientY);
    const groupsOf = (ids) => [...canvas.querySelectorAll('.av-node')].filter((group) => ids.includes(group.dataset.id));
    const local = (event) => {
      const frame = stage.getBoundingClientRect();
      return { x: event.clientX - frame.left, y: event.clientY - frame.top };
    };

    // --- connecting: a handle on a box or port, a line drawn from it ------
    // The handle sits on the box or port under the pointer, else on the
    // selected box so it can be reached by Tab. It lives in the frame, not
    // on the sheet: it keeps its size whatever the zoom.
    const handle = document.createElement('div');
    handle.className = 'arcviz-handle';
    handle.hidden = true;
    handle.setAttribute('role', 'button');
    handle.setAttribute('tabindex', '0');
    handle.setAttribute('title', 'Drag to another box or port to connect them. Enter (or c, with a box selected) starts the same from the keyboard: choose the target with the arrow keys, confirm with Enter, leave with Escape.');
    let hovered = null; // the box or port under the pointer
    let anchor = null; // the one the handle sits on
    let connect = null; // the gesture under way: { from, candidates, armed, picked }
    const endOf = (element) => (element && element.closest ? element.closest('.av-port, .av-node') : null);
    const endUnder = (event) => endOf(under(event));
    const onHandle = (element) => element === handle || handle.contains(element);

    /** Where a line leaves a box or port, in frame coordinates: the middle of the box's right edge, the middle of the port. */
    function spotOf(element) {
      const frame = stage.getBoundingClientRect();
      const box = element.getBoundingClientRect();
      const x = element.classList.contains('av-port') ? box.left + box.width / 2 : box.right;
      return { x: x - frame.left, y: box.top + box.height / 2 - frame.top };
    }
    function placeHandle() {
      if (!anchor) return;
      if (!anchor.isConnected) {
        anchor = null;
        handle.hidden = true;
        return;
      }
      const spot = spotOf(anchor);
      handle.style.translate = `${spot.x}px ${spot.y}px`; // not `transform`: the scale on hover composes with this, not under it
      handle.hidden = false;
    }
    /** The handle on the box or port under the pointer, else on the selected box, else away. */
    function refreshHandle() {
      if (!onConnect || connect) return;
      const next = (hovered && hovered.isConnected ? hovered : null) || canvas.querySelector('.av-node.is-selected');
      if (next !== anchor) {
        anchor = next;
        if (anchor) handle.setAttribute('aria-label', `Connect from ${nameOf(anchor)}`);
        else handle.hidden = true;
      }
      placeHandle();
    }
    const hover = (event) => {
      const hit = under(event);
      if (onHandle(hit)) return; // on the handle itself: it stays
      hovered = endOf(hit);
      refreshHandle();
    };
    function drawBand(to) {
      const from = spotOf(connect.from);
      const end = to || from;
      if (!ghost.isConnected) stage.appendChild(ghost);
      ghostLine.setAttribute('x1', String(from.x));
      ghostLine.setAttribute('y1', String(from.y));
      ghostLine.setAttribute('x2', String(end.x));
      ghostLine.setAttribute('y2', String(end.y));
    }
    /** Mark the end under the pointer when the line may end there. */
    const aimAt = (end) => markTarget(end && connect.candidates.has(end) ? end : null);
    /**
     * Start a line from `from`. The host says which ends may take it, once;
     * the others fade. `armed`: the target is chosen by a later tap, click
     * or key rather than by the drag under way.
     */
    function beginConnect(from, armed) {
      cancelConnect();
      const allowed = connectTargets ? connectTargets(from.dataset.id) : null;
      const valid = allowed ? new Set(allowed) : null;
      const owner = from.closest('.av-node');
      const candidates = new Set([...canvas.querySelectorAll('.av-node, .av-port')]
        .filter((end) => end !== owner && !owner.contains(end) && (!valid || valid.has(end.dataset.id))));
      for (const end of candidates) end.classList.add('is-connect-target');
      from.classList.add('is-connect-source');
      stage.classList.add('is-connecting');
      handle.setAttribute('aria-pressed', 'true');
      connect = { from, candidates, armed, picked: -1 };
      drawBand(null);
    }
    function cancelConnect() {
      if (!connect) return;
      for (const end of connect.candidates) end.classList.remove('is-connect-target');
      connect.from.classList.remove('is-connect-source');
      stage.classList.remove('is-connecting');
      handle.removeAttribute('aria-pressed');
      connect = null;
      if (ghost.isConnected) ghost.remove();
      markTarget(null);
      refreshHandle();
    }
    /** End the line on `end`: the host hears of it when the line may end there. */
    function finishConnect(end) {
      const done = connect;
      const taken = end && done.candidates.has(end) && done.from.isConnected && end.isConnected;
      cancelConnect();
      if (taken) onConnect(done.from, end);
    }
    /** From the keyboard: step through the ends the line may take. */
    function pickCandidate(step) {
      const ends = [...connect.candidates];
      if (ends.length === 0) return;
      connect.picked = (connect.picked + step + ends.length) % ends.length;
      const end = ends[connect.picked];
      markTarget(end);
      drawBand(spotOf(end));
    }
    handle.addEventListener('keydown', (event) => {
      if (connect || !anchor) return; // under way: the frame's keys apply
      if (event.key !== 'Enter' && event.key !== ' ') return;
      beginConnect(anchor, true);
      pickCandidate(1);
      event.preventDefault();
      event.stopPropagation();
    });
    if (onConnect) stage.appendChild(handle);

    // --- pointers: one drags or pans, two pinch ---------------------------
    const pointers = new Map();
    let pinch = null; // { distance, midX, midY } of the two pointers, in frame coordinates
    const pinchOf = () => {
      const [a, b] = [...pointers.values()];
      return { distance: Math.max(1, Math.hypot(b.x - a.x, b.y - a.y)), midX: (a.x + b.x) / 2, midY: (a.y + b.y) / 2 };
    };

    let drag = null;
    stage.addEventListener('pointerdown', (event) => {
      if (event.button !== 0 && event.button !== 1) return;
      stopMotion();
      pointers.set(event.pointerId, local(event));
      stage.setPointerCapture(event.pointerId);
      if (pointers.size === 2) {
        // A second finger: whatever the first was doing becomes a pinch.
        if (drag && drag.box) for (const group of drag.box.groups) group.removeAttribute('transform');
        drag = null;
        cancelConnect();
        pinch = pinchOf();
        stage.classList.add('is-panning');
        return;
      }
      if (pointers.size > 2) return;
      drag = { x: event.clientX, y: event.clientY, startX: event.clientX, startY: event.clientY, moved: false, box: null, from: null, connect: false, vx: 0, vy: 0, lastAt: performance.now() };
      if (event.button === 1) return; // the middle button always pans
      const hit = under(event);
      if (onConnect && onHandle(hit) && anchor && !connect) {
        // From the handle: a drag draws the line, a tap waits for the target.
        beginConnect(anchor, false);
        drag.connect = true;
        return;
      }
      if (connect) {
        // A line waiting for its end: this press chooses it, or lets go.
        drag.connect = true;
        return;
      }
      const node = hit && hit.closest ? hit.closest('.av-node') : null;
      if (arranging) {
        const ids = node ? carried(node.dataset.id) : [];
        if (ids.length > 0) drag.box = { id: node.dataset.id, groups: groupsOf(ids) };
      } else if (onDrop && node) {
        drag.from = node;
      }
    });
    stage.addEventListener('pointermove', (event) => {
      if (pointers.has(event.pointerId)) pointers.set(event.pointerId, local(event));
      if (pinch && pointers.size === 2) {
        const now = pinchOf();
        const target = scaledBy(now.distance / pinch.distance, pinch.midX, pinch.midY);
        Object.assign(at, target);
        at.x += now.midX - pinch.midX;
        at.y += now.midY - pinch.midY;
        apply();
        pinch = now;
        return;
      }
      if (!drag) {
        if (connect) {
          // A line waiting for its end follows the pointer meanwhile.
          drawBand(local(event));
          aimAt(endUnder(event));
        } else if (onConnect) hover(event);
        return;
      }
      const dx = event.clientX - drag.x;
      const dy = event.clientY - drag.y;
      if (!drag.moved && Math.hypot(event.clientX - drag.startX, event.clientY - drag.startY) < CLICK_SLOP) return;
      drag.moved = true;
      if (drag.connect) {
        if (!connect) return;
        drawBand(local(event));
        aimAt(endUnder(event));
        return;
      }
      if (drag.from) {
        // A line from the box to the pointer, and the box under it marked.
        if (!ghost.isConnected) stage.appendChild(ghost);
        const frame = stage.getBoundingClientRect();
        const box = drag.from.getBoundingClientRect();
        ghostLine.setAttribute('x1', String(box.left + box.width / 2 - frame.left));
        ghostLine.setAttribute('y1', String(box.top + box.height / 2 - frame.top));
        ghostLine.setAttribute('x2', String(event.clientX - frame.left));
        ghostLine.setAttribute('y2', String(event.clientY - frame.top));
        const over = under(event);
        const target = over && over.closest ? over.closest('.av-node') : null;
        markTarget(target && target !== drag.from ? target : null);
        stage.classList.add('is-dragging');
        return;
      }
      if (drag.box) {
        // The box follows the pointer; its exchanges are redrawn on drop.
        const shift = `translate(${(event.clientX - drag.startX) / at.scale} ${(event.clientY - drag.startY) / at.scale})`;
        for (const group of drag.box.groups) group.setAttribute('transform', shift);
        stage.classList.add('is-moving');
        return;
      }
      stage.classList.add('is-panning');
      const now = performance.now();
      const dt = Math.max(1, now - drag.lastAt);
      // Velocity in pixels per frame (16 ms), smoothed over the last moves.
      drag.vx = drag.vx * 0.4 + (dx / dt) * 16 * 0.6;
      drag.vy = drag.vy * 0.4 + (dy / dt) * 16 * 0.6;
      drag.lastAt = now;
      drag.x = event.clientX;
      drag.y = event.clientY;
      panBy(dx, dy);
    });
    const release = (event) => {
      if (event) pointers.delete(event.pointerId);
      if (pointers.size < 2) pinch = null;
      drag = null;
      stage.classList.remove('is-panning', 'is-moving', 'is-dragging');
      if (connect) return; // the line waits for its end
      if (ghost.isConnected) ghost.remove();
      markTarget(null);
    };
    stage.addEventListener('pointerup', (event) => {
      const done = drag;
      const pinched = pinch !== null;
      release(event);
      if (pinched || !done) return;
      if (done.connect) {
        if (!connect) return;
        const hit = under(event);
        if (done.moved || connect.armed) finishConnect(onHandle(hit) ? null : endOf(hit));
        else connect.armed = true; // a tap on the handle: the next tap names the end
        return;
      }
      if (!done.moved) {
        if (event.button === 0) onPick(under(event));
      } else if (done.box) {
        onMove(done.box.id, (event.clientX - done.startX) / at.scale, (event.clientY - done.startY) / at.scale);
      } else if (done.from) {
        const over = under(event);
        const target = over && over.closest ? over.closest('.av-node') : null;
        onDrop(done.from.dataset.id, target && target !== done.from ? target.dataset.id : null);
      } else if (performance.now() - done.lastAt < FLING_WINDOW_MS && Math.hypot(done.vx, done.vy) > 2) {
        glide(done.vx, done.vy);
      }
    });
    stage.addEventListener('pointercancel', (event) => {
      if (drag && drag.box) for (const group of drag.box.groups) group.removeAttribute('transform');
      release(event);
    });
    stage.addEventListener('dblclick', (event) => {
      if (event.button !== 0) return;
      event.preventDefault();
      const hit = under(event);
      const node = hit && hit.closest ? hit.closest('.av-node') : null;
      const point = local(event);
      if (event.shiftKey || event.altKey) zoomBy(1 / DOUBLE_CLICK_ZOOM, point.x, point.y, true);
      else if (node) onOpen(hit);
      else zoomBy(DOUBLE_CLICK_ZOOM, point.x, point.y, true);
    });
    stage.addEventListener('wheel', (event) => {
      event.preventDefault();
      stopMotion();
      const unit = event.deltaMode === 1 ? WHEEL_LINE : event.deltaMode === 2 ? stage.clientHeight : 1;
      if (event.ctrlKey || event.metaKey) {
        // A pinch on a trackpad arrives as a wheel with Ctrl held.
        const point = local(event);
        zoomBy(Math.exp(-event.deltaY * unit * WHEEL_ZOOM_RATE), point.x, point.y, false);
      } else {
        // Two fingers, or the wheel (Shift turns it sideways), move the sheet.
        const sideways = event.shiftKey && event.deltaX === 0;
        panBy(-(sideways ? event.deltaY : event.deltaX) * unit, -(sideways ? 0 : event.deltaY) * unit);
      }
    }, { passive: false });
    stage.addEventListener('keydown', (event) => {
      if (event.ctrlKey || event.metaKey || event.altKey) return; // browser shortcuts
      if (connect) {
        if (event.key === 'Escape') cancelConnect();
        else if (!connect.armed) return;
        else if (event.key === 'Enter') finishConnect([...connect.candidates][connect.picked] || null);
        else if (event.key === 'ArrowRight' || event.key === 'ArrowDown' || (event.key === 'Tab' && !event.shiftKey)) pickCandidate(1);
        else if (event.key === 'ArrowLeft' || event.key === 'ArrowUp' || event.key === 'Tab') pickCandidate(-1);
        else return;
        event.preventDefault();
        return;
      }
      if (onConnect && event.key === 'c') {
        const selected = canvas.querySelector('.av-node.is-selected');
        if (!selected) return;
        beginConnect(selected, true);
        pickCandidate(1);
        event.preventDefault();
        return;
      }
      if (onKey(event)) {
        event.preventDefault();
        return;
      }
      const pan = { ArrowLeft: [PAN_STEP, 0], ArrowRight: [-PAN_STEP, 0], ArrowUp: [0, PAN_STEP], ArrowDown: [0, -PAN_STEP] }[event.key];
      if (pan) {
        stopMotion();
        panBy(pan[0], pan[1]);
      } else if (event.key === '+' || event.key === '=') zoomBy(ZOOM_STEP, undefined, undefined, true);
      else if (event.key === '-') zoomBy(1 / ZOOM_STEP, undefined, undefined, true);
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
      zoomIn: () => zoomBy(ZOOM_STEP, undefined, undefined, true),
      zoomOut: () => zoomBy(1 / ZOOM_STEP, undefined, undefined, true),
      centerOn,
      toggleFullscreen,
      /** Where the sheet is: kept across a redraw that does not change its size much. */
      position: () => ({ ...at }),
      restore: (position) => {
        stopMotion();
        Object.assign(at, position);
        apply();
      },
      setArranging: (on) => {
        arranging = on;
        stage.classList.toggle('is-arranging', on);
      },
      /** Put the handle where it belongs after the selection changed. */
      refreshHandle,
      /** Drop a line under way, as before a redraw. */
      cancelConnect,
    };
  }

  root.ArcVizStage = { create };
})(typeof globalThis !== 'undefined' ? globalThis : this);
