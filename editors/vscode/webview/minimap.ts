// The minimap (Shift+M): the whole sheet in a corner, the frame on it; a
// click or a drag there brings that point of the sheet to the middle.
import type { ViewerApi } from './globals';
import { el } from './dom';

export interface MinimapHost {
  viewer: () => ViewerApi | null;
  /** Changes when what the sheet shows changes, so the picture is redrawn. */
  key: () => string;
}

export function createMinimap(sheet: HTMLElement, host: HTMLElement, owner: MinimapHost): { toggle: (force?: boolean) => boolean } {
  const minimap = el('div', { class: 'wb-minimap', hidden: true, 'aria-label': 'Overview of the diagram' });
  const image = el('img', { alt: '' });
  const frameBox = el('div', { class: 'wb-minimap-view' });
  minimap.append(image, frameBox);
  sheet.appendChild(minimap);
  let url: string | null = null;
  let drawnFor = '';

  /** Show or hide it; true when it shows. */
  function toggle(force?: boolean): boolean {
    minimap.hidden = force === undefined ? !minimap.hidden : !force;
    if (!minimap.hidden) refresh(true);
    return !minimap.hidden;
  }

  /** The drawn sheet and where the frame is on it. */
  function geometry(): { svg: SVGSVGElement; width: number; height: number; at: { scale: number; x: number; y: number }; frame: DOMRect } | null {
    const svg = host.querySelector<SVGSVGElement>('.arcviz-stage svg');
    const stage = host.querySelector<HTMLElement>('.arcviz-stage');
    const position = owner.viewer()?.current?.()?.position as { scale: number; x: number; y: number } | undefined;
    if (!svg || !stage || !position) return null;
    const width = Number(svg.getAttribute('width')) || svg.viewBox.baseVal.width;
    const height = Number(svg.getAttribute('height')) || svg.viewBox.baseVal.height;
    if (!width || !height) return null;
    return { svg, width, height, at: position, frame: stage.getBoundingClientRect() };
  }

  function refresh(redraw = false): void {
    if (minimap.hidden) return;
    const g = geometry();
    if (!g) return;
    const key = `${owner.key()}:${g.width}x${g.height}`;
    if (redraw || key !== drawnFor) {
      drawnFor = key;
      if (url) URL.revokeObjectURL(url);
      url = URL.createObjectURL(new Blob([new XMLSerializer().serializeToString(g.svg)], { type: 'image/svg+xml' }));
      image.src = url;
    }
    const box = minimap.getBoundingClientRect();
    const k = Math.min(box.width / g.width, box.height / g.height);
    const offsetX = (box.width - g.width * k) / 2;
    const offsetY = (box.height - g.height * k) / 2;
    const { scale, x, y } = g.at;
    frameBox.style.left = `${offsetX + (-x / scale) * k}px`;
    frameBox.style.top = `${offsetY + (-y / scale) * k}px`;
    frameBox.style.width = `${(g.frame.width / scale) * k}px`;
    frameBox.style.height = `${(g.frame.height / scale) * k}px`;
  }

  function jump(event: PointerEvent): void {
    const g = geometry();
    if (!g) return;
    const box = minimap.getBoundingClientRect();
    const k = Math.min(box.width / g.width, box.height / g.height);
    const sheetX = (event.clientX - box.left - (box.width - g.width * k) / 2) / k;
    const sheetY = (event.clientY - box.top - (box.height - g.height * k) / 2) / k;
    const { scale } = g.at;
    owner.viewer()?.restore?.({ scale, x: g.frame.width / 2 - sheetX * scale, y: g.frame.height / 2 - sheetY * scale });
    refresh();
  }
  minimap.addEventListener('pointerdown', (event) => {
    event.preventDefault();
    event.stopPropagation();
    minimap.setPointerCapture(event.pointerId);
    jump(event);
    const move = (e: PointerEvent) => jump(e);
    const up = () => {
      minimap.removeEventListener('pointermove', move);
      minimap.removeEventListener('pointerup', up);
    };
    minimap.addEventListener('pointermove', move);
    minimap.addEventListener('pointerup', up);
  });
  setInterval(() => refresh(), 250);
  return { toggle };
}
