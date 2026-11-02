/*
 * ArcViz toolbar — the parts the viewer builds its tool row from: icon
 * buttons drawn as small inline SVGs (no icon font: a page with a strict
 * content policy loads nothing else), drop-down menus that work from the
 * keyboard as well as the pointer, and a row that, when its frame narrows,
 * first drops the words of its menus, then moves whole tools into a "⋯"
 * menu rather than wrapping or cutting them.
 *
 * Chrome is built with DOM APIs and textContent only.
 */
(function (root) {
  'use strict';

  const SVG = 'http://www.w3.org/2000/svg';

  // 16 × 16 outlines in the codicon style, stroked 1.5 in the text color.
  // Each entry is a list of [element, attributes].
  const ICONS = {
    back: [['path', { d: 'M9.5 3.5 5 8l4.5 4.5' }]],
    undo: [['path', { d: 'M5.5 3.5 2.5 6.5l3 3' }], ['path', { d: 'M3 6.5h6.25a3.75 3.75 0 0 1 0 7.5H6.5' }]],
    redo: [['path', { d: 'M10.5 3.5l3 3-3 3' }], ['path', { d: 'M13 6.5H6.75a3.75 3.75 0 0 0 0 7.5H9.5' }]],
    'zoom-out': [['path', { d: 'M3.5 8h9' }]],
    'zoom-in': [['path', { d: 'M8 3.5v9M3.5 8h9' }]],
    fit: [['path', { d: 'M2.5 5.5v-3h3M10.5 2.5h3v3M13.5 10.5v3h-3M5.5 13.5h-3v-3' }], ['rect', { x: 5.5, y: 5.5, width: 5, height: 5, rx: 0.5 }]],
    fullscreen: [['path', { d: 'M9.5 2.5h4v4M13.5 2.5 9.25 6.75M6.5 13.5h-4v-4M2.5 13.5l4.25-4.25' }]],
    layout: [['rect', { x: 2, y: 2.5, width: 5, height: 4, rx: 0.5 }], ['rect', { x: 9, y: 9.5, width: 5, height: 4, rx: 0.5 }], ['path', { d: 'M4.5 6.5v5H9M7 4.5h4.5v5' }]],
    align: [['path', { d: 'M2.5 2v12' }], ['rect', { x: 5, y: 3.5, width: 8, height: 3, rx: 0.5 }], ['rect', { x: 5, y: 9.5, width: 5, height: 3, rx: 0.5 }]],
    filter: [['path', { d: 'M2 3h12l-4.75 5.5v4.75L6.75 12V8.5z' }]],
    export: [['path', { d: 'M8 2.5v8M4.75 7.25 8 10.5l3.25-3.25M2.5 13.5h11' }]],
    theme: [['circle', { cx: 8, cy: 8, r: 5.75 }], ['path', { d: 'M8 2.25v11.5a5.75 5.75 0 0 0 0-11.5z', fill: 'currentColor', stroke: 'none' }]],
    more: [['circle', { cx: 3.5, cy: 8, r: 0.9, fill: 'currentColor' }], ['circle', { cx: 8, cy: 8, r: 0.9, fill: 'currentColor' }], ['circle', { cx: 12.5, cy: 8, r: 0.9, fill: 'currentColor' }]],
    caret: [['path', { d: 'M4.5 6.5 8 10l3.5-3.5' }]],
    check: [['path', { d: 'M3.5 8.5 6.5 11.5 12.5 4.5' }]],
    tree: [['path', { d: 'M3 3h4M5 3v9.5h3.5M5 8h3.5' }], ['rect', { x: 9, y: 6.5, width: 4.5, height: 3, rx: 0.5 }], ['rect', { x: 9, y: 11, width: 4.5, height: 3, rx: 0.5 }]],
    palette: [['rect', { x: 2, y: 2.5, width: 12, height: 11, rx: 1 }], ['path', { d: 'M6.5 2.5v11' }]],
    properties: [['rect', { x: 2, y: 2.5, width: 12, height: 11, rx: 1 }], ['path', { d: 'M9.5 2.5v11M11 5.5h1.5M11 8h1.5' }]],
    minimap: [['rect', { x: 2, y: 2.5, width: 12, height: 11, rx: 1 }], ['rect', { x: 7.5, y: 7, width: 4.5, height: 4.5, rx: 0.5 }]],
    // zorder hook: the Layers panel (two sheets, one over the other).
    layers: [['path', { d: 'M8 2.5 13.5 5.5 8 8.5 2.5 5.5z' }], ['path', { d: 'M2.5 8.25 8 11.25l5.5-3' }], ['path', { d: 'M2.5 10.75 8 13.75l5.5-3' }]],
  };

  /** An icon of the set, as an SVG element hidden from assistive technology. */
  function icon(name, size) {
    const svg = document.createElementNS(SVG, 'svg');
    const px = String(size || 16);
    for (const [key, value] of Object.entries({ viewBox: '0 0 16 16', width: px, height: px, fill: 'none', stroke: 'currentColor', 'stroke-width': '1.5', 'stroke-linecap': 'round', 'stroke-linejoin': 'round', 'aria-hidden': 'true', focusable: 'false', class: `arcviz-icon arcviz-icon-${name}` })) {
      svg.setAttribute(key, value);
    }
    for (const [tag, attributes] of ICONS[name] || []) {
      const part = document.createElementNS(SVG, tag);
      for (const [key, value] of Object.entries(attributes)) part.setAttribute(key, String(value));
      svg.appendChild(part);
    }
    return svg;
  }

  /** True on a Mac (and iPad): shortcuts are written ⌘Z there, Ctrl+Z elsewhere. */
  function isMac(nav) {
    const n = nav || (typeof navigator !== 'undefined' ? navigator : {});
    const platform = (n.userAgentData && n.userAgentData.platform) || n.platform || n.userAgent || '';
    return /mac|iphone|ipad/i.test(platform);
  }

  /**
   * A shortcut as the platform writes it: 'Mod+Shift+Z' is ⇧⌘Z on a Mac,
   * Ctrl+Shift+Z elsewhere. A single key ('0', '+') is left as it is.
   */
  function shortcutLabel(keys, mac) {
    if (!keys) return '';
    const text = String(keys);
    // A trailing '+' is the key itself ('+', 'Mod++').
    const parts = text.endsWith('+') ? [...text.slice(0, -1).split('+').filter(Boolean), '+'] : text.split('+').filter(Boolean);
    if (!mac) return parts.map((part) => (part === 'Mod' ? 'Ctrl' : part)).join('+');
    const glyph = { Mod: '⌘', Shift: '⇧', Alt: '⌥', Ctrl: '⌃' };
    const order = ['Ctrl', 'Alt', 'Shift', 'Mod'];
    const mods = parts.filter((part) => glyph[part]).sort((a, b) => order.indexOf(a) - order.indexOf(b));
    const rest = parts.filter((part) => !glyph[part]);
    return mods.map((part) => glyph[part]).join('') + rest.join('+');
  }

  /** A tooltip: the name, then the shortcut in parentheses when there is one. */
  function tooltip(label, keys, mac) {
    const shortcut = shortcutLabel(keys, mac);
    return shortcut ? `${label} (${shortcut})` : label;
  }

  /** The zoom of the sheet as a reader reads it: 1.234 → "123 %". */
  function formatZoom(scale) {
    const percent = Math.round((Number(scale) || 0) * 100);
    return `${percent} %`;
  }

  /** Where the focus goes in a menu of `count` items after `key`, from `index` (-1: none). */
  function nextIndex(index, count, key) {
    if (count === 0) return -1;
    if (key === 'Home') return 0;
    if (key === 'End') return count - 1;
    if (key === 'ArrowDown') return index < 0 ? 0 : (index + 1) % count;
    if (key === 'ArrowUp') return index < 0 ? count - 1 : (index - 1 + count) % count;
    return index;
  }

  // One menu open at a time across the page; a press elsewhere closes it.
  let openMenu = null;
  if (typeof document !== 'undefined') {
    document.addEventListener('pointerdown', (event) => {
      if (openMenu && !openMenu.contains(event.target)) openMenu.close(false);
    }, true);
    window.addEventListener('blur', () => { if (openMenu) openMenu.close(false); });
    // Esc closes the open menu even when its focused item went away (a
    // filter redraws its list), before the page reads the key.
    document.addEventListener('keydown', (event) => {
      if (event.key !== 'Escape' || !openMenu) return;
      event.preventDefault();
      event.stopPropagation();
      openMenu.close(true);
    }, true);
    window.addEventListener('resize', () => { if (openMenu) openMenu.close(false); });
  }

  /**
   * The builder of one tool row.
   * deps: { el } — the viewer's element helper.
   */
  function create(deps) {
    const el = deps.el;
    const mac = isMac();

    /**
     * A button showing an icon only, its name in the tooltip and to
     * assistive technology. spec: { id, icon, label, keys, run, pressed }.
     */
    function iconButton(spec) {
      const button = el('button', { type: 'button', class: 'arcviz-tool arcviz-icon-tool', id: spec.id, onclick: () => { if (!isOff(button)) spec.run(); } }, [icon(spec.icon)]);
      button.dataset.keys = spec.keys || '';
      if (spec.hint) button.dataset.hint = spec.hint;
      setLabel(button, spec.label);
      if (spec.pressed !== undefined) button.setAttribute('aria-pressed', String(Boolean(spec.pressed)));
      return button;
    }

    /** Rename a button: its accessible name and its tooltip with the shortcut. */
    function setLabel(button, label, reason) {
      button.setAttribute('aria-label', label);
      button.title = reason ? `${label}: ${reason}` : tooltip(button.dataset.hint || label, button.dataset.keys, mac);
    }

    const isOff = (button) => button.disabled || button.getAttribute('aria-disabled') === 'true';

    /**
     * A drop-down: a button with an optional icon, a text label and a caret,
     * over a menu built each time it opens (so what it offers is current).
     * spec: { id, icon, label, title, items: () => [item], panel: () => Element,
     *         className, keepLabel (the words stay in a narrow row),
     *         heading (its title in "⋯"), describe: (label) -> accessible name }
     * item: { label, run, disabled, reason, checked, radio, keys, action, title }
     *       | 'separator' | { heading }
     */
    function menuButton(spec) {
      const text = el('span', { class: 'arcviz-tool-label', text: spec.label });
      const badge = el('span', { class: 'arcviz-badge', hidden: true });
      const button = el('button', {
        type: 'button',
        class: `arcviz-tool arcviz-menu-button${spec.icon ? '' : ' is-text'}`,
        id: spec.id,
        'aria-haspopup': spec.panel ? 'dialog' : 'menu',
        'aria-expanded': 'false',
      }, [spec.icon ? icon(spec.icon) : null, text, badge, icon('caret', 12)]);
      // The menu itself is not inside the button's wrapper: the row moves it
      // to its own end, so a tool moved into "⋯" (hidden) can still show it.
      const pop = el('div', { class: `arcviz-menu-pop${spec.className ? ` ${spec.className}-pop` : ''}`, role: spec.panel ? 'dialog' : 'menu', tabindex: '-1', hidden: true });
      const wrap = el('span', { class: `arcviz-menu${spec.className ? ` ${spec.className}` : ''}${spec.keepLabel ? ' keeps-label' : ''}` }, [button]);
      let label = spec.label;
      let reason = '';
      let returnTo = button;
      const describe = () => {
        const count = badge.hidden ? '' : badge.textContent;
        const base = spec.describe ? spec.describe(label) : label;
        const name = count && /\d/.test(count) ? `${base}, ${count} active` : base;
        button.setAttribute('aria-label', name);
        button.title = reason ? `${label}: ${reason}` : (spec.title || label);
      };
      describe();

      const focusables = () => [...pop.querySelectorAll('[role^="menuitem"], input, select')].filter((node) => !node.closest('[hidden]'));

      function fill() {
        pop.textContent = '';
        if (spec.panel) {
          pop.setAttribute('aria-label', label);
          pop.appendChild(spec.panel());
          return;
        }
        pop.setAttribute('aria-label', label);
        for (const item of spec.items()) pop.appendChild(menuEntry(item, () => close(true)));
      }

      function place(anchor) {
        const box = anchor.getBoundingClientRect();
        pop.style.left = '0px';
        pop.style.top = `${Math.round(box.bottom + 2)}px`;
        const width = pop.offsetWidth;
        const left = Math.max(4, Math.min(box.left, window.innerWidth - width - 4));
        pop.style.left = `${Math.round(left)}px`;
        pop.style.maxHeight = `${Math.max(120, window.innerHeight - box.bottom - 12)}px`;
      }

      /** Open under `anchor` (the button by default); `focusFirst` from the keyboard. */
      function open(focusFirst, anchor) {
        if (isOff(button) && !anchor) return;
        if (openMenu && openMenu !== api) openMenu.close(false);
        returnTo = anchor || button;
        fill();
        pop.hidden = false;
        wrap.classList.add('is-open');
        button.setAttribute('aria-expanded', 'true');
        place(returnTo);
        openMenu = api;
        const first = focusables()[0];
        if (focusFirst && first) first.focus();
        else pop.focus({ preventScroll: true });
      }

      function close(restoreFocus) {
        if (pop.hidden) return;
        pop.hidden = true;
        wrap.classList.remove('is-open');
        button.setAttribute('aria-expanded', 'false');
        if (openMenu === api) openMenu = null;
        if (restoreFocus && returnTo && returnTo.isConnected && !returnTo.closest('[hidden]')) returnTo.focus({ preventScroll: true });
        else if (restoreFocus && button.isConnected) button.focus({ preventScroll: true });
      }

      button.addEventListener('click', (event) => {
        if (!pop.hidden) close(false);
        else open(event.detail === 0); // a click from Enter or Space focuses the first item
      });
      button.addEventListener('keydown', (event) => {
        // Enter and Space open on the first item (not every host reports a
        // keyboard click as one).
        if ((event.key === 'Enter' || event.key === ' ') && pop.hidden) {
          event.preventDefault();
          open(true);
          return;
        }
        if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
          event.preventDefault();
          open(true);
          if (event.key === 'ArrowUp') { const all = focusables(); if (all.length) all[all.length - 1].focus(); }
        }
      });
      pop.addEventListener('keydown', (event) => {
        if (event.key === 'Escape') {
          event.preventDefault();
          event.stopPropagation();
          close(true);
          return;
        }
        if (event.key === 'Tab') { close(false); return; }
        if (!['ArrowDown', 'ArrowUp', 'Home', 'End'].includes(event.key)) return;
        const all = focusables();
        const index = nextIndex(all.indexOf(document.activeElement), all.length, event.key);
        if (index >= 0) all[index].focus();
        event.preventDefault();
        event.stopPropagation();
      });

      const api = {
        root: wrap,
        button,
        pop,
        open,
        close,
        contains: (node) => wrap.contains(node) || pop.contains(node),
        isOpen: () => !pop.hidden,
        setLabel: (next) => { label = next; text.textContent = next; describe(); },
        /** A count beside the label ('' hides it). */
        setBadge: (value) => { badge.textContent = value ? String(value) : ''; badge.hidden = !value; describe(); },
        /** Off with a reason: the button stays focusable, its tooltip says why. */
        setDisabled: (off, why) => {
          reason = off ? why || '' : '';
          button.setAttribute('aria-disabled', String(Boolean(off)));
          if (off) close(false);
          describe();
        },
        /** What it offers, for the overflow menu. */
        entries: () => (spec.panel
          ? [{ label: badge.hidden ? `${label}…` : `${label} (${badge.textContent})…`, keepOpen: true, run: (anchor) => open(true, anchor), disabled: isOff(button), reason }]
          : [{ heading: typeof spec.heading === 'function' ? spec.heading(label) : spec.heading || label }, ...(isOff(button) ? [{ label, disabled: true, reason }] : spec.items())]),
      };
      return api;
    }

    /** One line of a menu. */
    function menuEntry(item, done) {
      if (item === 'separator') return el('div', { class: 'arcviz-menu-rule', role: 'separator' });
      if (item.heading) return el('div', { class: 'arcviz-menu-heading', role: 'presentation', text: item.heading });
      const checkable = item.checked !== undefined;
      const entry = el('button', {
        type: 'button',
        class: 'arcviz-menu-item',
        role: checkable ? (item.radio ? 'menuitemradio' : 'menuitemcheckbox') : 'menuitem',
        'aria-checked': checkable ? String(Boolean(item.checked)) : null,
        'aria-disabled': item.disabled ? 'true' : null,
        'data-action': item.action || null,
        title: item.disabled && item.reason ? item.reason : (item.title || null),
      }, [
        el('span', { class: 'arcviz-menu-check' }, [checkable && item.checked ? icon('check', 14) : null]),
        el('span', { class: 'arcviz-menu-label', text: item.label }),
        item.keys ? el('span', { class: 'arcviz-menu-keys', text: shortcutLabel(item.keys, mac) }) : null,
      ]);
      entry.addEventListener('click', () => {
        if (item.disabled) return;
        if (item.keepOpen) { done(false); item.run(entry); return; }
        done(true);
        item.run();
      });
      return entry;
    }

    /** A run of tools between two separators. */
    function group(label, items) {
      return el('div', { class: 'arcviz-tool-group', role: 'group', 'aria-label': label }, items);
    }

    /**
     * The row itself: `groups` between separators, the `menus` among them
     * (their lists are kept at the end of the row). Tools with a `data-drop`
     * rank move into the "⋯" menu when the row is too narrow, the highest
     * rank first; tools without one always stay. `entriesOf(tool)` says what
     * a moved tool offers there.
     */
    function row(groups, menus, entriesOf) {
      const more = menuButton({ id: 'arcviz-more', icon: 'more', label: 'More tools', className: 'arcviz-more', items: () => overflowItems() });
      more.root.hidden = true;
      const toolbar = el('div', { class: 'arcviz-tools', role: 'toolbar', 'aria-label': 'Diagram tools' }, [...groups, more.root]);
      for (const menu of [...menus, more]) toolbar.appendChild(menu.pop);
      const droppable = () => [...toolbar.querySelectorAll('[data-drop]')].sort((a, b) => Number(b.dataset.drop) - Number(a.dataset.drop));

      function overflowItems() {
        const items = [];
        for (const tool of [...toolbar.querySelectorAll('[data-drop]')].filter((node) => node.classList.contains('is-overflowed'))) {
          const entries = entriesOf(tool);
          if (entries.length === 0) continue;
          if (items.length) items.push('separator');
          items.push(...entries);
        }
        return items;
      }

      const overflows = () => toolbar.scrollWidth > toolbar.clientWidth + 1;
      function syncGroups() {
        for (const g of groups) g.hidden = [...g.children].every((child) => child.hidden || child.classList.contains('is-overflowed'));
      }
      /** Fit the row to its width: words first, then whole tools into "⋯". */
      function fit() {
        if (!toolbar.isConnected || toolbar.clientWidth === 0) return;
        toolbar.classList.remove('is-compact');
        for (const tool of droppable()) tool.classList.remove('is-overflowed');
        more.root.hidden = true;
        syncGroups();
        if (!overflows()) return;
        toolbar.classList.add('is-compact');
        if (!overflows()) return;
        more.root.hidden = false;
        for (const tool of droppable()) {
          if (tool.hidden) continue;
          tool.classList.add('is-overflowed');
          syncGroups();
          if (!overflows()) break;
        }
      }
      let pending = 0;
      const schedule = () => {
        if (pending) return;
        pending = requestAnimationFrame(() => { pending = 0; fit(); });
      };
      if (typeof ResizeObserver !== 'undefined') {
        let width = -1;
        new ResizeObserver(() => {
          if (Math.abs(toolbar.clientWidth - width) < 1) return;
          width = toolbar.clientWidth;
          schedule();
        }).observe(toolbar);
      }
      return { toolbar, more, fit: schedule, fitNow: fit };
    }

    return { iconButton, menuButton, menuEntry, group, row, setLabel, isOff, mac };
  }

  root.ArcVizToolbar = { create, icon, isMac, shortcutLabel, tooltip, formatZoom, nextIndex };
})(typeof globalThis !== 'undefined' ? globalThis : this);
