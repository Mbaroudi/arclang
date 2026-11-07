// The PDF view's messages and its zoom, kept free of the DOM and of VS Code.

export type FromPdfView = { type: 'ready' } | { type: 'outside' };
export type ToPdfView = { type: 'pdf'; name: string; base64: string } | { type: 'error'; text: string };

export type Fit = 'width' | 'page' | 'none';

export const ZOOM_STEPS = [0.25, 0.33, 0.5, 0.67, 0.75, 0.9, 1, 1.1, 1.25, 1.5, 1.75, 2, 2.5, 3, 4];
const MIN = ZOOM_STEPS[0];
const MAX = ZOOM_STEPS[ZOOM_STEPS.length - 1];

/** The next step in a direction from any scale (a fitted scale falls between steps). */
export function stepZoom(scale: number, direction: 1 | -1): number {
  const next = direction > 0 ? ZOOM_STEPS.find((s) => s > scale + 1e-3) : [...ZOOM_STEPS].reverse().find((s) => s < scale - 1e-3);
  return next ?? (direction > 0 ? MAX : MIN);
}

/** The scale at which a page of `page` points fits `view` pixels (less a margin), by width or whole. */
export function fitScale(fit: Exclude<Fit, 'none'>, page: { width: number; height: number }, view: { width: number; height: number }, margin = 24): number {
  const byWidth = (view.width - 2 * margin) / page.width;
  const scale = fit === 'width' ? byWidth : Math.min(byWidth, (view.height - 2 * margin) / page.height);
  return Math.min(MAX, Math.max(MIN, scale));
}

/** The page shown: the one whose top is nearest above the middle of the view. */
export function currentPage(tops: number[], scrollTop: number, viewHeight: number): number {
  const middle = scrollTop + viewHeight / 3;
  let page = 1;
  tops.forEach((top, index) => {
    if (top <= middle) page = index + 1;
  });
  return page;
}
