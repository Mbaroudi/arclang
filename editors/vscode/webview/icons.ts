// The kind icons of src/pure/kind-icons.ts as SVG nodes, the same the
// side bar tree shows and the diagram's glyphs echo.
import { kindIcon } from '../src/pure/kind-icons';

const SVG = 'http://www.w3.org/2000/svg';

export function iconNode(kind: string, size = 16, title?: string): SVGSVGElement {
  const icon = kindIcon(kind);
  const svg = document.createElementNS(SVG, 'svg');
  svg.setAttribute('viewBox', '0 0 16 16');
  svg.setAttribute('width', String(size));
  svg.setAttribute('height', String(size));
  svg.setAttribute('class', `wb-kind-icon wb-kind-${icon.family}`);
  svg.setAttribute('stroke-width', '1.2');
  svg.setAttribute('stroke-linecap', 'round');
  svg.setAttribute('stroke-linejoin', 'round');
  if (title) {
    const t = document.createElementNS(SVG, 'title');
    t.textContent = title;
    svg.appendChild(t);
  } else svg.setAttribute('aria-hidden', 'true');
  for (const shape of icon.shapes) {
    const node = document.createElementNS(SVG, shape.tag);
    node.setAttribute('fill', icon.fill);
    node.setAttribute('stroke', icon.stroke);
    for (const [key, value] of Object.entries(shape.attrs)) node.setAttribute(key, String(value));
    svg.appendChild(node);
  }
  return svg;
}
