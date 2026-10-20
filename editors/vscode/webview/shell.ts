// The frame of the page: a bar (layers and the viewer's tools), then a row
// with the palette rail, the canvas and the properties drawer (resized by
// its splitter), then one status line. The widths and what is open are
// remembered by the page.
import { button, el } from './dom';

export interface ShellLayout {
  paletteOpen: boolean;
  drawerOpen: boolean;
  drawerWidth: number;
  rightTab: number;
}

export interface ShellHooks {
  /** The layout changed: remember it. */
  save: () => void;
  /** The palette was opened or closed. */
  palette: (open: boolean) => void;
  minimap: () => void;
}

/**
 * A panel toggle of the bar: the viewer's icon set, a pressed state, the
 * name beside the icon when the bar has room (workbench.css), the shortcut
 * in the tooltip.
 */
function toggle(icon: string, label: string, keys: string, onClick: () => void, className: string): HTMLButtonElement {
  const kit = window.ArcVizToolbar;
  const node = el('button', { type: 'button', class: `wb-icon-button wb-bar-toggle ${className}`, 'aria-label': label, 'aria-pressed': 'false', title: kit.tooltip(label, keys, kit.isMac()) }, [kit.icon(icon), el('span', { class: 'wb-toggle-label', text: label })]);
  node.addEventListener('click', onClick);
  return node;
}

export function createShell(root: HTMLElement, layout: ShellLayout, hooks: ShellHooks) {
  const bar = el('header', { class: 'wb-bar' });
  const layerBar = el('nav', { class: 'wb-layers', 'aria-label': 'Arcadia layers' });
  const toolRow = el('div', { class: 'wb-toolrow' });
  const tabs = el('div', { class: 'wb-tabs wb-hidden' });
  const tools = el('div', { class: 'wb-viewer-tools' });
  const paletteToggle = toggle('palette', 'Palette', '', () => setPalette(!layout.paletteOpen), 'wb-palette-toggle');
  const minimapToggle = toggle('minimap', 'Minimap', 'Shift+M', () => hooks.minimap(), 'wb-minimap-toggle');
  const drawerToggle = toggle('properties', 'Properties', '', () => setDrawer(!layout.drawerOpen), 'wb-drawer-toggle');
  toolRow.append(paletteToggle, tools, minimapToggle, drawerToggle);
  bar.append(layerBar, toolRow);

  const body = el('div', { class: 'wb-body' });
  const left = el('aside', { class: 'wb-left' });
  const sheet = el('main', { class: 'wb-sheet' });
  const host = el('div', { class: 'wb-host' });
  const modeChip = el('div', { class: 'wb-mode', hidden: true });
  const toast = el('div', { class: 'wb-toast', hidden: true, role: 'status', 'aria-live': 'polite' });
  const progress = el('div', { class: 'wb-progress', hidden: true });
  const layoutNotice = el('div', { class: 'wb-notice', hidden: true, role: 'status' });
  sheet.append(progress, host, modeChip, toast, layoutNotice);
  const splitter = el('div', { class: 'wb-splitter', role: 'separator', 'aria-orientation': 'vertical', title: 'Drag to resize' });
  const right = el('aside', { class: 'wb-right' });
  const bottom = el('footer', { class: 'wb-bottom' });
  const hiddenTree = el('div', { class: 'wb-hidden' });
  const hiddenFoot = el('div', { class: 'wb-hidden' });
  const detailsSlot = el('div', { class: 'wb-details' });
  const relatedSlot = el('div', { class: 'wb-related' });

  const rightTabs = el('div', { class: 'wb-right-tabs', role: 'tablist' });
  const propertiesPane = el('div', { class: 'wb-pane' });
  const browserPane = el('div', { class: 'wb-pane' });
  const detailsPane = el('div', { class: 'wb-pane' }, [detailsSlot, relatedSlot]);
  const panes: [string, HTMLElement][] = [
    ['Properties', propertiesPane],
    ['Relations', browserPane],
    ['Details', detailsPane],
  ];
  panes.forEach(([label, pane], index) => {
    rightTabs.appendChild(button(label, () => showRightTab(index), { class: 'wb-right-tab', role: 'tab' }));
    pane.setAttribute('role', 'tabpanel');
  });
  function showRightTab(index: number): void {
    layout.rightTab = index;
    panes.forEach(([, pane], i) => pane.classList.toggle('is-active', i === index));
    [...rightTabs.children].forEach((tab, i) => {
      tab.classList.toggle('is-active', i === index);
      tab.setAttribute('aria-selected', String(i === index));
    });
    hooks.save();
  }
  right.append(rightTabs, propertiesPane, browserPane, detailsPane);
  body.append(left, sheet, splitter, right);
  root.append(bar, body, bottom, hiddenTree, hiddenFoot);
  showRightTab(layout.rightTab);

  function setPalette(open: boolean): void {
    layout.paletteOpen = open;
    left.classList.toggle('is-collapsed', !open);
    paletteToggle.setAttribute('aria-pressed', String(open));
    hooks.palette(open);
    hooks.save();
    fitStage();
  }

  function setDrawer(open: boolean): void {
    layout.drawerOpen = open;
    right.hidden = !open;
    splitter.hidden = !open;
    right.style.width = `${layout.drawerWidth}px`;
    drawerToggle.setAttribute('aria-pressed', String(open));
    hooks.save();
    fitStage();
  }

  // The drawer is resized by dragging the splitter; the width is remembered.
  splitter.addEventListener('pointerdown', (down) => {
    down.preventDefault();
    splitter.setPointerCapture(down.pointerId);
    const startX = down.clientX;
    const startWidth = layout.drawerWidth;
    const move = (event: PointerEvent) => {
      layout.drawerWidth = Math.min(480, Math.max(240, startWidth - (event.clientX - startX)));
      right.style.width = `${layout.drawerWidth}px`;
      fitStage();
    };
    const up = () => {
      splitter.removeEventListener('pointermove', move);
      splitter.removeEventListener('pointerup', up);
      hooks.save();
    };
    splitter.addEventListener('pointermove', move);
    splitter.addEventListener('pointerup', up);
  });

  /** The viewer sizes its stage from a variable: it gets what the sheet has left under its own chrome. */
  function fitStage(): void {
    const stage = host.querySelector<HTMLElement>('.arcviz-stage');
    if (!stage) return;
    const above = stage.getBoundingClientRect().top - sheet.getBoundingClientRect().top;
    let below = 0;
    let after = stage.nextElementSibling as HTMLElement | null;
    while (after) {
      if (!after.hidden) below += after.getBoundingClientRect().height;
      after = after.nextElementSibling as HTMLElement | null;
    }
    const height = Math.max(160, sheet.clientHeight - above - below);
    root.style.setProperty('--av-stage-height', `${Math.floor(height)}px`);
  }
  new ResizeObserver(() => fitStage()).observe(sheet);

  /** The minimap shows (or not): its toggle says so. */
  function setMinimapShown(shown: boolean): void {
    minimapToggle.setAttribute('aria-pressed', String(shown));
  }

  return { setMinimapShown, layerBar, tabs, tools, left, sheet, host, modeChip, toast, progress, layoutNotice, right, bottom, hiddenTree, hiddenFoot, detailsSlot, relatedSlot, propertiesPane, browserPane, showRightTab, setPalette, setDrawer, fitStage };
}
